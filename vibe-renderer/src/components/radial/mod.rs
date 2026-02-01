mod descriptor;

pub use descriptor::*;

use super::{Component, Rgba, Vec2f};
use crate::Renderable;
use cgmath::{Deg, Matrix2, Rad, Vector2};
use std::num::NonZero;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, SampleProcessor,
};
use wgpu::{include_wgsl, util::DeviceExt};

/// Entrypoints for the vertex shader
#[derive(Clone, Copy)]
enum VertexEntrypoint {
    BassTreble,
    TrebleBass,
}

impl VertexEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            Self::BassTreble => "bass_treble",
            Self::TrebleBass => "treble_bass",
        }
    }
}

/// Entrypoints for the fragment shader
#[derive(Clone, Copy)]
enum FragmentEntrypoint {
    Color,
    HeightGradient,
}

impl FragmentEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Color => "fs_color",
            Self::HeightGradient => "fs_height_gradient",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CirclePart {
    Half,
    Full,
}

impl CirclePart {
    pub fn radians(&self) -> f32 {
        match self {
            Self::Half => std::f32::consts::PI,
            Self::Full => std::f32::consts::PI * 2f32,
        }
    }
}

type PositionOffset = Vec2f;
type CircleRadius = f32;
type AspectRatio = f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {
    position_offset: PositionOffset,
    circle_radius: CircleRadius,
    aspect_ratio: AspectRatio,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    color1: Rgba,
    color2: Rgba,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexFragmentParams {
    bar_width: f32,
    bar_height_sensitivity: f32,
}

struct PipelineCtx {
    bind_group1: wgpu::BindGroup,
    freq_buffer: wgpu::Buffer,
    _rotations_buffer: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,
}

pub struct Radial {
    bar_processor: BarProcessor,

    bind_group0: wgpu::BindGroup,
    vertex_params_buffer: wgpu::Buffer,
    _fragment_params_buffer: wgpu::Buffer,
    _vertex_fragment_params_buffer: wgpu::Buffer,

    left: PipelineCtx,
    right: Option<PipelineCtx>,

    amount_bars: NonZero<u16>,
}

impl Radial {
    pub fn new<F: Fetcher>(desc: &RadialDescriptor<F>) -> Self {
        let device = desc.renderer.device();
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.processor, desc.audio_conf.clone());

        let vertex_params_buffer = {
            let position_offset = {
                let x_factor = desc.position.0.clamp(0., 1.);
                let y_factor = desc.position.1.clamp(0., 1.);

                let coord_system_origin: Vector2<f32> = Vector2::from((-1., 1.)); // top left in vertex space
                coord_system_origin + Vector2::from((2. * x_factor, 2. * -y_factor))
            };

            let vertex_params = VertexParams {
                position_offset: position_offset.into(),
                circle_radius: desc.circle_radius,
                aspect_ratio: 0.,
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: Vertex params buffer"),
                contents: bytemuck::bytes_of(&vertex_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        let fragment_params_buffer = {
            let (color1, color2) = match desc.variant {
                RadialVariant::Color(rgba) => (rgba, rgba),
                RadialVariant::HeightGradient { inner, outer } => (inner, outer),
            };

            let fragment_params = FragmentParams { color1, color2 };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: Fragment params buffer"),
                contents: bytemuck::bytes_of(&fragment_params),
                usage: wgpu::BufferUsages::UNIFORM,
            })
        };

        let vertex_fragment_params_buffer = {
            let vertex_fragment_params = VertexFragmentParams {
                bar_height_sensitivity: desc.bar_height_sensitivity,
                bar_width: desc.bar_width,
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: Vertex Fragment buffer"),
                contents: bytemuck::bytes_of(&vertex_fragment_params),
                usage: wgpu::BufferUsages::UNIFORM,
            })
        };

        let fragment_entrypoint = match desc.variant {
            RadialVariant::Color(_) => FragmentEntrypoint::Color,
            RadialVariant::HeightGradient { .. } => FragmentEntrypoint::HeightGradient,
        };

        let circle_part = match desc.format {
            RadialFormat::BassTreble | RadialFormat::TrebleBass => CirclePart::Full,
            RadialFormat::TrebleBassTreble | RadialFormat::BassTrebleBass => CirclePart::Half,
        };

        let left = {
            let vertex_entry_point = match desc.format {
                RadialFormat::BassTreble => VertexEntrypoint::BassTreble,
                RadialFormat::TrebleBass => VertexEntrypoint::TrebleBass,
                // So, in the middle of the circle should be the treble.
                // Regarding the left rotations: The first left rotation should is in the middle of the circle,
                // so we need to start with `Treble` and keep rotating to the left (counter clock wise)
                // which adds the bass bars.
                RadialFormat::BassTrebleBass => VertexEntrypoint::TrebleBass,
                // same as `RadialFormat::BassTrebleBass`
                RadialFormat::TrebleBassTreble => VertexEntrypoint::BassTreble,
            };

            let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Radial: Left freq buffer"),
                size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let rotations_buffer = {
                let rotations =
                    compute_rotations(circle_part, amount_bars, desc.init_rotation, Direction::Ccw);

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `left rotations` buffer"),
                    contents: bytemuck::cast_slice(&rotations),
                    usage: wgpu::BufferUsages::STORAGE,
                })
            };

            let pipeline = create_pipeline(
                device,
                desc.output_texture_format,
                vertex_entry_point,
                fragment_entrypoint,
            );

            let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Radial: Left bind group 1"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: freqs_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: rotations_buffer.as_entire_binding(),
                    },
                ],
            });

            PipelineCtx {
                bind_group1,
                freq_buffer: freqs_buffer,
                _rotations_buffer: rotations_buffer,
                pipeline,
            }
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radial: Bind group 0"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: vertex_fragment_params_buffer.as_entire_binding(),
                },
            ],
        });

        // right half of radial
        let right = {
            let vertex_entry_point = match desc.format {
                RadialFormat::BassTrebleBass => Some(VertexEntrypoint::TrebleBass),
                RadialFormat::TrebleBassTreble => Some(VertexEntrypoint::BassTreble),
                RadialFormat::BassTreble | RadialFormat::TrebleBass => None,
            };

            vertex_entry_point.map(|vertex_entry_point| {
                let freq_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: Right freq buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                debug_assert_eq!(circle_part, CirclePart::Half, concat![
                    "`right`s only task is to render the second (circle) half if there needs to be two audio channels to be rendered!\n",
                    "So `left` should also render half of the circle (which means `circle_part` should be `CirclePart::Half`)!"
                ]);
                let rotations_buffer = {
                    let rotations = compute_rotations(
                        circle_part,
                        amount_bars,
                        desc.init_rotation,
                        Direction::Cw,
                    );

                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `right rotations` buffer"),
                        contents: bytemuck::cast_slice(&rotations),
                        usage: wgpu::BufferUsages::STORAGE,
                    })
                };

                let pipeline = create_pipeline(
                    device,
                    desc.output_texture_format,
                    vertex_entry_point,
                    fragment_entrypoint,
                );

                let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Radial:Right bind group 1"),
                    layout: &pipeline.get_bind_group_layout(1),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: freq_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: rotations_buffer.as_entire_binding(),
                        },
                    ],
                });

                PipelineCtx {
                    pipeline,

                    bind_group1,
                    freq_buffer,
                    _rotations_buffer: rotations_buffer,
                }
            })
        };

        Self {
            bar_processor,

            bind_group0,
            vertex_params_buffer,
            _fragment_params_buffer: fragment_params_buffer,
            _vertex_fragment_params_buffer: vertex_fragment_params_buffer,

            left,
            right,

            amount_bars,
        }
    }
}

