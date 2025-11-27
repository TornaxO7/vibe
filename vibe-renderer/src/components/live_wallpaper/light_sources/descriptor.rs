use std::{num::NonZero, ops::Range};

use vibe_audio::{fetcher::Fetcher, SampleProcessor};

pub struct LightSourcesDescriptor<'a, F: Fetcher> {
    pub renderer: &'a crate::Renderer,
    pub format: wgpu::TextureFormat,
    pub processor: &'a SampleProcessor<F>,

    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,

    pub wallpaper: image::DynamicImage,
    pub sources: &'a [LightSourceData],
    // if each light source should listen to *one* bar
    pub uniform_pulse: bool,
    // set if the sources should be shown as full circles for debugging
    pub debug_sources: bool,
}

pub struct LightSourceData {
    // must be within the range [0, 1] for each value
    pub center: [f32; 2],
    pub radius: f32,
}
