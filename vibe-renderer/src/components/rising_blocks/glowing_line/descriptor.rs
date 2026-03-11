use crate::Renderer;

pub struct GlowingLineDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub format: wgpu::TextureFormat,
}
