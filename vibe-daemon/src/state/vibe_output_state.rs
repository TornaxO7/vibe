use pollster::FutureExt;
use shady::{Shady, ShadyRenderPipeline};
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wgpu::{
    naga::{front::glsl::Options, ShaderStage},
    Device, Queue, ShaderSource, Surface, SurfaceConfiguration,
};

use super::wayland_handle::WaylandHandle;

pub struct VibeOutputState {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    shady: Shady,
    pipeline: Option<ShadyRenderPipeline>,

    pub layer: LayerSurface,
}

impl VibeOutputState {
    pub fn new(wl_handle: WaylandHandle, layer: LayerSurface, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::default();

        // static lifetime: Well, our WlSurface also has a static lifetime, so it should be fine... I hope... ;-;
        let surface: Surface<'static> = instance
            .create_surface(wl_handle.into_surface_target())
            .unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        let config = {
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let alpha_mode = surface_caps
                .alpha_modes
                .iter()
                .find(|&&a| a == wgpu::CompositeAlphaMode::Auto)
                .copied()
                .unwrap_or(surface_caps.alpha_modes[0]);

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode,
                view_formats: vec![],
                desired_maximum_frame_latency: 3,
            }
        };
        surface.configure(&device, &config);

        let shady = Shady::new(shady::ShadyDescriptor { device: &device });

        let pipeline = {
            let shader_code = std::fs::read_to_string("/home/tornax/shaders/music.glsl").unwrap();
            let source = wgpu::naga::front::glsl::Frontend::default()
                .parse(&Options::from(ShaderStage::Fragment), &shader_code)
                .unwrap();

            Some(shady::create_render_pipeline(
                &device,
                ShaderSource::Naga(std::borrow::Cow::Owned(source)),
                &config.format,
            ))
        };

        Self {
            device,
            queue,
            config,
            shady,
            pipeline,
            surface,

            layer,
        }
    }

    pub fn prepare_next_frame(&mut self) {
        self.shady.update_audio_buffer(&mut self.queue);
        self.shady.update_time_buffer(&mut self.queue);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0);
        debug_assert!(height > 0);

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(pipeline) = &self.pipeline {
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            self.shady.add_render_pass(&mut encoder, &view, pipeline);
            self.queue.submit(std::iter::once(encoder.finish()));

            output.present();
        }

        Ok(())
    }
}
