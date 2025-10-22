mod descriptor;

pub use descriptor::*;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{texture_generation::LightSources, Component, Renderable};

const LABEL: &str = "Pulse light sources";

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DataBinding {
    resolution: [f32; 2],
    wallpaper_brightness: f32,

    _padding: f32,
}

pub struct PulseLightSources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    data_binding: DataBinding,
    data_binding_buffer: wgpu::Buffer,
}

impl PulseLightSources {
    pub fn new(desc: &PulseLightSourcesDescriptor) -> Self {
        let renderer = desc.renderer;
        let device = renderer.device();
        let queue = renderer.queue();

        let wallpaper = crate::util::load_img_to_texture(device, queue, &desc.img);

        let light_source_map = renderer.generate(&LightSources {
            src: &desc.img,
            light_threshold: desc.light_threshold,
        });

        let data_binding = DataBinding {
            resolution: [0f32; 2],
            wallpaper_brightness: desc.wallpaper_brightness.clamp(0., 1.),

            _padding: 0f32,
        };

        let data_binding_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pulse light sources: Data binding buffer"),
            contents: bytemuck::bytes_of(&data_binding),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Pulse light sources: Sampler"),
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 1.,
            lod_max_clamp: 1.,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

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
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &wallpaper.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &light_source_map.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: data_binding_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,

            data_binding,
            data_binding_buffer,
        }
    }
}

impl Renderable for PulseLightSources {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}

impl Component for PulseLightSources {
    fn update_audio(
        &mut self,
        _queue: &wgpu::Queue,
        _processor: &vibe_audio::SampleProcessor<vibe_audio::fetcher::SystemAudioFetcher>,
    ) {
    }

    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        self.data_binding.resolution = [new_resolution[0] as f32, new_resolution[1] as f32];

        queue.write_buffer(
            &self.data_binding_buffer,
            0,
            bytemuck::bytes_of(&self.data_binding),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
