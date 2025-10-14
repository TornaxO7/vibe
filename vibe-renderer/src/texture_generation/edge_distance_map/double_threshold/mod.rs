use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::TextureGeneratorStep;

pub struct DoubleThresholdDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
    pub high_threshold_ratio: f32,
    pub low_threshold_ratio: f32,
}

pub struct DoubleThreshold {
    max_value_pipeline: wgpu::ComputePipeline,
    max_value_bind_group: wgpu::BindGroup,
    _max_value_buffer: wgpu::Buffer,

    _ratios_buffer: wgpu::Buffer,

    dt_pipeline: wgpu::ComputePipeline,
    dt_bind_group: wgpu::BindGroup,
}

impl DoubleThreshold {
    pub fn step(desc: DoubleThresholdDescriptor) -> Box<dyn TextureGeneratorStep> {
        let DoubleThresholdDescriptor {
            device,
            src,
            dst,
            high_threshold_ratio,
            low_threshold_ratio,
        } = desc;

        assert!(high_threshold_ratio >= low_threshold_ratio);

        let max_value_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DT: Max value buffer"),
            contents: bytemuck::bytes_of(&0u32),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // `max_value` relevant stuff
        let max_value_pipeline = {
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

        let max_value_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DT: Max value bind group"),
            layout: &max_value_pipeline.get_bind_group_layout(0),
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

        let ratios_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DT: Ratios buffer"),
            contents: bytemuck::bytes_of(&RatiosBinding {
                upper: high_threshold_ratio,
                lower: low_threshold_ratio,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // dt (double-thresholt) relevant stuff
        let dt_pipeline = {
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

        let dt_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DT: Bind group"),
            layout: &dt_pipeline.get_bind_group_layout(0),
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

        Box::new(Self {
            max_value_pipeline,
            max_value_bind_group,
            _max_value_buffer: max_value_buffer,

            _ratios_buffer: ratios_buffer,

            dt_pipeline,
            dt_bind_group,
        })
    }
}

impl TextureGeneratorStep for DoubleThreshold {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            // find max value first...
            pass.set_pipeline(&self.max_value_pipeline);
            pass.set_bind_group(0, &self.max_value_bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);

            // ...then apply double thresholds per texel
            pass.set_pipeline(&self.dt_pipeline);
            pass.set_bind_group(0, &self.dt_bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn amount_steps(&self) -> u32 {
        2
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RatiosBinding {
    upper: f32,
    lower: f32,
}
