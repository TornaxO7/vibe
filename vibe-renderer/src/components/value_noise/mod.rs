use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable};

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

#[repr(u32)]
enum Bindings0 {
    Octaves,
    Seed,
    Brightness,
    CanvasSize,
}

pub struct ValueNoiseDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub octaves: u32,
    // should be within the range [0, 1]
    pub brightness: f32,
}

pub struct ValueNoise {
    bind_group0: BindGroupManager,
    pipeline: wgpu::RenderPipeline,
    vbuffer: wgpu::Buffer,
}

impl ValueNoise {
    pub fn new(desc: &ValueNoiseDescriptor) -> Self {
        let device = desc.device;

        let mut bind_group0 = BindGroupManager::new(Some("Value noise bind group 0"));

        bind_group0.insert_buffer(
            Bindings0::Octaves as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Value noise: `octaves` buffer"),
                contents: bytemuck::bytes_of(&desc.octaves),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::Seed as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Value noise: `seed` buffer"),
                contents: bytemuck::bytes_of(&rand::random_range(15.0f32..35.0)),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::Brightness as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Value noise: `brightness` buffer"),
                contents: bytemuck::bytes_of(&desc.brightness.clamp(0., 1.)),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        bind_group0.insert_buffer(
            Bindings0::CanvasSize as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Value noise: `canvas_size` buffer"),
                contents: bytemuck::cast_slice(&[desc.width as f32, desc.height as f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Value noise: vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Value noise: `pipeline layout`"),
                bind_group_layouts: &[&bind_group0.get_bind_group_layout(device)],
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

        bind_group0.build_bind_group(device);

        Self {
            bind_group0,
            pipeline,
            vbuffer,
        }
    }
}

impl Renderable for ValueNoise {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for ValueNoise {
    fn update_audio(&mut self, _queue: &wgpu::Queue, _processor: &shady_audio::SampleProcessor) {}

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::CanvasSize as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }
}
