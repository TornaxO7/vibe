use serde::{Deserialize, Serialize};
use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::TextureGenerator;

const WORKGROUP_SIZE: u32 = 16;

pub struct SdfMask {
    pub pattern: SdfPattern,
    pub texture_size: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SdfPattern {
    Box = 0,
    Circle = 1,
    Heart = 2,
}

impl SdfPattern {
    pub fn id(&self) -> u32 {
        *self as u32
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DataBinding {
    canvas_size: f32,
    pattern: u32,
}

impl TextureGenerator for SdfMask {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SdfMask: Texture"),
            size: wgpu::Extent3d {
                width: self.texture_size,
                height: self.texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sdfmask: Data buffer"),
            contents: bytemuck::bytes_of(&DataBinding {
                canvas_size: self.texture_size as f32,
                pattern: self.pattern.id(),
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pipeline = {
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Sdfmask: Compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sdfmask: Bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: data_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Sdfmask: Command encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Sdfmask: Compute pas"),
                timestamp_writes: None,
            });

            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_pipeline(&pipeline);
            pass.dispatch_workgroups(
                self.texture_size.div_ceil(WORKGROUP_SIZE),
                self.texture_size.div_ceil(WORKGROUP_SIZE),
                1,
            );
        }

        queue.submit(std::iter::once(encoder.finish()));

        texture
    }
}

#[cfg(test)]
mod tests {
    use crate::Renderer;

    use super::*;

    fn generate(pattern: SdfPattern) {
        let renderer = Renderer::default();

        renderer.generate(&SdfMask {
            pattern,
            texture_size: 256,
        });
    }

    #[test]
    fn generate_box() {
        generate(SdfPattern::Box);
    }

    #[test]
    fn generate_circle() {
        generate(SdfPattern::Circle);
    }

    #[test]
    fn generate_heart() {
        generate(SdfPattern::Heart);
    }
}
