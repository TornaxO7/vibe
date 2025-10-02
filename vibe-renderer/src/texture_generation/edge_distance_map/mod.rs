use crate::texture_generation::TextureGenerator;

pub struct EdgeDistanceMap {
    pub src: wgpu::TextureView,
}

impl TextureGenerator for EdgeDistanceMap {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let src = gray_scale_and_blur(&self.src, device, queue);
        todo!()
    }
}

fn gray_scale_and_blur(
    src: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::Texture {
    todo!()
}
