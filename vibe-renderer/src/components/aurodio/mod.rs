use std::{num::NonZero, ops::Range};

use rand::Rng;
use shady_audio::{BarProcessor, BarProcessorConfig, SampleProcessor, StandardEasing};
use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable, Renderer};

use super::Component;

type Rgb = [f32; 3];

const ENTRY_POINT: &str = "main";

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings0 {
    IResolution,
    Points,
    PointsWidth,
    ZoomFactors,
    RandomSeeds,
    BaseColor,
    ValueNoiseTexture,
    ValueNoiseSampler,
    MovementSpeed,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings1 {
    ITime,
    Freqs,
}

type VertexPosition = [f32; 2];

#[rustfmt::skip]
const VERTICES: [VertexPosition; 4] = [
    [1.0, 1.0],   // top right
    [-1.0, 1.0],  // top left
    [1.0, -1.0],  // bottom right
    [-1.0, -1.0]  // bottom left
];

pub struct AurodioDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub sample_processor: &'a SampleProcessor,
    pub texture_format: wgpu::TextureFormat,

    pub base_color: Rgb,
    // should be very low (recommended: 0.001)
    pub movement_speed: f32,

    // audio config
    pub freq_ranges: &'a [Range<NonZero<u16>>],
    pub easing: StandardEasing,
    pub sensitivity: f32,
}

pub struct Aurodio {
    bar_processors: Box<[BarProcessor]>,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Aurodio {
    pub fn new(desc: &AurodioDescriptor) -> Self {
        let amount_layers = desc.freq_ranges.len();
        let device = desc.renderer.device();
        let bar_processors = {
            let mut bar_processors = Vec::new();

            for freq_range in desc.freq_ranges.iter() {
                bar_processors.push(BarProcessor::new(
                    desc.sample_processor,
                    BarProcessorConfig {
                        amount_bars: NonZero::new(1).unwrap(),
                        freq_range: freq_range.clone(),
                        sensitivity: desc.sensitivity,
                        easer: desc.easing,
                        ..Default::default()
                    },
                ));
            }

            bar_processors.into_boxed_slice()
        };

        let mut bind_group0_builder = BindGroupManager::builder(Some("Aurodio: Bind group 0"));
        let mut bind_group1_builder = BindGroupManager::builder(Some("Aurodio: Bind group 1"));

        bind_group0_builder.insert_buffer(
            Bindings0::IResolution as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Aurodio: `iResolution` buffer"),
                size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        {
            let (points, width) = get_points(amount_layers);

            bind_group0_builder.insert_buffer(
                Bindings0::Points as u32,
                wgpu::ShaderStages::FRAGMENT,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `points` buffer"),
                    contents: bytemuck::cast_slice(&points),
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            );

            bind_group0_builder.insert_buffer(
                Bindings0::PointsWidth as u32,
                wgpu::ShaderStages::FRAGMENT,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `points_width` buffer"),
                    contents: bytemuck::bytes_of(&width),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            )
        }

        {
            let zoom_factors: Vec<f32> = get_zoom_factors(amount_layers);

            bind_group0_builder.insert_buffer(
                Bindings0::ZoomFactors as u32,
                wgpu::ShaderStages::FRAGMENT,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `zoom_factors` buffer"),
                    contents: bytemuck::cast_slice(&zoom_factors),
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            );
        }

        {
            let random_seeds: Vec<f32> = get_random_seeds(amount_layers);

            bind_group0_builder.insert_buffer(
                Bindings0::RandomSeeds as u32,
                wgpu::ShaderStages::FRAGMENT,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `random_seeds` buffer"),
                    contents: bytemuck::cast_slice(&random_seeds),
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            );
        }

        bind_group0_builder.insert_buffer(
            Bindings0::BaseColor as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: `base_color` buffer"),
                contents: bytemuck::cast_slice(&desc.base_color),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        {
            let value_noise_texture = desc.renderer.create_value_noise_texture(256, 256, 1.);
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Aurodio: Value noise sampler"),
                address_mode_u: wgpu::AddressMode::MirrorRepeat,
                address_mode_v: wgpu::AddressMode::MirrorRepeat,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mipmap_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mag_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            bind_group0_builder.insert_texture(
                Bindings0::ValueNoiseTexture as u32,
                wgpu::ShaderStages::FRAGMENT,
                value_noise_texture,
            );

            bind_group0_builder.insert_sampler(
                Bindings0::ValueNoiseSampler as u32,
                wgpu::ShaderStages::FRAGMENT,
                sampler,
            );
        }

        bind_group0_builder.insert_buffer(
            Bindings0::MovementSpeed as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Aurodio: `movement_speed` buffer"),
                contents: bytemuck::bytes_of(&desc.movement_speed),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group1_builder.insert_buffer(
            Bindings1::ITime as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Aurodio: `iTime` buffer"),
                size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        bind_group1_builder.insert_buffer(
            Bindings1::Freqs as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Aurodio: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * amount_layers) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Aurodio: Vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline = {
            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Aurodio: Vertex module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/vertex_shader.wgsl").into(),
                ),
            });

            let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Aurodio: Fragment module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/fragment_shader.wgsl").into(),
                ),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Aurodio: Pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0_builder.get_bind_group_layout(device),
                    &bind_group1_builder.get_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Aurodio: Render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: Some(ENTRY_POINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<VertexPosition>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_module,
                    entry_point: Some(ENTRY_POINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: desc.texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };

        Self {
            bar_processors,

            bind_group0: bind_group0_builder.build(device),
            bind_group1: bind_group1_builder.build(device),

            vbuffer,
            pipeline,
        }
    }

    pub fn amount_layers(&self) -> usize {
        self.bar_processors.len()
    }
}

impl Renderable for Aurodio {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for Aurodio {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Freqs as u32) {
            let mut bar_values = Vec::with_capacity(self.amount_layers());

            for bar_processor in self.bar_processors.iter_mut() {
                bar_values.push(bar_processor.process_bars(processor)[0]);
            }

            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values));
        }
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::ITime as u32) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
        }
    }

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::IResolution as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }
}

fn get_points(amount_layers: usize) -> (Vec<[f32; 2]>, u32) {
    let mut points = Vec::with_capacity(amount_layers);

    let width = amount_layers + 2; // `+2` one square for the left/top and right/bottom
    let height = width;
    let mut rng = rand::rng();
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

fn get_zoom_factors(amount_layers: usize) -> Vec<f32> {
    (0..(amount_layers as u8))
        .into_iter()
        .map(|layer_idx| ((layer_idx + 1).pow(2)) as f32)
        .collect()
}

fn get_random_seeds(amount_layers: usize) -> Vec<f32> {
    let mut seeds = vec![0f32; amount_layers];

    rand::fill(&mut seeds[..]);

    seeds
}
