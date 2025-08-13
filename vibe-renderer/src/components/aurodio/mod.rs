mod descriptor;

pub use descriptor::*;

use super::Component;
use crate::{resource_manager::ResourceManager, Renderable};
use rand::Rng;
use shady_audio::{BarProcessor, BarProcessorConfig};
use std::num::NonZero;
use wgpu::util::DeviceExt;

type VertexPosition = [f32; 2];

#[rustfmt::skip]
const VERTICES: [VertexPosition; 4] = [
    [1.0, 1.0],   // top right
    [-1.0, 1.0],  // top left
    [1.0, -1.0],  // bottom right
    [-1.0, -1.0]  // bottom left
];

const ENTRY_POINT: &str = "main";

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const POINTS: u32 = 1;
    pub const POINTS_WIDTH: u32 = 2;
    pub const ZOOM_FACTORS: u32 = 3;
    pub const RANDOM_SEEDS: u32 = 4;
    pub const BASE_COLOR: u32 = 5;
    pub const VALUE_NOISE_TEXTURE: u32 = 6;
    pub const VALUE_NOISE_SAMPLER: u32 = 7;
    pub const MOVEMENT_SPEED: u32 = 8;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::ZoomFactors, crate::util::buffer(ZOOM_FACTORS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::RandomSeeds, crate::util::buffer(RANDOM_SEEDS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::BaseColor, crate::util::buffer(BASE_COLOR, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::MovementSpeed, crate::util::buffer(MOVEMENT_SPEED, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Points, crate::util::buffer(POINTS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::PointsWidth, crate::util::buffer(POINTS_WIDTH, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::ValueNoiseTexture, crate::util::texture(VALUE_NOISE_TEXTURE, wgpu::ShaderStages::FRAGMENT)),
            (ResourceID::ValueNoiseSampler, crate::util::sampler(VALUE_NOISE_SAMPLER, wgpu::ShaderStages::FRAGMENT)),
        ])
    }
}
mod bindings1 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const TIME: u32 = 0;
    pub const FREQS: u32 = 1;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Time, crate::util::buffer(TIME, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    Points,
    PointsWidth,
    ZoomFactors,
    RandomSeeds,
    BaseColor,
    ValueNoiseTexture,
    ValueNoiseSampler,
    MovementSpeed,

    Time,
    Freqs,
}

pub struct Aurodio {
    bar_processors: Box<[BarProcessor]>,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Aurodio {
    pub fn new(desc: &AurodioDescriptor) -> Self {
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

        let mut resource_manager = ResourceManager::new();

        let bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Aurodio: `iResolution` buffer"),
                    size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            {
                let zoom_factors: Vec<f32> =
                    desc.layers.iter().map(|layer| layer.zoom_factor).collect();

                (
                    ResourceID::ZoomFactors,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Aurodio: `zoom_factors` buffer"),
                        contents: bytemuck::cast_slice(&zoom_factors),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                )
            },
            {
                let random_seeds: Vec<f32> = get_random_seeds(amount_layers);

                (
                    ResourceID::RandomSeeds,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Aurodio: `random_seeds` buffer"),
                        contents: bytemuck::cast_slice(&random_seeds),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                )
            },
            (
                ResourceID::BaseColor,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `base_color` buffer"),
                    contents: bytemuck::cast_slice(&desc.base_color),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::MovementSpeed,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Aurodio: `movement_speed` buffer"),
                    contents: bytemuck::bytes_of(&desc.movement_speed),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Time,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Aurodio: `iTime` buffer"),
                    size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Aurodio: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * amount_layers) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        {
            let (points, width) = get_points(amount_layers * 2);

            resource_manager.extend_buffers([
                (
                    ResourceID::Points,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Aurodio: `points` buffer"),
                        contents: bytemuck::cast_slice(&points),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                ),
                (
                    ResourceID::PointsWidth,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Aurodio: `points_width` buffer"),
                        contents: bytemuck::bytes_of(&width),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                ),
            ]);
        }

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

            resource_manager.insert_texture(ResourceID::ValueNoiseTexture, value_noise_texture);
            resource_manager.insert_sampler(ResourceID::ValueNoiseSampler, sampler);
        }

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Aurodio: Vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "Aurodio: Bind group 0",
            device,
            &bind_group0_mapping,
        );

        let (bind_group1, bind_group1_layout) = resource_manager.build_bind_group(
            "Aurodio: Bind group 1",
            device,
            &bind_group1_mapping,
        );

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
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Aurodio: Render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some(ENTRY_POINT),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<VertexPosition>()
                                as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some(ENTRY_POINT),
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

        Self {
            bar_processors,

            resource_manager,

            bind_group0,
            bind_group1,

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
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for Aurodio {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        let mut bar_values: Vec<f32> = Vec::with_capacity(self.amount_layers());

        for bar_processor in self.bar_processors.iter_mut() {
            // we only have one bar
            bar_values.push(bar_processor.process_bars(processor)[0][0]);
        }

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Time).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let buffer = self
            .resource_manager
            .get_buffer(ResourceID::Resolution)
            .unwrap();

        queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
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

fn get_random_seeds(amount_layers: usize) -> Vec<f32> {
    let mut seeds = Vec::with_capacity(amount_layers);
    let mut rng = rand::rng();

    for _ in 0..amount_layers {
        seeds.push(rng.random_range(0f32..100f32));
    }

    seeds
}
