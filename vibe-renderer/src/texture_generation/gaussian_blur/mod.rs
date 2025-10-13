use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::TextureGenerator;

const WORKGROUP_SIZE: u32 = 16;

pub struct GaussianBlur<'a> {
    pub src: &'a image::DynamicImage,

    pub sigma: f32,
    pub kernel_size: usize,
}

impl<'a> TextureGenerator for GaussianBlur<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        assert!(self.kernel_size % 2 == 1);

        let img_texture = crate::util::load_img_to_texture(device, queue, &self.src);

        let tmp_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gaussian blur: Temp texture"),
            size: img_texture.size(),
            mip_level_count: img_texture.mip_level_count(),
            sample_count: img_texture.sample_count(),
            dimension: img_texture.dimension(),
            format: img_texture.format(),
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gaussian blur: Kernel buffer"),
            contents: bytemuck::cast_slice(&generate_kernel(self.kernel_size, self.sigma)),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        let horizontal_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gaussian blur: Horizontal pipeline"),
                layout: None,
                module: &shader,
                entry_point: Some("horizontal"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let vertical_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gaussian blur: Vertical pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("vertical"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let horizontal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gaussian blur: Horizontal bind group"),
            layout: &horizontal_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &img_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
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

        let vertical_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gaussian blur: Vertical bind group"),
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
                    resource: wgpu::BindingResource::TextureView(
                        &img_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Gaussian blur: Command encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Gaussian blur: Compute pass"),
                timestamp_writes: None,
            });

            // horizontal pass first...
            pass.set_bind_group(0, &horizontal_bind_group, &[]);
            pass.set_pipeline(&horizontal_pipeline);
            pass.dispatch_workgroups(
                tmp_texture.width().div_ceil(WORKGROUP_SIZE),
                tmp_texture.height().div_ceil(WORKGROUP_SIZE),
                1,
            );

            // ... then vertical
            pass.set_bind_group(0, &vertical_bind_group, &[]);
            pass.set_pipeline(&vertical_pipeline);
            pass.dispatch_workgroups(
                img_texture.width().div_ceil(WORKGROUP_SIZE),
                img_texture.height().div_ceil(WORKGROUP_SIZE),
                1,
            );
        }

        queue.submit(std::iter::once(encoder.finish()));

        img_texture
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
        * std::f32::consts::E.powf(-(x * x) / (2. * sigma * sigma))
}
