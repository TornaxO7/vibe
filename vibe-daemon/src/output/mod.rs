pub mod component;
pub mod config;
mod shader;

use std::borrow::Cow;

use anyhow::{anyhow, Context};
use component::ComponentConfig;
use shader::config::{ShaderCode, ShaderConf};
use shady_audio::SampleProcessor;
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerSurface},
        WaylandSurface,
    },
};

use vibe_renderer::{
    components::{Bars, Component, FragmentCanvas, FragmentCanvasDescriptor},
    Renderer,
};
use wayland_client::QueueHandle;
use wgpu::{
    naga::{
        front::{glsl, wgsl},
        Module, ShaderStage,
    },
    PresentMode, ShaderSource, Surface, SurfaceConfiguration,
};

use crate::{state::State, types::size::Size};
use config::OutputConfig;

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

        let components = {
            let mut components = Vec::with_capacity(config.components.len());

            for comp_conf in config.components {
                let component: Box<dyn Component> = match comp_conf {
                    ComponentConfig::Bars(bars_conf) => {
                        let bars = Bars::new(&vibe_renderer::components::BarsDescriptor {
                            device: renderer.device(),
                            sample_processor,
                            audio_conf: bars_conf.audio_conf,
                            texture_format: surface_config.format,
                            fragment_source: bars_conf.fragment_source,
                        });

                        Box::new(bars)
                    }
                    ComponentConfig::FragmentCanvas(canvas_conf) => {
                        let fragment_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
                            sample_processor,
                            audio_config: canvas_conf.audio_conf,
                            device: renderer.device(),
                            format: surface_config.format,
                            fragment_source: canvas_conf.fragment_source,
                            resolution: [0, 0],
                        });

                        Box::new(fragment_canvas)
                    }
                };

                components.push(component);
            }

            components
        };

        // let shaders = {
        //     let mut shaders = Vec::with_capacity(config.shaders.len());

        //     for shader_conf in config.shaders.iter() {
        //         let shader_resource =
        //             ShaderResources::new(renderer.device(), sample_processor, shader_conf);

        //         let pipeline = {
        //             let shader_source = match get_shader_source(shader_conf) {
        //                 Ok(source) => source,
        //                 Err(err) => {
        //                     error!("Couldn't parse shader:\n\n{}", err);
        //                     continue;
        //                 }
        //             };

        //             let bind_group_layouts = [
        //                 output_resources.bind_group_layout(),
        //                 global_resources.bind_group_layout(),
        //                 shader_resource.bind_group_layout(),
        //             ];

        //             renderer.create_render_pipeline(
        //                 shader_source,
        //                 &bind_group_layouts,
        //                 surface_config.format,
        //             )
        //         };

        //         let shader = Shader::new(shader_resource, pipeline);

        //         shaders.push(shader);
        //     }

        //     shaders
        // };

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

            self.resources.set_resolution(new_size);
            self.resources.update_ressource_buffers(renderer.queue());
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

// impl ResourceCollection for OutputCtx {
//     fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
//         self.resources.bind_group_layout()
//     }

//     fn bind_group(&self) -> &wgpu::BindGroup {
//         self.resources.bind_group()
//     }

//     fn update_ressource_buffers(&self, queue: &wgpu::Queue) {
//         for shader in self.shaders.iter() {
//             shader.update_render_buffers(queue);
//         }
//     }
// }

fn get_shader_source(shader_conf: &ShaderConf) -> anyhow::Result<ShaderSource> {
    let fragment_module = match &shader_conf.code {
        ShaderCode::Glsl(code) => get_glsl_module(code)?,
        ShaderCode::Wgsl(code) => get_wgsl_module(code)?,
        ShaderCode::VibeShader(dir_name) => {
            let url = format!("https://raw.githubusercontent.com/TornaxO7/vibe-shaders/refs/heads/main/{}/code.toml", dir_name);
            let body = reqwest::blocking::get(url)
                .context("Send http request to fetch shader code")?
                .text()
                .unwrap();
            let shader_code: ShaderCode = toml::from_str(&body)?;

            match shader_code {
                ShaderCode::Glsl(code) => get_glsl_module(code)?,
                ShaderCode::Wgsl(code) => get_wgsl_module(code)?,
                ShaderCode::VibeShader(_) => {
                    anyhow::bail!("The shader in '{}' refers to another shader. Please create an issue this shouldn't happen! Going to skip this shader...", dir_name);
                }
            }
        }
    };

    Ok(ShaderSource::Naga(Cow::Owned(fragment_module)))
}

fn get_glsl_module(code: impl AsRef<str>) -> anyhow::Result<Module> {
    let mut frontend = glsl::Frontend::default();
    frontend
        .parse(&glsl::Options::from(ShaderStage::Fragment), code.as_ref())
        .map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
}

fn get_wgsl_module(code: impl AsRef<str>) -> anyhow::Result<Module> {
    wgsl::parse_str(code.as_ref()).map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
}
