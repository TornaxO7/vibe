use anyhow::{anyhow, Context};
use reqwest::blocking::Client;
use shady::{Shady, ShadyRenderPipeline};
use std::borrow::Cow;
use tracing::error;

use wgpu::{
    naga::{
        front::{glsl, wgsl},
        Module, ShaderStage,
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
        size: Size,
        gpu: &GpuCtx,
        surface: Surface<'static>,
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

        let surface_config = {
            let surface_caps = surface.get_capabilities(gpu.adapter());
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

            let client = Client::new();

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
                        get_glsl_module(code, shader_index, num_abbreviation, name)
                    }
                    ShaderCode::Wgsl(code) => {
                        get_wgsl_module(code, shader_index, num_abbreviation, name)
                    }
                    ShaderCode::VibeShader(dir_name) => {
                        let url = format!("https://raw.githubusercontent.com/TornaxO7/vibe-shaders/refs/heads/main/{}/code.toml", dir_name);
                        let body = client
                            .get(url)
                            .send()
                            .context("Send http request to fetch shader code")?
                            .text()
                            .unwrap();
                        let shader_code: ShaderCode = toml::from_str(&body)?;

                        match shader_code {
                            ShaderCode::Glsl(code) => {
                                get_glsl_module(code, shader_index, num_abbreviation, name)
                            }
                            ShaderCode::Wgsl(code) => {
                                get_wgsl_module(code, shader_index, num_abbreviation, name)
                            }
                            ShaderCode::VibeShader(_) => {
                                error!("The shader in '{}' refers to another shader. Please create an issue this shouldn't happen! Going to skip this shader...", dir_name);
                                continue;
                            }
                        }
                    }
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

fn get_glsl_module(
    code: impl AsRef<str>,
    shader_index: usize,
    num_abbreviation: &str,
    output_name: &str,
) -> anyhow::Result<Module> {
    let mut frontend = glsl::Frontend::default();
    frontend
        .parse(&glsl::Options::from(ShaderStage::Fragment), code.as_ref())
        .map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
        .with_context(|| {
            format!(
                "your {}{}shader (it's a glsl shader) of '{}' is invalid",
                shader_index, num_abbreviation, output_name
            )
        })
}

fn get_wgsl_module(
    code: impl AsRef<str>,
    shader_index: usize,
    num_abbreviation: &str,
    output_name: &str,
) -> anyhow::Result<Module> {
    wgsl::parse_str(code.as_ref())
        .map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
        .with_context(|| {
            format!(
                "your {}{} shader (it's a wgsl shader) of '{}' is invalid",
                shader_index, num_abbreviation, output_name
            )
        })
}
