use std::num::NonZero;

use cgmath::Deg;
use vibe_audio::fetcher::Fetcher;

use crate::components::Rgba;

pub struct GraphDescriptor<'a, F: Fetcher> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a vibe_audio::SampleProcessor<F>,
    pub audio_conf: vibe_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,

    pub variant: GraphVariant,
    pub max_height: f32,
    pub placement: GraphPlacement,
}

#[derive(Debug, Clone, Copy)]
pub enum GraphPlacement {
    Bottom,
    Top,
    Right,
    Left,
    Custom {
        // Convention:
        //   (0, 0) => top left corner
        //   (1., 1.) => bottom right corner
        offset: [f32; 2],
        rotation: Deg<f32>,
        // aka: width. This will override the amount bars of the given `audio_conf.amount_bars` of the descriptor
        amount_bars: NonZero<u16>,
    },
}

#[derive(Debug, Clone)]
pub enum GraphVariant {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}
