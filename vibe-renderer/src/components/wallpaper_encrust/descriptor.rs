pub struct WallpaperEncrustDescriptor<'a> {
    pub renderer: &'a crate::Renderer,
    // pub sample_processor: &'a SampleProcessor<F>,
    pub texture_format: wgpu::TextureFormat,

    // pub variant: WallpaperEncrustVariant,
    pub img: image::DynamicImage,
}

// pub enum WallpaperEncrustVariant {
//     EdgeGlow,
// }
