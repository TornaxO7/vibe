use std::{borrow::Cow, num::NonZero};

use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::{Component, ShaderCode, ShaderCodeError, ShaderLanguage};

// assuming each value is positive
pub type Rgba = [f32; 4];

const SHADER_ENTRYPOINT: &str = "main";

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

#[derive(Debug, Clone)]
pub enum BarVariant {
    Color(Rgba),
    PresenceGradient { high: Rgba, low: Rgba },
    FragmentCode(ShaderCode),
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Bindings0 {
    ColumnWidth = 0,
    Padding = 1,
    MaxHeight = 2,
    Color = 3,
    Resolution = 4,
    GradientHighPresenceColor = 5,
    GradientLowPresenceColor = 6,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Bindings1 {
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

        let mut bind_group0_builder = BindGroupManager::builder(Some("Bars: Bind group 0"));
        let mut bind_group1_builder = BindGroupManager::builder(Some("Bars: Bind group 1"));

        let column_width = VERTEX_SURFACE_WIDTH / u16::from(amount_bars) as f32;
        let padding = column_width / 5.;

        bind_group0_builder.insert_buffer(
            Bindings0::ColumnWidth as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `column_width` buffer"),
                contents: bytemuck::bytes_of(&column_width),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0_builder.insert_buffer(
            Bindings0::Padding as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `padding` buffer"),
                contents: bytemuck::bytes_of(&padding),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0_builder.insert_buffer(
            Bindings0::MaxHeight as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bar: `max_height` buffer"),
                contents: bytemuck::bytes_of(&desc.max_height),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group1_builder.insert_buffer(
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
                bind_group0_builder.insert_buffer(
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
                bind_group0_builder.insert_buffer(
                    Bindings0::GradientHighPresenceColor as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bar: `high_prescene_color` buffer"),
                        contents: bytemuck::cast_slice(high),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0_builder.insert_buffer(
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
                bind_group0_builder.insert_buffer(
                    Bindings0::Resolution as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Bar: iResolution buffer"),
                        size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                bind_group1_builder.insert_buffer(
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
                    &bind_group0_builder.get_bind_group_layout(device),
                    &bind_group1_builder.get_bind_group_layout(device),
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

        Ok(Self {
            amount_bars,
            bar_processor,

            bind_group0: bind_group0_builder.build(device),
            bind_group1: bind_group1_builder.build(device),

            pipeline,
        })
    }
}

impl Renderable for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        if !self.bind_group0.is_empty() {
            pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        }

        if !self.bind_group1.is_empty() {
            pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);
        }

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

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::Resolution as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }
}
