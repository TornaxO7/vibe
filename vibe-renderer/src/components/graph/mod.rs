mod descriptor;

pub use descriptor::*;

use super::Component;
use crate::{Renderable, Renderer};
use cgmath::{Deg, Matrix2, Vector2};
use std::num::NonZero;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig,
};
use wgpu::{include_wgsl, util::DeviceExt};

/// Each graph is put inside a box with 4 vertices.
const AMOUNT_VERTICES: u32 = 4;

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

#[derive(Debug, Clone, Copy)]
enum VertexEntrypoint {
    BassTreble,
    TrebleBass,
}

impl VertexEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            VertexEntrypoint::BassTreble => "bass_treble",
            VertexEntrypoint::TrebleBass => "treble_bass",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FragmentEntrypoint {
    Color,
    HorizontalGradient,
    VerticalGradient,
}

impl FragmentEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            FragmentEntrypoint::Color => "color",
            FragmentEntrypoint::HorizontalGradient => "horizontal_gradient",
            FragmentEntrypoint::VerticalGradient => "vertical_gradient",
        }
    }
}

struct PipelineCtx {
    pipeline: wgpu::RenderPipeline,

    bind_group1: wgpu::BindGroup,
    freqs_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {
    right: [f32; 2],
    bottom_left_corner: [f32; 2],
    up: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    color1: [f32; 4],
    color2: [f32; 4],
}

pub struct Graph {
    bar_processor: vibe_audio::BarProcessor,

    bind_group0: wgpu::BindGroup,
    vertex_params_buffer: wgpu::Buffer,
    _fragment_params_buffer: wgpu::Buffer,

    left: PipelineCtx,
    right: Option<PipelineCtx>,

    // data used to update the values inside the buffers
    amount_bars: GraphAmountBars,
    angle: Deg<f32>,
}

impl Graph {
    pub fn new<F: Fetcher>(desc: &GraphDescriptor<F>) -> Self {
        let device = desc.device;

        let amount_bars = match desc.placement {
            GraphPlacement::Bottom | GraphPlacement::Top => GraphAmountBars::ScreenWidth,
            GraphPlacement::Right | GraphPlacement::Left => GraphAmountBars::ScreenHeight,
            GraphPlacement::Custom { amount_bars, .. } => GraphAmountBars::Custom(amount_bars),
        };

        let angle = match desc.placement {
            GraphPlacement::Bottom => Deg(0.),
            GraphPlacement::Right => Deg(90.),
            GraphPlacement::Top => Deg(180.),
            GraphPlacement::Left => Deg(270.),
            GraphPlacement::Custom { rotation, .. } => rotation,
        };

        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            BarProcessorConfig {
                amount_bars: amount_bars.get(),
                ..desc.audio_conf.clone()
            },
        );

        let vertex_params_buffer = {
            let bottom_left_corner = match desc.placement {
                GraphPlacement::Bottom => Vector2::from([-1., -1.]),
                GraphPlacement::Right => Vector2::from([1., -1.]),
                GraphPlacement::Top => Vector2::from([1., 1.]),
                GraphPlacement::Left => Vector2::from([-1., 1.]),
                GraphPlacement::Custom {
                    bottom_left_corner, ..
                } => {
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
                }
            };

            let rotation = Matrix2::from_angle(angle);

            let right = rotation * Vector2::unit_y();
            let up = {
                let mut up = Vector2::unit_y();
                up = rotation * up;
                // stretch the up vector accordingly to the vertex space
                up * desc.max_height.clamp(0., 1.) * VERTEX_SURFACE_WIDTH
            };

            let vertex_params = VertexParams {
                bottom_left_corner: Into::<[f32; 2]>::into(bottom_left_corner),
                right: Into::<[f32; 2]>::into(right),
                up: Into::<[f32; 2]>::into(up),
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Graph: Vertex params buffer"),
                contents: bytemuck::bytes_of(&vertex_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        let fragment_params_buffer = {
            let (color1, color2) = match desc.variant {
                GraphVariant::Color(color) => (color, color),
                GraphVariant::HorizontalGradient { left, right } => (left, right),
                GraphVariant::VerticalGradient { top, bottom } => (top, bottom),
            };

            let fragment_params = FragmentParams { color1, color2 };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Graph: Fragment params buffer"),
                contents: bytemuck::bytes_of(&fragment_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        let fragment_entrypoint = match desc.variant {
            GraphVariant::Color(_) => FragmentEntrypoint::Color,
            GraphVariant::HorizontalGradient { .. } => FragmentEntrypoint::HorizontalGradient,
            GraphVariant::VerticalGradient { .. } => FragmentEntrypoint::VerticalGradient,
        };

        let left = {
            let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Graph: Left freqs buffer"),
                size: (std::mem::size_of::<f32>() * amount_bars.get().get() as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let vertex_entrypoint = match desc.format {
                GraphFormat::BassTreble | GraphFormat::BassTrebleBass => {
                    VertexEntrypoint::BassTreble
                }
                GraphFormat::TrebleBass | GraphFormat::TrebleBassTreble => {
                    VertexEntrypoint::TrebleBass
                }
            };

            let pipeline = create_pipeline(
                device,
                desc.output_texture_format,
                vertex_entrypoint,
                fragment_entrypoint,
            );

            let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Graph: Left bind group 1"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: freqs_buffer.as_entire_binding(),
                }],
            });

            PipelineCtx {
                pipeline,
                bind_group1,
                freqs_buffer,
            }
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Graph: Bind group 0"),
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

        let right = {
            let vertex_entrypoint = match desc.format {
                GraphFormat::BassTreble | GraphFormat::TrebleBass => None,
                GraphFormat::BassTrebleBass => Some(VertexEntrypoint::TrebleBass),
                GraphFormat::TrebleBassTreble => Some(VertexEntrypoint::BassTreble),
            };

            vertex_entrypoint.map(|vertex_entrypoint| {
                let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: Right freqs buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get().get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let pipeline = create_pipeline(
                    device,
                    desc.output_texture_format,
                    vertex_entrypoint,
                    fragment_entrypoint,
                );

                let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Graph: Right bind group 1"),
                    layout: &pipeline.get_bind_group_layout(1),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: freqs_buffer.as_entire_binding(),
                    }],
                });

                PipelineCtx {
                    pipeline,
                    bind_group1,
                    freqs_buffer,
                }
            })
        };

