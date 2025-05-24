use std::{borrow::Cow, num::NonZero};

use cgmath::{Deg, InnerSpace, Matrix2, Vector2};
use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::{Component, Rgba, ShaderCode, ShaderCodeError, ShaderLanguage};

const SHADER_ENTRYPOINT: &str = "main";

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

#[derive(Debug, Clone)]
pub enum BarsPlacement {
    Custom {
        // Convention:
        // - (0., 0.) is the top left corner
        // - (1., 1.) is the bottom right corner
        bottom_left_corner: (f32, f32),
        // percentage of the screen width (so it should be within the range [0, 1])
        width_factor: f32,
        rotation: Deg<f32>,
    },
    Bottom,
    Top,
    Right,
    Left,
}

#[derive(Debug, Clone)]
pub enum BarVariant {
    Color(Rgba),
    PresenceGradient { high: Rgba, low: Rgba },
    FragmentCode(ShaderCode),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings0 {
    BottomLeftCorner = 0,
    UpDirection = 1,
    ColumnDirection = 2,
    Padding = 3,
    MaxHeight = 4,

    Color = 5,
    Resolution = 6,
    GradientHighPresenceColor = 7,
    GradientLowPresenceColor = 8,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings1 {
    Freqs = 0,
    Time = 1,
}

pub struct BarsDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    pub variant: BarVariant,
    pub max_height: f32,

    pub placement: BarsPlacement,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    pipeline: wgpu::RenderPipeline,
}

impl Bars {
    pub fn new(desc: &BarsDescriptor) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut bind_group0 = BindGroupManager::new(Some("Bars: Bind group 0"));
        let mut bind_group1 = BindGroupManager::new(Some("Bars: Bind group 1"));

        // `bottom_left_corner`: In vertex space coords
        let (bottom_left_corner, up_direction, column_direction) = {
            let (bottom_left_corner, rotation, width_factor) = match desc.placement {
                BarsPlacement::Bottom => {
                    (Vector2::from([-1., -1.]), Matrix2::from_angle(Deg(0.)), 1.)
                }
                BarsPlacement::Right => {
                    (Vector2::from([1., -1.]), Matrix2::from_angle(Deg(90.)), 1.)
                }
                BarsPlacement::Top => (Vector2::from([1., 1.]), Matrix2::from_angle(Deg(180.)), 1.),
                BarsPlacement::Left => {
                    (Vector2::from([-1., 1.]), Matrix2::from_angle(Deg(270.)), 1.)
                }
                BarsPlacement::Custom {
                    bottom_left_corner,
                    width_factor,
                    rotation,
                } => {
                    let bottom_left_corner = {
                        let mut pos = (2. * Vector2::from(bottom_left_corner)
                            - Vector2::from([1f32; 2])) // remap [0, 1] x [0, 1] to [-1, 1] x [-1, 1]
                            + Vector2::from((-1., 1.)); // use it as an offset of the top left corner
                        pos.x = pos.x.clamp(-1., 1.);
                        pos.y = pos.y.clamp(-1., 1.);
                        pos
                    };
                    let width = width_factor.clamp(0., 1.);
                    let rotation = Matrix2::from_angle(rotation);

                    (bottom_left_corner, rotation, width)
                }
            };

            let up_direction = (rotation * Vector2::unit_y()).normalize();
            let column_direction = {
                let column_width = (VERTEX_SURFACE_WIDTH * width_factor)
                    / u16::from(desc.audio_conf.amount_bars) as f32;
                let direction = rotation * Vector2::unit_x();

                direction.normalize_to(column_width)
            };

            (bottom_left_corner, up_direction, column_direction)
        };

        let padding = column_direction * 0.2;

        bind_group0.insert_buffer(
            Bindings0::BottomLeftCorner as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `bottom_left_corner` buffer"),
                contents: bytemuck::cast_slice(&[bottom_left_corner.x, bottom_left_corner.y]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::UpDirection as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `up_direction` buffer"),
                contents: bytemuck::cast_slice(&[up_direction.x, up_direction.y]),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::ColumnDirection as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `column_direction` buffer"),
                contents: bytemuck::cast_slice(&[column_direction.x, column_direction.y]),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::Padding as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `padding` buffer"),
                contents: bytemuck::cast_slice(&[padding.x, padding.y]),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::MaxHeight as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `max_height` buffer"),
                contents: bytemuck::bytes_of(&(desc.max_height * VERTEX_SURFACE_WIDTH)),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group1.insert_buffer(
            Bindings1::Freqs as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Bar: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars))) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let fragment_module = match &desc.variant {
            BarVariant::Color(rgba) => {
                bind_group0.insert_buffer(
                    Bindings0::Color as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bar: `color` buffer"),
                        contents: bytemuck::cast_slice(rgba),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Bar: Color fragment module"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("./shaders/fragment_color.wgsl").into(),
                    ),
                })
            }
            BarVariant::PresenceGradient { high, low } => {
                bind_group0.insert_buffer(
                    Bindings0::GradientHighPresenceColor as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bar: `high_prescene_color` buffer"),
                        contents: bytemuck::cast_slice(high),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0.insert_buffer(
                    Bindings0::GradientLowPresenceColor as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bar: `low_presence_color` buffer"),
                        contents: bytemuck::cast_slice(low),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Bar: Gradient fragment module"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("./shaders/fragment_presence_gradient.wgsl").into(),
                    ),
                })
            }
            BarVariant::FragmentCode(code) => {
                bind_group0.insert_buffer(
                    Bindings0::Resolution as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Bar: iResolution buffer"),
                        size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                bind_group1.insert_buffer(
                    Bindings1::Time as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Bar: iTime buffer"),
                        size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                let module = {
                    let source = code.source().map_err(ShaderCodeError::from)?;

                    match code.language {
                        ShaderLanguage::Wgsl => {
                            const PREAMBLE: &str =
                                include_str!("./shaders/fragment_code_preamble.wgsl");
                            super::parse_wgsl_fragment_code(PREAMBLE, &source)
                                .map_err(ShaderCodeError::ParseError)?
                        }
                        ShaderLanguage::Glsl => {
                            const PREAMBLE: &str =
                                include_str!("./shaders/fragment_code_preamble.glsl");
                            super::parse_glsl_fragment_code(PREAMBLE, &source)
                                .map_err(ShaderCodeError::ParseError)?
                        }
                    }
                };

                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Bar: Fragment code module"),
                    source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
                })
            }
        };

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bar pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0.get_bind_group_layout(device),
                    &bind_group1.get_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bar vertex module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/vertex_shader.wgsl").into(),
                ),
            });

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bar render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: Some(SHADER_ENTRYPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
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
                    entry_point: Some(SHADER_ENTRYPOINT),
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

        bind_group0.build_bind_group(device);
        bind_group1.build_bind_group(device);

        Ok(Self {
            amount_bars,
            bar_processor,

            bind_group0,
            bind_group1,

            pipeline,
        })
    }
}

impl Renderable for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);

        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..u16::from(self.amount_bars) as u32);
    }
}

impl Component for Bars {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Freqs as u32) {
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
        }
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Time as u32) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
        }
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::Resolution as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }
}
