mod descriptor;

pub use descriptor::*;
use std::{collections::HashMap, num::NonZero};

use cgmath::{Deg, Matrix2, Rad, Vector2};
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, SampleProcessor,
};
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{resource_manager::ResourceManager, Renderable};

use super::Component;

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const BAR_WIDTH: u32 = 0;
    pub const CIRCLE_RADIUS: u32 = 1;
    pub const ASPECT_RATIO: u32 = 2;
    pub const BAR_HEIGHT_SENSITIVITY: u32 = 3;
    pub const POSITION_OFFSET: u32 = 4;

    pub const COLOR1: u32 = 5;
    pub const COLOR2: u32 = 6;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::BarWidth, crate::util::buffer(BAR_WIDTH, wgpu::ShaderStages::VERTEX_FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::CircleRadius, crate::util::buffer(CIRCLE_RADIUS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::AspectRatio, crate::util::buffer(ASPECT_RATIO, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::BarHeightSensitivity, crate::util::buffer(BAR_HEIGHT_SENSITIVITY, wgpu::ShaderStages::VERTEX_FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::PositionOffset, crate::util::buffer(POSITION_OFFSET, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),

            (ResourceID::Color1, crate::util::buffer(COLOR1, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Color2, crate::util::buffer(COLOR2, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

mod bindings1 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const FREQS: u32 = 0;
    pub const ROTATIONS: u32 = 1;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::LeftFreqs, crate::util::buffer( FREQS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::LeftRotations, crate::util::buffer( ROTATIONS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true })),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    BarWidth,
    CircleRadius,
    AspectRatio,
    BarHeightSensitivity,
    PositionOffset,

    RightRotations,
    RightFreqs,

    LeftRotations,
    LeftFreqs,

    Color1,
    Color2,
}

/// Entrypoints for the vertex shader
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

struct PipelineCtx {
    bind_group1: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

pub struct Radial {
    bar_processor: BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    left: PipelineCtx,
    right: Option<PipelineCtx>,

    amount_bars: NonZero<u16>,
}

impl Radial {
    pub fn new<F: Fetcher>(desc: &RadialDescriptor<F>) -> Self {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();

        let bind_group0_mapping = bindings0::init_mapping();

        let (fragment_entrypoint, color1, color2) = match desc.variant {
            RadialVariant::Color(rgba) => ("color_entrypoint", rgba, rgba),
            RadialVariant::HeightGradient { inner, outer } => {
                ("height_gradient_entrypoint", inner, outer)
            }
        };

        resource_manager.extend_buffers([
            (
                ResourceID::BarWidth,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `bar_width` buffer"),
                    contents: bytemuck::bytes_of(&desc.bar_width),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::CircleRadius,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `circle_radius` buffer"),
                    contents: bytemuck::bytes_of(&desc.circle_radius),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::AspectRatio,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `aspect_ratio` buffer"),
                    size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::BarHeightSensitivity,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `bar_height_sensitivity` buffer"),
                    contents: bytemuck::bytes_of(&desc.bar_height_sensitivity),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (ResourceID::PositionOffset, {
                let x_factor = desc.position.0.clamp(0., 1.);
                let y_factor = desc.position.1.clamp(0., 1.);

                let coord_system_origin: Vector2<f32> = Vector2::from((-1., 1.)); // top left in vertex space
                let pos_offset =
                    coord_system_origin + Vector2::from((2. * x_factor, 2. * -y_factor));

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `position_offset` buffer"),
                    contents: bytemuck::cast_slice(&[pos_offset.x, pos_offset.y]),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            }),
            (
                ResourceID::Color1,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `color1` buffer"),
                    contents: bytemuck::cast_slice(&color1),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Color2,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `color2` buffer"),
                    contents: bytemuck::cast_slice(&color2),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
        ]);

        let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

        let fragment_targets = [Some(wgpu::ColorTargetState {
            format: desc.output_texture_format,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::all(),
        })];

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Radial: Bind group 0", device, &bind_group0_mapping);

        // left side of radial
        let (left_entry_point, left_circle_part) = match desc.format {
            RadialFormat::BassTreble => (VertexEntrypoint::BassTreble, CirclePart::Full),
            RadialFormat::TrebleBass => (VertexEntrypoint::TrebleBass, CirclePart::Full),
            // So, in the middle of the circle should be the treble.
            // Regarding the left rotations: The first left rotation should is in the middle of the circle,
            // so we need to start with `Treble` and keep rotating to the left (counter clock wise)
            // which adds the bass bars.
            RadialFormat::BassTrebleBass => (VertexEntrypoint::TrebleBass, CirclePart::Half),
            // same as `RadialFormat::BassTrebleBass`
            RadialFormat::TrebleBassTreble => (VertexEntrypoint::BassTreble, CirclePart::Half),
        };

        resource_manager.extend_buffers([
            (
                ResourceID::LeftFreqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `left freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            {
                let rotations = compute_rotations(
                    left_circle_part,
                    amount_bars,
                    desc.init_rotation,
                    Direction::Ccw,
                );

                (
                    ResourceID::LeftRotations,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `left rotations` buffer"),
                        contents: bytemuck::cast_slice(&rotations),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                )
            },
        ]);

        let left_bind_group_mapping = bindings1::init_mapping();

        let (left_bind_group1, left_bind_group1_layout) = resource_manager.build_bind_group(
            "Radial: `left` bind group 1",
            device,
            &left_bind_group_mapping,
        );

        let left_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Radial: `left` pipeline layout"),
            bind_group_layouts: &[&bind_group0_layout, &left_bind_group1_layout],
            push_constant_ranges: &[],
        });

        let left_pipeline_descriptor =
            crate::util::simple_pipeline_descriptor(crate::util::SimpleRenderPipelineDescriptor {
                label: "Radial: `left` render pipeline",

                // Note:
                // Because of this you can't move the whole logic which is relevant for the left side of the radial circle
                // into a block because this layout needs to live until the right layout has been created (if needed)
                layout: &left_pipeline_layout,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(left_entry_point.as_str()),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fragment_entrypoint),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &fragment_targets,
                },
            });

        let (left, left_pipeline_descriptor) = (
            PipelineCtx {
                bind_group1: left_bind_group1,
                pipeline: device.create_render_pipeline(&left_pipeline_descriptor),
            },
            left_pipeline_descriptor,
        );

        // right half of radial
        let right = {
            let entry_point = match desc.format {
                RadialFormat::BassTrebleBass => Some(VertexEntrypoint::TrebleBass),
                RadialFormat::TrebleBassTreble => Some(VertexEntrypoint::BassTreble),
                RadialFormat::BassTreble | RadialFormat::TrebleBass => None,
            };

            entry_point.map(|entry_point| {
                let rotations = compute_rotations(
                    CirclePart::Half,
                    amount_bars,
                    desc.init_rotation,
                    Direction::Cw,
                );

                resource_manager.extend_buffers([
                    (
                        ResourceID::RightFreqs,
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Radial: `right freqs` buffer"),
                            size: (std::mem::size_of::<f32>() * amount_bars.get() as usize)
                                as wgpu::BufferAddress,
                            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }),
                    ),
                    (
                        ResourceID::RightRotations,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Radial `right rotations` buffer"),
                            contents: bytemuck::cast_slice(&rotations),
                            usage: wgpu::BufferUsages::STORAGE,
                        }),
                    ),
                ]);

                let bind_group_mapping = HashMap::from([
                    (
                        ResourceID::RightFreqs,
                        crate::util::buffer(
                            bindings1::FREQS,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Storage { read_only: true },
                        ),
                    ),
                    (
                        ResourceID::RightRotations,
                        crate::util::buffer(
                            bindings1::ROTATIONS,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Storage { read_only: true },
                        ),
                    ),
                ]);

                let (bind_group1, bind_group1_layout) = resource_manager.build_bind_group(
                    "Radial: `right` bind group 1",
                    device,
                    &bind_group_mapping,
                );

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Radial: `right` pipeline layout"),
                        bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                        push_constant_ranges: &[],
                    });

                let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
                    label: Some("Radial: `right` render pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        entry_point: Some(entry_point.as_str()),
                        ..left_pipeline_descriptor.vertex.clone()
                    },
                    ..left_pipeline_descriptor.clone()
                };

                let pipeline = device.create_render_pipeline(&pipeline_descriptor);

                PipelineCtx {
                    pipeline,
                    bind_group1,
                }
            })
        };

        Self {
            bar_processor,
            resource_manager,

            bind_group0,

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

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::LeftFreqs)
                .unwrap();

            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
        }

        if let Some(buffer) = self.resource_manager.get_buffer(ResourceID::RightFreqs) {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[1]));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::AspectRatio)
                .unwrap();

            let aspect_ratio = new_resolution[0] as f32 / new_resolution[1] as f32;
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&aspect_ratio));
        }
    }
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
