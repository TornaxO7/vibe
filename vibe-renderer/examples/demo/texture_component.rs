use vibe_renderer::{Component, Renderable, ResourceManager};
use wgpu::util::DeviceExt;

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

    pub const RESOLUTION: u32 = 0;
    pub const SAMPLER: u32 = 1;
    pub const TEXTURE: u32 = 2;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, vibe_renderer::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Sampler, vibe_renderer::util::sampler(SAMPLER, wgpu::ShaderStages::FRAGMENT)),
            (ResourceID::Texture, vibe_renderer::util::texture(TEXTURE, wgpu::ShaderStages::FRAGMENT)),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    Sampler,
    Texture,
}

pub struct TextureComponentDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
}

pub struct TextureComponent {
    resource_manager: ResourceManager<ResourceID>,
    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    vbuffer: wgpu::Buffer,
}

impl TextureComponent {
    pub fn new(desc: &TextureComponentDescriptor) -> Self {
        let device = desc.device;

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();

        resource_manager.extend_buffers([(
            ResourceID::Resolution,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Texture component: iResolution buffer"),
                size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        )]);

        resource_manager.insert_sampler(
            ResourceID::Sampler,
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture component: Sampler"),
                ..vibe_renderer::util::DEFAULT_SAMPLER_DESCRIPTOR
            }),
        );

        resource_manager.insert_texture(ResourceID::Texture, desc.texture.clone());

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture component: vbuffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "Texture component: Bind group 0",
            device,
            &bind_group0_mapping,
        );

        let pipeline = {
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Texture component: Shader module"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./texture_component_shader.wgsl").into(),
                ),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Texture component: Pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&vibe_renderer::util::simple_pipeline_descriptor(
                vibe_renderer::util::SimpleRenderPipelineDescriptor {
                    label: "Texture component: Render pipeline",
                    layout: Some(&pipeline_layout),
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
            resource_manager,
            bind_group0,

            pipeline,
            vbuffer,
        }
    }
}

impl Renderable for TextureComponent {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

impl Component for TextureComponent {
    fn update_audio(
        &mut self,
        _queue: &wgpu::Queue,
        _processor: &vibe_audio::SampleProcessor<vibe_audio::fetcher::SystemAudioFetcher>,
    ) {
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &vibe_renderer::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::Resolution)
                .unwrap();

            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
