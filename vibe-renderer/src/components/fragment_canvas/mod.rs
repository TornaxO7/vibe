use shady_audio::{BarProcessor, SampleProcessor};
use wgpu::util::DeviceExt;

use crate::components::bind_group_entry;

use super::{bind_group_layout_entry, Component};

const ENTRYPOINT: &str = "main";

type VertexPosition = [f32; 2];

#[rustfmt::skip]
const VERTICES: [VertexPosition; 4] = [
    [1.0, 1.0],   // top right
    [-1.0, 1.0],  // top left
    [1.0, -1.0],  // bottom right
    [-1.0, -1.0]  // bottom left
];

pub struct FragmentCanvasDescriptor<'a> {
    pub sample_processor: &'a SampleProcessor,
    pub audio_config: shady_audio::Config,

    pub device: &'a wgpu::Device,

    pub format: wgpu::TextureFormat,

    /// Canvas/Resolution size: (width, height).
    pub resolution: [u32; 2],

    pub fragment_source: wgpu::ShaderSource<'a>,
}

pub struct FragmentCanvas {
    bar_processor: BarProcessor,

    time_buffer: wgpu::Buffer,
    _resolution_buffer: wgpu::Buffer,
    freqs_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    fragment_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new(desc: &FragmentCanvasDescriptor) -> Self {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_config.clone());

        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas time buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment canvas resolution buffer"),
            contents: bytemuck::cast_slice(&[desc.resolution[0] as f32, desc.resolution[1] as f32]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let freqs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas frequency buffer"),
            size: (std::mem::size_of::<f32>()
                * usize::from(u16::from(desc.audio_config.amount_bars))) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fragment canvas vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (pipeline, fragment_bind_group) = {
            let fragment_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Fragment canvas bind group layout"),
                    entries: &[
                        bind_group_layout_entry(
                            0,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                        bind_group_layout_entry(
                            1,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                        bind_group_layout_entry(
                            2,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Storage { read_only: true },
                        ),
                    ],
                });

            let fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fragment canvas bind group"),
                layout: &fragment_bind_group_layout,
                entries: &[
                    bind_group_entry(0, &time_buffer),
                    bind_group_entry(1, &resolution_buffer),
                    bind_group_entry(2, &freqs_buffer),
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fragment canvas pipeline layout"),
                bind_group_layouts: &[&fragment_bind_group_layout],
                push_constant_ranges: &[],
            });

            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment canvas vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./vertex_shader.wgsl").into()),
            });

            let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment canvas fragment module"),
                source: desc.fragment_source.clone(),
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            });

            (pipeline, fragment_bind_group)
        };

        Self {
            bar_processor,

            freqs_buffer,
            time_buffer,
            _resolution_buffer: resolution_buffer,
            vertex_buffer,

            fragment_bind_group,
            pipeline,
        }
    }

    pub fn update_audio(&mut self, processor: &SampleProcessor, queue: &wgpu::Queue) {
        let bar_values = self.bar_processor.process_bars(processor);
        queue.write_buffer(&self.freqs_buffer, 0, bytemuck::cast_slice(bar_values));
    }

    pub fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[new_time]));
    }
}

impl Component for FragmentCanvas {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.fragment_bind_group, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..1);
    }
}

impl Component for &FragmentCanvas {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        (*self).render_with_renderpass(pass);
    }
}
