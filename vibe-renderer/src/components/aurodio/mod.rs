use std::num::NonZero;

use rand::Rng;
use shady_audio::{BarProcessor, BarProcessorConfig, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{components::bind_group_manager::BindGroupManager, Renderable};

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
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a SampleProcessor,
    pub audio_conf: BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,

    pub amount_layers: NonZero<u8>,
    pub base_color: Rgb,
}

pub struct Aurodio {
    bar_processor: BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Aurodio {
    pub fn new(desc: &AurodioDescriptor) -> Self {
        let device = desc.device;
        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            BarProcessorConfig {
                amount_bars: NonZero::new(u16::from(u8::from(desc.amount_layers))).unwrap(),
                ..desc.audio_conf.clone()
            },
        );

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
            let (points, width) = get_points(desc.amount_layers);

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
            let zoom_factors: Vec<f32> = get_zoom_factors(desc.amount_layers);

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
            let random_seeds: Vec<f32> = get_random_seeds(desc.amount_layers);

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
                size: (std::mem::size_of::<f32>()
                    * usize::from(u16::from(desc.audio_conf.amount_bars)))
                    as wgpu::BufferAddress,
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
            bar_processor,

            bind_group0: bind_group0_builder.build(device),
            bind_group1: bind_group1_builder.build(device),

            vbuffer,
            pipeline,
        }
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
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
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

fn get_points(amount_layers: NonZero<u8>) -> (Vec<[f32; 2]>, u32) {
    let amount_layers_usize = usize::from(u8::from(amount_layers));
    let mut points = Vec::with_capacity(amount_layers_usize);

    let width = amount_layers_usize + 2; // `+2` one square for the left/top and right/bottom
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

fn get_zoom_factors(amount_layers: NonZero<u8>) -> Vec<f32> {
    (0..u8::from(amount_layers))
        .into_iter()
        .map(|layer_idx| (layer_idx * 2 + 1) as f32)
        .collect()
}

fn get_random_seeds(amount_layers: NonZero<u8>) -> Vec<f32> {
    let amount_layers_usize = usize::from(u8::from(amount_layers));
    let mut seeds = vec![0f32; amount_layers_usize];

    rand::rng().fill(&mut seeds[..]);

    seeds
}
