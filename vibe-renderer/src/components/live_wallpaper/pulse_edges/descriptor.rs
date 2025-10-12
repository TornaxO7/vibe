use std::{num::NonZero, ops::Range};
use vibe_audio::{fetcher::Fetcher, SampleProcessor};

pub struct PulseEdgesDescriptor<'a, F: Fetcher> {
    pub renderer: &'a crate::Renderer,
    pub sample_processor: &'a SampleProcessor<F>,
    pub texture_format: wgpu::TextureFormat,

    pub img: image::DynamicImage,
    pub freq_range: Range<NonZero<u16>>,
    pub audio_sensitivity: f32,

    pub high_threshold_ratio: f32,
    pub low_threshold_ratio: f32,
    pub wallpaper_brightness: f32,
    pub edge_width: f32,
    pub pulse_brightness: f32,

    pub sigma: f32,
    pub kernel_size: usize,
}
