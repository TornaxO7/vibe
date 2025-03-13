#[derive(Debug)]
pub struct RenderCtx<'a> {
    pub output: &'a wgpu::SurfaceTexture,
    pub bind_groups: &'a [wgpu::BindGroup],
    pub pipelines: &'a [wgpu::RenderPipeline],
}
