use std::sync::{Arc, Mutex};

use super::{Fetcher, SampleBuffer};
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioNode, AnalyserNode};

/// Fetcher for WebAudio API in WASM environments.
///
/// Uses an `AnalyserNode` to pull time-domain audio data each frame.
/// The caller is responsible for calling [`update`](Self::update) on each
/// animation frame to feed new samples into the shared `SampleBuffer`.
pub struct WebAudioFetcher {
    sample_buffer: Arc<Mutex<SampleBuffer>>,
    analyser: AnalyserNode,
    channels: u16,
    time_data: Vec<f32>,
}

impl WebAudioFetcher {
    /// Create a new `WebAudioFetcher`.
    ///
    /// * `audio_context` – the `AudioContext` to create the analyser from.
    /// * `source_node` – an `AudioNode` (e.g. `MediaStreamAudioSourceNode`) to
    ///   connect as input.
    /// * `channels` – number of audio channels to report.
    pub fn new(
        audio_context: &AudioContext,
        source_node: &AudioNode,
        channels: u16,
    ) -> Result<Self, JsValue> {
        let analyser = audio_context.create_analyser()?;
        analyser.set_fft_size(2048);
        source_node.connect_with_audio_node(&analyser)?;

        let buffer_len = analyser.frequency_bin_count() as usize;
        let time_data = vec![0.0f32; buffer_len];

        let sample_rate = audio_context.sample_rate() as u32;
        let sample_buffer = Arc::new(Mutex::new(SampleBuffer::new(sample_rate)));

        Ok(Self {
            sample_buffer,
            analyser,
            channels,
            time_data,
        })
    }

    /// Pull new audio data from the `AnalyserNode` into the shared buffer.
    ///
    /// Call this once per animation frame.
    pub fn update(&mut self) {
        self.analyser
            .get_float_time_domain_data(&mut self.time_data);
        if let Ok(mut buf) = self.sample_buffer.lock() {
            buf.push_before(&self.time_data);
        }
    }
}

impl Fetcher for WebAudioFetcher {
    fn sample_buffer(&self) -> Arc<Mutex<SampleBuffer>> {
        self.sample_buffer.clone()
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}
