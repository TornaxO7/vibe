mod descriptor;

pub use descriptor::*;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{texture_generation::EdgeDistanceMap, Component, Renderable};

type VertexPosition = [f32; 2];
#[rustfmt::skip]
const VERTICES: [VertexPosition; 3] = [
    [-3., -1.], // bottom left
    [1., -1.], // bottom right
    [1., 3.] // top right
];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DataBinding {
    resolution: [f32; 2],
    time: f32,

    _padding: f32,
}

pub struct EncrustWallpaper {
    data_binding_buffer: wgpu::Buffer,
    vbuffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    data_binding: DataBinding,
}

impl EncrustWallpaper {
    pub fn new(desc: &WallpaperEncrustDescriptor) -> Self {
        let renderer = desc.renderer;
        let device = renderer.device();
        let queue = renderer.queue();

        let edge_texture = renderer.generate(EdgeDistanceMap { src: &desc.img });

        let img_texture = {
            let img = desc.img.to_rgba8();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Encrust Wallpaper: Image texture"),
                size: wgpu::Extent3d {
                    width: img.width(),
                    height: img.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                img.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(std::mem::size_of::<[u8; 4]>() as u32 * img.width()),
                    rows_per_image: Some(img.height()),
                },
                texture.size(),
            );

            texture
        };

        let data_binding = DataBinding {
            resolution: [1f32; 2],
            time: 0.,

            _padding: 0.,
        };

        let data_binding_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Encrust Wallpaper: data-binding buffer"),
            size: std::mem::size_of::<DataBinding>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Encrust Wallpaper: Vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Encrust Wallpaper: Sampler"),
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
            let shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Encrust Wallpaper: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Encrust Wallpaper: Bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &img_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &edge_texture.create_view(&wgpu::TextureViewDescriptor::default()),
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
            vbuffer,

            data_binding,
            bind_group,
            pipeline,

            data_binding_buffer,
        }
    }
}

impl Renderable for EncrustWallpaper {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..VERTICES.len() as u32, 0..1);
    }
}

impl Component for EncrustWallpaper {
    fn update_audio(
        &mut self,
        _queue: &wgpu::Queue,
        _processor: &vibe_audio::SampleProcessor<vibe_audio::fetcher::SystemAudioFetcher>,
    ) {
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        self.data_binding.time = new_time;

        queue.write_buffer(
            &self.data_binding_buffer,
            0,
            bytemuck::bytes_of(&self.data_binding),
        );
    }

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
