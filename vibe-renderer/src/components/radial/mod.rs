use std::num::NonZero;

use cgmath::{Deg, Matrix2, Rad, SquareMatrix, Vector2};
use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::{Component, Rgba};

#[repr(u32)]
enum Binding0 {
    BarRotation = 0,
    InverseBarRotation = 1,
    BarWidth = 2,
    CircleRadius = 3,
    Resolution = 4,
    BarHeightSensitivity = 5,

    Color = 6,
    PositionOffset = 7,
}

#[repr(u32)]
enum Binding1 {
    Freqs = 0,
}

pub enum RadialVariant {
    Color(Rgba),
}

pub struct RadialDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub processor: &'a SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub output_texture_format: wgpu::TextureFormat,
    pub variant: RadialVariant,

    pub init_rotation: Deg<f32>,
    pub circle_radius: f32,
    pub bar_height_sensitivity: f32,
    pub bar_width: f32,
    // [0, 0]: top left corner
    // [1, 1]: bottom right corner
    pub position: (f32, f32),
}

pub struct Radial {
    bar_processor: BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    pipeline: wgpu::RenderPipeline,
    pipeline_inverted: wgpu::RenderPipeline,

    amount_bars: NonZero<u16>,
}

impl Radial {
    pub fn new(desc: &RadialDescriptor) -> Self {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.processor, desc.audio_conf.clone());
        let mut bind_group0 = BindGroupManager::new(Some("Radial: `Bind group 0`"));
        let mut bind_group1 = BindGroupManager::new(Some("Radial: `Bind group 1`"));

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

            bind_group0.insert_buffer(
                Binding0::BarRotation as u32,
                wgpu::ShaderStages::VERTEX,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `bar_rotation` buffer"),
                    contents: bytemuck::cast_slice(&bar_rotations_as_arrays),
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            );

            bind_group0.insert_buffer(
                Binding0::InverseBarRotation as u32,
                wgpu::ShaderStages::VERTEX,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `inverse_bar_rotation` buffer"),
                    contents: bytemuck::cast_slice(&inverse_bar_rotations_as_arrays),
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            );
        }

        bind_group0.insert_buffer(
            Binding0::BarWidth as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: `bar_width` buffer"),
                contents: bytemuck::bytes_of(&desc.bar_width),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Binding0::CircleRadius as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: `circle_radius` buffer"),
                contents: bytemuck::bytes_of(&desc.circle_radius),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Binding0::Resolution as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Radial: `iResolution` buffer"),
                size: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        bind_group0.insert_buffer(
            Binding0::BarHeightSensitivity as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Radial: `bar_height_sensitivity` buffer"),
                contents: bytemuck::bytes_of(&desc.bar_height_sensitivity),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        {
            let x_factor = desc.position.0.clamp(0., 1.);
            let y_factor = desc.position.1.clamp(0., 1.);

            let coord_system_origin: Vector2<f32> = Vector2::from((-1., 1.));
            let pos_offset = coord_system_origin + Vector2::from((2. * x_factor, 2. * -y_factor));

            bind_group0.insert_buffer(
                Binding0::PositionOffset as u32,
                wgpu::ShaderStages::VERTEX,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Radial: `position_offset` buffer"),
                    contents: bytemuck::cast_slice(&[pos_offset.x, pos_offset.y]),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            );
        }

        match desc.variant {
            RadialVariant::Color(rgba) => {
                bind_group0.insert_buffer(
                    Binding0::Color as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Radial: `color` buffer"),
                        contents: bytemuck::bytes_of(&rgba),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );
            }
        };

        bind_group1.insert_buffer(
            Binding1::Freqs as u32,
            wgpu::ShaderStages::VERTEX,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Radial: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars)))
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        bind_group0.build_bind_group(device);
        bind_group1.build_bind_group(device);

        let (pipeline, pipeline_inverted) = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Radial: Pipelinelayout"),
                bind_group_layouts: &[
                    &bind_group0.get_bind_group_layout(device),
                    &bind_group1.get_bind_group_layout(device),
                ],
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
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);

        // render the bars of the first half of the circle
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..u32::from(u16::from(self.amount_bars)));

        // render the bars of the other half of the circle
        pass.set_pipeline(&self.pipeline_inverted);
        pass.draw(0..4, 0..u32::from(u16::from(self.amount_bars)));
    }
}

impl Component for Radial {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor) {
        let buffer = self.bind_group1.get_buffer(Binding1::Freqs as u32).unwrap();
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let buffer = self
            .bind_group0
            .get_buffer(Binding0::Resolution as u32)
            .unwrap();
        queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }
}
