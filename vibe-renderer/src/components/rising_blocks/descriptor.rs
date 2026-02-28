use crate::Renderer;
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig, SampleProcessor};

pub struct RisingBlocksDescriptor<'a, F: Fetcher> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: BarProcessorConfig,
    pub format: wgpu::TextureFormat,

    /// The canvas height.
    ///
    /// `0`: Well... zero height...
    /// `1`: The full screen height
    pub canvas_height: f32,
}
