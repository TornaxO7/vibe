use std::{
    num::NonZero,
    sync::{Arc, Mutex},
};

use crate::SampleRate;
use realfft::{num_complex::Complex32, RealFftPlanner};

use crate::fetcher::{Fetcher, SampleBuffer};

/// Prepares the samples of the fetcher for the [crate::BarProcessor].
pub struct SampleProcessor<F: Fetcher> {
    planner: RealFftPlanner<f32>,
    hann_window: Box<[f32]>,

    // The fft context (fft output + additional buffers) per channel
    channels: Box<[FftContext]>,
    sample_buffer: Arc<Mutex<SampleBuffer>>,

    // aka fft input length
    fft_size: usize,

    // Relevant for the system audio fetcher, otherwise it will get dropped and so the stream as well.
    _fetcher: F,
}

impl<F: Fetcher> SampleProcessor<F> {
    /// Creates a new instance with the given fetcher where the audio samples are fetched from.
    pub fn new(fetcher: F) -> Self {
        let sample_buffer = fetcher.sample_buffer();

        let fft_size = {
            let fft_input = sample_buffer.lock().unwrap();
            fft_input.capacity()
        };

        let fft_out_size = fft_size / 2 + 1;

        let hann_window = apodize::hanning_iter(fft_size)
            .map(|val| val as f32)
            .collect::<Vec<f32>>()
            .into_boxed_slice();

        let channels = vec![FftContext::new(fft_size, fft_out_size); fetcher.channels() as usize]
            .into_boxed_slice();

        Self {
            planner: RealFftPlanner::new(),
            hann_window,

            channels,

            sample_buffer,
            fft_size,
            _fetcher: fetcher,
        }
    }

    /// Tell the processor to take some samples of the fetcher and prepare them
    /// for the [crate::BarProcessor]s.
    pub fn process_next_samples(&mut self) {
        let amount_channels = self.channels.len();

        // fetch the latest data
        {
            let fft_input = self.sample_buffer.lock().unwrap();

            for (sample_idx, samples) in
                fft_input.buffer().chunks_exact(amount_channels).enumerate()
            {
                for (channel_idx, channel) in self.channels.iter_mut().enumerate() {
                    channel.fft_in[sample_idx] =
                        samples[channel_idx] * self.hann_window[sample_idx];
                }
            }
        }

        let fft = self.planner.plan_fft_forward(self.fft_size);
        for channel in self.channels.iter_mut() {
            fft.process_with_scratch(
                channel.fft_in.as_mut(),
                channel.fft_out.as_mut(),
                channel.scratch_buffer.as_mut(),
            )
            .unwrap();
        }
    }
}

impl<F: Fetcher> SampleProcessor<F> {
    pub(crate) fn fft_size(&self) -> usize {
        self.fft_size
    }

    pub(crate) fn fft_out(&self) -> &[FftContext] {
        &self.channels
    }

    pub(crate) fn sample_rate(&self) -> SampleRate {
        self.sample_buffer.lock().unwrap().sample_rate()
    }

    pub(crate) fn amount_channels(&self) -> NonZero<u8> {
        NonZero::new(self.channels.len() as u8).unwrap()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FftContext {
    fft_in: Box<[f32]>,
    pub fft_out: Box<[Complex32]>,
    scratch_buffer: Box<[Complex32]>,
}

impl FftContext {
    fn new(fft_size: usize, fft_out_size: usize) -> Self {
        let fft_in = vec![0.; fft_size].into_boxed_slice();
        let fft_out = vec![Complex32::ZERO; fft_out_size].into_boxed_slice();
        let scratch_buffer = fft_out.clone();

        Self {
            fft_in,
            fft_out,
            scratch_buffer,
        }
    }
}
