use wgpu::include_wgsl;

pub struct EdgeTrackingDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,

    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,

    pub iterations: usize,
}

pub fn apply(desc: EdgeTrackingDescriptor) {
    let EdgeTrackingDescriptor {
        device,
        queue,
        src,
        dst,
        iterations,
    } = desc;

    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Edge tracking: Compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Edge tracking: Bind group (src, dst)"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&dst),
            },
        ],
    });

    let bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Edge tracking: Bind group (dst, src)"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&dst),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&src),
            },
        ],
    });

    for _ in 0..(iterations / 2) {
        super::start_computing(
            "Edge tracking (src => dst)",
            device,
            &dst.texture(),
            queue,
            &pipeline,
            &bind_group1,
        );

        super::start_computing(
            "Edge tracking (dst => src)",
            device,
            &src.texture(),
            queue,
            &pipeline,
            &bind_group2,
        );
    }

    super::start_computing(
        "Edge tracking (src => dst)",
        device,
        &dst.texture(),
        queue,
        &pipeline,
        &bind_group1,
    );
}
