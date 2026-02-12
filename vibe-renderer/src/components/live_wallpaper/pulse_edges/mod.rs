mod descriptor;

pub use descriptor::*;

use crate::{
    components::ComponentAudio,
    texture_generation::{edge_distance_map::EdgeDistanceMap, GaussianBlur},
    Component, Renderable,
};
use std::num::NonZero;
use vibe_audio::{fetcher::Fetcher, BarProcessor, SampleProcessor};
use wgpu::include_wgsl;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DataBinding {
    resolution: [f32; 2],
    time: f32,
    freq: f32,

    wallpaper_brightness: f32,
    edge_width: f32,
    pulse_brightness: f32,

    _padding: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum PulseEdgesError {
    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Cache(#[from] crate::cache::CacheError),

    #[error("The given kernel size must be odd but '{0}' was given")]
    EvenKernelSize(usize),
}

pub struct PulseEdges {
    bar_processor: BarProcessor,

    data_binding_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    data_binding: DataBinding,
}

impl PulseEdges {
    pub fn new<F: Fetcher>(desc: &PulseEdgesDescriptor<F>) -> Result<Self, PulseEdgesError> {
        if desc.kernel_size.is_multiple_of(2) {
            return Err(PulseEdgesError::EvenKernelSize(desc.kernel_size));
        }

        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            vibe_audio::BarProcessorConfig {
                amount_bars: NonZero::new(1).unwrap(),
                freq_range: desc.freq_range.clone(),
                sensitivity: desc.audio_sensitivity,
                ..Default::default()
            },
        );

        let renderer = desc.renderer;
        let device = renderer.device();
        let queue = renderer.queue();

        #[cfg(not(target_arch = "wasm32"))]
        let distance_map = crate::cache::load(
            renderer,
            &EdgeDistanceMap {
                src: &desc.img,
                high_threshold_ratio: desc.high_threshold_ratio,
                low_threshold_ratio: desc.low_threshold_ratio,

                sigma: desc.sigma,
                kernel_size: desc.kernel_size,
            },
        )?;

        #[cfg(target_arch = "wasm32")]
        let distance_map = renderer.generate(&EdgeDistanceMap {
            src: &desc.img,
            high_threshold_ratio: desc.high_threshold_ratio,
            low_threshold_ratio: desc.low_threshold_ratio,

            sigma: desc.sigma,
            kernel_size: desc.kernel_size,
        });

        let gaussian_blur = renderer.generate(&GaussianBlur {
            src: &desc.img,
            sigma: 5.,
            kernel_size: 9,
        });

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
            freq: 0.,
            wallpaper_brightness: desc.wallpaper_brightness.clamp(0., 1.),
            edge_width: {
                // invert, so that `edge_width` high => bigger width for the edge
                1. / desc.edge_width.max(f32::EPSILON)
            },
            pulse_brightness: desc.pulse_brightness,
            _padding: 0,
        };

        let data_binding_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Encrust Wallpaper: data-binding buffer"),
            size: std::mem::size_of::<DataBinding>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Encrust Wallpaper: Sampler"),
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            lod_min_clamp: 1.,
            lod_max_clamp: 1.,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let pipeline = {
            let vertex_shader =
                device.create_shader_module(include_wgsl!("../../utils/full_screen_vertex.wgsl"));

            let fragment_shader = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Encrust Wallpaper: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_shader,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_shader,
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
                        &distance_map.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &gaussian_blur.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: data_binding_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            bar_processor,

            data_binding,
            bind_group,
            pipeline,

            data_binding_buffer,
        })
    }
}

impl Renderable for PulseEdges {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl<F: Fetcher> ComponentAudio<F> for PulseEdges {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>) {
        let bars = self.bar_processor.process_bars(processor);

        self.data_binding.freq = bars[0][0];

        queue.write_buffer(
            &self.data_binding_buffer,
            0,
            bytemuck::bytes_of(&self.data_binding),
        );
    }
}

impl Component for PulseEdges {
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
