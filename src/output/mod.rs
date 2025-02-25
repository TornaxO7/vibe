pub mod config;
mod shader_context;

use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
        WaylandSurface,
    },
};
use wayland_client::{Connection, QueueHandle};

use crate::{gpu_context::GpuCtx, state::State};
use config::OutputConfig;
use shader_context::ShaderCtx;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<&OutputInfo> for Size {
    fn from(value: &OutputInfo) -> Self {
        let (width, height) = value
            .logical_size
            .map(|(width, height)| (width as u32, height as u32))
            .unwrap();

        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

pub struct OutputCtx {
    shader_ctx: ShaderCtx,
    layer_surface: LayerSurface,
}

impl OutputCtx {
    pub fn new(
        conn: &Connection,
        info: OutputInfo,
        layer_surface: LayerSurface,
        gpu: &GpuCtx,
        config: OutputConfig,
    ) -> anyhow::Result<Self> {
        let size = Size::from(&info);

        layer_surface.set_exclusive_zone(-69); // nice, arbitrary chosen
        layer_surface.set_anchor(Anchor::BOTTOM);
        layer_surface.set_size(size.width, size.height);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();

        let shader_ctx = ShaderCtx::new(conn, size, &layer_surface, gpu, &config)?;

        Ok(Self {
            shader_ctx,
            layer_surface,
        })
    }

    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer_surface
    }

    pub fn surface(&self) -> &wgpu::Surface<'static> {
        self.shader_ctx.surface()
    }

    pub fn request_redraw(&self, qh: &QueueHandle<State>) {
        let surface = self.layer_surface.wl_surface();

        let size = self.shader_ctx.size();
        surface.damage(
            0,
            0,
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        );
        surface.frame(qh, surface.clone());
        self.layer_surface.commit();
    }

    pub fn resize(&mut self, gpu: &GpuCtx, new_size: Size) {
        self.shader_ctx.resize(gpu, new_size);
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        self.shader_ctx.update_buffers(queue);
    }

    pub fn add_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.shader_ctx.add_render_pass(encoder, view);
    }
}
