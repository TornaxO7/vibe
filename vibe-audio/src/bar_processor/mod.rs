mod channel_ctx;
mod config;

use crate::{fetcher::Fetcher, SampleProcessor};
use channel_ctx::ChannelCtx;
use cpal::SampleRate;
use std::num::NonZero;

pub use config::{BarProcessorConfig, InterpolationVariant};

/// The struct which computes the bar values of the samples of the fetcher.
pub struct BarProcessor {
    // The final bar values.
    //
    // Mapping:
    // 1st Index: Channel
    // 2nd index: Bar value
    bar_values: Box<[Box<[f32]>]>,

    // channels[i] = contains the interpolation context for the i-th channel
    channels: Box<[ChannelCtx]>,

    config: BarProcessorConfig,
    sample_rate: SampleRate,
    sample_len: usize,
}

impl BarProcessor {
    /// Creates a new instance.
    ///
    /// See the examples of this crate to see it's usage.
    pub fn new<F: Fetcher>(processor: &SampleProcessor<F>, config: BarProcessorConfig) -> Self {
        let sample_rate = processor.sample_rate();
        let sample_len = processor.fft_size();
        let amount_channels = processor.amount_channels();

        let (channels, bar_values) =
            Self::get_channels_and_bar_values(&config, amount_channels, sample_rate, sample_len);

        Self {
            config,
            channels,
            bar_values,

            sample_rate,
            sample_len,
        }
    }

    /// Returns the bar values for each channel.
    ///
    /// If you access the returned value like this: `bar_processor.process_bars(&processor)[i][j]` then this would mean:
    /// You are accessing the `j`th bar value of the `i`th audio channel.
    pub fn process_bars<F: Fetcher>(&mut self, processor: &SampleProcessor<F>) -> &[Box<[f32]>] {
        for ((channel_idx, channel), fft_ctx) in self
            .channels
            .iter_mut()
            .enumerate()
            .zip(processor.fft_out().iter())
        {
            channel.update_supporting_points(&fft_ctx.fft_out);
            channel.interpolate(&mut self.bar_values[channel_idx]);
        }

        &self.bar_values
    }

    pub fn config(&self) -> &BarProcessorConfig {
        &self.config
    }

    /// Change the amount of bars which should be returned.
    ///
    /// # Example
    /// ```rust
    /// use vibe_audio::{SampleProcessor, BarProcessor, BarProcessorConfig, fetcher::DummyFetcher};
    ///
    /// let mut sample_processor = SampleProcessor::new(DummyFetcher::new(1));
    /// let mut bar_processor = BarProcessor::new(
    ///     &sample_processor,
    ///     BarProcessorConfig {
    ///         amount_bars: std::num::NonZero::new(10).unwrap(),
    ///         ..Default::default()
    ///     }
    /// );
    /// sample_processor.process_next_samples();
    ///
    /// let bars = bar_processor.process_bars(&sample_processor);
    /// // the dummy just has one channel
    /// assert_eq!(bars.len(), 1);
    /// // but it should have ten bars
    /// assert_eq!(bars[0].len(), 10);
    ///
    /// // change the amount of bars
    /// bar_processor.set_amount_bars(std::num::NonZero::new(20).unwrap());
    /// let bars = bar_processor.process_bars(&sample_processor);
    /// assert_eq!(bars.len(), 1);
    /// assert_eq!(bars[0].len(), 20);
    /// ```
    pub fn set_amount_bars(&mut self, amount_bars: NonZero<u16>) {
        self.config.amount_bars = amount_bars;
        let amount_channels = self.channels.len();

        let (channels, bar_values) = Self::get_channels_and_bar_values(
            &self.config,
            amount_channels,
            self.sample_rate,
            self.sample_len,
        );

        self.channels = channels;
        self.bar_values = bar_values;
    }

    fn get_channels_and_bar_values(
        config: &BarProcessorConfig,
        amount_channels: usize,
        sample_rate: SampleRate,
        sample_len: usize,
    ) -> (Box<[ChannelCtx]>, Box<[Box<[f32]>]>) {
        let mut channels = Vec::with_capacity(amount_channels);
        let bar_values =
            vec![vec![0f32; config.amount_bars.get() as usize].into_boxed_slice(); amount_channels];

        for _ in 0..amount_channels {
            channels.push(ChannelCtx::new(config, sample_rate, sample_len));
        }

        (channels.into_boxed_slice(), bar_values.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::fetcher::DummyFetcher;

    use super::*;

    #[test]
    fn one_channel_u16_max_bars() {
        let processor = SampleProcessor::new(DummyFetcher::new(1));
        let mut bar_processor = BarProcessor::new(
            &processor,
            BarProcessorConfig {
                amount_bars: NonZero::new(u16::MAX).unwrap(),
                ..Default::default()
            },
        );

        let bars = bar_processor.process_bars(&processor);
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].len(), u16::MAX as usize);
    }

    #[test]
    fn two_channels_u16_max_bars() {
        let processor = SampleProcessor::new(DummyFetcher::new(2));
        let mut bar_processor = BarProcessor::new(
            &processor,
            BarProcessorConfig {
                amount_bars: NonZero::new(u16::MAX).unwrap(),
                ..Default::default()
            },
        );

        let bars = bar_processor.process_bars(&processor);
        assert_eq!(bars.len(), 2);

        for channel in bars {
            assert_eq!(channel.len(), u16::MAX as usize);
        }
    }
}
