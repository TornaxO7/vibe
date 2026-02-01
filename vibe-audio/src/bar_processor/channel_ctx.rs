use std::ops::Range;

use cpal::SampleRate;
use realfft::num_complex::Complex32;
use tracing::debug;

use crate::{
    bar_processor::config::BarDistribution,
    interpolation::{
        CubicSplineInterpolation, Interpolater, InterpolationInner, LinearInterpolation,
        NothingInterpolation, SupportingPoint,
    },
    BarProcessorConfig, InterpolationVariant, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};

/// Contains every additional information for a channel to be processed.
pub struct ChannelCtx {
    // The interpolation strategy for this channel
    interpolator: Box<dyn Interpolater>,
    // Contains the index range for each supporting point within the fft output for each supporting point
    supporting_point_fft_ranges: Box<[Range<usize>]>,

    normalize_factor: f32,
    sensitivity: f32,

    // Contains the raw previous bar values
    prev: Box<[f32]>,
    // Contains the last peak value
    peak: Box<[f32]>,
    // Contains the time how long the i-th bar is falling
    fall: Box<[f32]>,
    // Contains the previous, smoothened bar values
    mem: Box<[f32]>,
}

/// Construction relevant methods
impl ChannelCtx {
    pub fn new(config: &BarProcessorConfig, sample_rate: SampleRate, fft_size: usize) -> Self {
        let (interpolator, supporting_point_fft_ranges) =
            Self::new_interpolation_data(config, sample_rate, fft_size);

        let peak = vec![0f32; u16::from(config.amount_bars) as usize].into_boxed_slice();
        let fall = peak.clone();
        let mem = peak.clone();
        let prev = peak.clone();

        Self {
            interpolator,
            supporting_point_fft_ranges,
            normalize_factor: 1.,
            sensitivity: config.sensitivity,

            prev,
            peak,
            fall,
            mem,
        }
    }

    /// Calculates the indexes for the fft output on how to distribute them to each bar.
    fn new_interpolation_data(
        config: &BarProcessorConfig,
        sample_rate: SampleRate,
        sample_len: usize,
    ) -> (Box<dyn Interpolater>, Box<[Range<usize>]>) {
        // == preparations
        let weights = {
            let amount_bars = config.amount_bars.get() as u32;

            (0..amount_bars)
                .map(|index| exp_fun((index + 1) as f32 / (amount_bars + 1) as f32))
                .collect::<Vec<f32>>()
        };
        debug!("Weights: {:?}", weights);

        let amount_bins = {
            let freq_resolution = sample_rate as f32 / sample_len as f32;
            debug!("Freq resolution: {}", freq_resolution);

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((u16::from(config.freq_range.start) as f32 / freq_resolution) as usize)
                    .max(1),
                end: (u16::from(config.freq_range.end) as f32 / freq_resolution).ceil() as usize,
            };
            debug!("Bin range: {:?}", bin_range);
            bin_range.len()
        };
        debug!("Available bins: {}", amount_bins);

        // == supporting points
        let (supporting_points, supporting_point_fft_ranges) = {
            let mut supporting_points = Vec::new();
            let mut supporting_point_fft_ranges = Vec::new();

            let mut prev_fft_range = 0..0;
            for (bar_idx, weight) in weights.iter().enumerate() {
                let end =
                    ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

                let new_fft_range = prev_fft_range.end..end;

                let is_supporting_point =
                    new_fft_range != prev_fft_range && !new_fft_range.is_empty();
                if is_supporting_point {
                    supporting_points.push(SupportingPoint { x: bar_idx, y: 0. });

                    debug_assert!(!new_fft_range.is_empty());
                    supporting_point_fft_ranges.push(new_fft_range.clone());
                }

                prev_fft_range = new_fft_range;
            }

            // re-adjust the supporting points if needed
            match config.bar_distribution {
                BarDistribution::Uniform => {
                    let step = config.amount_bars.get() as f32 / supporting_points.len() as f32;
                    let supporting_points_len = supporting_points.len();
                    for (idx, supporting_point) in supporting_points
                        [..supporting_points_len.saturating_sub(1)]
                        .iter_mut()
                        .enumerate()
                    {
                        supporting_point.x = (idx as f32 * step) as usize;
                    }
                }
                BarDistribution::Natural => {}
            }

            (supporting_points, supporting_point_fft_ranges)
        };

