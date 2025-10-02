use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::include_wgsl;

use crate::texture_generation::edge_distance_map::EdgeDistanceMapStep;

pub struct EdgeTrackingDescriptor<'a> {
    pub device: &'a wgpu::Device,

    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,

    pub iterations: u32,
}

pub struct EdgeTracking {
    pipeline: wgpu::ComputePipeline,
    src_to_dst_bind_group: wgpu::BindGroup,
    dst_to_src_bind_group: wgpu::BindGroup,

    iterations: u32,
}

impl EdgeTracking {
    pub fn step(desc: EdgeTrackingDescriptor) -> Box<dyn EdgeDistanceMapStep> {
        let EdgeTrackingDescriptor {
            device,
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

        let src_to_dst_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let dst_to_src_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        Box::new(Self {
            pipeline,
            src_to_dst_bind_group,
            dst_to_src_bind_group,
            iterations,
        })
    }
}

impl EdgeDistanceMapStep for EdgeTracking {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();
        for _ in 0..(self.iterations / 2) {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                pass.set_pipeline(&self.pipeline);

                // src => dst (= 1 iteration)
                pass.set_bind_group(0, &self.src_to_dst_bind_group, &[]);
                pass.dispatch_workgroups(x, y, 1);
                span.pb_inc(1);

                // dst => src (= 1 iteration)
                pass.set_bind_group(0, &self.dst_to_src_bind_group, &[]);
                pass.dispatch_workgroups(x, y, 1);
                span.pb_inc(1);
            }

            queue.submit([encoder.finish()]);
        }

        // last step to save to `dst`
        {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.src_to_dst_bind_group, &[]);
                pass.dispatch_workgroups(x, y, 1);
                span.pb_inc(1);
            }

            queue.submit([encoder.finish()]);
        }
    }

    fn amount_steps(&self) -> u32 {
        (self.iterations / 2) * 2 + 1
    }
}
