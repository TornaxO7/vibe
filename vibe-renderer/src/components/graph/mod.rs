mod descriptor;

pub use descriptor::*;

use super::Component;
use crate::{
    resource_manager::ResourceManager, util::SimpleRenderPipelineDescriptor, Renderable, Renderer,
};
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor,
};
use wgpu::{include_wgsl, util::DeviceExt};

type VertexPosition = [f32; 2];
const POSITIONS: [VertexPosition; 3] = [
    [1., 1.],  // top right
    [1., -3.], // right bottom corner
    [-3., 1.], // top left corner
];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const MAX_HEIGHT: u32 = 1;
    pub const COLOR: u32 = 2;
    pub const HORIZONTAL_GRADIENT_LEFT: u32 = 3;
    pub const HORIZONTAL_GRADIENT_RIGHT: u32 = 4;

    pub const VERTICAL_GRADIENT_TOP: u32 = 5;
    pub const VERTICAL_GRADIENT_BOTTOM: u32 = 6;
    pub const SMOOTHNESS: u32 = 7;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::MaxHeight, crate::util::buffer(MAX_HEIGHT, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Smoothness, crate::util::buffer(SMOOTHNESS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
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
    MaxHeight,
    Color,

    HorizontalGradientLeft,
    HorizontalGradientRight,

    VerticalGradientTop,
    VerticalGradientBottom,

    Smoothness,

    Freqs,
}

pub struct Graph {
    placement: GraphPlacement,
    bar_processor: vibe_audio::BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Graph {
    pub fn new<F: Fetcher>(desc: &GraphDescriptor<F>) -> Self {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();
        let mut bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Graph: Vertex buffer => positions"),
            contents: bytemuck::cast_slice(&POSITIONS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `iResolution` buffer"),
                    size: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::MaxHeight,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `max_height` buffer"),
                    contents: bytemuck::bytes_of(&desc.max_height),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Smoothness,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Graph: `smoothness` buffer"),
                    contents: bytemuck::bytes_of(&desc.smoothness),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>()
                        * u16::from(desc.audio_conf.amount_bars) as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let fragment_entrypoint = match desc.placement {
            GraphPlacement::Bottom => "bottom",
            GraphPlacement::Top => "top",
            GraphPlacement::Right => "right",
            GraphPlacement::Left => "left",
        };

        let fragment_shader = match &desc.variant {
            GraphVariant::Color(rgba) => {
                resource_manager.insert_buffer(
                    ResourceID::Color,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Graph: `color` buffer"),
                        contents: bytemuck::cast_slice(rgba),
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

                device.create_shader_module(include_wgsl!("./fragment_color.wgsl"))
            }
            GraphVariant::HorizontalGradient { left, right } => {
                resource_manager.extend_buffers([
                    (
                        ResourceID::HorizontalGradientLeft,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Graph: `color_left` buffer"),
                            contents: bytemuck::cast_slice(left),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                    (
                        ResourceID::HorizontalGradientRight,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Graph: `color_right` buffer"),
                            contents: bytemuck::cast_slice(right),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                ]);

                bind_group0_mapping.extend([
                    (
                        ResourceID::HorizontalGradientLeft,
                        crate::util::buffer(
                            bindings0::HORIZONTAL_GRADIENT_LEFT,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                    (
                        ResourceID::HorizontalGradientRight,
                        crate::util::buffer(
                            bindings0::HORIZONTAL_GRADIENT_RIGHT,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                ]);

                device.create_shader_module(include_wgsl!("./fragment_horizontal_gradient.wgsl"))
            }
            GraphVariant::VerticalGradient { top, bottom } => {
                resource_manager.extend_buffers([
                    (
                        ResourceID::VerticalGradientTop,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Graph: `color_top` buffer"),
                            contents: bytemuck::cast_slice(top),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                    (
                        ResourceID::VerticalGradientBottom,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Graph: `color_bottom` buffer"),
                            contents: bytemuck::cast_slice(bottom),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                ]);

                bind_group0_mapping.extend([
                    (
                        ResourceID::VerticalGradientTop,
                        crate::util::buffer(
                            bindings0::VERTICAL_GRADIENT_TOP,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                    (
                        ResourceID::VerticalGradientBottom,
                        crate::util::buffer(
                            bindings0::VERTICAL_GRADIENT_BOTTOM,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                ]);

                device.create_shader_module(include_wgsl!("./fragment_vertical_gradient.wgsl"))
            }
        };

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
            let amount_bars = match self.placement {
                GraphPlacement::Bottom | GraphPlacement::Top => new_resolution[0] as usize,
                GraphPlacement::Left | GraphPlacement::Right => new_resolution[1] as usize,
            };

            self.bar_processor
                .set_amount_bars(std::num::NonZero::new(amount_bars as u16).unwrap());

            self.resource_manager.replace_buffer(
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Graph: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * amount_bars) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            );
        }
    }
}
