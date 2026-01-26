use std::borrow::Cow;
use std::io::Write;

use pollster::FutureExt;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, BarProcessorConfig, BpmDetector, BpmDetectorConfig, SampleProcessor,
};
use wgpu::include_wgsl;

use crate::{resource_manager::ResourceManager, Renderable};

use super::{Component, ShaderCode, ShaderCodeError};

const ENTRYPOINT: &str = "main";

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const FREQS: u32 = 1;
    pub const TIME: u32 = 2;
    pub const MOUSE: u32 = 3;
    pub const BPM: u32 = 4;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution, crate::util::buffer(RESOLUTION, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
            (ResourceID::Time, crate::util::buffer(TIME, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Mouse, crate::util::buffer(MOUSE, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
            (ResourceID::Bpm, crate::util::buffer(BPM, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform)),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    Freqs,
    Time,
    Mouse,
    Bpm,
}

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
    bpm_detector: BpmDetector,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new<F: Fetcher>(desc: &FragmentCanvasDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());
        let bpm_detector = BpmDetector::new(desc.sample_processor, BpmDetectorConfig::default());

        let mut resource_manager = ResourceManager::new();

        let bind_group0_mapping = bindings0::init_mapping();

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
            (
                ResourceID::Mouse,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fragment canvas: `iMouse` buffer"),
                    size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Bpm,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fragment canvas: `iBPM` buffer"),
                    size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let (bind_group0, bind_group0_layout) = resource_manager.build_bind_group(
            "Fragment canvas: Bind group 0",
            device,
            &bind_group0_mapping,
        );

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Fragment canvas pipeline layout"),
                bind_group_layouts: &[&bind_group0_layout],
                ..Default::default()
            });

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

        Ok(Self {
            bar_processor,
            bpm_detector,

            resource_manager,

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

    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) {
        // Update frequency bars
        let bar_values = self.bar_processor.process_bars(processor);
        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));

        // Update BPM
        let bpm = self.bpm_detector.process(processor);
        let buffer = self.resource_manager.get_buffer(ResourceID::Bpm).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&bpm));

        // Write BPM to file for external tools (waybar, etc.)
        if let Ok(mut file) = std::fs::File::create("/tmp/vibe-bpm") {
            let _ = writeln!(file, "{:.0}", bpm);
        }
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Time).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_mouse_position(&mut self, queue: &wgpu::Queue, new_pos: (f32, f32)) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Mouse).unwrap();

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[new_pos.0, new_pos.1]));
    }
}
