pub struct WhiteNoiseDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub format: wgpu::TextureFormat,

    pub seed: f32,
}
