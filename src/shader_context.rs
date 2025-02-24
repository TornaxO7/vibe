use anyhow::anyhow;
use shady_audio::{config::ShadyAudioConfig, fetcher::SystemAudioFetcher, ShadyAudio};
use std::{borrow::Cow, ptr::NonNull, time::Instant};
use tracing::info;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::shell::{wlr_layer::LayerSurface, WaylandSurface};
use wayland_client::{Connection, Proxy};
use wgpu::{
    naga::{front::glsl, ShaderStage},
    BindGroup, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, PresentMode, RenderPipeline, ShaderStages, Surface, SurfaceConfiguration,
};

use crate::{
    gpu_context::GpuCtx,
    output_config::{OutputConfig, ShaderCode},
    output_context::Size,
};

pub struct ShaderCtx {
    audio: Buffer,
    resolution: Buffer,
    time: Buffer,

    instant: Instant,
    shady: ShadyAudio,

    bind_group: BindGroup,
    pipeline: RenderPipeline,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl ShaderCtx {
    pub fn new(
        conn: &Connection,
        size: Size,
        layer_surface: &LayerSurface,
        gpu: &GpuCtx,
        config: &OutputConfig,
    ) -> anyhow::Result<Self> {
        let shady = ShadyAudio::new(
            SystemAudioFetcher::default(|err| panic!("{}", err)).unwrap(),
            ShadyAudioConfig {
                amount_bars: config.amount_bars,
                ..Default::default()
            },
        )
        .unwrap();

        let audio = gpu.device().create_buffer(&BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<f32>() * usize::from(config.amount_bars)) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let resolution = gpu.device().create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let time = gpu.device().create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let surface: Surface<'static> = {
            let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
            ));

            let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
                NonNull::new(layer_surface.wl_surface().id().as_ptr() as *mut _).unwrap(),
            ));

            unsafe {
                gpu.instance()
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            }
        };

        let surface_config = {
            let surface_caps = surface.get_capabilities(gpu.adapter());
            let format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap();

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
                view_formats: vec![],
                desired_maximum_frame_latency: 3,
            }
        };
        surface.configure(gpu.device(), &surface_config);

        let vertex_module = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/vertex_shader.wgsl").into(),
                ),
            });

        let fragment_module = match &config.shader_code {
            ShaderCode::Glsl(code) => {
                let mut frontend = glsl::Frontend::default();
                let module = frontend
                    .parse(&glsl::Options::from(ShaderStage::Fragment), &code)
                    .map_err(|err| anyhow!("{}", err.emit_to_string(code)))?;

                gpu.device()
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
                    })
            }
        };

        let (pipeline, bind_group) = {
            let bind_group_layout =
                gpu.device()
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            // iAudio
                            bind_group_layout_entry(
                                0,
                                BufferBindingType::Storage { read_only: true },
                            ),
                            // iResolution
                            bind_group_layout_entry(1, BufferBindingType::Uniform),
                            // iTime
                            bind_group_layout_entry(2, BufferBindingType::Uniform),
                        ],
                    });

            let pipeline_layout =
                gpu.device()
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    });

            let pipeline = gpu
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some("vertex_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[crate::vertices::BUFFER_LAYOUT],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some("main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                    cache: None,
                });

            let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: audio.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: resolution.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: time.as_entire_binding(),
                    },
                ],
            });

            (pipeline, bind_group)
        };

        Ok(Self {
            shady,
            audio,
            time,
            resolution,
            surface,
            config: surface_config,
            pipeline,
            bind_group,
            instant: Instant::now(),
        })
    }

    pub fn resize(&mut self, gpu: &GpuCtx, new_size: Size) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(gpu.device(), &self.config);

            gpu.queue().write_buffer(
                &self.resolution,
                0,
                bytemuck::cast_slice(&[self.config.width as f32, self.config.height as f32]),
            );
        }
    }

    pub fn size(&self) -> Size {
        Size::from((self.config.width, self.config.height))
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        // iAudio
        {
            let bars = self.shady.get_bars();
            queue.write_buffer(&self.audio, 0, bytemuck::cast_slice(bars));
        }
        // iTime
        {
            let time = self.instant.elapsed().as_secs_f32();
            queue.write_buffer(&self.time, 0, bytemuck::cast_slice(&time.to_ne_bytes()));
        }
    }
}

fn bind_group_layout_entry(binding: u32, ty: BufferBindingType) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
