use std::num::NonZero;

use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use super::Component;

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

const SHADER_ENTRYPOINT: &str = "main";

pub struct BarsDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub sample_processor: &'a SampleProcessor,
    pub audio_conf: shady_audio::Config,
    pub texture_format: wgpu::TextureFormat,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    freqs_buffer: wgpu::Buffer,
    _column_width_buffer: wgpu::Buffer,
    _padding_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Bars {
    pub fn new(desc: &BarsDescriptor) -> Self {
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

        let (pipeline, bind_group) = {
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bar bind group layout"),
                    entries: &[
                        bind_group_layout_entry(
                            0,
                            wgpu::BufferBindingType::Storage { read_only: true },
                        ),
                        bind_group_layout_entry(1, wgpu::BufferBindingType::Uniform),
                        bind_group_layout_entry(2, wgpu::BufferBindingType::Uniform),
                    ],
                });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bar bind group"),
                layout: &bind_group_layout,
                entries: &[
                    bind_group_entry(0, &freqs_buffer),
                    bind_group_entry(1, &column_width_buffer),
                    bind_group_entry(2, &padding_buffer),
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bar pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bar vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
            });

            let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Bar fragment module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./fragment_shader.wgsl").into()),
            });

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

            (pipeline, bind_group)
        };

        Self {
            amount_bars,
            bar_processor,

            freqs_buffer,
            _column_width_buffer: column_width_buffer,
            _padding_buffer: padding_buffer,

            pipeline,
            bind_group,
        }
    }

    pub fn update_audio(&mut self, processor: &SampleProcessor, queue: &wgpu::Queue) {
        let bar_values = self.bar_processor.process_bars(processor);
        queue.write_buffer(&self.freqs_buffer, 0, bytemuck::cast_slice(bar_values));
    }
}

impl Component for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.pipeline);

        pass.draw(0..4, 0..u16::from(self.amount_bars) as u32);
    }
}

impl Component for &Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        (*self).render_with_renderpass(pass);
    }
}

fn bind_group_layout_entry(
    binding: u32,
    ty: wgpu::BufferBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bind_group_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
