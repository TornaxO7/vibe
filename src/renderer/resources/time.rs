use std::{fmt, time::Instant};

use super::{Resource, ResourceInstantiator, TemplateGenerator};

const BINDING: u32 = super::BindingValue::Time as u32;
const BUFFER_TYPE: wgpu::BufferBindingType = wgpu::BufferBindingType::Uniform;

#[derive(Debug)]
pub struct Time {
    time: Instant,

    buffer: wgpu::Buffer,
}

impl ResourceInstantiator for Time {
    fn new(device: &wgpu::Device) -> Self {
        let buffer = super::create_uniform_buffer(
            "Shady iTime buffer",
            device,
            std::mem::size_of::<f32>() as u64,
        );

        Self {
            time: Instant::now(),
            buffer,
        }
    }
}

impl Resource for Time {
    fn update_buffer(&self, queue: &wgpu::Queue) {
        let elapsed_time = self.time.elapsed().as_secs_f32();
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[elapsed_time]));
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    fn binding(&self) -> u32 {
        BINDING
    }

    fn buffer_type(&self) -> wgpu::BufferBindingType {
        BUFFER_TYPE
    }
}

impl TemplateGenerator for Time {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
@group({}) @binding({})
var<uniform> iTime: f32;
",
            bind_group_index, BINDING,
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
layout(binding = {}) uniform float iTime;
",
            BINDING
        ))
    }
}
