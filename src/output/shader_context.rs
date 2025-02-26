use anyhow::{anyhow, Context};
use shady::{Shady, ShadyRenderPipeline};
use std::{borrow::Cow, ptr::NonNull};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::shell::{wlr_layer::LayerSurface, WaylandSurface};
use wayland_client::{Connection, Proxy};
use wgpu::{
    naga::{
        front::{glsl, wgsl},
        ShaderStage,
    },
    PresentMode, ShaderSource, Surface, SurfaceConfiguration,
};

use crate::gpu::GpuCtx;

use super::{
    config::{OutputConfig, ShaderCode},
    Size,
};

pub struct ShaderCtx {
    shady: Shady,
    pipelines: Vec<ShadyRenderPipeline>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl ShaderCtx {
    pub fn new(
        name: &str,
        conn: &Connection,
        size: Size,
        layer_surface: &LayerSurface,
        gpu: &GpuCtx,
        config: &OutputConfig,
    ) -> anyhow::Result<Self> {
        let shady = {
            let mut shady = Shady::new(shady::ShadyDescriptor {
                device: gpu.device(),
            });
            shady.set_audio_bars(gpu.device(), config.amount_bars);
            shady.set_resolution(size.width, size.height);
            shady
        };

        let surface: Surface<'static> = {
            let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
            ));

            let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
                NonNull::new(layer_surface.wl_surface().id().as_ptr() as *mut _).unwrap(),
            ));

            unsafe {
                gpu.instance()
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            }
        };

        let surface_config = {
            let surface_caps = surface.get_capabilities(gpu.adapter());
            let format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap();

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
                view_formats: vec![],
                desired_maximum_frame_latency: 3,
            }
        };
        surface.configure(gpu.device(), &surface_config);

        let pipelines = {
            let mut pipelines = Vec::new();

            for (i, shader_code) in config.shader_code.iter().enumerate() {
                let shader_index = i + 1; // `i` starts with 0
                let num_abbreviation = match shader_index {
                    1 => "st",
                    2 => "nd",
                    3 => "rd",
                    _ => "th",
                };

                let fragment_module = match shader_code {
                    ShaderCode::Glsl(code) => {
                        let mut frontend = glsl::Frontend::default();
                        frontend
                            .parse(&glsl::Options::from(ShaderStage::Fragment), code)
                            .map_err(|err| anyhow!("{}", err.emit_to_string(code)))
                            .with_context(|| {
                                let shader_pos = i + 1; // `i` starts with 0
                                format!(
                                    "your {}{}shader (it's a glsl shader) of '{}' is invalid",
                                    shader_pos, num_abbreviation, name
                                )
                            })
                    }
                    ShaderCode::Wgsl(code) => wgsl::parse_str(code)
                        .map_err(|err| anyhow!("{}", err.emit_to_string(code)))
                        .with_context(|| {
                            format!(
                                "your {}{} shader (it's a wgsl shader) of '{}' is invalid",
                                shader_index, num_abbreviation, name
                            )
                        }),
                }?;

                let pipeline = shady::create_render_pipeline(
                    gpu.device(),
                    ShaderSource::Naga(Cow::Owned(fragment_module)),
                    &surface_config.format,
                );

                pipelines.push(pipeline);
            }

            pipelines
        };

        Ok(Self {
            shady,
            surface,
            config: surface_config,
            pipelines,
        })
    }

    pub fn resize(&mut self, gpu: &GpuCtx, new_size: Size) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(gpu.device(), &self.config);

            self.shady
                .set_resolution(self.config.width, self.config.height);
            self.shady.update_resolution_buffer(gpu.queue());
        }
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        self.shady.update_audio_buffer(queue);
        self.shady.update_time_buffer(queue);
    }

    pub fn size(&self) -> Size {
        Size::from((self.config.width, self.config.height))
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn add_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.shady.add_render_pass(encoder, view, &self.pipelines);
    }
}
