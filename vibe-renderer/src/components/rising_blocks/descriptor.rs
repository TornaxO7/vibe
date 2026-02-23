use vibe_audio::{fetcher::Fetcher, BarProcessorConfig, SampleProcessor};

use crate::Renderer;

pub struct RisingBlocksDescriptor<'a, F: Fetcher> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: BarProcessorConfig,
    pub format: wgpu::TextureFormat,
}
