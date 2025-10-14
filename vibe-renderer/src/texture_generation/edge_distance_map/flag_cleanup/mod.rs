use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::include_wgsl;

use crate::texture_generation::TextureGeneratorStep;

pub struct FlagCleanupDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
}

pub struct FlagCleanup {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

impl FlagCleanup {
    pub fn step(desc: FlagCleanupDescriptor) -> Box<dyn TextureGeneratorStep> {
        let FlagCleanupDescriptor { device, src, dst } = desc;

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

        Box::new(Self {
            pipeline,
            bind_group,
        })
    }
}

impl TextureGeneratorStep for FlagCleanup {
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

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn amount_steps(&self) -> u32 {
        1
    }
}
