mod descriptor;

use std::num::NonZero;

pub use descriptor::*;
use vibe_audio::{
    fetcher::Fetcher, BarProcessor, BarProcessorConfig, NothingInterpolation, SampleProcessor,
};
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{components::ComponentAudio, Component, Renderable};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct BindingGeneralData {
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct BindingLightData {
    center: [f32; 2],
    freq: f32,
    radius: f32,
}

impl BindingLightData {
    fn new(source: &LightSourceData) -> Self {
        Self {
            center: source.center,
            freq: 0.,
            radius: source.radius,
        }
    }

    fn from_sources(sources: &[LightSourceData]) -> Vec<Self> {
        let mut datas = Vec::with_capacity(sources.len());

        for source in sources {
            datas.push(Self::new(source));
        }

        datas
    }
}

pub struct LightSources {
    bar_processor: BarProcessor<NothingInterpolation>,

    amount_light_sources: usize,
    uniform_pulse: bool,

    _wallpaper: wgpu::Texture,
    _sampler: wgpu::Sampler,
    general_data_buffer: wgpu::Buffer,
    light_sources_buffer: wgpu::Buffer,

    bind_group0: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl LightSources {
    pub fn new<F: Fetcher>(desc: &LightSourcesDescriptor<F>) -> Self {
        let device = desc.renderer.device();
        let queue = desc.renderer.queue();

        let bar_processor = {
            let amount_bars = if desc.uniform_pulse {
                1u16
            } else {
                desc.sources.len() as u16
            };

            BarProcessor::new(
                desc.processor,
                BarProcessorConfig {
                    amount_bars: NonZero::new(amount_bars).unwrap(),
                    freq_range: desc.freq_range.clone(),
                    down: desc.sensitivity,
                    ..Default::default()
                },
            )
        };

        let general_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light sources: General data buffer"),
            contents: bytemuck::bytes_of(&BindingGeneralData {
                resolution: [0f32; 2],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let wallpaper = crate::util::load_img_to_texture(device, queue, &desc.wallpaper);

        let sampler = device.create_sampler(&crate::util::DEFAULT_SAMPLER_DESCRIPTOR);

        let (amount_light_sources, light_sources_buffer) = {
            let datas = BindingLightData::from_sources(desc.sources);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light sources: Light data buffer"),
                contents: bytemuck::cast_slice(&datas),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            (datas.len(), buffer)
        };

        let pipeline = {
            let vertex_shader =
                device.create_shader_module(include_wgsl!("../../utils/full_screen_vertex.wgsl"));

            let fragment_shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));
            let entry_point = if desc.debug_sources { "debug" } else { "main" };

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Light sources: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_shader,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_shader,
                        entry_point: Some(entry_point),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light sources: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: general_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &wallpaper.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_sources_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bar_processor,

            _wallpaper: wallpaper,
            _sampler: sampler,
            general_data_buffer,
            light_sources_buffer,

            amount_light_sources,
            uniform_pulse: desc.uniform_pulse,

            bind_group0,
            pipeline,
        }
    }
}

impl Renderable for LightSources {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl<F: Fetcher> ComponentAudio<F> for LightSources {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>) {
        let channels = self.bar_processor.process_bars(processor);
        let bars = &channels[0];

        const REL_OFFSET_SIZE: usize = std::mem::size_of::<[f32; 2]>();
        if self.uniform_pulse {
            let freq = bars[0];

            for i in 0..self.amount_light_sources {
                let offset = (std::mem::size_of::<BindingLightData>()) * i + REL_OFFSET_SIZE;
                queue.write_buffer(
                    &self.light_sources_buffer,
                    offset as wgpu::BufferAddress,
                    bytemuck::bytes_of(&freq),
                );
            }
        } else {
            for i in 0..self.amount_light_sources {
                let offset = (std::mem::size_of::<BindingLightData>()) * i + REL_OFFSET_SIZE;
                queue.write_buffer(
                    &self.light_sources_buffer,
                    offset as wgpu::BufferAddress,
                    bytemuck::bytes_of(&bars[i]),
                );
            }
        }
    }
}

impl Component for LightSources {
    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let offset = std::mem::offset_of!(BindingGeneralData, resolution);

        queue.write_buffer(
            &self.general_data_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
