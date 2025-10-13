mod descriptor;

pub use descriptor::*;
use wgpu::include_wgsl;

use crate::{Component, Renderable};

const LABEL: &str = "Pulse light sources";

pub struct PulseLightSources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl PulseLightSources {
    pub fn new(desc: &PulseLightSourcesDescriptor) -> Self {
        let device = desc.renderer.device();

        let pipeline = {
            let vertex_module =
                device.create_shader_module(include_wgsl!("../../utils/full_screen_vertex.wgsl"));

            let fragment_module =
                device.create_shader_module(include_wgsl!("./fragment_shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: LABEL,
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[],
        });

        todo!()
    }
}

impl Renderable for PulseLightSources {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        todo!()
    }
}

impl Component for PulseLightSources {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &vibe_audio::SampleProcessor<vibe_audio::fetcher::SystemAudioFetcher>,
    ) {
        todo!()
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        todo!()
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
