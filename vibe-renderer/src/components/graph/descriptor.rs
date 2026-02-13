use std::num::NonZero;

use cgmath::Deg;
use vibe_audio::fetcher::Fetcher;

use crate::{components::Rgba, Renderer};

pub struct GraphDescriptor<'a, F: Fetcher> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a vibe_audio::SampleProcessor<F>,
    // NOTE: Maybe it's better to create a custom struct for the audio config
    // and remove the `amount_bars` from `audio_conf` since we only need it,
    // if the placement is `GraphPlacement::Custom` and we are
    // ignoring `audio_conf.amount_bars` anyhow.
    pub audio_conf: vibe_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,

    pub variant: GraphVariant,

    // relative screen height
    pub max_height: f32,
    pub placement: GraphPlacement,
    pub format: GraphFormat,
    pub border: Option<GraphBorder>,
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
        bottom_left_corner: [f32; 2],
        rotation: Deg<f32>,
        amount_bars: NonZero<u16>,
    },
}

#[derive(Debug, Clone)]
pub enum GraphVariant {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphFormat {
    BassTreble,
    TrebleBass,
    BassTrebleBass,
    TrebleBassTreble,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphBorder {
    pub width: f32,
    pub color: Rgba,
}
