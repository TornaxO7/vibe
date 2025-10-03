use wgpu::include_wgsl;

pub struct GrayScaleDescriptor<'a> {
    pub src: &'a image::DynamicImage,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub fn apply(desc: GrayScaleDescriptor) -> wgpu::Texture {
    let GrayScaleDescriptor { src, device, queue } = desc;

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
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

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

    super::start_computing("Gray scale", device, &texture, queue, pipeline, bind_group);

    texture
}
