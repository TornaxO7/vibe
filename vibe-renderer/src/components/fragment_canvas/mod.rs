use std::borrow::Cow;

use shady_audio::{BarProcessor, BarProcessorConfig, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, Renderable};

use super::{Component, ShaderCode, ShaderCodeError};

const ENTRYPOINT: &str = "main";

type VertexPosition = [f32; 2];

#[repr(u32)]
enum Bindings0 {
    IResolution = 0,
}

#[repr(u32)]
enum Bindings1 {
    Freqs = 0,
    ITime = 1,
}

#[rustfmt::skip]
const VERTICES: [VertexPosition; 4] = [
    [1.0, 1.0],   // top right
    [-1.0, 1.0],  // top left
    [1.0, -1.0],  // bottom right
    [-1.0, -1.0]  // bottom left
];

pub struct FragmentCanvasDescriptor<'a> {
    pub sample_processor: &'a SampleProcessor,
    pub audio_conf: BarProcessorConfig,
    pub device: &'a wgpu::Device,
    pub format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    /// Canvas/Resolution size: (width, height).
    pub resolution: [u32; 2],
    pub fragment_code: ShaderCode,
}

pub struct FragmentCanvas {
    bar_processor: BarProcessor,

    vbuffer: wgpu::Buffer,

    bind_group0: BindGroupManager,
    bind_group1: BindGroupManager,

    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new(desc: &FragmentCanvasDescriptor) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut bind_group0_builder =
            BindGroupManager::builder(Some("Fragment canvas: Bind group 0"));
        let mut bind_group1_builder =
            BindGroupManager::builder(Some("Fragment canvas: Bind group 1"));

        bind_group0_builder.insert_buffer(
            Bindings0::IResolution as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fragment canvas: `iResolution` buffer"),
                contents: bytemuck::bytes_of(&desc.resolution),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        );

        bind_group1_builder.insert_buffer(
            Bindings1::Freqs as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Fragment canvas: `freqs` buffer"),
                size: (std::mem::size_of::<f32>()
                    * usize::from(u16::from(desc.audio_conf.amount_bars)))
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        bind_group1_builder.insert_buffer(
            Bindings1::ITime as u32,
            wgpu::ShaderStages::FRAGMENT,
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Fragment canvas: `iTime` buffer"),
                size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        );

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment canvas vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fragment canvas pipeline layout"),
                bind_group_layouts: &[
                    &bind_group0_builder.get_bind_group_layout(device),
                    &bind_group1_builder.get_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment canvas vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
            });

            let fragment_module = {
                let source = desc.fragment_code.source().map_err(ShaderCodeError::from)?;

                let module = match desc.fragment_code.language {
                    super::ShaderLanguage::Wgsl => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.wgsl");
                        super::parse_wgsl_fragment_code(PREAMBLE, &source)
                            .map_err(ShaderCodeError::ParseError)?
                    }
                    super::ShaderLanguage::Glsl => {
                        const PREAMBLE: &str = include_str!("./fragment_preamble.glsl");
                        super::parse_glsl_fragment_code(PREAMBLE, &source)
                            .map_err(ShaderCodeError::ParseError)?
                    }
                };

                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment canvas fragment module"),
                    source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
                })
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Fragment canvas render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vertex_module,
                    entry_point: Some(ENTRYPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<VertexPosition>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
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
                    entry_point: Some(ENTRYPOINT),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: desc.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };

        Ok(Self {
            bar_processor,

            vbuffer,

            bind_group0: bind_group0_builder.build(device),
            bind_group1: bind_group1_builder.build(device),

            pipeline,
        })
    }
}

impl Renderable for FragmentCanvas {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        if !self.bind_group0.is_empty() {
            pass.set_bind_group(0, self.bind_group0.get_bind_group(), &[]);
        }

        if !self.bind_group1.is_empty() {
            pass.set_bind_group(1, self.bind_group1.get_bind_group(), &[]);
        }

        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for FragmentCanvas {
    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        if let Some(buffer) = self.bind_group0.get_buffer(Bindings0::IResolution as u32) {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
            );
        }
    }

    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::Freqs as u32) {
            let bar_values = self.bar_processor.process_bars(processor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(bar_values));
        }
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        if let Some(buffer) = self.bind_group1.get_buffer(Bindings1::ITime as u32) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
        }
    }
}
