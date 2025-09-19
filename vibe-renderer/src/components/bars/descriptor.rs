use cgmath::Deg;
use vibe_audio::{fetcher::Fetcher, SampleProcessor};

use crate::components::{Rgba, ShaderCode};

pub struct BarsDescriptor<'a, F: Fetcher> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: vibe_audio::BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    pub variant: BarVariant,
    pub max_height: f32,

    pub placement: BarsPlacement,
    pub format: BarsFormat,
}

#[derive(Debug, Clone)]
pub enum BarsPlacement {
    Custom {
        // Convention:
        // - (0., 0.) is the top left corner
        // - (1., 1.) is the bottom right corner
        bottom_left_corner: (f32, f32),
        // percentage of the screen width (so it should be within the range [0, 1])
        width_factor: f32,
        rotation: Deg<f32>,
        height_mirrored: bool,
    },
    Bottom,
    Top,
    Right,
    Left,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BarsFormat {
    #[default]
    BassTreble,
    TrebleBass,
    TrebleBassTreble,
    BassTrebleBass,
}

#[derive(Debug, Clone)]
pub enum BarVariant {
    Color(Rgba),
    PresenceGradient { high: Rgba, low: Rgba },
    FragmentCode(ShaderCode),
}
