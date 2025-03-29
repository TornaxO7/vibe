pub mod config;
mod resources;

pub use resources::ShaderResources;
use vibe_daemon::{renderer::RenderShader, resources::ResourceCollection};

pub struct Shader {
    pub resources: ShaderResources,

    pipeline: wgpu::RenderPipeline,
}

impl Shader {
    pub fn new(resources: ShaderResources, pipeline: wgpu::RenderPipeline) -> Self {
        Self {
            resources,
            pipeline,
        }
    }
}

impl RenderShader for &Shader {
    fn bind_group(&self) -> &wgpu::BindGroup {
        self.resources.bind_group()
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
