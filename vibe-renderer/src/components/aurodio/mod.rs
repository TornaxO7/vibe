mod descriptor;
pub use descriptor::*;

use super::Component;
use crate::{texture_generation::ValueNoise, Renderable};
use std::num::NonZero;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig,
};
use wgpu::{include_wgsl, util::DeviceExt};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    base_color: [f32; 3],
    time: f32,
    resolution: [f32; 2],
    movement_speed: f32,
    points_width: u32,
}

pub struct Aurodio {
    bar_processors: Box<[BarProcessor]>,

    bind_group0: wgpu::BindGroup,
    fragment_params_buffer: wgpu::Buffer,
    _points_buffer: wgpu::Buffer,
    _zoom_factors_buffer: wgpu::Buffer,
    _random_seeds_buffer: wgpu::Buffer,
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
                        sensitivity: desc.sensitivity,
                        ..Default::default()
                    },
                ));
            }

            bar_processors.into_boxed_slice()
        };

        let (fragment_params_buffer, points_buffer) = {
            let (points, points_width) = get_points(amount_layers * 2);

            let fragment_params = FragmentParams {
                base_color: desc.base_color,
                time: 0f32,
                resolution: [0f32; 2],
                movement_speed: desc.movement_speed,
                points_width,
            };

            let fragment_params_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: Fragment params buffer"),
                    contents: bytemuck::bytes_of(&fragment_params),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

            let points_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: `points` buffer"),
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::STORAGE,
            });

            (fragment_params_buffer, points_buffer)
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

        let random_seeds_buffer = {
            let random_seeds: Vec<f32> = get_random_seeds(amount_layers);
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: `random_seeds` buffer"),
                contents: bytemuck::cast_slice(&random_seeds),
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
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Aurodio: Value noise sampler"),
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
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
                    resource: points_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: zoom_factors_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: random_seeds_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &value_noise_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: freqs_buffer.as_entire_binding(),
                },
            ],
        });

        let bar_values_buffer = vec![0f32; bar_processors.len()].into_boxed_slice();

        Self {
            bar_processors,

            bind_group0,
            fragment_params_buffer,
            _points_buffer: points_buffer,
            _zoom_factors_buffer: zoom_factors_buffer,
            _random_seeds_buffer: random_seeds_buffer,
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

impl Component for Aurodio {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &vibe_audio::SampleProcessor<SystemAudioFetcher>,
    ) {
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

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        queue.write_buffer(
            &self.fragment_params_buffer,
            std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let offset = {
            let base_color_size = std::mem::size_of::<[f32; 3]>();
            let time_size = std::mem::size_of::<f32>();

            base_color_size + time_size
        };

        queue.write_buffer(
            &self.fragment_params_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

fn get_points(amount_layers: usize) -> (Vec<[f32; 2]>, u32) {
    let mut points = Vec::with_capacity(amount_layers);

    let width = amount_layers + 2; // `+2` one square for the left/top and right/bottom
    let height = width;
    let mut rng = fastrand::Rng::new();
    for _y in 0..height {
        for _x in 0..width {
            let mut point = [0u8; 2];
            rng.fill(&mut point[..]);

            points.push([
                (point[0] as f32 / u8::MAX as f32),
                (point[1] as f32 / u8::MAX as f32),
            ]);
        }
    }

    (points, width as u32)
}

fn get_random_seeds(amount_layers: usize) -> Vec<f32> {
    let mut seeds = Vec::with_capacity(amount_layers);
    let mut rng = fastrand::Rng::new();

    for _ in 0..amount_layers {
        // range: 0..100
        seeds.push(rng.f32() * 100.);
    }

    seeds
}
