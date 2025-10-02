use wgpu::include_wgsl;

pub struct EdgeDetectionDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
}

pub struct EdgeDetection {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,

    info_texture: wgpu::Texture,
}

impl EdgeDetection {
    pub fn step(desc: EdgeDetectionDescriptor) -> Self {
        let EdgeDetectionDescriptor { device, src } = desc;

        let src_texture = src.texture();
        let info_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Edge information texture"),
            size: src_texture.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: src_texture.dimension(),
            format: wgpu::TextureFormat::Rg32Float,
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
                        &info_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            info_texture,
        }
    }

    pub fn info_texture(&self) -> wgpu::TextureView {
        self.info_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn prepare_pass(&self, pass: &mut wgpu::ComputePass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
    }

    pub fn amount_steps(&self) -> u32 {
        1
    }
}
