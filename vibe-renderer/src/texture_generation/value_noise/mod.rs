use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::TextureGenerator;

const WORKGROUP_SIZE: u32 = 16;

pub struct ValueNoise {
    pub texture_size: u32,
    pub octaves: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DataBinding {
    octaves: u32,
    seed: f32,
    canvas_size: f32,
}

impl TextureGenerator for ValueNoise {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Value noise: Texture"),
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
            label: Some("Value noise: Data buffer"),
            contents: bytemuck::bytes_of(&DataBinding {
                octaves: self.octaves,
                canvas_size: self.texture_size as f32,
                // range: [15, 35]
                seed: fastrand::f32() * 20. + 15.,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pipeline = {
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Value noise: Pipeline"),
                layout: None,
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Value noise: Bind group"),
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
            label: Some("Value noise: Command encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Value noise: Compute pass"),
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

    #[test]
    fn general() {
        let renderer = Renderer::default();

        renderer.generate(ValueNoise {
            texture_size: 50,
            octaves: 7,
        });
    }
}
