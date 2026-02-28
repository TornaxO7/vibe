mod fft_out_metadata;
mod padding;

use crate::{
    interpolation::{Interpolater, InterpolatorDescriptor},
    BarProcessorConfig, PaddingSize,
};
use cpal::SampleRate;
use fft_out_metadata::{FftOutMetadata, FftOutMetadataDescriptor};
use padding::PaddingCtx;
use realfft::num_complex::Complex32;
use std::ops::Range;

const INIT_NORMALIZATION_FACTOR: f32 = 1.;
const DEFAULT_PADDING_SIZE: usize = 5;

/// Contains every additional information for a channel to be processed.
pub struct ChannelCtx<I: Interpolater> {
    // The interpolation strategy for this channel
    interpolator: I,
    // Contains the index range for each supporting point within the fft output for each supporting point
    fft_out_ranges: Box<[Range<usize>]>,

    padding: Option<PaddingCtx>,

    normalize_factor: f32,

    up: f32,
    down: f32,
    correction_offset: f32,

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
impl<I: Interpolater> ChannelCtx<I> {
    pub fn new(config: &BarProcessorConfig, sample_rate: SampleRate, fft_size: usize) -> Self {
        let mut data = FftOutMetadata::interpret_fft_context(FftOutMetadataDescriptor {
            amount_bars: config.amount_bars,
            sample_rate,
            fft_size,
            freq_range: config.freq_range.clone(),
        })
        .fillup(config.amount_bars)
        .redistribute(config.bar_distribution);

        let padding = config.padding.as_ref().map(|conf| {
            let size = match conf.size {
                PaddingSize::Auto => match config.bar_distribution {
                    crate::BarDistribution::Uniform => {
                        if data.supporting_points.len() > 1 {
                            let first = data.supporting_points[0].x;
                            let second = data.supporting_points[1].x;

                            (second - first) * 4
                        } else {
                            DEFAULT_PADDING_SIZE
                        }
                    }
                    crate::BarDistribution::Natural => DEFAULT_PADDING_SIZE,
                },
                PaddingSize::Custom(size) => size.get().into(),
            };

            let ctx = PaddingCtx::new(size, conf.side.clone());
            ctx.adjust_supporting_points(&mut data.supporting_points);
            ctx
        });

        let FftOutMetadata {
            supporting_points,
            supporting_points_fft_ranges,
        } = data;

        let interpolator = I::new(InterpolatorDescriptor { supporting_points });
        let covered_bar_range = interpolator.covered_bar_range();

        let peak = vec![0f32; covered_bar_range.len()].into_boxed_slice();
        let fall = peak.clone();
        let mem = peak.clone();
        let prev = peak.clone();

        let ctx = Self {
            interpolator,
            fft_out_ranges: supporting_points_fft_ranges,
            padding: padding.clone(),

            normalize_factor: INIT_NORMALIZATION_FACTOR,

            up: config.up,
            down: config.down,
            correction_offset: config.correction_offset,

            prev,
            peak,
            fall,
            mem,
        };

        assert!(ctx.total_amount_bars() <= (u16::MAX as usize),
            "The configured amount of bars ({}) and the padding size ({}) exceeds the limit of {} bars (total amount bars: {})", config.amount_bars.get(), padding.as_ref().map(|ctx| ctx.amount_bars()).unwrap_or(0), u16::MAX, ctx.total_amount_bars());

        ctx
    }
}

/// Processing relevant methods
impl<I: Interpolater> ChannelCtx<I> {
    pub fn update_supporting_points(&mut self, fft_out: &[Complex32]) {
        let mut overshoot = false;
        let mut is_silent = true;

        let amount_bars = self.prev.len();

        for (sup_idx, (supporting_point, fft_range)) in self
            .interpolator
            .supporting_points_mut()
            .iter_mut()
            .zip(self.fft_out_ranges.iter())
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

                // reduce the bass change (low `x` value) and increase the change of the treble (high `x` value)
                let correction = normalized_x.powf(2.) + self.correction_offset;

                raw_bar_val * self.normalize_factor * correction
            };

            debug_assert!(!prev_magnitude.is_nan());
            debug_assert!(!next_magnitude.is_nan());

            // IDEA: Maybe replace the whole smoothing stuff with
            // supporting_point.y = supporting_point.y * 0.5 + next_magnitude;
            // for _beat detection_ for better efficiency <.<

            // shoutout to `cava` for their computation on how to make the falling look smooth.
            // Really nice idea!
            if next_magnitude < self.prev[sup_idx] {
                next_magnitude =
                    self.peak[sup_idx] * (1. - (self.fall[sup_idx].powf(2.) * self.down));

                if next_magnitude < 0. {
                    next_magnitude = 0.;
                }
                self.fall[sup_idx] += 0.028;
            } else {
                self.peak[sup_idx] = next_magnitude;
                self.fall[sup_idx] = 0.0;
            }
            self.prev[sup_idx] = next_magnitude;

            supporting_point.y = self.mem[sup_idx] * self.up + next_magnitude;
            self.mem[sup_idx] = supporting_point.y;

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

        if let Some(ctx) = &self.padding {
            ctx.apply(bar_values);
        }
    }

    /// Returns the total amount of bars which are going to be rendered which includes
    /// the configured amount of bars _and_ the amount of padded bars.
    pub fn total_amount_bars(&self) -> usize {
        let unpadded_amount = self.interpolator.covered_bar_range().len();

        let padding_size = self
            .padding
            .as_ref()
            .map(|ctx| ctx.amount_bars())
            .unwrap_or(0);

        unpadded_amount + padding_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod total_amount_bars {
        use crate::{NothingInterpolation, PaddingConfig};

        use super::*;
        use std::num::NonZero;

        const DUMMY_FFT_SIZE: usize = 2048;

        #[test]
        fn one_bar_no_padding() {
            let ctx: ChannelCtx<NothingInterpolation> = ChannelCtx::new(
                &BarProcessorConfig {
                    amount_bars: NonZero::new(1).unwrap(),
                    ..Default::default()
                },
                crate::DEFAULT_SAMPLE_RATE,
                DUMMY_FFT_SIZE,
            );

            assert_eq!(ctx.total_amount_bars(), 1);
        }

        #[test]
        fn one_bar_with_oneside_padding() {
            let ctx: ChannelCtx<NothingInterpolation> = ChannelCtx::new(
                &BarProcessorConfig {
                    amount_bars: NonZero::new(1).unwrap(),
                    padding: Some(PaddingConfig {
                        side: crate::PaddingSide::Left,
                        size: PaddingSize::Custom(NonZero::new(10).unwrap()),
                    }),
                    ..Default::default()
                },
                crate::DEFAULT_SAMPLE_RATE,
                DUMMY_FFT_SIZE,
            );

            assert_eq!(ctx.total_amount_bars(), 11);
        }

        #[test]
        fn one_bar_with_twoside_padding() {
            let ctx: ChannelCtx<NothingInterpolation> = ChannelCtx::new(
                &BarProcessorConfig {
                    amount_bars: NonZero::new(1).unwrap(),
                    padding: Some(PaddingConfig {
                        side: crate::PaddingSide::Both,
                        size: PaddingSize::Custom(NonZero::new(10).unwrap()),
                    }),
                    ..Default::default()
                },
                crate::DEFAULT_SAMPLE_RATE,
                DUMMY_FFT_SIZE,
            );

            assert_eq!(ctx.total_amount_bars(), 21);
        }
    }
}
