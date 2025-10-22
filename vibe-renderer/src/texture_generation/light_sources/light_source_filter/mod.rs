use crate::texture_generation::TextureGeneratorStep;

pub struct LightSourceFilterDescriptor<'a> {
    pub device: &'a wgpu::Device,

    pub src: wgpu::TextureView,
}

pub struct LightSourceFilter {}

impl LightSourceFilter {
    pub fn step(desc: LightSourceFilterDescriptor) -> Box<dyn TextureGeneratorStep> {
        let LightSourceFilterDescriptor { device, src } = desc;

        todo!()
    }
}

impl TextureGeneratorStep for LightSourceFilter {
    fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: u32, y: u32) {
        todo!()
    }

    fn amount_steps(&self) -> u32 {
        todo!()
    }
}
