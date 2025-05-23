use cgmath::Deg;
use shady_audio::BarProcessor;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{bind_group_manager::BindGroupManager, Renderable, Renderer};

use super::{Component, Rgba};

const SHADER_ENTRYTPOINT: &str = "main";

type VertexPosition = [f32; 2];
const POSITIONS: [VertexPosition; 3] = [
    [1., 1.],  // top right
    [1., -3.], // right bottom corner
    [-3., 1.], // top left corner
];

#[derive(Debug, Clone)]
pub enum GraphPlacement {
    Bottom,
    Top,
    Right,
    Left,
    Custom {
        // Convention: (0., 0.) is the top left corner of the screen
        bottom_left_corner: (f32, f32),
        rotation: Deg<f32>,
    },
}

#[derive(Debug, Clone)]
pub enum GraphVariant {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Binding0 {
    CanvasHeight = 0,
    MaxHeight = 1,
    Color = 2,
    HorizontalGradientLeft = 3,
    HorizontalGradientRight = 4,
    CanvasWidth = 5,
    VerticalGradientTop = 6,
    VerticalGradientBottom = 7,
    Smoothness = 8,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Binding1 {
    Freqs = 0,
}

pub struct GraphDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a shady_audio::SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,

    pub variant: GraphVariant,
    pub max_height: f32,
    pub smoothness: f32,
    pub placement: GraphPlacement,
}

pub struct Graph {
    bar_processor: shady_audio::BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Graph {
    pub fn new(desc: &GraphDescriptor) -> Self {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut bind_group0 = BindGroupManager::new(Some("Graph: Bind group 0"));
        let mut bind_group1 = BindGroupManager::new(Some("Graph: Bind group 1"));

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Graph: Vertex buffer => positions"),
            contents: bytemuck::cast_slice(&POSITIONS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        bind_group0.insert_buffer(
            Binding0::CanvasHeight as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Graph: `canvas_height` buffer"),
                contents: bytemuck::bytes_of(&0f32),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );

        bind_group0.insert_buffer(
            Binding0::MaxHeight as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Graph: `max_height` buffer"),
                contents: bytemuck::bytes_of(&desc.max_height),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Binding0::Smoothness as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Graph: `smoothness` buffer"),
                contents: bytemuck::bytes_of(&desc.smoothness),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group1.insert_buffer(
            Binding1::Freqs as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Graph: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * u16::from(desc.audio_conf.amount_bars) as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let fragment_shader = match &desc.variant {
            GraphVariant::Color(rgba) => {
                bind_group0.insert_buffer(
                    Binding0::Color as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color` buffer"),
                        contents: bytemuck::cast_slice(rgba),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(include_wgsl!("./fragment_color.wgsl"))
            }
            GraphVariant::HorizontalGradient { left, right } => {
                bind_group0.insert_buffer(
                    Binding0::CanvasWidth as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Graph: `canvas_width` buffer"),
                        size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                bind_group0.insert_buffer(
                    Binding0::HorizontalGradientLeft as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color_left` buffer"),
                        contents: bytemuck::cast_slice(left),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0.insert_buffer(
                    Binding0::HorizontalGradientRight as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color_right` buffer"),
                        contents: bytemuck::cast_slice(right),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(include_wgsl!("./fragment_horizontal_gradient.wgsl"))
            }
            GraphVariant::VerticalGradient { top, bottom } => {
                bind_group0.insert_buffer(
                    Binding0::VerticalGradientTop as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color_top` buffer"),
                        contents: bytemuck::cast_slice(top),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0.insert_buffer(
                    Binding0::VerticalGradientBottom as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color_bottom` buffer"),
                        contents: bytemuck::cast_slice(bottom),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(include_wgsl!("./fragment_vertical_gradient.wgsl"))
            }
        };

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Graph: Pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0.get_bind_group_layout(device),
                    &bind_group1.get_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

            let vertex_shader = device.create_shader_module(include_wgsl!("./vertex_shader.wgsl"));

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Graph: Render pipeline`"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: Some(SHADER_ENTRYTPOINT),
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
                    topology: wgpu::PrimitiveTopology::TriangleList,
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
                    module: &fragment_shader,
                    entry_point: Some(SHADER_ENTRYTPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: desc.output_texture_format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };

        bind_group0.build_bind_group(device);
        bind_group1.build_bind_group(device);

        Self {
            bar_processor,
            vbuffer,
            pipeline,

            bind_group0,
            bind_group1,
        }
    }
}

impl Renderable for Graph {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);
        pass.draw(0..3, 0..1);
    }
}

impl Component for Graph {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Binding1::Freqs as u32) {
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let device = renderer.device();

        self.bar_processor
            .set_amount_bars(std::num::NonZero::new(new_resolution[0] as u16).unwrap());

        if let Some(buffer) = self.bind_group0.get_buffer(Binding0::CanvasWidth as u32) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&(new_resolution[0] as f32)));
        }

        if let Some(buffer) = self.bind_group0.get_buffer(Binding0::CanvasHeight as u32) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&(new_resolution[1] as f32)));
        }

        self.bind_group1.replace_buffer(
            Binding1::Freqs as u32,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Graph: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * new_resolution[0] as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        self.bind_group1.build_bind_group(device);
    }
}
