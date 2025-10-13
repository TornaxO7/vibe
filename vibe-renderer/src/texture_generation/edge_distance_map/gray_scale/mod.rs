use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use wgpu::include_wgsl;

use crate::texture_generation::edge_distance_map::EdgeDistanceMapStep;

pub struct GrayScaleDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,

    pub src: &'a image::DynamicImage,
    pub dst: wgpu::TextureView,
}

pub struct GrayScale {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,

    _img_texture: wgpu::Texture,
}

impl GrayScale {
    pub fn step(desc: GrayScaleDescriptor) -> Box<dyn EdgeDistanceMapStep> {
        let GrayScaleDescriptor {
            src,
            device,
            queue,
            dst,
        } = desc;

        let img_texture = crate::util::load_img_to_texture(device, queue, &src);

        let pipeline = {
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gray scale: Pipeline"),
                layout: None,
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gray scale: Bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &img_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
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
            _img_texture: img_texture,
        })
    }
}

impl EdgeDistanceMapStep for GrayScale {
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
