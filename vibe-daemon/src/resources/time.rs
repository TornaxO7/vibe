use std::time::Instant;

use super::Resource;

#[derive(Debug)]
pub struct Time {
    time: Instant,

    buffer: wgpu::Buffer,
    binding: u32,
}

impl Time {
    pub fn new(device: &wgpu::Device, binding: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Time buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            time: Instant::now(),
            buffer,
            binding,
        }
    }
}

impl Resource for Time {
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
        let value = self.time.elapsed().as_secs_f32();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[value]));
    }
}

// impl ResourceBuffer for Time {
//     fn update_buffer(&self, queue: &wgpu::Queue) {
//         let elapsed_time = self.time.elapsed().as_secs_f32();
//         queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[elapsed_time]));
//     }

//     fn buffer(&self) -> &wgpu::Buffer {
//         &self.buffer
//     }

//     fn buffer_type(&self) -> wgpu::BufferBindingType {
//         BUFFER_TYPE
//     }
// }

// impl TemplateGenerator for Time {
//     fn write_wgsl_template(
//         writer: &mut dyn std::fmt::Write,
//         bind_group_index: u32,
//     ) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// @group({}) @binding({})
// var<uniform> iTime: f32;
// ",
//             bind_group_index, BINDING,
//         ))
//     }

//     fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
//         writer.write_fmt(format_args!(
//             "
// layout(binding = {}) uniform float iTime;
// ",
//             BINDING
//         ))
//     }
// }
