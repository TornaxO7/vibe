mod descriptor;

use std::{collections::HashMap, num::NonZero};

use cgmath::{Deg, Matrix, Matrix2};
pub use descriptor::*;

use super::Component;
use crate::{
    resource_manager::ResourceManager, util::SimpleRenderPipelineDescriptor, Renderable, Renderer,
};
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig,
};
use wgpu::{include_wgsl, util::DeviceExt};

type VertexPosition = [f32; 2];
const POSITIONS: [VertexPosition; 3] = [
    [1., 1.],  // top right
    [1., -3.], // right bottom corner
    [-3., 1.], // top left corner
];

/// This value is only used for the placements `TOP`, `BOTTOM`, `RIGHT` and `LEFT` since
/// the amount of bars needs to adjust to the screen height/width anyhow.
const DEFAULT_AMOUNT_BARS: NonZero<u16> = NonZero::new(128).unwrap();

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const OFFSET: u32 = 1;
    pub const ROTATION: u32 = 2;
    pub const MAX_HEIGHT: u32 = 3;
    pub const COLOR1: u32 = 4;
    pub const COLOR2: u32 = 5;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Offset, crate::util::buffer(OFFSET, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Rotation, crate::util::buffer(ROTATION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::MaxHeight, crate::util::buffer(MAX_HEIGHT, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Color1, crate::util::buffer(COLOR1, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Color2, crate::util::buffer(COLOR2, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
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
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    Offset,
    Rotation,
    MaxHeight,
    Color1,
    Color2,

    Freqs,
}

pub struct Graph {
    placement: GraphPlacement,
    bar_processor: vibe_audio::BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    bind_group1_mapping: HashMap<ResourceID, wgpu::BindGroupLayoutEntry>,

    amount_bars: NonZero<u16>,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Graph {
    pub fn new<F: Fetcher>(desc: &GraphDescriptor<F>) -> Self {
        let device = desc.device;

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        let (offset, rotation, amount_bars) = {
            let (offset, deg, amount_bars) = match desc.placement {
                GraphPlacement::Top => ([0., 0.], Deg(0.), DEFAULT_AMOUNT_BARS),
                GraphPlacement::Left => ([0., 1.], Deg(90.), DEFAULT_AMOUNT_BARS),
                GraphPlacement::Bottom => ([1., 1.], Deg(180.), DEFAULT_AMOUNT_BARS),
                GraphPlacement::Right => ([1., 0.], Deg(270.), DEFAULT_AMOUNT_BARS),
                GraphPlacement::Custom {
                    offset,
                    rotation,
                    amount_bars,
                } => (offset, rotation, amount_bars),
            };

            (offset, Matrix2::from_angle(-deg), amount_bars)
        };
        let bar_processor = {
            let audio_conf = BarProcessorConfig {
                amount_bars,
                ..desc.audio_conf.clone()
            };
            BarProcessor::new(desc.sample_processor, audio_conf)
        };

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Graph: vertex buffer"),
            contents: bytemuck::cast_slice(&POSITIONS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (fragment_entrypoint, color1, color2) = match desc.variant {
            GraphVariant::Color(color) => ("color", color, color),
            GraphVariant::HorizontalGradient { left, right } => {
                ("horizontal_gradient", left, right)
            }
            GraphVariant::VerticalGradient { top, bottom } => ("vertical_gradient", top, bottom),
        };

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `iResolution` buffer"),
                    size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Offset,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `offset` buffer"),
                    contents: bytemuck::cast_slice(&offset),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            {
                // we need to transpose, because otherwise `.as_ref()` would return columns first
                let t = rotation.transpose();
                let array: &[f32; 4] = t.as_ref();

                (
                    ResourceID::Rotation,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `rotation` buffer"),
                        contents: bytemuck::cast_slice(array),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                )
            },
            (
                ResourceID::MaxHeight,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `max_height` buffer"),
                    contents: bytemuck::bytes_of(&desc.max_height),
                    usage: wgpu::BufferUsages::UNIFORM,
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
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * desc.audio_conf.amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Graph: fragment shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./fragment.wgsl").into()),
        });

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Graph: Bind group 0", device, &bind_group0_mapping);

        let (bind_group1, bind_group1_layout) =
            resource_manager.build_bind_group("Graph: Bind group 1", device, &bind_group1_mapping);

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Graph: Pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let vertex_shader = device.create_shader_module(include_wgsl!("./vertex_shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                SimpleRenderPipelineDescriptor {
                    label: "Graph: Render pipeline`",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_shader,
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<VertexPosition>()
                                as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: (wgpu::FragmentState {
                        module: &fragment_shader,
                        entry_point: Some(fragment_entrypoint),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.output_texture_format,
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                },
            ))
        };

        Self {
            placement: desc.placement,
            bar_processor,
            vbuffer,
            pipeline,

            resource_manager,

            bind_group0,
            bind_group1,

            bind_group1_mapping,

            amount_bars,
        }
    }
}

impl Renderable for Graph {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl Component for Graph {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &vibe_audio::SampleProcessor<SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let device = renderer.device();

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::Resolution)
                .unwrap();

            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }

        {
            self.amount_bars = {
                let amount_bars = match self.placement {
                    GraphPlacement::Bottom | GraphPlacement::Top => new_resolution[0],
                    GraphPlacement::Left | GraphPlacement::Right => new_resolution[1],
                    GraphPlacement::Custom { .. } => self.amount_bars.get() as u32,
                };

                NonZero::new(amount_bars as u16).unwrap()
            };

            self.bar_processor.set_amount_bars(self.amount_bars);

            self.resource_manager.replace_buffer(
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * self.amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            );

            let (bind_group, _layout) = self.resource_manager.build_bind_group(
                "Graph: Bind group 1",
                device,
                &self.bind_group1_mapping,
            );

            self.bind_group1 = bind_group;
        }
    }
}
