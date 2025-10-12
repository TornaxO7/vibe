use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::include_wgsl;

use crate::texture_generation::edge_distance_map::EdgeDistanceMapStep;

pub struct ComputeDistanceMapDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub src: wgpu::TextureView,
    pub dst: wgpu::TextureView,
    pub iterations: u32,
}

pub struct ComputeDistanceMap {
    init_pipeline: wgpu::ComputePipeline,
    init_bind_group: wgpu::BindGroup,

    update_pipeline: wgpu::ComputePipeline,
    src_to_dst_bind_group: wgpu::BindGroup,
    dst_to_src_bind_group: wgpu::BindGroup,

    iterations: u32,
}

impl ComputeDistanceMap {
    pub fn step(desc: ComputeDistanceMapDescriptor) -> Box<dyn EdgeDistanceMapStep> {
        let ComputeDistanceMapDescriptor {
            device,
            src,
            dst,
            iterations,
        } = desc;

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        // init fields
        let init_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Distance map: Init pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("init_map"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let init_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Distance map: Bind group"),
            layout: &init_pipeline.get_bind_group_layout(0),
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

        // start updating the fields

        let update_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Distance map: Update dist"),
            layout: None,
            module: &shader,
            entry_point: Some("update_dist"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let src_to_dst_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Distance map: Bind group (src => dst)"),
            layout: &update_pipeline.get_bind_group_layout(0),
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
            label: Some("Distance map: Bind group (dst => src)"),
            layout: &update_pipeline.get_bind_group_layout(0),
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
            init_pipeline,
            init_bind_group,

            update_pipeline,

            src_to_dst_bind_group,
            dst_to_src_bind_group,

            iterations,
        })
    }
}

impl EdgeDistanceMapStep for ComputeDistanceMap {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        let span = Span::current();
        // init fields first
        {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

                pass.set_pipeline(&self.init_pipeline);
                pass.set_bind_group(0, &self.init_bind_group, &[]);
                pass.dispatch_workgroups(x, y, 1);
                span.pb_inc(1);
            }

            queue.submit(std::iter::once(encoder.finish()));
        }

        // now update the distances for each step
        {
            for _ in 0..self.iterations.div_ceil(2) {
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let mut pass =
                        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

                    pass.set_pipeline(&self.update_pipeline);

                    // dst => src (= 1 iteration)
                    pass.set_bind_group(0, &self.dst_to_src_bind_group, &[]);
                    pass.dispatch_workgroups(x, y, 1);
                    span.pb_inc(1);

                    // src => dst (= 1 iteration)
                    pass.set_bind_group(0, &self.src_to_dst_bind_group, &[]);
                    pass.dispatch_workgroups(x, y, 1);
                    span.pb_inc(1);
                }

                queue.submit(std::iter::once(encoder.finish()));
            }
        }
    }

    fn amount_steps(&self) -> u32 {
        1 + self.iterations.div_ceil(2) * 2
    }
}
