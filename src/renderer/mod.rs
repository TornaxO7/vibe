mod config;
mod shader_context;
mod vertices;

pub use config::GraphicsConfig;
pub use shader_context::RenderShader;

use pollster::FutureExt;
use tracing::info;
use wgpu::ShaderSource;

const FRAGMENT_ENTRYPOINT: &str = "main";
const VBUFFER_INDEX: u32 = 0;

pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(config: &GraphicsConfig) -> Self {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: config.backend,
                ..Default::default()
            }
            .with_env(),
        );

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                ..Default::default()
            })
            .block_on()
            .expect("Couldn't find GPU device.");

        info!("Choosing for rendering: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        let vbuffer = vertices::vertex_buffer(&device);
        let ibuffer = vertices::index_buffer(&device);

        Self {
            instance,
            adapter,
            device,
            queue,

            vbuffer,
            ibuffer,
        }
    }

    pub fn render<'a>(
        &self,
        view: &'a wgpu::TextureView,
        global_bind_groups: impl IntoIterator<Item = &'a wgpu::BindGroup>,
        shaders: impl IntoIterator<Item = impl RenderShader>,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_vertex_buffer(VBUFFER_INDEX, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);

            let mut bind_group_idx = 0;
            for global_bind_group in global_bind_groups.into_iter() {
                render_pass.set_bind_group(bind_group_idx, global_bind_group, &[]);
                bind_group_idx += 1;
            }

            for shader in shaders.into_iter() {
                render_pass.set_bind_group(bind_group_idx, shader.bind_group(), &[]);
                render_pass.set_pipeline(shader.pipeline());

                render_pass.draw_indexed(vertices::index_buffer_range(), 0, 0..1);
                bind_group_idx += 1;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn create_render_pipeline(
        &self,
        shader_source: ShaderSource,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        texture_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex_shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Vertex shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("vertex_shader.wgsl").into()),
            });

        let fragment_shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment shader"),
                source: shader_source,
            });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: Some("vertex_main"),
                    buffers: &[vertices::BUFFER_LAYOUT],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: Some(FRAGMENT_ENTRYPOINT),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                multiview: None,
                cache: None,
            })
    }
}

// getter functions
impl Renderer {
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
