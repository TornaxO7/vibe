use cgmath::{Deg, Matrix2};
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{
    bind_group_manager::BindGroupManager, util::SimpleRenderPipelineDescriptor, Renderable,
};

use super::{Component, Rgba};

type VertexPosition = [f32; 2];

const SHADER_ENTRYPOINT: &str = "main";
const POSITIONS: [VertexPosition; 3] = [
    [1., 1.],  // Top right corner
    [-3., 1.], // Top left corner
    [1., -3.], // Bottom right corner
];

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings0 {
    Resolution = 0,
    CircleRadius = 1,
    Rotation = 2,
    SpikeSensitivity = 3,

    // The radiant distance between two frequency spikes.
    // `0.9` instead of `1.0` due to floating point errors
    FreqRadiantStep = 4,
    WaveColor = 5,
    PositionOffset = 6,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings1 {
    Freqs,
}

pub enum CircleVariant {
    Graph { spike_sensitivity: f32, color: Rgba },
}

pub struct CircleDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a shady_audio::SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,
    pub variant: CircleVariant,

    pub radius: f32,
    pub rotation: Deg<f32>,
    // (0, 0) is top left
    pub position: (f32, f32),
}

pub struct Circle {
    bar_processor: shady_audio::BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Circle {
    pub fn new(desc: &CircleDescriptor) -> Self {
        let device = desc.device;

        let bar_processor =
            shady_audio::BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());
        let mut bind_group0 = BindGroupManager::new(Some("Circle: Bind group 0"));
        let mut bind_group1 = BindGroupManager::new(Some("Circle: Bind group 1"));

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle: Vertex buffer"),
            contents: bytemuck::cast_slice(&POSITIONS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        bind_group0.insert_buffer(
            Bindings0::Resolution as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Circle: `iResolution` buffer"),
                size: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::CircleRadius as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle: `circle_radius` buffer"),
                contents: bytemuck::bytes_of(&desc.radius),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        let rotation: [[f32; 2]; 2] = Matrix2::from_angle(desc.rotation).into();
        bind_group0.insert_buffer(
            Bindings0::Rotation as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle: `rotation` buffer"),
                contents: bytemuck::cast_slice(&rotation),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::FreqRadiantStep as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Circle: `freq_radiant_step` buffer"),
                contents: bytemuck::bytes_of(
                    &(std::f32::consts::PI
                        / (u16::from(desc.audio_conf.amount_bars) as f32 + 0.99)),
                ),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        {
            let rel_x_offset = desc.position.0.clamp(0., 1.);
            let rel_y_offset = desc.position.1.clamp(0., 1.);

            bind_group0.insert_buffer(
                Bindings0::PositionOffset as u32,
                wgpu::ShaderStages::FRAGMENT,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Circle: `position_offset` buffer"),
                    contents: bytemuck::cast_slice(&[rel_x_offset, rel_y_offset]),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            );
        }

        bind_group1.insert_buffer(
            Bindings1::Freqs as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Circle: `freqs` buffer"),
                size: (std::mem::size_of::<f32>() * u16::from(desc.audio_conf.amount_bars) as usize)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let fragment_module = match &desc.variant {
            CircleVariant::Graph {
                spike_sensitivity: max_radius,
                color,
            } => {
                bind_group0.insert_buffer(
                    Bindings0::SpikeSensitivity as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Circle: `spike-sensitivity` buffer"),
                        contents: bytemuck::bytes_of(max_radius),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                bind_group0.insert_buffer(
                    Bindings0::WaveColor as u32,
                    wgpu::ShaderStages::FRAGMENT,
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Circle: `wave: color` buffer"),
                        contents: bytemuck::cast_slice(color),
                        usage: wgpu::BufferUsages::UNIFORM,
                    }),
                );

                device.create_shader_module(include_wgsl!("./fragment_graph.wgsl"))
            }
        };

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Circle: Pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0.get_bind_group_layout(device),
                    &bind_group1.get_bind_group_layout(device),
                ],
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

        bind_group0.build_bind_group(device);
        bind_group1.build_bind_group(device);

        Self {
            bar_processor,
            bind_group0,
            bind_group1,

            vbuffer,
            pipeline,
        }
    }
}

impl Renderable for Circle {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);

        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

impl Component for Circle {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Freqs as u32) {
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::Resolution as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }
}
