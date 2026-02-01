mod descriptor;

pub use descriptor::*;

use super::{Component, Pixels, Rgba, ShaderCodeError, Vec2f};
use crate::Renderable;
use cgmath::{Deg, Matrix2, Vector2};
use std::num::NonZero;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, SampleProcessor,
};
use wgpu::{include_wgsl, util::DeviceExt};

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

// The actual column direction needs to be computed first after we know
// the size of the screen.
const INIT_COLUMN_DIRECTION: Vector2<f32> = Vector2::new(1.0, 0.0);

const TRUE: u32 = 1;
const FALSE: u32 = 0;

type ColumnDirection = Vec2f;
type BottomLeftCorner = Vec2f;
type UpDirection = Vec2f;
type MaxHeight = f32;
type HeightMirrored = u32;
type AmountBars = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct VertexParams {
    column_direction: ColumnDirection,
    bottom_left_corner: BottomLeftCorner,
    up_direction: UpDirection,
    max_height: MaxHeight,
    // should be a boolean, but... you know, it's not possible due to `bytemuck::Pod`.
    // So, it's meaning is: 1 = True, 0 = False
    height_mirrored: HeightMirrored,
    amount_bars: AmountBars,

    // memory padding
    _padding1: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct FragmentParams {
    color1: Rgba,
    color2: Rgba,
}

#[derive(Debug, Clone, Copy)]
enum FragmentEntrypoint {
    Color,
    Presence,
    HorizontalGradient,
    VerticalGradient,
}

impl FragmentEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            FragmentEntrypoint::Color => "fs_color",
            FragmentEntrypoint::Presence => "fs_presence",
            FragmentEntrypoint::HorizontalGradient => "fs_horizontal_gradient",
            FragmentEntrypoint::VerticalGradient => "fs_vertical_gradient",
        }
    }
}

// The render context to render the bars with the given format
struct RenderCtx {
    freq_buffer: wgpu::Buffer,

    bind_group1: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    // `left` and `right` share the same bind group 0
    bind_group0: wgpu::BindGroup,
    vertex_params_buffer: wgpu::Buffer,
    _fragment_params_buffer: wgpu::Buffer,

    left: RenderCtx,
    // This is set if a second format should be displayed like `BassTrebleBass` => left `BassTreble`, right: `TrebleBass`
    right: Option<RenderCtx>,

    // things we need to update the column-direction-vector
    angle: Deg<f32>,
    width: BarsWidth,
}

impl Bars {
    pub fn new<F: Fetcher>(desc: &BarsDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let (bottom_left_corner, angle, width) = match desc.placement {
            BarsPlacement::Bottom => (Vector2::from([-1., -1.]), Deg(0.), BarsWidth::ScreenWidth),
            BarsPlacement::Right => (Vector2::from([1., -1.]), Deg(90.), BarsWidth::ScreenHeight),
            BarsPlacement::Top => (Vector2::from([1., 1.]), Deg(180.), BarsWidth::ScreenWidth),
            BarsPlacement::Left => (Vector2::from([-1., 1.]), Deg(270.), BarsWidth::ScreenHeight),
            BarsPlacement::Custom {
                bottom_left_corner,
                width,
                rotation,
                ..
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

                (bottom_left_corner, rotation, BarsWidth::Custom(width))
            }
        };

        let vparams = {
            let rotation = Matrix2::from_angle(angle);
            let up_direction = rotation * Vector2::unit_y();
            let column_direction = INIT_COLUMN_DIRECTION;
            let height_mirrored = match desc.placement {
                BarsPlacement::Custom {
                    height_mirrored, ..
                } => match height_mirrored {
                    true => TRUE,
                    false => FALSE,
                },
                _ => FALSE,
            };

            VertexParams {
                bottom_left_corner: bottom_left_corner.into(),
                up_direction: up_direction.into(),
                column_direction: column_direction.into(),
                max_height: desc.max_height * VERTEX_SURFACE_WIDTH,
                height_mirrored,
                amount_bars: amount_bars.get() as AmountBars,
                _padding1: 0,
            }
        };

        let (fragment_entrypoint, fragment_params) = match &desc.variant {
            BarVariant::Color(rgba) => (
                FragmentEntrypoint::Color,
                FragmentParams {
                    color1: *rgba,
                    color2: *rgba,
                },
            ),
            BarVariant::PresenceGradient { high, low } => (
                FragmentEntrypoint::Presence,
                FragmentParams {
                    color1: *low,
                    color2: *high,
                },
            ),

            BarVariant::HorizontalGradient { left, right } => (
                FragmentEntrypoint::HorizontalGradient,
                FragmentParams {
                    color1: *left,
                    color2: *right,
                },
            ),
            BarVariant::VerticalGradient { top, bottom } => (
                FragmentEntrypoint::VerticalGradient,
                FragmentParams {
                    color1: *bottom,
                    color2: *top,
                },
            ),
        };

        let vertex_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bars: Vertex params buffer"),
            contents: bytemuck::bytes_of(&vparams),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fragment_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bars: Fragment params buffer"),
            contents: bytemuck::bytes_of(&fragment_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let left = {
            let freq_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Bars: Left freq buffer"),
                size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let pipeline = {
                let entry_point = match desc.format {
                    BarsFormat::BassTreble | BarsFormat::BassTrebleBass => "bass_treble",
                    BarsFormat::TrebleBass | BarsFormat::TrebleBassTreble => "treble_bass",
                };

                create_pipeline(
                    device,
                    desc.texture_format,
                    entry_point,
                    fragment_entrypoint,
                )
            };

            let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bars: Left bind group 1"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: freq_buffer.as_entire_binding(),
                }],
            });

            RenderCtx {
                freq_buffer,
                bind_group1,
                pipeline,
            }
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bars: Bind group 0"),
            layout: &left.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fragment_params_buffer.as_entire_binding(),
                },
            ],
        });

        // TODO: Use the same approach as in radial. Looks cleaner
        let right = match &desc.format {
            BarsFormat::TrebleBass | BarsFormat::BassTreble => None,
            f @ (BarsFormat::TrebleBassTreble | BarsFormat::BassTrebleBass) => {
                let freq_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Bars: Right freq buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let entry_point = match f {
                    BarsFormat::TrebleBassTreble => "bass_treble",
                    BarsFormat::BassTrebleBass => "treble_bass",
                    _ => unreachable!(),
                };

                let pipeline = create_pipeline(
                    device,
                    desc.texture_format,
                    entry_point,
                    fragment_entrypoint,
                );

                let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bars: Right bind group 1"),
                    layout: &pipeline.get_bind_group_layout(1),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: freq_buffer.as_entire_binding(),
                    }],
                });

                Some(RenderCtx {
                    freq_buffer,
                    bind_group1,
                    pipeline,
                })
            }
        };

        Ok(Self {
            amount_bars,
            bar_processor,

            bind_group0,
            vertex_params_buffer,
            _fragment_params_buffer: fragment_params_buffer,

            left,
            right,

            angle,
            width,
        })
    }
}

