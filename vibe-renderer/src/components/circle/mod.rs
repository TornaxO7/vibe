mod descriptor;

pub use descriptor::*;

use super::Component;
use crate::{resource_manager::ResourceManager, util::SimpleRenderPipelineDescriptor, Renderable};
use cgmath::Matrix2;
use wgpu::{include_wgsl, util::DeviceExt};

type VertexPosition = [f32; 2];

const SHADER_ENTRYPOINT: &str = "main";
const POSITIONS: [VertexPosition; 3] = [
    [1., 1.],  // Top right corner
    [-3., 1.], // Top left corner
    [1., -3.], // Bottom right corner
];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const CIRCLE_RADIUS: u32 = 1;
    pub const ROTATION: u32 = 2;
    pub const SPIKE_SENSITIVITY: u32 = 3;

    // The radiant distance between two frequency spikes.
    // `0.9` instead of `1.0` due to floating point errors
    pub const FREQ_RADIANT_STEP: u32 = 4;
    pub const WAVE_COLOR: u32 = 5;
    pub const POSITION_OFFSET: u32 = 6;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::CircleRadius, crate::util::buffer(CIRCLE_RADIUS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Rotation, crate::util::buffer(ROTATION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::FreqRadiantStep, crate::util::buffer(FREQ_RADIANT_STEP, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::PositionOffset, crate::util::buffer(POSITION_OFFSET, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
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
    CircleRadius,
    Rotation,
    SpikeSensitivity,
    FreqRadiantStep,
    WaveColor,
    PositionOffset,
    Freqs,
}

pub struct Circle {
    bar_processor: shady_audio::BarProcessor,

    resource_manager: ResourceManager<ResourceID>,
    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Circle {
    pub fn new(desc: &CircleDescriptor) -> Self {
        let device = desc.device;
        let bar_processor =
            shady_audio::BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();

        let mut bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle: Vertex buffer"),
            contents: bytemuck::cast_slice(&POSITIONS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Circle: `iResolution` buffer"),
                    size: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::CircleRadius,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Circle: `circle_radius` buffer"),
                    contents: bytemuck::bytes_of(&desc.radius),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            {
                let rotation: [[f32; 2]; 2] = Matrix2::from_angle(desc.rotation).into();

                (
                    ResourceID::Rotation,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Circle: `rotation` buffer"),
                        contents: bytemuck::cast_slice(&rotation),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                )
            },
            (
                ResourceID::FreqRadiantStep,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Circle: `freq_radiant_step` buffer"),
                    contents: bytemuck::bytes_of(
                        &(std::f32::consts::PI
                            / (u16::from(desc.audio_conf.amount_bars) as f32 + 0.99)),
                    ),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            {
                let rel_x_offset = desc.position.0.clamp(0., 1.);
                let rel_y_offset = desc.position.1.clamp(0., 1.);

                (
                    ResourceID::PositionOffset,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Circle: `position_offset` buffer"),
                        contents: bytemuck::cast_slice(&[rel_x_offset, rel_y_offset]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                )
            },
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Circle: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>()
                        * u16::from(desc.audio_conf.amount_bars) as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let fragment_module = match &desc.variant {
            CircleVariant::Graph {
                spike_sensitivity: max_radius,
                color,
            } => {
                resource_manager.extend_buffers([
                    (
                        ResourceID::SpikeSensitivity,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Circle: `spike-sensitivity` buffer"),
                            contents: bytemuck::bytes_of(max_radius),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                    (
                        ResourceID::WaveColor,
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Circle: `wave: color` buffer"),
                            contents: bytemuck::cast_slice(color),
                            usage: wgpu::BufferUsages::UNIFORM,
                        }),
                    ),
                ]);

                bind_group0_mapping.extend([
                    (
                        ResourceID::SpikeSensitivity,
                        crate::util::buffer(
                            bindings0::SPIKE_SENSITIVITY,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                    (
                        ResourceID::WaveColor,
                        crate::util::buffer(
                            bindings0::WAVE_COLOR,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ),
                ]);

                device.create_shader_module(include_wgsl!("./fragment_graph.wgsl"))
            }
        };

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Circle: Bind group 0", device, &bind_group0_mapping);

        let (bind_group1, bind_group1_layout) =
            resource_manager.build_bind_group("Circle: Bind group 1", device, &bind_group1_mapping);

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Circle: Pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(include_wgsl!("./vertex_shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                SimpleRenderPipelineDescriptor {
                    label: "Circle: Render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some(SHADER_ENTRYPOINT),
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
                        module: &fragment_module,
                        entry_point: Some(SHADER_ENTRYPOINT),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.texture_format,
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                },
            ))
        };

        Self {
            bar_processor,

            resource_manager,

            bind_group0,
            bind_group1,

            vbuffer,
            pipeline,
        }
    }
}

impl Renderable for Circle {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);

        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

impl Component for Circle {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        let bar_values = self.bar_processor.process_bars(processor);

        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

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
}
