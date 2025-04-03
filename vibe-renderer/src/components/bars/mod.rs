use std::{borrow::Cow, num::NonZero};

use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use super::{bind_group_entry, bind_group_layout_entry, Component, ParseErrorMsg, ShaderCode};

const SHADER_ENTRYPOINT: &str = "main";

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

pub struct BarsDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a SampleProcessor,
    pub audio_conf: shady_audio::Config,
    pub texture_format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    pub fragment_code: ShaderCode,
    /// width, height
    pub resolution: [u32; 2],
    pub max_height: f32,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    freqs_buffer: wgpu::Buffer,
    _column_width_buffer: wgpu::Buffer,
    _padding_buffer: wgpu::Buffer,
    time_buffer: wgpu::Buffer,
    resolution_buffer: wgpu::Buffer,
    _max_height_buffer: wgpu::Buffer,

    column_padding_resolution_bind_group: wgpu::BindGroup,
    freqs_time_bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl Bars {
    pub fn new(desc: &BarsDescriptor) -> Result<Self, ParseErrorMsg> {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let column_width = VERTEX_SURFACE_WIDTH / u16::from(amount_bars) as f32;
        let padding = column_width / 5.;

        let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bar freq buffer"),
            size: (std::mem::size_of::<f32>() * usize::from(u16::from(amount_bars))) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let column_width_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bar column width buffer"),
            contents: bytemuck::cast_slice(&[column_width]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let padding_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bar padding buffer"),
            contents: bytemuck::cast_slice(&[padding]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bar time buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bar resolution buffer"),
            contents: bytemuck::cast_slice(&[desc.resolution[0] as f32, desc.resolution[1] as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let max_height_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bar max_height buffer"),
            contents: bytemuck::cast_slice(&[desc.max_height]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let (pipeline, column_padding_bind_group, freqs_time_bind_group) = {
            let column_padding_resolution_height_bind_group_layout = device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bar column padding bind group layout"),
                    entries: &[
                        bind_group_layout_entry(
                            0,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Uniform,
                        ),
                        bind_group_layout_entry(
                            1,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Uniform,
                        ),
                        bind_group_layout_entry(
                            2,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                        bind_group_layout_entry(
                            3,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ],
                });

            let column_padding_resolution_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bar column padding bind group"),
                    layout: &column_padding_resolution_height_bind_group_layout,
                    entries: &[
                        bind_group_entry(0, &column_width_buffer),
                        bind_group_entry(1, &padding_buffer),
                        bind_group_entry(2, &resolution_buffer),
                        bind_group_entry(3, &max_height_buffer),
                    ],
                });

            let freqs_time_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bar freqs time bind group layout"),
                    entries: &[
                        bind_group_layout_entry(
                            0,
                            wgpu::ShaderStages::VERTEX,
                            wgpu::BufferBindingType::Storage { read_only: true },
                        ),
                        bind_group_layout_entry(
                            1,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    ],
                });

            let freqs_time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bar freqs time bind group"),
                layout: &freqs_time_bind_group_layout,
                entries: &[
                    bind_group_entry(0, &freqs_buffer),
                    bind_group_entry(1, &time_buffer),
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bar pipeline layout"),
                bind_group_layouts: &[
                    &column_padding_resolution_height_bind_group_layout,
                    &freqs_time_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bar vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
            });

            let fragment_module = {
                let module = match &desc.fragment_code {
                    ShaderCode::Wgsl(code) => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.wgsl");
                        super::parse_wgsl_fragment_code(PREAMBLE, code)?
                    }
                    ShaderCode::Glsl(code) => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.glsl");
                        super::parse_glsl_fragment_code(PREAMBLE, code)?
                    }
                };
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Bar fragment module"),
                    source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
                })
            };

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Bar render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: Some(SHADER_ENTRYPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_module,
                    entry_point: Some(SHADER_ENTRYPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: desc.texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            });

            (
                pipeline,
                column_padding_resolution_bind_group,
                freqs_time_bind_group,
            )
        };

        Ok(Self {
            amount_bars,
            bar_processor,

            freqs_buffer,
            _column_width_buffer: column_width_buffer,
            _padding_buffer: padding_buffer,
            time_buffer,
            resolution_buffer,
            _max_height_buffer: max_height_buffer,

            pipeline,
            column_padding_resolution_bind_group: column_padding_bind_group,
            freqs_time_bind_group,
        })
    }
}

impl Component for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.column_padding_resolution_bind_group, &[]);
        pass.set_bind_group(1, &self.freqs_time_bind_group, &[]);
        pass.set_pipeline(&self.pipeline);

        pass.draw(0..4, 0..u16::from(self.amount_bars) as u32);
    }

    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor) {
        let bar_values = self.bar_processor.process_bars(processor);
        queue.write_buffer(&self.freqs_buffer, 0, bytemuck::cast_slice(bar_values));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[new_time]));
    }

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        queue.write_buffer(
            &self.resolution_buffer,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }
}
