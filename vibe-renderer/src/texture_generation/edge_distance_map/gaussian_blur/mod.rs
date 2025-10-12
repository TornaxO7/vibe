use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::edge_distance_map::EdgeDistanceMapStep;

pub struct GaussianBlurDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,

    pub sigma: f32,
    pub kernel_size: usize,
}

pub struct GaussianBlur {
    horizontal_pipeline: wgpu::ComputePipeline,
    horizontal_bind_group: wgpu::BindGroup,

    vertical_pipeline: wgpu::ComputePipeline,
    vertical_bind_group: wgpu::BindGroup,

    _tmp_texture: wgpu::Texture,
    _kernel_buffer: wgpu::Buffer,
}

impl GaussianBlur {
    pub fn step(desc: GaussianBlurDescriptor) -> Box<dyn EdgeDistanceMapStep> {
        assert!(desc.kernel_size % 2 == 1);

        let device = desc.device;
        let dst_texture = desc.dst.texture();

        let tmp_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gaussian blur: Tmp texture"),
            size: dst_texture.size(),
            mip_level_count: dst_texture.mip_level_count(),
            sample_count: dst_texture.sample_count(),
            dimension: dst_texture.dimension(),
            format: dst_texture.format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gaussian blur: Kernel"),
            contents: bytemuck::cast_slice(&generate_kernel(desc.kernel_size, desc.sigma)),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let horizontal_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Horizontal gaussian blur: Pipeline"),
                layout: None,
                module: &shader,
                entry_point: Some("horizontal"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let horizontal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Horizontal gaussian blur: Bind group"),
            layout: &horizontal_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&desc.src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &tmp_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        let vertical_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Vertical gaussian blur: Pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("vertical"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let vertical_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Vertical gaussian blur: Bind group"),
            layout: &vertical_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &tmp_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&desc.dst),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        Box::new(Self {
            horizontal_pipeline,
            horizontal_bind_group,

            vertical_pipeline,
            vertical_bind_group,

            _tmp_texture: tmp_texture,
            _kernel_buffer: kernel_buffer,
        })
    }
}

impl EdgeDistanceMapStep for GaussianBlur {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            // horizontal first...
            pass.set_pipeline(&self.horizontal_pipeline);
            pass.set_bind_group(0, &self.horizontal_bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);

            // ... then vertical
            pass.set_pipeline(&self.vertical_pipeline);
            pass.set_bind_group(0, &self.vertical_bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn amount_steps(&self) -> u32 {
        2
    }
}

fn generate_kernel(size: usize, sigma: f32) -> Vec<f32> {
    assert!(size % 2 == 1);

    let mut kernel = Vec::with_capacity(size);

    let mut sum = 0.;
    let half_size = (size / 2) as isize;
    for x in (-half_size)..=half_size {
        let value = gauss(sigma, x as f32);
        kernel.push(value);
        sum += value;
    }

    // normamlize kernel
    for value in kernel.iter_mut() {
        *value /= sum;
    }

    kernel
}

fn gauss(sigma: f32, x: f32) -> f32 {
    (1. / (2. * std::f32::consts::PI * sigma * sigma))
        * std::f32::consts::E.powf(-(x * x) / (2. * sigma * sigma).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_size() {
        let size = 5;

        assert_eq!(generate_kernel(size, 1.0).len(), size);
    }
}
