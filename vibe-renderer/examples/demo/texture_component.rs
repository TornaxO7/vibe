use vibe_renderer::{Component, Renderable};
use wgpu::{include_wgsl, util::DeviceExt};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    resolution: [f32; 2],
}

pub struct TextureComponentDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
}

pub struct TextureComponent {
    bind_group0: wgpu::BindGroup,
    fragment_params_buffer: wgpu::Buffer,
    _sampler: wgpu::Sampler,
    _texture: wgpu::Texture,

    pipeline: wgpu::RenderPipeline,
}

impl TextureComponent {
    pub fn new(desc: &TextureComponentDescriptor) -> Self {
        let device = desc.device;

        let fragment_params_buffer = {
            let fragment_params = FragmentParams {
                resolution: [0f32; 2],
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Texture component: iResolution buffer"),
                contents: bytemuck::bytes_of(&fragment_params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture component: Sampler"),
            ..vibe_renderer::util::DEFAULT_SAMPLER_DESCRIPTOR
        });

        let texture = desc.texture.clone();

        let pipeline = {
            let module =
                device.create_shader_module(include_wgsl!("./texture_component_shader.wgsl"));

            device.create_render_pipeline(&vibe_renderer::util::simple_pipeline_descriptor(
                vibe_renderer::util::SimpleRenderPipelineDescriptor {
                    label: "Texture component: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("main_vs"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &module,
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

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture component: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fragment_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        Self {
            bind_group0,
            fragment_params_buffer,
            _sampler: sampler,
            _texture: texture,

            pipeline,
        }
    }
}

impl Renderable for TextureComponent {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
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
            queue.write_buffer(
                &self.fragment_params_buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
