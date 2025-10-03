use wgpu::include_wgsl;

pub struct NMSDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub edge_infos: wgpu::Texture,
    pub dst: wgpu::TextureView,
}

pub fn apply(desc: NMSDescriptor) {
    let NMSDescriptor {
        device,
        queue,
        edge_infos,
        dst,
    } = desc;

    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("NMS: Compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("NMS: Bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &edge_infos.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&dst),
            },
        ],
    });

    super::start_computing("NMS", device, dst.texture(), queue, pipeline, bind_group);
}
