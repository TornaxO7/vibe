use crate::{components::Rgba, Renderer};
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

    // TODO: This should be in a `BlocksVariant`
    pub spawn_random: bool,
    pub speed: f32,
    pub easing: Option<RisingBlocksEasing>,
    pub beat_threshold: f32,
    pub background: RisingBlocksBackground,
    pub foreground: RisingBlocksForeground,
}

#[derive(Debug, Clone, Copy)]
pub enum RisingBlocksEasing {
    InSine,
    OutSine,
    InOutSine,

    InCubic,
    OutCubic,
    InOutCubic,
}

impl RisingBlocksEasing {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InSine => "in_sine",
            Self::OutSine => "out_sine",
            Self::InOutSine => "in_out_sine",

            Self::InCubic => "in_cubic",
            Self::OutCubic => "out_cubic",
            Self::InOutCubic => "in_out_cubic",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RisingBlocksBackground {
    Color(Rgba),
}

#[derive(Debug, Clone)]
pub enum RisingBlocksForeground {
    Color(Rgba),
}
