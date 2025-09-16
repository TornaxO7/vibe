use std::borrow::Cow;

use pollster::FutureExt;
use vibe_audio::{fetcher::Fetcher, BarProcessor, BarProcessorConfig, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::{resource_manager::ResourceManager, Renderable};

use super::{Component, ShaderCode, ShaderCodeError};

const ENTRYPOINT: &str = "main";

type VertexPosition = [f32; 2];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

mod bindings1 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const FREQS: u32 = 0;
    pub const TIME: u32 = 1;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::Time, crate::util::buffer(TIME, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    Freqs,
    Time,
}

#[rustfmt::skip]
const VERTICES: [VertexPosition; 3] = [
    [-3., -1.], // bottom left
    [1., -1.], // bottom right
    [1., 3.] // top right
];

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

    vbuffer: wgpu::Buffer,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new<F: Fetcher>(desc: &FragmentCanvasDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let mut resource_manager = ResourceManager::new();

        let bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fragment canvas: `iResolution` buffer"),
                    size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fragment canvas: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>()
                        * usize::from(u16::from(desc.audio_conf.amount_bars)))
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Time,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fragment canvas: `iTime` buffer"),
                    size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment canvas vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "Fragment canvas: Bind group 0",
            device,
            &bind_group0_mapping,
        );

        let (bind_group1, bind_group1_layout) = resource_manager.build_bind_group(
            "Fragment canvas: Bind group 1",
            device,
            &bind_group1_mapping,
        );

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fragment canvas pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment canvas vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
            });

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
                            defines: wgpu::naga::FastHashMap::default(),
                        }
                    }
                };

                device.push_error_scope(wgpu::ErrorFilter::Validation);
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment canvas fragment module"),
                    source: shader_source,
                });

                if let Some(err) = device.pop_error_scope().block_on() {
                    return Err(ShaderCodeError::ParseError(err));
                }

                module
            };

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Fragment canvas render pipeline",
                    layout: &pipeline_layout,
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

        Ok(Self {
            bar_processor,

            vbuffer,

            resource_manager,

            bind_group0,
            bind_group1,

            pipeline,
        })
    }
}

impl Renderable for FragmentCanvas {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);

        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..VERTICES.len() as u32, 0..1);
    }
}

impl<F: Fetcher> Component<F> for FragmentCanvas {
    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

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

    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>) {
        let bar_values = self.bar_processor.process_bars(processor);

        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Time).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_sample_processor(&mut self, processor: &SampleProcessor<F>) {
        self.bar_processor.update_sample_processor(processor);
    }
}
