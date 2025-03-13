pub trait RenderShader {
    fn bind_group(&self) -> &wgpu::BindGroup;

    fn pipeline(&self) -> &wgpu::RenderPipeline;
}
