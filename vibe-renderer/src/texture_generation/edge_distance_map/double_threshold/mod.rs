use wgpu::{include_wgsl, util::DeviceExt};

pub struct DoubleThresholdDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
    pub high_threshold_ratio: f32,
    pub low_threshold_ratio: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RatiosBinding {
    upper: f32,
    lower: f32,
}

pub fn apply(desc: DoubleThresholdDescriptor) {
    let DoubleThresholdDescriptor {
        device,
        queue,
        src,
        dst,
        high_threshold_ratio,
        low_threshold_ratio,
    } = desc;

    let max_value_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("DT: Max value buffer"),
        contents: bytemuck::bytes_of(&0u32),
        usage: wgpu::BufferUsages::STORAGE,
    });

    // set `max_value_buffer`
    {
        let pipeline = {
            let shader = device.create_shader_module(include_wgsl!("./max_value.wgsl"));

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("DT: Max value pipeline"),
                layout: None,
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DT: Max value bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: max_value_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&src),
                },
            ],
        });

        super::start_computing(
            "DT: Max value",
            device,
            &src.texture(),
            queue,
            pipeline,
            bind_group,
        );
    }

    let ratios_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("DT: Ratios buffer"),
        contents: bytemuck::bytes_of(&RatiosBinding {
            upper: high_threshold_ratio,
            lower: low_threshold_ratio,
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // apply dt
    let pipeline = {
        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("DT: Compute pipeline"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    };

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("DT: Bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: max_value_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: ratios_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&src),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&dst),
            },
        ],
    });

    super::start_computing(
        "Double Threshold",
        device,
        &dst.texture(),
        queue,
        pipeline,
        bind_group,
    );
}
