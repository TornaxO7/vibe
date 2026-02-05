mod descriptor;

pub use descriptor::*;

use super::{Component, Vec2f};
use crate::{
    texture_generation::{SdfMask, SdfPattern},
    Renderable,
};
use vibe_audio::{fetcher::Fetcher, BarProcessor};
use wgpu::{include_wgsl, util::DeviceExt};

// this texture size seems good enough for a 1920x1080 screen.
const DEFAULT_SDF_TEXTURE_SIZE: u32 = 512;

type Resolution = Vec2f;
type Time = f32;
type MovementSpeed = f32;
type ZoomFactor = f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct Data {
    resolution: Resolution,
    time: Time,
    movement_speed: MovementSpeed,
    zoom_factor: ZoomFactor,
    _padding: f32,
}

pub struct Chessy {
    bar_processor: BarProcessor,

    data_buffer: wgpu::Buffer,
    freqs_buffer: wgpu::Buffer,
    grid_texture: wgpu::Texture,
    grid_sampler: wgpu::Sampler,

    bind_group0: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    // data to recreate the grid texture
    pattern: SdfPattern,
}

impl Chessy {
    pub fn new<F: Fetcher>(desc: &ChessyDescriptor<F>) -> Self {
        let renderer = desc.renderer;
        let device = renderer.device();
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_config.clone());
        let total_amount_bars = bar_processor.total_amount_bars();

        let data = Data {
            resolution: Resolution::default(),
            time: 0f32,
            movement_speed: desc.movement_speed,
            zoom_factor: desc.zoom_factor,
            ..Default::default()
        };

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chessy: Data buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chessy: `freqs` buffer"),
            size: (std::mem::size_of::<f32>() * total_amount_bars) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // arbitrary size for the beginning
        let grid_texture = desc.renderer.generate(&SdfMask {
            texture_size: 50,
            pattern: desc.pattern,
        });

        let grid_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Chessy: Grid texture sampler"),

            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline = {
            let vertex_module =
                device.create_shader_module(include_wgsl!("../utils/full_screen_vertex.wgsl"));

            let fragment_module =
                device.create_shader_module(include_wgsl!("./shaders/fragment_shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Chessy: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chessy: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &grid_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&grid_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: freqs_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bar_processor,

            data_buffer,
            freqs_buffer,
            grid_texture,
            grid_sampler,

            bind_group0,
            pipeline,

            pattern: desc.pattern,
        }
    }
}

impl Renderable for Chessy {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for Chessy {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &vibe_audio::SampleProcessor<vibe_audio::fetcher::SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(&self.freqs_buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let resolution_size = 8;

        queue.write_buffer(
            &self.data_buffer,
            resolution_size,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let device = renderer.device();

        {
            // idea: If someone has a 4k screen for example, we'd need to double the sdf texture to
            // avoid blurring of each cell.
            // Why doubling if 4k:
            //
            // The longest side of a typical 4k (3840x2160) screen is 3840 ...
            let max_length = new_resolution.iter().max().unwrap();
            // ... so it's `3840 / 1920 = 2` twice is big as the texture size for a full hd screen...
            let factor = *max_length as f32 / 1920f32;
            // ... so we double it ~~and give it to the next person~~
            let new_size = DEFAULT_SDF_TEXTURE_SIZE as f32 * factor;

            self.grid_texture = renderer.generate(&SdfMask {
                texture_size: new_size.ceil() as u32,
                pattern: self.pattern,
            });

            self.bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Chessy: Bind group 0"),
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self
                                .grid_texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.grid_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.freqs_buffer.as_entire_binding(),
                    },
                ],
            });

            // update `resolution` values
            queue.write_buffer(
                &self.data_buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
