use vibe_audio::fetcher::SystemAudioFetcher;
use wgpu::util::DeviceExt;

use crate::{resource_manager::ResourceManager, Renderable};

use super::Component;

const ENTRYPOINT: &str = "main";

type VertexPosition = [f32; 2];

#[rustfmt::skip]
const VERTICES: [VertexPosition; 4] = [
    [1.0, 1.0],   // top right
    [-1.0, 1.0],  // top left
    [1.0, -1.0],  // bottom right
    [-1.0, -1.0]  // bottom left
];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const OCTAVES: u32 = 0;
    pub const SEED: u32 = 1;
    pub const BRIGHTNESS: u32 = 2;
    pub const CANVASSIZE: u32 = 3;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Octaves, crate::util::buffer(OCTAVES, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Seed, crate::util::buffer(SEED, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Brightness, crate::util::buffer(BRIGHTNESS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::CanvasSize, crate::util::buffer(CANVASSIZE, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Octaves,
    Seed,
    Brightness,
    CanvasSize,
}

pub struct ValueNoiseDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub texture_size: u32,
    pub format: wgpu::TextureFormat,
    pub octaves: u32,
    // should be within the range [0, 1]
    pub brightness: f32,
}

pub struct ValueNoise {
    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    vbuffer: wgpu::Buffer,
}

impl ValueNoise {
    pub fn new(desc: &ValueNoiseDescriptor) -> Self {
        let device = desc.device;

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();

        resource_manager.extend_buffers([
            (
                ResourceID::Octaves,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Value noise: `octaves` buffer"),
                    contents: bytemuck::bytes_of(&desc.octaves),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Seed,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Value noise: `seed` buffer"),
                    contents: bytemuck::bytes_of(&rand::random_range(15.0f32..35.0)),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Brightness,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Value noise: `brightness` buffer"),
                    contents: bytemuck::bytes_of(&desc.brightness.clamp(0., 1.)),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::CanvasSize,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Value noise: `canvas_size` buffer"),
                    contents: bytemuck::bytes_of(&(desc.texture_size as f32)),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
            ),
        ]);

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Value noise: vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "Value noise: Bind group 0",
            device,
            &bind_group0_mapping,
        );

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Value noise: `pipeline layout`"),
                bind_group_layouts: &[&bind_group0_layout],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Value noise: vertex shader module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/vertex_shader.wgsl").into(),
                ),
            });

            let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Value noise: fragment shader module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/fragment_shader.wgsl").into(),
                ),
            });

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Value noise: render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some(ENTRYPOINT),
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
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some(ENTRYPOINT),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        Self {
            bind_group0,

            resource_manager,

            pipeline,
            vbuffer,
        }
    }
}

impl Renderable for ValueNoise {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for ValueNoise {
    fn update_audio(
        &mut self,
        _queue: &wgpu::Queue,
        _processor: &vibe_audio::SampleProcessor<SystemAudioFetcher>,
    ) {
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let buffer = self
            .resource_manager
            .get_buffer(ResourceID::CanvasSize)
            .unwrap();

        queue.write_buffer(
            buffer,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }
}
