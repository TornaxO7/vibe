use crate::Renderer;

pub struct PulseLightSourcesDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub format: wgpu::TextureFormat,
}
