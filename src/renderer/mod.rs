mod config;
mod resources;
mod shader_context;
mod template;
mod vertices;

pub use config::GraphicsConfig;
use resources::Resources;
pub use shader_context::ShaderCtx;

use pollster::FutureExt;
use tracing::info;

const FRAGMENT_ENTRYPOINT: &str = "main";
const BIND_GROUP_INDEX: u32 = 0;
const VBUFFER_INDEX: u32 = 0;

pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    resources: Resources,

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

        let resources = Resources::new(&device);

        let vbuffer = vertices::vertex_buffer(&device);
        let ibuffer = vertices::index_buffer(&device);

        Self {
            instance,
            adapter,
            device,
            queue,

            resources,

            vbuffer,
            ibuffer,
        }
    }

    pub fn apply_render_pass<'a>(
        &mut self,
        output: &wgpu::SurfaceTexture,
        shaders: impl IntoIterator<Item = &'a ShaderCtx>,
    ) {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_bind_group(BIND_GROUP_INDEX, &self.resources.bind_group, &[]);
            render_pass.set_vertex_buffer(VBUFFER_INDEX, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);

            for shader in shaders {
                render_pass.set_pipeline(&shader.pipeline);
                render_pass.draw_indexed(vertices::index_buffer_range(), 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
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

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.resources.bind_group_layout
    }
}
