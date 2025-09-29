mod descriptor;

use std::{collections::HashMap, num::NonZero};

use cgmath::{Deg, Matrix2, Vector2};
pub use descriptor::*;

use super::Component;
use crate::{resource_manager::ResourceManager, Renderable, Renderer};
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig,
};
use wgpu::{include_wgsl, util::DeviceExt};

/// Each graph is put inside a box with 4 vertices.
const AMOUNT_VERTICES: u32 = 4;

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

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

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const BOTTOM_LEFT_CORNER: u32 = 0;
    pub const RIGHT: u32 = 1;
    pub const UP: u32 = 2;
    pub const AMOUNT_BARS: u32 = 3;

    pub const COLOR1: u32 = 4;
    pub const COLOR2: u32 = 5;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::BottomLeftCorner, crate::util::buffer(BOTTOM_LEFT_CORNER, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Right, crate::util::buffer(RIGHT, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Up, crate::util::buffer(UP, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::AmountBars, crate::util::buffer(AMOUNT_BARS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),

            (ResourceID::Color1, crate::util::buffer(COLOR1, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Color2, crate::util::buffer(COLOR2, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

mod bindings1 {
    pub const FREQS: u32 = 0;
}

struct PipelineCtx {
    pipeline: wgpu::RenderPipeline,
    bind_group1: wgpu::BindGroup,
    bind_group1_mapping: HashMap<ResourceID, wgpu::BindGroupLayoutEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    BottomLeftCorner,
    Right,
    Up,
    AmountBars,

    Color1,
    Color2,

    Freqs1,
    Freqs2,
}

pub struct Graph {
    bar_processor: vibe_audio::BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    left: PipelineCtx,
    right: Option<PipelineCtx>,

    amount_bars: GraphAmountBars,
    angle: Deg<f32>,
}

impl Graph {
    pub fn new<F: Fetcher>(desc: &GraphDescriptor<F>) -> Self {
        let device = desc.device;

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();

        let amount_bars = match desc.placement {
            GraphPlacement::Bottom | GraphPlacement::Top => GraphAmountBars::ScreenWidth,
            GraphPlacement::Right | GraphPlacement::Left => GraphAmountBars::ScreenHeight,
            GraphPlacement::Custom { amount_bars, .. } => GraphAmountBars::Custom(amount_bars),
        };

        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            BarProcessorConfig {
                amount_bars: amount_bars.get(),
                ..desc.audio_conf.clone()
            },
        );

        let (bottom_left_corner, angle) = match desc.placement {
            GraphPlacement::Bottom => (Vector2::from([-1., -1.]), Deg(0.)),
            GraphPlacement::Right => (Vector2::from([1., -1.]), Deg(90.)),
            GraphPlacement::Top => (Vector2::from([1., 1.]), Deg(180.)),
            GraphPlacement::Left => (Vector2::from([-1., 1.]), Deg(270.)),
            GraphPlacement::Custom {
                bottom_left_corner,
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

                (bottom_left_corner, rotation)
            }
        };
        let rotation = Matrix2::from_angle(angle);

        let (fragment_entrypoint, color1, color2) = match desc.variant {
            GraphVariant::Color(color) => (FragmentEntrypoint::Color, color, color),
            GraphVariant::HorizontalGradient { left, right } => {
                (FragmentEntrypoint::HorizontalGradient, left, right)
            }
            GraphVariant::VerticalGradient { top, bottom } => {
                (FragmentEntrypoint::VerticalGradient, top, bottom)
            }
        };

        resource_manager.extend_buffers([
            (
                ResourceID::BottomLeftCorner,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `bottom_left_corner` buffer"),
                    contents: bytemuck::cast_slice(&[bottom_left_corner.x, bottom_left_corner.y]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
            ),
            (ResourceID::Right, {
                let right = rotation * Vector2::unit_y();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `right` buffer"),
                    contents: bytemuck::cast_slice(&[right.x, right.y]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            }),
            (ResourceID::Up, {
                let mut up = Vector2::unit_y();
                up = rotation * up;
                // stretch the up vector accordingly to the vertex space
                up = up * desc.max_height.clamp(0., 1.) * VERTEX_SURFACE_WIDTH;

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `up` buffer"),
                    contents: bytemuck::cast_slice(&[up.x, up.y]),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            }),
            (
                ResourceID::AmountBars,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `amount_bars` buffer"),
                    contents: bytemuck::bytes_of(&u32::from(amount_bars.get().get())),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
            ),
            (
                ResourceID::Color1,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `color1` buffer"),
                    contents: bytemuck::cast_slice(&color1),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Color2,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `color2` buffer"),
                    contents: bytemuck::cast_slice(&color2),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Freqs1,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs1` buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get().get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let (left_vertex_entrypoint, right_vertex_entrypoint) = match desc.format {
            GraphFormat::BassTreble => (VertexEntrypoint::BassTreble, None),
            GraphFormat::TrebleBass => (VertexEntrypoint::TrebleBass, None),
            GraphFormat::BassTrebleBass => (
                VertexEntrypoint::BassTreble,
                Some(VertexEntrypoint::TrebleBass),
            ),
            GraphFormat::TrebleBassTreble => (
                VertexEntrypoint::TrebleBass,
                Some(VertexEntrypoint::BassTreble),
            ),
        };

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Graph: Bind group 0", device, &bind_group0_mapping);

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));
        let left = {
            let bind_group1_mapping = HashMap::from([(
                ResourceID::Freqs1,
                crate::util::buffer(
                    bindings1::FREQS,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            )]);

            let (bind_group1, bind_group1_layout) = resource_manager.build_bind_group(
                "Graph: Left bind group 1",
                device,
                &bind_group1_mapping,
            );

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Grgaph: Left pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Graph: Left render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some(left_vertex_entrypoint.as_str()),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(fragment_entrypoint.as_str()),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.output_texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ));

            PipelineCtx {
                pipeline,
                bind_group1,
                bind_group1_mapping,
            }
        };

        let right = right_vertex_entrypoint.map(|vertex_entrypoint| {
            resource_manager.insert_buffer(
                ResourceID::Freqs2,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs2` buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get().get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            );

            let bind_group1_mapping = HashMap::from([(
                ResourceID::Freqs2,
                crate::util::buffer(
                    bindings1::FREQS,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            )]);

            let (bind_group1, bind_group1_layout) = resource_manager.build_bind_group(
                "Graph: Right bind group 1",
                device,
                &bind_group1_mapping,
            );

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Graph: Right pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Graph: Right render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some(vertex_entrypoint.as_str()),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(fragment_entrypoint.as_str()),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.output_texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ));

            PipelineCtx {
                pipeline,
                bind_group1,
                bind_group1_mapping,
            }
        });

        Self {
            bar_processor,

            resource_manager,

            bind_group0,

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

        let buffer = self
            .resource_manager
            .get_buffer(ResourceID::Freqs1)
            .unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));

        if let Some(buffer) = self.resource_manager.get_buffer(ResourceID::Freqs2) {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[1]));
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

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::AmountBars)
                .unwrap();

            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&(amount_bars.get() as u32)));
        }

        {
            let buffer = self.resource_manager.get_buffer(ResourceID::Right).unwrap();

            let pixel_width_in_vertex_space =
                1. / (new_resolution[0] as f32 / VERTEX_SURFACE_WIDTH);

            let rotation = Matrix2::from_angle(self.angle);
            let right_dir = rotation * Vector2::new(pixel_width_in_vertex_space, 0.);
            let mut right = amount_bars.get() as f32 * right_dir;

            let renders_two_audio_channel = self.right.is_some();
            if renders_two_audio_channel {
                right /= 2.;
            }

            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[right.x, right.y]));
        }

        self.bar_processor.set_amount_bars(amount_bars);

        let buffer_desc = wgpu::BufferDescriptor {
            label: Some("Graph: `freqs1` buffer"),
            size: (std::mem::size_of::<f32>() * amount_bars.get() as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        // update left `freqs` buffer and bindings
        {
            self.resource_manager
                .replace_buffer(ResourceID::Freqs1, device.create_buffer(&buffer_desc));

            let (bind_group, _layout) = self.resource_manager.build_bind_group(
                "Graph: Left bind group 1",
                device,
                &self.left.bind_group1_mapping,
            );
            self.left.bind_group1 = bind_group;
        }

        // update right `freqs` buffer and bindings
        if let Some(right) = &mut self.right {
            self.resource_manager.replace_buffer(
                ResourceID::Freqs2,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs2` buffer"),
                    ..buffer_desc.clone()
                }),
            );

            let (bind_group, _layout) = self.resource_manager.build_bind_group(
                "Graph: Right bind group 1",
                device,
                &right.bind_group1_mapping,
            );

            right.bind_group1 = bind_group;
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
