use wgpu::include_wgsl;

pub struct EdgeDetectionDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub src: wgpu::TextureView,
}

pub fn apply(desc: EdgeDetectionDescriptor) -> wgpu::Texture {
    let EdgeDetectionDescriptor { device, queue, src } = desc;

    let src_texture = src.texture();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Edge information texture"),
        size: src_texture.size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: src_texture.dimension(),
        format: wgpu::TextureFormat::Rg16Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Edge detection: Pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Edge detection: Bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    });

    super::start_computing(
        "Edge detection",
        device,
        &texture,
        queue,
        &pipeline,
        &bind_group,
    );

    texture
}