impl Renderable for Radial {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        // render the left half of the circle
        pass.set_bind_group(1, &self.left.bind_group1, &[]);
        pass.set_pipeline(&self.left.pipeline);
        pass.draw(0..4, 0..u32::from(self.amount_bars.get()));

        // render the right half of the circle
        if let Some(right) = &self.right {
            pass.set_bind_group(1, &right.bind_group1, &[]);
            pass.set_pipeline(&right.pipeline);
            pass.draw(0..4, 0..u32::from(self.amount_bars.get()));
        }
    }
}

impl Component for Radial {
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

        {
            let aspect_ratio = new_resolution[0] as f32 / new_resolution[1] as f32;

            let offset =
                std::mem::size_of::<PositionOffset>() + std::mem::size_of::<CircleRadius>();

            queue.write_buffer(
                &self.vertex_params_buffer,
                offset as wgpu::BufferAddress,
                bytemuck::bytes_of(&aspect_ratio),
            );
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    // Clock wise (for left side)
    Cw,
    // Counter-clock wise (for right side)
    Ccw,
}

fn compute_rotations(
    circle_part: CirclePart,
    amount_bars: NonZero<u16>,
    init_rotation_deg: Deg<f32>,
    dir: Direction,
) -> Box<[[f32; 4]]> {
    let bar_rotation_radians = {
        let sign = match dir {
            Direction::Cw => -1f32,
            Direction::Ccw => 1f32,
        };
        Rad(sign * circle_part.radians() / amount_bars.get() as f32)
    };

    // example: Assuming `amount_bars` is `1`, we don't want to let the bar be at radiant `PI`, it should be at `PI/2` instead
    let center_bars_radians = bar_rotation_radians / 2.;

    let bar_rotation = Matrix2::from_angle(bar_rotation_radians);

    let init_rotation =
        Matrix2::from_angle(center_bars_radians) * Matrix2::from_angle(init_rotation_deg);

    let mut rotation = init_rotation;
    let mut rotations = Vec::with_capacity(amount_bars.get() as usize);

    for _offset in 0..amount_bars.get() {
        let rotation_as_array = *<Matrix2<f32> as AsRef<[f32; 4]>>::as_ref(&rotation);
        rotations.push(rotation_as_array);
        rotation = bar_rotation * rotation;
    }

    rotations.into_boxed_slice()
}

fn create_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    vertex_entrypoint: VertexEntrypoint,
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
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
                entry_point: Some(vertex_entrypoint.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: wgpu::FragmentState {
                module: &module,
                entry_point: Some(fragment_entrypoint.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            },
        },
    ))
}
