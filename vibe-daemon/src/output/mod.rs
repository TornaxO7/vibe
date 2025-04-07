pub mod config;

use shady_audio::SampleProcessor;
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
        WaylandSurface,
    },
};

use tracing::error;
use vibe_renderer::{
    components::{
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, Component,
        FragmentCanvas, FragmentCanvasDescriptor,
    },
    Renderer,
};
use wayland_client::QueueHandle;
use wgpu::{PresentMode, Surface, SurfaceConfiguration};

use crate::{state::State, types::size::Size};
use config::{
    component::{BarVariantConfig, ComponentConfig},
    OutputConfig,
};

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
        sample_processor: &SampleProcessor,
        config: OutputConfig,
    ) -> Self {
        let size = Size::from(&info);

        layer_surface.set_exclusive_zone(-1); // nice! (arbitrary chosen :P hehe)
        layer_surface.set_anchor(Anchor::all());
        layer_surface.set_size(size.width, size.height);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();

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
                error!(concat![
                    "Ok, now this is getting tricky (great to hear that from a software, right?).\n",
                    "\tSimply speaking: For the time being I'm expecting that the selected gpu supports the 'PreMultiplied'-'feature'\n",
                    "\tbut the selected gpu only supports: {:?}\n",
                    "\tPlease create an issue (or give the existing issue an upvote) that you've encountered this so I can priotize this problem."
                ], &surface_caps.alpha_modes);

                todo!("Sorry :(");
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

        let components = {
            let mut components = Vec::with_capacity(config.components.len());

            for comp_conf in config.components {
                let component: Box<dyn Component> = match comp_conf {
                    ComponentConfig::Bars {
                        audio_conf,
                        max_height,
                        variant,
                    } => {
                        let variant = match variant {
                            BarVariantConfig::Color(rgba) => {
                                BarVariant::Color(rgba.gamma_corrected())
                            }
                            BarVariantConfig::PresenceGradient {
                                high_presence,
                                low_presence,
                            } => BarVariant::PresenceGradient {
                                high: high_presence.gamma_corrected(),
                                low: low_presence.gamma_corrected(),
                            },
                            BarVariantConfig::FragmentCode(code) => BarVariant::FragmentCode {
                                resolution: [size.width, size.height],
                                code,
                            },
                        };

                        Bars::new(&vibe_renderer::components::BarsDescriptor {
                            device: renderer.device(),
                            sample_processor,
                            audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                            texture_format: surface_config.format,
                            max_height,
                            variant,
                        })
                        .map(|bars| Box::new(bars) as Box<dyn Component>)
                    }
                    ComponentConfig::FragmentCanvas {
                        audio_conf,
                        fragment_code,
                    } => FragmentCanvas::new(&FragmentCanvasDescriptor {
                        sample_processor,
                        audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                        device: renderer.device(),
                        format: surface_config.format,
                        fragment_code,
                        resolution: [size.width, size.height],
                    })
                    .map(|canvas| Box::new(canvas) as Box<dyn Component>),
                    ComponentConfig::Aurodio {
                        base_color,
                        movement_speed,
                        audio_conf,
                        layers,
                    } => {
                        let layers: Vec<AurodioLayerDescriptor> = layers
                            .iter()
                            .map(|layer| AurodioLayerDescriptor {
                                freq_range: layer.freq_range.clone(),
                                zoom_factor: layer.zoom_factor,
                            })
                            .collect();

                        Ok(Box::new(Aurodio::new(&AurodioDescriptor {
                            renderer,
                            sample_processor,
                            texture_format: surface_config.format,
                            base_color: base_color.gamma_corrected(),
                            movement_speed,
                            easing: audio_conf.easing,
                            sensitivity: audio_conf.sensitivity,
                            layers: &layers,
                        })) as Box<dyn Component>)
                    }
                }
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

    pub fn resize(&mut self, renderer: &Renderer, new_size: Size) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;

            self.surface
                .configure(renderer.device(), &self.surface_config);

            for component in self.components.iter_mut() {
                component.update_resolution(renderer.queue(), [new_size.width, new_size.height]);
            }
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
