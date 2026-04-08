mod descriptor;

use crate::{
    components::utils::wgsl_types::{Vec3f, Vec4f},
    Component, Renderable,
};
use wgpu::{include_wgsl, util::DeviceExt};

pub use descriptor::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {
    pub canvas_height: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    pub color1: Vec4f,

    pub time: f32,
    _padding: Vec3f,
}

/// Renders the glowing line where the blocks spawn.
pub struct GlowingLineRenderer {
    pipeline: wgpu::RenderPipeline,

    _vp: wgpu::Buffer,
    fp: wgpu::Buffer,
    bind_group0: wgpu::BindGroup,
}

impl GlowingLineRenderer {
    pub fn new(desc: &GlowingLineDescriptor) -> Self {
        let device = desc.renderer.device();

        let vp = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rising blocks, glowing line: Vertex params"),
            contents: bytemuck::bytes_of(&VertexParams {
                canvas_height: desc.canvas_height,
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let fp = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rising blocks, glowing line: Fragment parameters"),
            contents: bytemuck::bytes_of(&FragmentParams {
                time: 0f32,
                color1: desc.color1,
                ..Default::default()
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let pipeline = {
            let module = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Glowing line: Render pipeline",
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Glowing line: Render pipeline layout"),
                            bind_group_layouts: &[Some(&device.create_bind_group_layout(
                                &wgpu::BindGroupLayoutDescriptor {
                                    label: Some("Glowing line: Bind group 0 layout"),
                                    entries: &[
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 0,
                                            visibility: wgpu::ShaderStages::VERTEX,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Uniform,
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 1,
                                            visibility: wgpu::ShaderStages::FRAGMENT,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Uniform,
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                    ],
                                },
                            ))],
                            ..Default::default()
                        }),
                    ),
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
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vp.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: fp.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            _vp: vp,
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
        queue.write_buffer(
            &self.fp,
            std::mem::offset_of!(FragmentParams, time) as u64,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, _renderer: &crate::Renderer, _new_resolution: [u32; 2]) {}

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