impl Renderable for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        let amount_bars = self.amount_bars.get() as u32;

        pass.set_bind_group(0, &self.bind_group0, &[]);

        // left half
        pass.set_bind_group(1, &self.left.bind_group1, &[]);
        pass.set_pipeline(&self.left.pipeline);
        pass.draw(0..4, 0..amount_bars);

        // right half (if it exists)
        if let Some(right) = &self.right {
            pass.set_bind_group(1, &right.bind_group1, &[]);
            pass.set_pipeline(&right.pipeline);
            pass.draw(0..4, amount_bars..(2 * amount_bars));
        }
    }
}

impl Component for Bars {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(
            &self.left.freq_buffer,
            0,
            bytemuck::cast_slice(&bar_values[0]),
        );

        if let Some(right) = &self.right {
            queue.write_buffer(&right.freq_buffer, 0, bytemuck::cast_slice(&bar_values[1]));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let new_width = new_resolution[0] as f32;
        let new_height = new_resolution[1] as f32;

        // update the column direction vector
        {
            let rotation = Matrix2::from_angle(self.angle);
            let aspect_ratio = new_width / new_height;

            let component_width = match self.width {
                BarsWidth::ScreenWidth => new_width,
                BarsWidth::ScreenHeight => new_height,
                BarsWidth::Custom(custom) => custom.get() as f32,
            };

            let bar_len = component_width / self.amount_bars.get() as f32;
            let mut column_direction = rotation * Vector2::unit_x();

            column_direction = bar_len * column_direction;

            let is_mono_channel = self.right.is_none();
            if is_mono_channel {
                column_direction *= 2.;
            }

            // apply aspect ratio
            column_direction.y *= aspect_ratio;
            column_direction /= new_width;

            let array: [f32; 2] = column_direction.into();

            queue.write_buffer(&self.vertex_params_buffer, 0, bytemuck::cast_slice(&array));
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

fn create_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    vertex_entrypoint: &'static str,
    fragment_entrypoint: FragmentEntrypoint,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(include_wgsl!("./shader.wgsl"));

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Bars: Pipeline layout"),
        bind_group_layouts: &[
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bars: Bind group 0 layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }),
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bars: Bind group 1 layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        ],
        ..Default::default()
    });

    device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
        crate::util::SimpleRenderPipelineDescriptor {
            label: "Bar: Render pipeline",
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some(vertex_entrypoint),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: wgpu::FragmentState {
                module: &module,
                entry_point: Some(fragment_entrypoint.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            },
        },
    ))
}

enum BarsWidth {
    ScreenWidth,
    ScreenHeight,
    Custom(Pixels<u16>),
}
