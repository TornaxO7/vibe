use cgmath::Deg;

use crate::components::Rgba;

pub struct CircleDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a shady_audio::SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,
    pub variant: CircleVariant,

    pub radius: f32,
    pub rotation: Deg<f32>,
    // (0, 0) is top left
    pub position: (f32, f32),
}

pub enum CircleVariant {
    Graph { spike_sensitivity: f32, color: Rgba },
}
