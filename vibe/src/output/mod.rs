pub mod config;

use crate::{output::config::component::ComponentConfig, state::State, types::size::Size};
use config::OutputConfig;
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{
        wlr_layer::{Anchor, LayerSurface},
        WaylandSurface,
    },
};
use tracing::error;
use vibe_audio::{fetcher::SystemAudioFetcher, SampleProcessor};
use vibe_renderer::{components::Component, Renderer};
use wayland_client::QueueHandle;
use wgpu::{PresentMode, Surface, SurfaceConfiguration};

/// Contains every relevant information for an output.
pub struct OutputCtx {
    pub components: Vec<Box<dyn Component>>,

    // don't know if this is required, but better drop `surface` first before
    // `layer_surface`
    surface: Surface<'static>,
    layer_surface: LayerSurface,
    surface_config: SurfaceConfiguration,
}

impl OutputCtx {
    pub fn new(
        info: OutputInfo,
        surface: Surface<'static>,
        layer_surface: LayerSurface,
        renderer: &Renderer,
        sample_processor: &SampleProcessor<SystemAudioFetcher>,
        config: OutputConfig,
    ) -> Self {
        let size = Size::from(&info);

        // Should be "-1" otherwise: https://github.com/TornaxO7/vibe/issues/167 happens
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_anchor(Anchor::all());
        layer_surface.set_size(size.width, size.height);
        layer_surface.commit();

        let surface_config = get_surface_config(renderer.adapter(), &surface, size);
        surface.configure(renderer.device(), &surface_config);

        let components = {
            let mut components = Vec::with_capacity(config.components.len());

            for comp_conf in config.components {
                let component: Box<dyn Component> = comp_conf
                    .create_component(renderer, sample_processor, surface_config.format)
                    .unwrap_or_else(|msg| {
                        error!("{}", msg);
                        panic!("Invalid fragment shader code");
                    });

                components.push(component);
            }

            components
        };

        Self {
            surface_config,
            surface,
            layer_surface,
            components,
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

    /// Update the internal data to the new output size.
    pub fn resize(&mut self, renderer: &Renderer, new_size: Size) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;

            self.surface
                .configure(renderer.device(), &self.surface_config);

            for component in self.components.iter_mut() {
                component.update_resolution(renderer, [new_size.width, new_size.height]);
            }
        }
    }

    pub fn update_mouse_position(&mut self, queue: &wgpu::Queue, new_pos: (f64, f64)) {
        let normalized_pos = (
            new_pos.0 as f32 / self.surface_config.width as f32,
            new_pos.1 as f32 / self.surface_config.height as f32,
        );

        for component in self.components.iter_mut() {
            component.update_mouse_position(queue, normalized_pos);
        }
    }
}

// getters
impl OutputCtx {
    pub fn layer_surface(&self) -> &LayerSurface {
        &self.layer_surface
    }

    pub fn surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
    }
}

pub fn get_surface_config(
    adapter: &wgpu::Adapter,
    surface: &wgpu::Surface<'_>,
    size: Size,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    // Prefer common renderable non-sRGB formats; some GPUs (AMD) expose
    // Rgba16Unorm first which isn't renderable.
    let format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| {
            !f.is_srgb()
                && matches!(
                    f,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
        })
        .or_else(|| surface_caps.formats.iter().find(|f| !f.is_srgb()).copied())
        .unwrap_or(surface_caps.formats[0]);

    if !surface_caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
    {
        error!(concat![
                    "Ok, now this is getting tricky (great to hear that from a software, right?).\n",
                    "\tSimply speaking: For the time being I'm expecting that the selected gpu supports the 'PreMultiplied'-'feature'\n",
                    "\tbut the selected gpu only supports: {:?}\n",
                    "\tPlease create an issue (or give the existing issue an upvote) that you've encountered this so I can prioritize this problem."
                ], &surface_caps.alpha_modes);

        todo!("Sorry :(");
    }

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
}
