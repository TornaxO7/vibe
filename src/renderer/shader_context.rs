use std::borrow::Cow;

use anyhow::{anyhow, Context};

use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::{
    naga::{
        front::{glsl, wgsl},
        Module, ShaderStage,
    },
    RenderPipeline, ShaderSource,
};

use crate::output::config::{ShaderCode, ShaderConf};

use super::Renderer;

/// Everything relevant for the shader to be rendered by the renderer.
pub struct ShaderCtx {
    pub bar_processor: Option<BarProcessor>,
    pub pipeline: RenderPipeline,
}

impl ShaderCtx {
    pub fn new(
        conf: &ShaderConf,
        renderer: &Renderer,
        sample_processor: &SampleProcessor,
        texture_format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        let bar_processor = conf.amount_bars.map(|amount| {
            BarProcessor::new(
                sample_processor,
                shady_audio::Config {
                    amount_bars: amount,
                    ..Default::default()
                },
            )
        });

        let pipeline = {
            let fragment_module = match &conf.code {
                ShaderCode::Glsl(code) => get_glsl_module(code)?,
                ShaderCode::Wgsl(code) => get_wgsl_module(code)?,
                ShaderCode::VibeShader(repo_dir_name) => {
                    let url = format!("https://raw.githubusercontent.com/TornaxO7/vibe-shaders/refs/heads/main/{}/code.toml", repo_dir_name);

                    let body = reqwest::blocking::get(url)
                        .context("Send http request to fetch shader code")?
                        .text()
                        .unwrap();
                    let shader_code: ShaderCode = toml::from_str(&body)?;

                    match shader_code {
                        ShaderCode::Glsl(code) => get_glsl_module(code)?,
                        ShaderCode::Wgsl(code) => get_wgsl_module(code)?,
                        ShaderCode::VibeShader(_) => {
                            anyhow::bail!("hello")
                        }
                    }
                }
            };

            get_render_pipeline(
                renderer,
                ShaderSource::Naga(Cow::Owned(fragment_module)),
                texture_format,
            )
        };

        Ok(Self {
            bar_processor,
            pipeline,
        })
    }
}

impl AsRef<ShaderCtx> for ShaderCtx {
    fn as_ref(&self) -> &ShaderCtx {
        self
    }
}

fn get_glsl_module(code: impl AsRef<str>) -> anyhow::Result<Module> {
    let mut frontend = glsl::Frontend::default();
    frontend
        .parse(&glsl::Options::from(ShaderStage::Fragment), code.as_ref())
        .map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
    // .with_context(|| {
    //     format!(
    //         "your {}{}shader (it's a glsl shader) of '{}' is invalid",
    //         shader_index, num_abbreviation, output_name
    //     )
    // })
}

fn get_wgsl_module(code: impl AsRef<str>) -> anyhow::Result<Module> {
    wgsl::parse_str(code.as_ref()).map_err(|err| anyhow!("{}", err.emit_to_string(code.as_ref())))
    // .with_context(|| {
    //     format!(
    //         "your {}{} shader (it's a wgsl shader) of '{}' is invalid",
    //         shader_index, num_abbreviation, output_name
    //     )
    // })
}

fn get_render_pipeline(
    renderer: &Renderer,
    shader_source: ShaderSource<'_>,
    texture_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let device = renderer.device();

    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shady vertex shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shady fragment shader"),
        source: shader_source,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shady pipeline layout"),
        bind_group_layouts: &[renderer.bind_group_layout()],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shady render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("vertex_main"),
            buffers: &[super::vertices::BUFFER_LAYOUT],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: Some(super::FRAGMENT_ENTRYPOINT),
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview: None,
        cache: None,
    })
}
