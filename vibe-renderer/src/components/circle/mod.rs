use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::Component;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings0 {
    iResolution,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Bindings1 {
    Freqs,
}

pub struct CircleDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a shady_audio::SampleProcessor,
    pub audio_conf: shady_audio::BarProcessorConfig,
    pub texture_format: wgpu::TextureFormat,
}

pub struct Circle {
    bar_processor: shady_audio::BarProcessor,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    pipeline: wgpu::RenderPipeline,
}

impl Circle {
    pub fn new(desc: &CircleDescriptor) -> Self {
        let mut bind_group0_builder = BindGroupManager::builder(Some("Circle: Bind group 0"));
        let mut bind_group1_builder = BindGroupManager::builder(Some("Circle: Bind group 1"));

        todo!()
    }
}

impl Renderable for Circle {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        if !self.bind_group0.is_empty() {
            pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        }

        if !self.bind_group1.is_empty() {
            pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);
        }

        pass.set_pipeline(&self.pipeline);
        todo!()
    }
}

impl Component for Circle {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &shady_audio::SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Freqs as u32) {
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
        }
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::iResolution as u32) {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&new_resolution));
        }
    }
}
