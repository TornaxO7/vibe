use cgmath::Deg;
use shady_audio::SampleProcessor;

use crate::components::Rgba;

pub struct RadialDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub processor: &'a SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,
    pub variant: RadialVariant,

    pub init_rotation: Deg<f32>,
    pub circle_radius: f32,
    pub bar_height_sensitivity: f32,
    pub bar_width: f32,
    // [0, 0]: top left corner
    // [1, 1]: bottom right corner
    pub position: (f32, f32),
}

pub enum RadialVariant {
    Color(Rgba),
}
