use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::texture_generation::TextureGeneratorStep;

const LABEL: &str = "Light threshold";

pub struct LightThresholdDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,

    pub threshold: f32,
}

pub struct LightThreshold {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

impl LightThreshold {
    pub fn step(desc: LightThresholdDescriptor) -> Box<dyn TextureGeneratorStep> {
        assert!(0. <= desc.threshold && desc.threshold <= 1.);

        let LightThresholdDescriptor {
            device,
            src,
            dst,
            threshold,
        } = desc;

        let threshold_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light source: Threshold buffer"),
            contents: bytemuck::bytes_of(&threshold),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pipeline = {
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(LABEL),
                layout: None,
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: threshold_buffer.as_entire_binding(),
                },
            ],
        });

        Box::new(Self {
            pipeline,
            bind_group,
        })
    }
}

impl TextureGeneratorStep for LightThreshold {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);
        }

        queue.submit([encoder.finish()]);
    }

    fn amount_steps(&self) -> u32 {
        1
    }
}
