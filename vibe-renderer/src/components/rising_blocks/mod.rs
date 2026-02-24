mod blocks;
mod descriptor;

use crate::{components::Rgba, Component, ComponentAudio, Renderable};
use blocks::{BlocksDescriptor, BlocksRenderer};
use vibe_audio::fetcher::Fetcher;

pub use descriptor::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    color1: Rgba,
}

pub struct RisingBlocks {
    blocks: BlocksRenderer,
}

impl RisingBlocks {
    pub fn new<F: Fetcher>(desc: &RisingBlocksDescriptor<F>) -> Self {
        let blocks = BlocksRenderer::new(&BlocksDescriptor {
            renderer: desc.renderer,
            sample_processor: desc.sample_processor,
            audio_conf: desc.audio_conf.clone(),
            format: desc.format,
            canvas_height: desc.canvas_height,
        });

        Self { blocks }
    }
}

impl Renderable for RisingBlocks {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        self.blocks.render_with_renderpass(pass);
    }
}

impl Component for RisingBlocks {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        self.blocks.update_time(queue, new_time);
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        self.blocks.update_resolution(renderer, new_resolution);
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

impl<F: Fetcher> ComponentAudio<F> for RisingBlocks {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        self.blocks.update_audio(queue, processor);
    }
}
