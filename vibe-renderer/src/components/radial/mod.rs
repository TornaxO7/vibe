mod descriptor;

pub use descriptor::*;
use std::num::NonZero;

use cgmath::{Matrix2, Rad, SquareMatrix, Vector2};
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
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Rotations,
    InverseRotations,

    BarWidth,
    CircleRadius,
    AspectRatio,
    BarHeightSensitivity,
    PositionOffset,

    Freqs1,
    Freqs2,

    Color1,
    Color2,
}

pub struct Radial {
    bar_processor: BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    left_bind_group1: wgpu::BindGroup,
    right_bind_group1: wgpu::BindGroup,

    left_pipeline: wgpu::RenderPipeline,
    right_pipeline: wgpu::RenderPipeline,

    amount_bars: NonZero<u16>,
}

impl Radial {
    pub fn new<F: Fetcher>(desc: &RadialDescriptor<F>) -> Self {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();

        let bind_group0_mapping = bindings0::init_mapping();
        let mut right_bind_group1_mapping = bindings1::init_mapping();
        let mut left_bind_group1_mapping = bindings1::init_mapping();

        // bar rotation
        {
            let amount_bars = usize::from(u16::from(amount_bars));

            let bar_rotation_radians = Rad(std::f32::consts::PI / amount_bars as f32);
            let center_bars_radians = bar_rotation_radians / 2.;

            let bar_rotation = Matrix2::from_angle(bar_rotation_radians);
            let inverse_bar_rotation = bar_rotation.invert().unwrap();

            let init_rotation =
                Matrix2::from_angle(center_bars_radians) * Matrix2::from_angle(desc.init_rotation);

            let mut rotation = init_rotation;
            let mut inverse_rotation = inverse_bar_rotation * init_rotation;

            let mut bar_rotations = Vec::with_capacity(amount_bars);
            let mut inverse_bar_rotations = Vec::with_capacity(amount_bars);

            for _offset in 0..amount_bars {
                bar_rotations.push(rotation);
                inverse_bar_rotations.push(inverse_rotation);

                rotation = bar_rotation * rotation;
                inverse_rotation = inverse_bar_rotation * inverse_rotation;
            }

            let bar_rotations_as_arrays = bar_rotations
                .iter()
                .cloned()
                .map(|matrix| matrix.into())
                .collect::<Vec<[[f32; 2]; 2]>>();

            let inverse_bar_rotations_as_arrays = inverse_bar_rotations
                .iter()
                .cloned()
                .map(|matrix| matrix.into())
                .collect::<Vec<[[f32; 2]; 2]>>();

            resource_manager.extend_buffers([
                (
                    ResourceID::Rotations,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `rotations` buffer"),
                        contents: bytemuck::cast_slice(&bar_rotations_as_arrays),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                ),
                (
                    ResourceID::InverseRotations,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: inverse `rotations` buffer"),
                        contents: bytemuck::cast_slice(&inverse_bar_rotations_as_arrays),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                ),
            ]);
        }

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

                let coord_system_origin: Vector2<f32> = Vector2::from((-1., 1.));
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
            (
                ResourceID::Freqs1,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `freqs1` buffer"),
                    size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars)))
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Freqs2,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `freqs2` buffer"),
                    size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars)))
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        left_bind_group1_mapping.extend([
            (
                ResourceID::Freqs1,
                crate::util::buffer(
                    bindings1::FREQS,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            ),
            (
                ResourceID::Rotations,
                crate::util::buffer(
                    bindings1::ROTATIONS,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            ),
        ]);

        right_bind_group1_mapping.extend([
            (
                ResourceID::Freqs2,
                crate::util::buffer(
                    bindings1::FREQS,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            ),
            (
                ResourceID::InverseRotations,
                crate::util::buffer(
                    bindings1::ROTATIONS,
                    wgpu::ShaderStages::VERTEX,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
            ),
        ]);

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Radial: Bind group 0", device, &bind_group0_mapping);

        let (left_bind_group1, left_bind_group1_layout) = resource_manager.build_bind_group(
            "Radial: (left) Bind group 1",
            device,
            &left_bind_group1_mapping,
        );

        let (right_bind_group1, right_bind_group1_layout) = resource_manager.build_bind_group(
            "Radial: (right) Bind group 1",
            device,
            &right_bind_group1_mapping,
        );

        let (right_pipeline, left_pipeline) = {
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            let left_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Radial: left pipeline layout"),
                    bind_group_layouts: &[&bind_group0_layout, &left_bind_group1_layout],
                    push_constant_ranges: &[],
                });

            let right_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Radial: right pipeline layout"),
                    bind_group_layouts: &[&bind_group0_layout, &right_bind_group1_layout],
                    push_constant_ranges: &[],
                });

            let fragment_targets = [Some(wgpu::ColorTargetState {
                format: desc.output_texture_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            })];

            let left_descriptor = crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Radial: Left renderpipeline",
                    layout: &left_pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vertex_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(fragment_entrypoint),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &fragment_targets,
                    },
                },
            );

            let right_descriptor = wgpu::RenderPipelineDescriptor {
                label: Some("Radial: Right renderpipeline"),
                layout: Some(&right_pipeline_layout),
                ..left_descriptor.clone()
            };

            let left_pipeline = device.create_render_pipeline(&left_descriptor);
            let right_pipeline = device.create_render_pipeline(&right_descriptor);

            (left_pipeline, right_pipeline)
        };

        Self {
            bar_processor,
            resource_manager,

            bind_group0,

            left_bind_group1,
            right_bind_group1,

            left_pipeline,
            right_pipeline,

            amount_bars,
        }
    }
}

impl Renderable for Radial {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        // render the left half of the circle
        pass.set_bind_group(1, &self.left_bind_group1, &[]);
        pass.set_pipeline(&self.left_pipeline);
        pass.draw(0..4, 0..u32::from(self.amount_bars.get()));

        // render the right half of the circle
        pass.set_bind_group(1, &self.right_bind_group1, &[]);
        pass.set_pipeline(&self.right_pipeline);
        pass.draw(0..4, 0..u32::from(self.amount_bars.get()));
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
                .get_buffer(ResourceID::Freqs1)
                .unwrap();

            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
        }
        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::Freqs2)
                .unwrap();

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
