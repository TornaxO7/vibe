use crate::{
    bar_processor::{
        config::{BarDistribution, PaddingSide},
        PaddingSize,
    },
    interpolation::SupportingPoint,
    BarProcessorConfig, MAX_HUMAN_FREQUENCY, MIN_HUMAN_FREQUENCY,
};
use cpal::SampleRate;
use std::ops::Range;
use tracing::debug;

/// Interprets the fft output. (dunno how to describe this in a short sentence)
///
/// # Returns
/// 1. The supporting points
/// 2. The ranges within the fft-output for the i-th supporting point.
pub fn compute(
    config: &BarProcessorConfig,
    sample_rate: SampleRate,
    fft_size: usize,
) -> (Vec<SupportingPoint>, Box<[Range<usize>]>) {
    // == preparations
    let weights = {
        let amount_bars = config.amount_bars.get() as u32;

        (0..amount_bars)
            .map(|index| exp_fun((index + 1) as f32 / (amount_bars + 1) as f32))
            .collect::<Vec<f32>>()
    };
    debug!("Weights: {:?}", weights);

    let amount_bins = {
        let freq_resolution = sample_rate as f32 / fft_size as f32;
        debug!("Freq resolution: {}", freq_resolution);

        // the relevant index range of the fft output which we should use for the bars
        let bin_range = Range {
            start: ((u16::from(config.freq_range.start) as f32 / freq_resolution) as usize).max(1),
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
            let end = ((weight / MAX_HUMAN_FREQUENCY as f32) * amount_bins as f32).ceil() as usize;

            let new_fft_range = prev_fft_range.end..end;

            let is_supporting_point = new_fft_range != prev_fft_range && !new_fft_range.is_empty();
            if is_supporting_point {
                supporting_points.push(SupportingPoint { x: bar_idx, y: 0. });

                debug_assert!(!new_fft_range.is_empty());
                supporting_point_fft_ranges.push(new_fft_range.clone());
            }

            prev_fft_range = new_fft_range;
        }

        // It could happen that we don't have enough supporting points yet to have the given amount of bars set in `config.amount_bars`.
        // So just add a supporting point in the end.
        {
            let last_x = supporting_points.last().unwrap().x + 1;
            let not_enough_supporting_points = last_x < config.amount_bars.get() as usize;
            if not_enough_supporting_points {
                supporting_points.push(SupportingPoint {
                    x: (config.amount_bars.get() - 1) as usize,
                    y: 0.,
                });
            }

            assert!(
                supporting_points.last().unwrap().x == (config.amount_bars.get() - 1) as usize,
                "The supporting points from '{:?}' to '{:?}' don't cover '{}' bars <.<",
                supporting_points.first().unwrap(),
                supporting_points.last().unwrap(),
                config.amount_bars.get()
            );
        }

        // re-adjust the supporting points if needed
        {
            match config.bar_distribution {
                BarDistribution::Uniform => {
                    // apply distribution
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
        }

        // apply padding
        if let Some(padding) = &config.padding {
            #[allow(clippy::infallible_destructuring_match)]
            let padding_size = match padding.size {
                PaddingSize::Custom(amount) => amount,
            };

            assert!(
                config.amount_bars.checked_add(padding_size.get()).is_some(),
                "The current amount of bars + padding bars are exceeding the maximum amount of bars.\nEither reduce the amount of bars ro reduce the padding size."
            );

            let requires_left_padding =
                [PaddingSide::Left, PaddingSide::Both].contains(&padding.side);

            if requires_left_padding {
                let mut padded_supporting_points = Vec::with_capacity(supporting_points.len() + 1);
                padded_supporting_points.push(SupportingPoint { x: 0, y: 0. });

                for mut sp in supporting_points {
                    sp.x += padding_size.get() as usize;
                    padded_supporting_points.push(sp);
                }

                supporting_points = padded_supporting_points;
            }

            let requires_right_padding =
                [PaddingSide::Both, PaddingSide::Right].contains(&padding.side);

            if requires_right_padding {
                let last_sp = supporting_points.last().unwrap();

                supporting_points.push(SupportingPoint {
                    x: last_sp.x + padding_size.get() as usize,
                    y: 0.,
                });
            }
        }

        (supporting_points, supporting_point_fft_ranges)
    };

    (
        supporting_points,
        supporting_point_fft_ranges.into_boxed_slice(),
    )
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
