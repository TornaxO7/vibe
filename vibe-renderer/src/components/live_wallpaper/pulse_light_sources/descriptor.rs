use crate::Renderer;

pub struct PulseLightSourcesDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub format: wgpu::TextureFormat,

    pub img: image::DynamicImage,
    pub light_threshold: f32,
    pub wallpaper_brightness: f32,
}