        Self {
            bar_processor,

            bind_group0,
            vertex_params_buffer,
            _fragment_params_buffer: fragment_params_buffer,

            left,
            right,

            amount_bars,
            angle,
        }
    }
}

impl Renderable for Graph {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        pass.set_pipeline(&self.left.pipeline);
        pass.set_bind_group(1, &self.left.bind_group1, &[]);
        pass.draw(0..AMOUNT_VERTICES, 0..1);

        if let Some(right) = &self.right {
            pass.set_pipeline(&right.pipeline);
            pass.set_bind_group(1, &right.bind_group1, &[]);
            pass.draw(0..AMOUNT_VERTICES, 1..2);
        }
    }
}

impl Component for Graph {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &vibe_audio::SampleProcessor<SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(
            &self.left.freqs_buffer,
            0,
            bytemuck::cast_slice(&bar_values[0]),
        );

        if let Some(right) = &self.right {
            queue.write_buffer(&right.freqs_buffer, 0, bytemuck::cast_slice(&bar_values[1]));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let device = renderer.device();

        let amount_bars = match self.amount_bars {
            GraphAmountBars::ScreenWidth => NonZero::new(new_resolution[0] as u16).unwrap(),
            GraphAmountBars::ScreenHeight => NonZero::new(new_resolution[1] as u16).unwrap(),
            GraphAmountBars::Custom(amount) => amount,
        };

        // update `right` vector
        {
            let pixel_width_in_vertex_space =
                1. / (new_resolution[0] as f32 / VERTEX_SURFACE_WIDTH);

            let rotation = Matrix2::from_angle(self.angle);
            let right_dir = rotation * Vector2::new(pixel_width_in_vertex_space, 0.);
            let mut right = amount_bars.get() as f32 * right_dir;

            let renders_two_audio_channel = self.right.is_some();
            if renders_two_audio_channel {
                right /= 2.;
            }

            queue.write_buffer(
                &self.vertex_params_buffer,
                0,
                bytemuck::cast_slice(&[right.x, right.y]),
            );
        }

        self.bar_processor.set_amount_bars(amount_bars);

        let buffer_desc = wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<f32>() * amount_bars.get() as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        // update left `freqs` buffer and bindings
        {
            let new_freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Graph: Left freqs buffer"),
                ..buffer_desc
            });

            let new_bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Graph: Left bind group 1"),
                layout: &self.left.pipeline.get_bind_group_layout(1),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: new_freqs_buffer.as_entire_binding(),
                }],
            });

            self.left.freqs_buffer = new_freqs_buffer;
            self.left.bind_group1 = new_bind_group1;
        }

        // update right `freqs` buffer and bindings
        if let Some(right) = &mut self.right {
            let new_freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Graph: Right freqs buffer"),
                ..buffer_desc
            });

            let new_bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Graph: Right bind group 1"),
                layout: &self.left.pipeline.get_bind_group_layout(1),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: new_freqs_buffer.as_entire_binding(),
                }],
            });

            right.freqs_buffer = new_freqs_buffer;
            right.bind_group1 = new_bind_group1;
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

enum GraphAmountBars {
    ScreenWidth,
    ScreenHeight,
    // In Pixels
    Custom(NonZero<u16>),
}

impl GraphAmountBars {
    const DEFAULT_AMOUNT_BARS: NonZero<u16> = NonZero::new(128).unwrap();

    fn get(&self) -> NonZero<u16> {
        match self {
            GraphAmountBars::ScreenWidth | GraphAmountBars::ScreenHeight => {
                Self::DEFAULT_AMOUNT_BARS
            }
            GraphAmountBars::Custom(non_zero) => *non_zero,
        }
    }
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
                ],
            }),
            &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bars: Bind group 1 layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            },
        },
    ))
}
