mod descriptor;

use crate::{Component, Renderable};
use wgpu::{include_wgsl, util::DeviceExt};

pub use descriptor::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    pub time: f32,
}

/// Renders the glowing line where the blocks spawn.
pub struct GlowingLineRenderer {
    pipeline: wgpu::RenderPipeline,

    fp: wgpu::Buffer,
    bind_group0: wgpu::BindGroup,
}

impl GlowingLineRenderer {
    pub fn new(desc: &GlowingLineDescriptor) -> Self {
        let device = desc.renderer.device();

        let fp = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rising blocks, glowing line: Fragment parameters"),
            contents: &bytemuck::bytes_of(&FragmentParams { time: 0f32 }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rising blocks, glowing line: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fp.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            fp,
            bind_group0,
        }
    }
}

impl Renderable for GlowingLineRenderer {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.draw(0..5, 0..1);
    }
}

impl Component for GlowingLineRenderer {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        queue.write_buffer(&self.fp, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_resolution(&mut self, _renderer: &crate::Renderer, _new_resolution: [u32; 2]) {}

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
