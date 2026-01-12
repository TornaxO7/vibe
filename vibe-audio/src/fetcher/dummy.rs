use std::sync::{Arc, Mutex};

use super::{Fetcher, SampleBuffer};

/// A dummy fetcher which does... nothing.
/// Mainly used for docs and tests.
pub struct DummyFetcher {
    sample_buffer: Arc<Mutex<SampleBuffer>>,

    amount_channels: u16,
}

impl DummyFetcher {
    /// Creates a new instance of this struct.
    pub fn new(amount_channels: u16) -> Self {
        Self {
            sample_buffer: Arc::new(Mutex::new(SampleBuffer::new(44_100))),
            amount_channels,
        }
    }
}

impl Fetcher for DummyFetcher {
    fn sample_buffer(&self) -> Arc<Mutex<SampleBuffer>> {
        self.sample_buffer.clone()
    }

    fn channels(&self) -> u16 {
        self.amount_channels
    }
}
