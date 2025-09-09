mod descriptor;

pub use descriptor::*;
use wgpu::util::DeviceExt;

use crate::{resource_manager::ResourceManager, Renderable};

type VertexPosition = [f32; 2];

#[rustfmt::skip]
const VERTICES: [VertexPosition; 3] = [
    [-3., -1.], // bottom left
    [1., -1.], // bottom right
    [1., 3.] // top right
];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const SEED: u32 = 0;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Seed, crate::util::buffer(SEED, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Seed,
}

pub struct WhiteNoise {
    _resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    vbuffer: wgpu::Buffer,
}

impl WhiteNoise {
    pub fn new(desc: &WhiteNoiseDescriptor) -> Self {
        let device = desc.device;

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();

        resource_manager.insert_buffer(
            ResourceID::Seed,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("WhiteNoise: `seed` buffer"),
                contents: bytemuck::bytes_of(&desc.seed),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        );

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("WhiteNoise: vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "WhiteNoise: bind group 0",
            device,
            &bind_group0_mapping,
        );

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("WhiteNoise: Pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout],
                push_constant_ranges: &[],
            });

            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("WhiteNoise: Shader module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
            });

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "WhiteNoise: Render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: Some("main_vs"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<VertexPosition>()
                                as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: Some("main_fs"),
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

        Self {
            _resource_manager: resource_manager,
            bind_group0,
            pipeline,
            vbuffer,
        }
    }
}

impl Renderable for WhiteNoise {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}
