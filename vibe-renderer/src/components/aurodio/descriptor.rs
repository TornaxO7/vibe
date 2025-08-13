use std::{num::NonZero, ops::Range};

use shady_audio::SampleProcessor;

use crate::{components::Rgb, Renderer};

pub struct AurodioLayerDescriptor {
    pub freq_range: Range<NonZero<u16>>,
    pub zoom_factor: f32,
}

pub struct AurodioDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor,
    pub texture_format: wgpu::TextureFormat,

    pub base_color: Rgb,
    // should be very low (recommended: 0.001)
    pub movement_speed: f32,

    // audio config
    pub layers: &'a [AurodioLayerDescriptor],
    pub sensitivity: f32,
}
