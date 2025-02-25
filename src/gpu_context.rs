use pollster::FutureExt;
use tracing::info;
use wgpu::{Adapter, Device, Instance, Queue};

use crate::output::OutputCtx;

pub struct GpuCtx {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
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

        Self {
            instance,
            adapter,
            device,
            queue,
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

        let output = output_ctx.surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        output_ctx.add_render_pass(&mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
