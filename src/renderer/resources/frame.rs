use std::fmt;

use crate::{template::TemplateGenerator, ShadyDescriptor};

use super::Resource;

pub struct Frame {
    value: u32,

    buffer: wgpu::Buffer,
}

impl Frame {
    pub fn inc(&mut self) {
        (self.value, _) = self.value.overflowing_add(1)
    }
}

impl Resource for Frame {
    fn new(desc: &ShadyDescriptor) -> Self {
        let buffer = Self::create_uniform_buffer(desc.device, std::mem::size_of::<u32>() as u64);

        Self { value: 0, buffer }
    }

    fn binding() -> u32 {
        super::BindingValue::Frame as u32
    }

    fn buffer_label() -> &'static str {
        "Shady iFrame buffer"
    }

    fn buffer_type() -> wgpu::BufferBindingType {
        wgpu::BufferBindingType::Uniform
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(self.buffer(), 0, &self.value.to_ne_bytes());
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl TemplateGenerator for Frame {
    fn write_wgsl_template(
        writer: &mut dyn std::fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
@group({}) @binding({})
var<uniform> iFrame: u32;
",
            bind_group_index,
            Self::binding()
        ))
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        writer.write_fmt(format_args!(
            "
layout(binding = {}) uniform uint iFrame;
",
            Self::binding()
        ))
    }
}
