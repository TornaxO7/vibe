use super::{Component, ShaderCode, ShaderCodeError};
use crate::Renderable;
use pollster::FutureExt;
use std::borrow::Cow;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig, SampleProcessor,
};
use wgpu::include_wgsl;

const ENTRYPOINT: &str = "main";

pub struct FragmentCanvasDescriptor<'a, F: Fetcher> {
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: BarProcessorConfig,
    pub device: &'a wgpu::Device,
    pub format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    pub fragment_code: ShaderCode,
}

pub struct FragmentCanvas {
    bar_processor: BarProcessor,

    iresolution: wgpu::Buffer,
    freqs: wgpu::Buffer,
    itime: wgpu::Buffer,
    imouse: wgpu::Buffer,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new<F: Fetcher>(desc: &FragmentCanvasDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let iresolution = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iResolution` buffer"),
            size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let freqs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `freqs` buffer"),
            size: (std::mem::size_of::<f32>() * usize::from(u16::from(desc.audio_conf.amount_bars)))
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let itime = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iTime` buffer"),
            size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let imouse = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iMouse` buffer"),
            size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group0_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment canvas: Bind group 0 layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline = {
            let vertex_module =
                device.create_shader_module(include_wgsl!("../utils/full_screen_vertex.wgsl"));

            let fragment_module = {
                let source = desc.fragment_code.source().map_err(ShaderCodeError::from)?;

                let shader_source = match desc.fragment_code.language {
                    super::ShaderLanguage::Wgsl => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.wgsl");
                        let full_code = format!("{}\n{}", PREAMBLE, &source);
                        wgpu::ShaderSource::Wgsl(Cow::Owned(full_code))
                    }
                    super::ShaderLanguage::Glsl => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.glsl");
                        let full_code = format!("{}\n{}", PREAMBLE, &source);
                        wgpu::ShaderSource::Glsl {
                            shader: Cow::Owned(full_code),
                            stage: wgpu::naga::ShaderStage::Fragment,
                            defines: &[],
                        }
                    }
                };

                let err_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment canvas fragment module"),
                    source: shader_source,
                });

                if let Some(err) = err_scope.pop().block_on() {
                    return Err(ShaderCodeError::ParseError(err));
                }

                module
            };

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fragment canvas: Pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout],
                ..Default::default()
            });

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Fragment canvas render pipeline",
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: Some(ENTRYPOINT),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fragment canvas: Bind group 0"),
            layout: &bind_group0_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: iresolution.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: freqs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: itime.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: imouse.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            bar_processor,

            iresolution,
            freqs,
            itime,
            imouse,

            bind_group0,

            pipeline,
        })
    }
}

impl Renderable for FragmentCanvas {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for FragmentCanvas {
    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        queue.write_buffer(
            &self.iresolution,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }

    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(&self.freqs, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        queue.write_buffer(&self.itime, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_mouse_position(&mut self, queue: &wgpu::Queue, new_pos: (f32, f32)) {
        queue.write_buffer(
            &self.imouse,
            0,
            bytemuck::cast_slice(&[new_pos.0, new_pos.1]),
        );
    }
}
