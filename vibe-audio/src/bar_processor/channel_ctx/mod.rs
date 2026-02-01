mod supporting_points;

use std::ops::Range;

use crate::{
    interpolation::{
        CubicSplineInterpolation, Interpolater, InterpolationInner, LinearInterpolation,
        NothingInterpolation,
    },
    BarProcessorConfig, InterpolationVariant,
};
use cpal::SampleRate;
use realfft::num_complex::Complex32;

const INIT_NORMALIZATION_FACTOR: f32 = 1.;

/// Contains every additional information for a channel to be processed.
pub struct ChannelCtx {
    // The interpolation strategy for this channel
    interpolator: Box<dyn Interpolater>,
    // Contains the index range for each supporting point within the fft output for each supporting point
    ranges: Box<[Range<usize>]>,

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
        let (supporting_points, ranges) = supporting_points::compute(config, sample_rate, fft_size);

        let interpolator: Box<dyn Interpolater> = match config.interpolation {
            InterpolationVariant::None => NothingInterpolation::boxed(supporting_points),
            InterpolationVariant::Linear => LinearInterpolation::boxed(supporting_points),
            InterpolationVariant::CubicSpline => CubicSplineInterpolation::boxed(supporting_points),
        };

        let peak = vec![0f32; u16::from(config.amount_bars) as usize].into_boxed_slice();
        let fall = peak.clone();
        let mem = peak.clone();
        let prev = peak.clone();

        Self {
            interpolator,
            ranges,

            normalize_factor: INIT_NORMALIZATION_FACTOR,
            sensitivity: config.sensitivity,

            prev,
            peak,
            fall,
            mem,
        }
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
            .zip(self.ranges.iter())
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
