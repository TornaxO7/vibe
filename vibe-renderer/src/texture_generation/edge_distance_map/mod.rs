use wgpu::include_wgsl;

use crate::texture_generation::TextureGenerator;

const WORKGROUP_SIZE: u32 = 16;

pub struct EdgeDistanceMap<'a> {
    pub src: &'a image::DynamicImage,
}

impl<'a> TextureGenerator for EdgeDistanceMap<'a> {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let texture1 = gray_scale_img(self.src, device, queue);
        let texture2 = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture 2"),
            size: texture1.size(),
            mip_level_count: texture1.mip_level_count(),
            sample_count: texture1.sample_count(),
            dimension: texture1.dimension(),
            format: texture1.format(),
            usage: texture1.usage(),
            view_formats: &[],
        });

        todo!()
    }
}

fn gray_scale_img(
    src: &image::DynamicImage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::Texture {
    let img_texture = {
        let src_img = src.to_rgba8();
        let (width, height) = src_img.dimensions();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image source texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 1,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            src_img.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<[u8; 4]>() as u32 * width),
                rows_per_image: Some(height),
            },
            texture.size(),
        );

        texture
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture 1"),
        size: img_texture.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R16Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./gray_scale.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gray scale: Pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Gray scale: Bind group"),
        layout: &pipeline.get_bind_group_layout(0),
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
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    });

    start_computing(
        "Gray scale",
        device,
        texture.width(),
        texture.height(),
        queue,
        pipeline,
        bind_group,
    );

    texture
}

fn start_computing(
    label_prefix: &'static str,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    queue: &wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
) {
    let command_encoder_label = format!("{}: Command encoder", label_prefix);
    let pass_label = format!("{}: Compute pass", label_prefix);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(&command_encoder_label),
    });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&pass_label),
            timestamp_writes: None,
        });

        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_pipeline(&pipeline);
        pass.dispatch_workgroups(
            width.div_ceil(WORKGROUP_SIZE),
            height.div_ceil(WORKGROUP_SIZE),
            1,
        );
    }

    queue.submit(std::iter::once(encoder.finish()));
}
