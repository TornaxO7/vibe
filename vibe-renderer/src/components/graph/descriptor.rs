use vibe_audio::fetcher::Fetcher;

use crate::components::Rgba;

pub struct GraphDescriptor<'a, F: Fetcher> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a vibe_audio::SampleProcessor<F>,
    pub audio_conf: vibe_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,

    pub variant: GraphVariant,
    pub max_height: f32,
    pub smoothness: f32,
    pub placement: GraphPlacement,
}

#[derive(Debug, Clone, Copy)]
pub enum GraphPlacement {
    Bottom,
    Top,
    Right,
    Left,
}

#[derive(Debug, Clone)]
pub enum GraphVariant {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}