        // create the interpolator
        let interpolator: Box<dyn Interpolater> = match config.interpolation {
            InterpolationVariant::None => NothingInterpolation::boxed(supporting_points),
            InterpolationVariant::Linear => LinearInterpolation::boxed(supporting_points),
            InterpolationVariant::CubicSpline => CubicSplineInterpolation::boxed(supporting_points),
        };

        (interpolator, supporting_point_fft_ranges.into_boxed_slice())
    }
}

/// Processing relevant methods
impl ChannelCtx {
    pub fn update_supporting_points(&mut self, fft_out: &[Complex32]) {
        let mut overshoot = false;
        let mut is_silent = true;

        let amount_bars = self.prev.len();

        for (bar_idx, (supporting_point, fft_range)) in self
            .interpolator
            .supporting_points_mut()
            .zip(self.supporting_point_fft_ranges.iter_mut())
            .enumerate()
        {
            let normalized_x = supporting_point.x as f32 / amount_bars as f32;

            let amount_bins = fft_range.len() as f32;
            let prev_magnitude = supporting_point.y;
            let mut next_magnitude = {
                let raw_bar_val = fft_out[fft_range.clone()]
                    .iter()
                    .map(|out| {
                        let mag = out.norm();
                        if mag > 0. {
                            is_silent = false;
                        }
                        mag
                    })
                    .sum::<f32>()
                    / amount_bins;

                // reduce the bass change (low x value) and increase the change of the treble (high x value)
                let correction = normalized_x.powf(2.) + 0.05;

                raw_bar_val * self.normalize_factor * correction
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            // shoutout to `cava` for their computation on how to make the falling look smooth.
            // Really nice idea!
            if next_magnitude < self.prev[bar_idx] {
                next_magnitude =
                    self.peak[bar_idx] * (1. - (self.fall[bar_idx].powf(2.) * self.sensitivity));

                if next_magnitude < 0. {
                    next_magnitude = 0.;
                }
                self.fall[bar_idx] += 0.028;
            } else {
                self.peak[bar_idx] = next_magnitude;
                self.fall[bar_idx] = 0.0;
            }
            self.prev[bar_idx] = next_magnitude;

            supporting_point.y = self.mem[bar_idx] * 0.77 + next_magnitude;
            self.mem[bar_idx] = supporting_point.y;

            if supporting_point.y > 1. {
                overshoot = true;
            }
        }

        if overshoot {
            self.normalize_factor *= 0.98;
        } else if !is_silent {
            self.normalize_factor *= 1.002;
        }
    }

    pub fn interpolate(&mut self, bar_values: &mut [f32]) {
        self.interpolator.interpolate(bar_values);
    }
}

// Bascially `inv_mel` but with the precondition that the argument `x` is within the range [0, 1]
// where:
//   - `0` = the minimum frequency which a human can hear
//   - `1` = the maximum frequency which a human can hear
fn exp_fun(x: f32) -> f32 {
    debug_assert!(0. <= x);
    debug_assert!(x <= 1.);

    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);

    // map [0, 1] => [min-mel-value, max-mel-value]
    let mapped_x = x * (max_mel_value - min_mel_value) + min_mel_value;
    inv_mel(mapped_x)
}

// https://en.wikipedia.org/wiki/Mel_scale
fn mel(x: f32) -> f32 {
    debug_assert!(MIN_HUMAN_FREQUENCY as f32 <= x);
    debug_assert!(x <= MAX_HUMAN_FREQUENCY as f32);

    2595. * (1. + x / 700.).log10()
}

fn inv_mel(x: f32) -> f32 {
    let min_mel_value = mel(MIN_HUMAN_FREQUENCY as f32);
    let max_mel_value = mel(MAX_HUMAN_FREQUENCY as f32);

    debug_assert!(min_mel_value <= x);
    debug_assert!(x <= max_mel_value);

    700. * (10f32.powf(x / 2595.) - 1.)
}
