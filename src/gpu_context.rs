use pollster::FutureExt;
use tracing::info;
use wgpu::{Adapter, Buffer, Device, Instance, Queue};

use crate::output_context::OutputCtx;

pub struct GpuCtx {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    vbuffer: Buffer,
    ibuffer: Buffer,
}

impl GpuCtx {
    pub fn new() -> Self {
        let instance = Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .unwrap();

        info!("Choosing for rendering: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        let vbuffer = crate::vertices::vertex_buffer(&device);
        let ibuffer = crate::vertices::index_buffer(&device);

        Self {
            instance,
            adapter,
            device,
            queue,

            vbuffer,
            ibuffer,
        }
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn render(&mut self, output_ctx: &mut OutputCtx) -> Result<(), wgpu::SurfaceError> {
        output_ctx.update_buffers(&self.queue);

        let shader_ctx = output_ctx.shader_ctx();

        let output = shader_ctx.surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
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

            render_pass.set_pipeline(shader_ctx.pipeline());
            render_pass.set_bind_group(0, shader_ctx.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vbuffer.slice(..));
            render_pass.set_index_buffer(self.ibuffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(crate::vertices::index_buffer_range(), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
