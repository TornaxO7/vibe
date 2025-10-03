use wgpu::include_wgsl;

pub struct FlagCleanupDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
}

pub fn apply(desc: FlagCleanupDescriptor) {
    let FlagCleanupDescriptor {
        device,
        queue,
        src,
        dst,
    } = desc;

    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Flag cleanup: Compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Flag cleanup: Bind group"),
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
        "Flag cleanup",
        device,
        &dst.texture(),
        queue,
        &pipeline,
        &bind_group,
    );
}
