mod descriptor;
pub use descriptor::*;

use super::{Component, Vec2f, Vec3f};
use crate::{components::ComponentAudio, texture_generation::ValueNoise, Renderable};
use std::num::NonZero;
use vibe_audio::{fetcher::Fetcher, BarProcessor, BarProcessorConfig, NothingInterpolation};
use wgpu::{include_wgsl, util::DeviceExt};

type BaseColor = Vec3f;
type Time = f32;
type Resolution = Vec2f;
type MovementSpeed = f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    base_color: BaseColor,
    time: Time,
    resolution: Resolution,
    movement_speed: MovementSpeed,
    _padding: u32,
}

pub struct Aurodio {
    bar_processors: Box<[BarProcessor<NothingInterpolation>]>,

    bind_group0: wgpu::BindGroup,
    fragment_params_buffer: wgpu::Buffer,
    _zoom_factors_buffer: wgpu::Buffer,
    freqs_buffer: wgpu::Buffer,

    bar_values_buffer: Box<[f32]>,

    pipeline: wgpu::RenderPipeline,
}

impl Aurodio {
    pub fn new<F: Fetcher>(desc: &AurodioDescriptor<F>) -> Self {
        let amount_layers = desc.layers.len();
        let device = desc.renderer.device();
        let bar_processors = {
            let mut bar_processors = Vec::new();

            for layer in desc.layers.iter() {
                bar_processors.push(BarProcessor::new(
                    desc.sample_processor,
                    BarProcessorConfig {
                        amount_bars: NonZero::new(1).unwrap(),
                        freq_range: layer.freq_range.clone(),
                        down: desc.sensitivity,
                        ..Default::default()
                    },
                ));
            }

            bar_processors.into_boxed_slice()
        };

        let fragment_params_buffer = {
            let fragment_params = FragmentParams {
                base_color: desc.base_color,
                time: Time::default(),
                resolution: Resolution::default(),
                movement_speed: desc.movement_speed,
                ..Default::default()
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: Fragment params buffer"),
                contents: bytemuck::bytes_of(&fragment_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        let zoom_factors_buffer = {
            let zoom_factors: Vec<f32> =
                desc.layers.iter().map(|layer| layer.zoom_factor).collect();

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: `zoom_factors` buffer"),
                contents: bytemuck::cast_slice(&zoom_factors),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Aurodio: `freqs` buffer"),
            size: (std::mem::size_of::<f32>() * amount_layers) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let value_noise_texture = desc.renderer.generate(&ValueNoise {
            texture_size: 256,
            octaves: 7,
            seed: desc.seed,
        });
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Aurodio: Sampler nearest"),
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mag_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline = {
            let vertex_module =
                device.create_shader_module(include_wgsl!("../utils/full_screen_vertex.wgsl"));

            let fragment_module = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Aurodio: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.texture_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Aurodio: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fragment_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: zoom_factors_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &value_noise_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: freqs_buffer.as_entire_binding(),
                },
            ],
        });

        let bar_values_buffer = vec![0f32; bar_processors.len()].into_boxed_slice();

        Self {
            bar_processors,

            bind_group0,
            fragment_params_buffer,
            _zoom_factors_buffer: zoom_factors_buffer,
            freqs_buffer,
            bar_values_buffer,

            pipeline,
        }
    }
}

impl Renderable for Aurodio {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl<F: Fetcher> ComponentAudio<F> for Aurodio {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        for (idx, bar_processor) in self.bar_processors.iter_mut().enumerate() {
            // we only have one bar
            self.bar_values_buffer[idx] = bar_processor.process_bars(processor)[0][0];
        }

        queue.write_buffer(
            &self.freqs_buffer,
            0,
            bytemuck::cast_slice(&self.bar_values_buffer),
        );
    }
}

impl Component for Aurodio {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let offset = std::mem::size_of::<BaseColor>();

        queue.write_buffer(
            &self.fragment_params_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let offset = std::mem::size_of::<BaseColor>() + std::mem::size_of::<Time>();

        queue.write_buffer(
            &self.fragment_params_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
