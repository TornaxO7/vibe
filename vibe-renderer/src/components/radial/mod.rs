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

    pub const BAR_ROTATION: u32 = 0;
    pub const INVERSE_BAR_ROTATION: u32 = 1;
    pub const BAR_WIDTH: u32 = 2;
    pub const CIRCLE_RADIUS: u32 = 3;
    pub const RESOLUTION: u32 = 4;
    pub const BAR_HEIGHT_SENSITIVITY: u32 = 5;

    pub const COLOR: u32 = 6;
    pub const POSITION_OFFSET: u32 = 7;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::BarRotation, crate::util::buffer(BAR_ROTATION, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::InverseBarRotation, crate::util::buffer(INVERSE_BAR_ROTATION, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::BarWidth, crate::util::buffer(BAR_WIDTH, wgpu::ShaderStages::VERTEX_FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::CircleRadius, crate::util::buffer(CIRCLE_RADIUS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::BarHeightSensitivity, crate::util::buffer(BAR_HEIGHT_SENSITIVITY, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
            (ResourceID::PositionOffset, crate::util::buffer(POSITION_OFFSET, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

mod bindings1 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const FREQS: u32 = 0;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true })),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    BarRotation,
    InverseBarRotation,
    BarWidth,
    CircleRadius,
    Resolution,
    BarHeightSensitivity,

    Color,
    PositionOffset,

    Freqs,
}

pub struct Radial {
    bar_processor: BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    pipeline_inverted: wgpu::RenderPipeline,

    amount_bars: NonZero<u16>,
}

impl Radial {
    pub fn new<F: Fetcher>(desc: &RadialDescriptor<F>) -> Self {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();

        let mut bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

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
                    ResourceID::BarRotation,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `bar_rotation` buffer"),
                        contents: bytemuck::cast_slice(&bar_rotations_as_arrays),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                ),
                (
                    ResourceID::InverseBarRotation,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `inverse_bar_rotation` buffer"),
                        contents: bytemuck::cast_slice(&inverse_bar_rotations_as_arrays),
                        usage: wgpu::BufferUsages::STORAGE,
                    }),
                ),
            ]);
        }

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
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `iResolution` buffer"),
                    size: (std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
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
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Radial: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars)))
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        {
            let x_factor = desc.position.0.clamp(0., 1.);
            let y_factor = desc.position.1.clamp(0., 1.);

            let coord_system_origin: Vector2<f32> = Vector2::from((-1., 1.));
            let pos_offset = coord_system_origin + Vector2::from((2. * x_factor, 2. * -y_factor));

            resource_manager.insert_buffer(
                ResourceID::PositionOffset,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `position_offset` buffer"),
                    contents: bytemuck::cast_slice(&[pos_offset.x, pos_offset.y]),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            );
        }

        match desc.variant {
            RadialVariant::Color(rgba) => {
                resource_manager.insert_buffer(
                    ResourceID::Color,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `color` buffer"),
                        contents: bytemuck::bytes_of(&rgba),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0_mapping.insert(
                    ResourceID::Color,
                    crate::util::buffer(
                        bindings0::COLOR,
                        wgpu::ShaderStages::FRAGMENT,
                        wgpu::BufferBindingType::Uniform,
                    ),
                );
            }
        };

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Radial: Bind group 0", device, &bind_group0_mapping);

        let (bind_group1, bind_group1_layout) =
            resource_manager.build_bind_group("Radial: Bind group 1", device, &bind_group1_mapping);

        let (pipeline, pipeline_inverted) = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Radial: Pipelinelayout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));
            let fragment_targets = [Some(wgpu::ColorTargetState {
                format: desc.output_texture_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::all(),
            })];

            let descriptor = crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Radial: Renderpipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vertex_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &fragment_targets,
                    },
                },
            );

            let inverse_descriptor = wgpu::RenderPipelineDescriptor {
                label: Some("Radial: Pipeline for other half of circle"),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main_inverted"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                ..descriptor.clone()
            };

            let pipeline = device.create_render_pipeline(&descriptor);
            let pipeline_inverted = device.create_render_pipeline(&inverse_descriptor);

            (pipeline, pipeline_inverted)
        };

        Self {
            bar_processor,

            resource_manager,

            bind_group0,
            bind_group1,

            pipeline,
            pipeline_inverted,

            amount_bars,
        }
    }
}

impl Renderable for Radial {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);

        // render the bars of the first half of the circle
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..u32::from(self.amount_bars.get()));

        // render the bars of the other half of the circle
        pass.set_pipeline(&self.pipeline_inverted);
        pass.draw(0..4, 0..u32::from(self.amount_bars.get()));
    }
}

impl Component for Radial {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::Resolution)
                .unwrap();

            let width = new_resolution[0] as f32;
            let height = new_resolution[1] as f32;
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[width, height, width / height]),
            );
        }
    }
}
