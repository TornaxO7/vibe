use wgpu::include_wgsl;

pub struct ComputeDistanceMapDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
    pub iterations: u32,
}

pub fn apply(desc: ComputeDistanceMapDescriptor) {
    let ComputeDistanceMapDescriptor {
        device,
        queue,
        src,
        dst,
        iterations,
    } = desc;

    let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

    // init field
    {
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Distance map: Init pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("init_map"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Distance map: Bind group"),
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

        super::start_computing(
            "Distance map (init mapping)",
            device,
            &dst.texture(),
            queue,
            &pipeline,
            &bind_group,
        );
    }

    // start updating the fields
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Distance map: Update dist"),
        layout: None,
        module: &shader,
        entry_point: Some("update_dist"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Distance map: Bind group (src => dst)"),
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
        label: Some("Distance map: Bind group (dst => src)"),
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
            "Distance map (src => dst)",
            device,
            &dst.texture(),
            queue,
            &pipeline,
            &bind_group1,
        );

        super::start_computing(
            "Distance map (dst => src)",
            device,
            &src.texture(),
            queue,
            &pipeline,
            &bind_group2,
        );
    }

    super::start_computing(
        "Distance map (src => dst)",
        device,
        &dst.texture(),
        queue,
        &pipeline,
        &bind_group1,
    );
}
