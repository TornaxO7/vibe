pub mod config;

use std::ptr::NonNull;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use shady_audio::SampleProcessor;
use smithay_client_toolkit::{
    compositor::{CompositorState, Region},
    output::OutputInfo,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
        WaylandSurface,
    },
};
use tracing::error;
use wayland_client::{Connection, Proxy, QueueHandle};
use wgpu::{PresentMode, Surface, SurfaceConfiguration};

use crate::{
    renderer::{Renderer, ShaderCtx},
    state::State,
};
use config::OutputConfig;

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

impl From<&SurfaceConfiguration> for Size {
    fn from(value: &SurfaceConfiguration) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

/// Contains every relevant information for an output.
pub struct OutputCtx {
    shaders: Vec<ShaderCtx>,

    surface_config: SurfaceConfiguration,

    // don't know if this is required, but better drop the surface first
    surface: Surface<'static>,
    layer_surface: LayerSurface,
}

impl OutputCtx {
    pub fn new(
        conn: &Connection,
        comp: &CompositorState,
        info: OutputInfo,
        layer_surface: LayerSurface,
        renderer: &Renderer,
        sample_processor: &SampleProcessor,
        config: OutputConfig,
    ) -> Self {
        let size = Size::from(&info);

        {
            let region = Region::new(comp).unwrap();
            layer_surface.set_input_region(Some(region.wl_region()));
        }
        layer_surface.set_exclusive_zone(-1); // nice! (arbitrary chosen :P hehe)
        layer_surface.set_anchor(Anchor::all());
        layer_surface.set_size(size.width, size.height);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();

        let surface: Surface<'static> = {
            let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
            ));

            let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
                NonNull::new(layer_surface.wl_surface().id().as_ptr() as *mut _).unwrap(),
            ));

            unsafe {
                renderer
                    .instance()
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            }
        };

        let surface_config = {
            let surface_caps = surface.get_capabilities(renderer.adapter());
            let format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap();

            if !surface_caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
            {
                todo!(concat![
                    "Ok, now this is getting tricky (great to hear that from a software, right?)\n",
                    "Simply speaking: For the time being I'm expecting that the selected gpu supports the 'PreMultiplied'-'feature'\n",
                    "but the selected gpu only supports: {:?}\n",
                    "Please create an issue (or give the existing issue an upvote) that you've encountered this so I can priotize this problem."
                ], &surface_caps.alpha_modes);
            }

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
                view_formats: vec![],
                desired_maximum_frame_latency: 3,
            };

            surface.configure(renderer.device(), &config);
            config
        };

        let shaders = {
            let mut shaders = Vec::with_capacity(config.shaders.len());

            for shader_conf in config.shaders.iter() {
                let shader = match ShaderCtx::new(
                    shader_conf,
                    renderer,
                    sample_processor,
                    surface_config.format,
                ) {
                    Ok(shader) => shader,
                    Err(err) => {
                        error!("Skipping shader due to error:\n\n{}", err);
                        continue;
                    }
                };

                shaders.push(shader);
            }

            shaders
        };

        Self {
            surface_config,
            surface,
            layer_surface,
            shaders,
        }
    }

    pub fn request_redraw(&self, qh: &QueueHandle<State>) {
        let surface = self.layer_surface.wl_surface();

        let size = Size::from(&self.surface_config);
        surface.damage(
            0,
            0,
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        );
        surface.frame(qh, surface.clone());
        self.layer_surface.commit();
    }

    pub fn resize(&mut self, renderer: &Renderer, new_size: Size) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;

            self.surface
                .configure(renderer.device(), &self.surface_config);
        }
    }
}

// getters
impl OutputCtx {
    pub fn shaders(&self) -> &[ShaderCtx] {
        &self.shaders
    }

    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer_surface
    }

    pub fn surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
    }
}
