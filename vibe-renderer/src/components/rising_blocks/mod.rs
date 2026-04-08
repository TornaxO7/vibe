mod blocks;
mod descriptor;
mod glowing_line;

use crate::{Component, ComponentAudio, Renderable};
use blocks::{BlocksColor, BlocksDescriptor, BlocksRenderer};
use glowing_line::{GlowingLineDescriptor, GlowingLineRenderer};
use vibe_audio::fetcher::Fetcher;

pub use descriptor::*;

pub struct RisingBlocks {
    blocks: BlocksRenderer,
    glowing_line: GlowingLineRenderer,
}

impl RisingBlocks {
    pub fn new<F: Fetcher>(desc: &RisingBlocksDescriptor<F>) -> Self {
        debug_assert!(0f32 <= desc.canvas_height && desc.canvas_height <= 1f32);

        let blocks = {
            #[allow(clippy::infallible_destructuring_match)]
            let color = match desc.foreground {
                RisingBlocksForeground::Color(color) => BlocksColor::Color(color),
            };

            BlocksRenderer::new(&BlocksDescriptor {
                renderer: desc.renderer,
                sample_processor: desc.sample_processor,
                audio_conf: desc.audio_conf.clone(),
                format: desc.format,
                canvas_height: desc.canvas_height,
                spawn_random: desc.spawn_random,
                speed: desc.speed,
                easing: desc.easing,
                beat_threshold: desc.beat_threshold,
                color,
            })
        };

        let glowing_line = {
            #[allow(clippy::infallible_destructuring_match)]
            let color1 = match desc.background {
                RisingBlocksBackground::Color(color) => color,
            };

            GlowingLineRenderer::new(&GlowingLineDescriptor {
                renderer: desc.renderer,
                format: desc.format,

                canvas_height: desc.canvas_height,

                color1,
            })
        };

        Self {
            blocks,
            glowing_line,
        }
    }
}

impl Renderable for RisingBlocks {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        self.blocks.render_with_renderpass(pass);
        self.glowing_line.render_with_renderpass(pass);
    }
}

impl Component for RisingBlocks {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        self.blocks.update_time(queue, new_time);
        self.glowing_line.update_time(queue, new_time);
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        self.blocks.update_resolution(renderer, new_resolution);
        self.glowing_line
            .update_resolution(renderer, new_resolution);
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

impl<F: Fetcher> ComponentAudio<F> for RisingBlocks {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        self.blocks.update_audio(queue, processor);
    }
}
