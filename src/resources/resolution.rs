use crate::types::size::Size;

use super::Resource;

#[derive(Debug)]
pub struct Resolution {
    width: u32,
    height: u32,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Resolution {
    pub fn new(device: &wgpu::Device, binding: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shady iResolution buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            width: 0,
            height: 0,
            buffer,
            binding,
        }
    }

    pub fn set(&mut self, size: Size) {
        let Size { width, height } = size;

        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }
}

impl Resource for Resolution {
    fn bind_group_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: self.binding,
            resource: wgpu::BindingResource::Buffer(self.buffer.as_entire_buffer_binding()),
        }
    }

    fn update_buffer(&self, queue: &wgpu::Queue) {
        let data = [self.width as f32, self.height as f32];
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&data));
    }
}

// impl TemplateGenerator for Resolution {
//     fn write_wgsl_template(
//         writer: &mut dyn std::fmt::Write,
//         bind_group_index: u32,
//     ) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// // x: width
// // y: height
// @group({}) @binding({})
// var<uniform> iResolution: vec2<f32>;
// ",
//             bind_group_index, BINDING
//         ))
//     }

//     fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// // x: width
// // y: height
// layout(binding = {}) uniform vec2 iResolution;
// ",
//             BINDING
//         ))
//     }
// }
