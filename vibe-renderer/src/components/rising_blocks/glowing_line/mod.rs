mod descriptor;

use crate::Renderable;
use wgpu::include_wgsl;

pub use descriptor::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {}

/// Renders the glowing line where the blocks spawn.
pub struct GlowingLineRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl GlowingLineRenderer {
    pub fn new(desc: &GlowingLineDescriptor) -> Self {
        let device = desc.renderer.device();

        let pipeline = {
            let module = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Glowing line: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::OVER,
                            }),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        Self { pipeline }
    }
}

impl Renderable for GlowingLineRenderer {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..5, 0..1);
    }
}
