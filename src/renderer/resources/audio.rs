use std::fmt;

use super::{Resource, ResourceInstantiator, TemplateGenerator};

const BINDING: u32 = super::BindingValue::Audio as u32;
const BUFFER_TYPE: wgpu::BufferBindingType = wgpu::BufferBindingType::Storage { read_only: true };

const DEFAULT_AMOUNT_BARS: usize = 60;
const DESCRIPTION: &str = "\
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.";

pub struct Audio {
    bar_values: Box<[f32]>,
    buffer: wgpu::Buffer,
}

impl ResourceInstantiator for Audio {
    fn new(device: &wgpu::Device) -> Self {
        let buffer = super::create_storage_buffer(
            "Shady iAudio buffer",
            device,
            std::mem::size_of::<[f32; DEFAULT_AMOUNT_BARS]>() as u64,
        );

        let audio_buffer = Box::new([0.; DEFAULT_AMOUNT_BARS]);

        Self {
            bar_values: audio_buffer,
            buffer,
        }
    }
}

impl Resource for Audio {
    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn buffer_type(&self) -> wgpu::BufferBindingType {
        BUFFER_TYPE
    }

    fn binding(&self) -> u32 {
        BINDING
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&self.bar_values));
    }
}

impl TemplateGenerator for Audio {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
{}
@group({}) @binding({})
var<storage, read> iAudio: array<f32>;
",
            DESCRIPTION, bind_group_index, BINDING,
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
{}
layout(binding = {}) readonly buffer iAudio {{
    float[] freqs;
}};
",
            DESCRIPTION, BINDING
        ))
    }
}
