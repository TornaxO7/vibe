use crate::{
    components::{rising_blocks::RisingBlocksEasing, Rgba},
    Renderer,
};
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig, SampleProcessor};

pub struct BlocksDescriptor<'a, F: Fetcher> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: BarProcessorConfig,
    pub format: wgpu::TextureFormat,

    /// The canvas height.
    ///
    /// `0`: Well... zero height...
    /// `1`: The full screen height
    pub canvas_height: f32,
    pub spawn_random: bool,
    pub speed: f32,
    pub easing: Option<RisingBlocksEasing>,

    /// The threshold when a bar value is detected as a beat.
    /// Needs to be within the range [0, 1].
    pub beat_threshold: f32,

    pub color: BlocksColor,
}

pub enum BlocksColor {
    Color(Rgba),
}
