use std::{borrow::Cow, num::NonZero};

use cgmath::{Deg, InnerSpace, Matrix2, Vector2};
use pollster::FutureExt;
use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::{Component, Rgba, ShaderCode, ShaderCodeError};

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BarsFormat {
    #[default]
    BassTreble,
    TrebleBass,
    TrebleBassTreble,
    BassTrebleBass,
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
    pub format: BarsFormat,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    pipelines: Box<[wgpu::RenderPipeline]>,
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
            let (bottom_left_corner, rotation, mut width_factor) = match desc.placement {
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
                        // remap [0, 1] x [0, 1] to [-1, 1] x [-1, 1]
                        let mut pos = {
                            let bottom_left_corner = Vector2::from(bottom_left_corner);

                            let x = 2. * bottom_left_corner.x - 1.0;
                            let y = -(2. * bottom_left_corner.y - 1.0);

                            Vector2::from((x, y))
                        };
                        pos.x = pos.x.clamp(-1., 1.);
                        pos.y = pos.y.clamp(-1., 1.);
                        pos
                    };
                    let width = width_factor.clamp(0., 1.);
                    let rotation = Matrix2::from_angle(rotation);

                    (bottom_left_corner, rotation, width)
                }
            };

            // if we have to render two "sections" of the bars, we need to divide the width by 2
            if [BarsFormat::TrebleBassTreble, BarsFormat::BassTrebleBass].contains(&desc.format) {
                width_factor /= 2.;
            }

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

        // == create buffers ==
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

        let pipelines: Box<[wgpu::RenderPipeline]> = {
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
                            label: Some("Bar: `high_presence_color` buffer"),
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

                    let fragment_module = {
                        let source = code.source().map_err(ShaderCodeError::from)?;

                        let shader_source = match code.language {
                            super::ShaderLanguage::Wgsl => {
                                const PREAMBLE: &str =
                                    include_str!("./shaders/fragment_code_preamble.wgsl");
                                let full_code = format!("{}\n{}", PREAMBLE, &source);
                                wgpu::ShaderSource::Wgsl(Cow::Owned(full_code))
                            }
                            super::ShaderLanguage::Glsl => {
                                const PREAMBLE: &str =
                                    include_str!("./shaders/fragment_code_preamble.glsl");
                                let full_code = format!("{}\n{}", PREAMBLE, &source);
                                wgpu::ShaderSource::Glsl {
                                    shader: Cow::Owned(full_code),
                                    stage: wgpu::naga::ShaderStage::Fragment,
                                    defines: wgpu::naga::FastHashMap::default(),
                                }
                            }
                        };

                        device.push_error_scope(wgpu::ErrorFilter::Validation);
                        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Fragment canvas fragment module"),
                            source: shader_source,
                        });

                        if let Some(err) = device.pop_error_scope().block_on() {
                            return Err(ShaderCodeError::ParseError(err));
                        }

                        module
                    };

                    fragment_module
                }
            };

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bar vertex module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/vertex_shader.wgsl").into(),
                ),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bar pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0.get_bind_group_layout(device),
                    &bind_group1.get_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

            let fragment_targets = [Some(wgpu::ColorTargetState {
                format: desc.texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            })];

            let bass_treble_pipeline_descriptor = crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Bar: Render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some("bass_treble"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some(SHADER_ENTRYPOINT),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &fragment_targets,
                    },
                },
            );

            let treble_bass_pipeline_descriptor = {
                let mut descriptor = bass_treble_pipeline_descriptor.clone();
                descriptor.vertex.entry_point = Some("treble_bass");
                descriptor
            };

            let bass_treble_pipeline =
                device.create_render_pipeline(&bass_treble_pipeline_descriptor);
            let treble_bass_pipeline =
                device.create_render_pipeline(&treble_bass_pipeline_descriptor);

            match desc.format {
                BarsFormat::BassTreble => {
                    vec![bass_treble_pipeline]
                }
                BarsFormat::TrebleBass => {
                    vec![treble_bass_pipeline]
                }
                BarsFormat::TrebleBassTreble => {
                    vec![treble_bass_pipeline, bass_treble_pipeline]
                }
                BarsFormat::BassTrebleBass => {
                    vec![bass_treble_pipeline, treble_bass_pipeline]
                }
            }
            .into_boxed_slice()
        };

        bind_group0.build_bind_group(device);
        bind_group1.build_bind_group(device);

        Ok(Self {
            amount_bars,
            bar_processor,

            bind_group0,
            bind_group1,

            pipelines,
        })
    }
}

impl Renderable for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);

        let mut instance_idx_range = 0..u16::from(self.amount_bars) as u32;
        for pipeline in self.pipelines.iter() {
            pass.set_pipeline(pipeline);
            pass.draw(0..4, instance_idx_range.clone());

            instance_idx_range.start = instance_idx_range.end;
            instance_idx_range.end += u16::from(self.amount_bars) as u32;
        }
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
