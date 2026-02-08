use std::sync::{Arc, Mutex};

use vibe_audio::fetcher::{Fetcher, SampleBuffer};

pub struct TestFetcher {
    buffer: Arc<Mutex<SampleBuffer>>,
}

impl TestFetcher {
    pub fn new() -> Self {
        let sample_snapshot = {
            let snapshot = include_str!("./audio_samples.txt");
            let mut buffer: Vec<f32> = Vec::new();

            for sample in snapshot.split(',') {
                let sample_float: f32 = sample.trim().parse().unwrap();
                buffer.push(sample_float);
            }

            buffer
        };

        let mut buffer = SampleBuffer::new(vibe_audio::DEFAULT_SAMPLE_RATE);
        buffer.push_before(&sample_snapshot);

        Self {
            buffer: Arc::new(Mutex::new(buffer)),
        }
    }
}

impl Fetcher for TestFetcher {
    fn sample_buffer(&self) -> Arc<Mutex<SampleBuffer>> {
        self.buffer.clone()
    }

    fn channels(&self) -> u16 {
        2
    }
}
