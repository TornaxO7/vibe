use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::include_wgsl;

use crate::texture_generation::edge_distance_map::{
    edge_detection::{EdgeDetection, EdgeDetectionDescriptor},
    EdgeDistanceMapStep,
};

pub struct NmsDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
}

pub struct Nms {
    edge_detection: EdgeDetection,

    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

impl Nms {
    pub fn step(desc: NmsDescriptor) -> Box<dyn EdgeDistanceMapStep> {
        let NmsDescriptor { device, src, dst } = desc;

        let edge_detection = EdgeDetection::step(EdgeDetectionDescriptor { device, src });

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
                    resource: wgpu::BindingResource::TextureView(&edge_detection.info_texture()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst),
                },
            ],
        });

        Box::new(Self {
            edge_detection,
            pipeline,
            bind_group,
        })
    }
}

impl EdgeDistanceMapStep for Nms {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            // detect edges first...
            self.edge_detection.prepare_pass(&mut pass);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(self.edge_detection.amount_steps() as u64);

            // ... then apply NMS
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(x, y, 1);
            span.pb_inc(1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn amount_steps(&self) -> u32 {
        self.edge_detection.amount_steps() + 1
    }
}
