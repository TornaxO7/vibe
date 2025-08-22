use vibe_audio::{fetcher::Fetcher, BarProcessorConfig, SampleProcessor};

use crate::{components::SdfPattern, Renderer};

pub struct ChessyDescriptor<'a, F: Fetcher> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_config: BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,

    pub movement_speed: f32,
    pub pattern: SdfPattern,
    pub zoom_factor: f32,
}
