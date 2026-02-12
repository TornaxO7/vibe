//! Each struct here can be used to fetch the audio data from various sources.
//! Pick the one you need to fetch from.
mod dummy;

#[cfg(not(target_arch = "wasm32"))]
mod system_audio;

#[cfg(target_arch = "wasm32")]
mod web_audio;

use crate::SampleRate;
use std::sync::{Arc, Mutex};

pub use dummy::DummyFetcher;

#[cfg(not(target_arch = "wasm32"))]
pub use system_audio::{
    Descriptor as SystemAudioFetcherDescriptor, SystemAudio as SystemAudioFetcher, SystemAudioError,
};

#[cfg(target_arch = "wasm32")]
pub use web_audio::WebAudioFetcher;

/// Interface for all structs (fetchers) which are listed in the [fetcher module](crate::fetcher).
pub trait Fetcher {
    /// Returns the [SampleBuffer] (aka the input for the fft calculations).
    fn sample_buffer(&self) -> Arc<Mutex<SampleBuffer>>;

    /// Returns the amount of channels which are used from the fetcher.
    fn channels(&self) -> u16;
}

/// Holds the audio samples which gets filled by the fetcher
#[derive(Debug, Clone)]
pub struct SampleBuffer {
    buffer: Box<[f32]>,
    sample_rate: SampleRate,
}

impl SampleBuffer {
    /// Create a new instance for the given sample rate.
    pub fn new(sample_rate: SampleRate) -> Self {
        // props to cava for this heuristic.
        let factor = if sample_rate < 8_125 {
            1
        } else if sample_rate <= 16_250 {
            2
        } else if sample_rate <= 32_500 {
            4
        } else if sample_rate <= 75_000 {
            8
        } else if sample_rate <= 150_000 {
            16
        } else if sample_rate <= 300_000 {
            32
        } else {
            64
        };

        let buffer = vec![0f32; factor * 128].into_boxed_slice();

        Self {
            buffer,
            sample_rate,
        }
    }

    /// Pushes the given data to the front of `buffer` and moves the current data to the right.
    /// Basically a `VecDeque::push_before` just on a `Box<[f32]>`.
    pub fn push_before(&mut self, data: &[f32]) {
        let data_len = data.len();
        let buffer_len = self.buffer.len();

        // split point
        let split_point = buffer_len.min(data_len);

        // move current values to the end/right of the buffer
        self.buffer
            .copy_within(..split_point, buffer_len - split_point);

        // write the new data [at the beginning]/[on the left] of the buffer
        self.buffer[..split_point].copy_from_slice(&data[..split_point]);
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn buffer(&self) -> &[f32] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sample_buffer {
        use super::*;

        #[test]
        fn push_more_than_capacity() {
            // buffer should have length of `1` * `128`
            let mut sample_buffer = SampleBuffer::new(1);
            sample_buffer.push_before(&[1f32; 129]);

            assert_eq!(sample_buffer.buffer.len(), 128);
            assert!(sample_buffer.buffer.iter().all(|&value| value == 1f32));
        }

        #[test]
        fn new_values_are_moved_to_the_beginning() {
            let mut sample_buffer = SampleBuffer::new(1);
            sample_buffer.push_before(&[69f32]);

            assert_eq!(sample_buffer.buffer.len(), 128);
            assert_eq!(sample_buffer.buffer[0], 69f32);
            assert!(sample_buffer.buffer[1..].iter().all(|&value| value == 0f32));
        }

        #[test]
        fn no_values_pushed() {
            let mut sample_buffer = SampleBuffer::new(1);
            sample_buffer.push_before(&[]);

            assert_eq!(sample_buffer.buffer.len(), 128);
            assert!(sample_buffer.buffer.iter().all(|&value| value == 0f32));
        }
    }
}
