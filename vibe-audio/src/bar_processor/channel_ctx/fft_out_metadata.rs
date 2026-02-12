use crate::{
    bar_processor::config::BarDistribution, interpolation::SupportingPoint, MAX_HUMAN_FREQUENCY,
    MIN_HUMAN_FREQUENCY,
};
use crate::SampleRate;
use std::{num::NonZero, ops::Range};
use tracing::debug;

#[derive(Debug)]
pub struct FftOutMetadataDescriptor {
    pub amount_bars: NonZero<u16>,
    pub sample_rate: SampleRate,
    pub fft_size: usize,
    pub freq_range: Range<NonZero<u16>>,
}

/// Stores information which is extracted of the context for the fft.
#[derive(Debug)]
pub struct FftOutMetadata {
    /// The supporting points which can then be interpolated later to create a smooth audio
    /// wave.
    pub supporting_points: Box<[SupportingPoint]>,
    /// The index ranges within the fft output for each supporting point.
    pub supporting_points_fft_ranges: Box<[Range<usize>]>,
}

impl FftOutMetadata {
    /// Given the fft-context (input size, sample rate, etc.), it computes the required
    /// preparations to process the fft-output.
    pub fn interpret_fft_context(desc: FftOutMetadataDescriptor) -> Self {
        // == preparations
        let weights = {
            let amount_bars = desc.amount_bars.get() as u32;

            (0..amount_bars)
                .map(|index| exp_fun((index + 1) as f32 / (amount_bars + 1) as f32))
                .collect::<Vec<f32>>()
        };
        debug!("Weights: {:?}", weights);

        let amount_bins = {
            let freq_resolution = desc.sample_rate as f32 / desc.fft_size as f32;
            debug!("Freq resolution: {}", freq_resolution);

            // the relevant index range of the fft output which we should use for the bars
            let bin_range = Range {
                start: ((desc.freq_range.start.get() as f32 / freq_resolution) as usize).max(1),
                end: (desc.freq_range.end.get() as f32 / freq_resolution).ceil() as usize,
            };
            debug!("Bin range: {:?}", bin_range);
            bin_range.len()
        };
        debug!("Available bins: {}", amount_bins);

        // == supporting points
        let mut supporting_points = Vec::new();
        let mut supporting_points_fft_ranges = Vec::new();

        let mut prev_fft_range = 0..0;
        for (bar_idx, weight) in weights.iter().enumerate() {
            let end = ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

            let new_fft_range = prev_fft_range.end..end;

            let is_supporting_point = new_fft_range != prev_fft_range && !new_fft_range.is_empty();
            if is_supporting_point {
                supporting_points.push(SupportingPoint { x: bar_idx, y: 0. });

                debug_assert!(!new_fft_range.is_empty());
                supporting_points_fft_ranges.push(new_fft_range.clone());
            }

            prev_fft_range = new_fft_range;
        }

        debug_assert_eq!(
            supporting_points.first().unwrap().x,
            0,
            "The first supporting point must be the first bar."
        );

        Self {
            supporting_points: supporting_points.into_boxed_slice(),
            supporting_points_fft_ranges: supporting_points_fft_ranges.into_boxed_slice(),
        }
    }

    /// Ensures that all supporting points cover the given amount of bars.
    pub fn fillup(mut self, amount_bars: NonZero<u16>) -> Self {
        // It could happen that we don't have enough supporting points yet to have the given amount of bars set in `config.amount_bars`.
        // So just add a supporting point in the end.
        let last_x = self.supporting_points.last().unwrap().x + 1;
        let not_enough_supporting_points = last_x < amount_bars.get() as usize;
        if not_enough_supporting_points {
            let mut filled_supporting_points = self.supporting_points.to_vec();
            filled_supporting_points.push(SupportingPoint {
                x: (amount_bars.get() - 1) as usize,
                y: 0.,
            });

            self.supporting_points = filled_supporting_points.into_boxed_slice();
        }

        assert!(
            self.supporting_points.last().unwrap().x == (amount_bars.get() - 1) as usize,
            "The supporting points from '{:?}' to '{:?}' don't cover '{}' bars <.<",
            self.supporting_points.first().unwrap(),
            self.supporting_points.last().unwrap(),
            amount_bars.get()
        );

        self
    }

    /// Reorder the supporting points by the given `distribution` policy.
    pub fn redistribute(mut self, distribution: BarDistribution) -> Self {
        match distribution {
            BarDistribution::Uniform => {
                let supporting_points_len = self.supporting_points.len();
                let step = self.covered_amount_bars() as f32 / supporting_points_len as f32;
                for (idx, supporting_point) in self.supporting_points
                    [..supporting_points_len.saturating_sub(1)]
                    .iter_mut()
                    .enumerate()
                {
                    supporting_point.x = (idx as f32 * step) as usize;
                }
            }
            BarDistribution::Natural => {}
        }

        self
    }

    /// Returns the amount of bars which the supporting points cover up.
    ///
    /// The first and last supporting point are the first and last bar.
    fn covered_amount_bars(&self) -> usize {
        let last = self.supporting_points.last().unwrap();
        let first = self.supporting_points.first().unwrap();

        (last.x + 1) - first.x
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
