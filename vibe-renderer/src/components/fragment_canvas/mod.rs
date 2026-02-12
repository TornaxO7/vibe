use super::{Component, ShaderCode, ShaderCodeError};
use crate::{components::ComponentAudio, Renderable, Renderer};
#[cfg(not(target_arch = "wasm32"))]
use pollster::FutureExt;
use std::borrow::Cow;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
use vibe_audio::{
    fetcher::Fetcher, BarProcessor, BarProcessorConfig, BpmDetector, BpmDetectorConfig,
    SampleProcessor,
};
use wgpu::include_wgsl;

const ENTRYPOINT: &str = "main";

pub struct FragmentCanvasDescriptor<'a, F: Fetcher> {
    pub sample_processor: &'a SampleProcessor<F>,
    pub audio_conf: BarProcessorConfig,
    pub renderer: &'a Renderer,
    pub format: wgpu::TextureFormat,

    // fragment shader relevant stuff
    pub fragment_code: ShaderCode,
    pub img: Option<image::DynamicImage>,
}

struct TextureCtx {
    sampler: wgpu::Sampler,
    _texture: wgpu::Texture,
    tv: wgpu::TextureView,
}

pub struct FragmentCanvas {
    bar_processor: BarProcessor,
    bpm_detector: BpmDetector,

    iresolution: wgpu::Buffer,
    freqs: wgpu::Buffer,
    itime: wgpu::Buffer,
    imouse: wgpu::Buffer,
    ibpm: wgpu::Buffer,
    icolors: wgpu::Buffer,
    imouseclick: wgpu::Buffer,
    _itexture: Option<TextureCtx>,

    last_click_pos: (f32, f32),
    last_click_time: f32,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl FragmentCanvas {
    pub fn new<F: Fetcher>(desc: &FragmentCanvasDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.renderer.device();
        let queue = desc.renderer.queue();
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());
        let bpm_detector = BpmDetector::new(desc.sample_processor, BpmDetectorConfig::default());
        let total_amount_bars = bar_processor.total_amount_bars();

        let iresolution = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iResolution` buffer"),
            size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let freqs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `freqs` buffer"),
            size: (std::mem::size_of::<f32>() * total_amount_bars) as wgpu::BufferAddress,
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

        let ibpm = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iBPM` buffer"),
            size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let icolors = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iColors` buffer"),
            // 4 colors as vec4f (vec4 for alignment, xyz = rgb, w = unused)
            size: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let imouseclick = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fragment canvas: `iMouseClick` buffer"),
            size: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let itexture = desc.img.as_ref().map(|img| {
            let sampler = device.create_sampler(&crate::util::DEFAULT_SAMPLER_DESCRIPTOR);
            let texture = crate::util::load_img_to_texture(device, queue, img);
            let tv = texture.create_view(&wgpu::TextureViewDescriptor::default());

            TextureCtx {
                sampler,
                _texture: texture,
                tv,
            }
        });

        let bind_group0_layout = {
            let mut entries = vec![
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
                // iBPM
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // iColors
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // iMouseClick
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ];

            if let Some(_texture) = &itexture {
                entries.extend_from_slice(&[
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ]);
            }

            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Fragment canvas: Bind group 0 layout"),
                entries: &entries,
            })
        };

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

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let err_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
                    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Fragment canvas fragment module"),
                        source: shader_source,
                    });

                    if let Some(err) = err_scope.pop().block_on() {
                        return Err(ShaderCodeError::ParseError(err));
                    }

                    module
                }

                #[cfg(target_arch = "wasm32")]
                {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Fragment canvas fragment module"),
                        source: shader_source,
                    })
                }
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

        let bind_group0 = {
            let mut entries = vec![
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
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: ibpm.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: icolors.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: imouseclick.as_entire_binding(),
                },
            ];

            if let Some(texture) = &itexture {
                entries.extend_from_slice(&[
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(&texture.tv),
                    },
                ]);
            }

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Fragment canvas: Bind group 0"),
                layout: &bind_group0_layout,
                entries: &entries,
            })
        };

        Ok(Self {
            bar_processor,
            bpm_detector,

            iresolution,
            freqs,
            itime,
            imouse,
            ibpm,
            icolors,
            imouseclick,
            _itexture: itexture,

            last_click_pos: (-1.0, -1.0),
            last_click_time: 0.0,

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

impl<F: Fetcher> ComponentAudio<F> for FragmentCanvas {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>) {
        let bar_values = self.bar_processor.process_bars(processor);
        queue.write_buffer(&self.freqs, 0, bytemuck::cast_slice(&bar_values[0]));

        // Update BPM
        let bpm = self.bpm_detector.process(processor);
        queue.write_buffer(&self.ibpm, 0, bytemuck::bytes_of(&bpm));

        // Write BPM to file for external tools (waybar, etc.)
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut file) = std::fs::File::create("/tmp/vibe-bpm") {
                let _ = writeln!(file, "{:.0}", bpm);
            }
        }
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

    fn update_mouse_click(&mut self, queue: &wgpu::Queue, pos: (f32, f32), time: f32) {
        self.last_click_pos = pos;
        self.last_click_time = time;
        queue.write_buffer(
            &self.imouseclick,
            0,
            bytemuck::cast_slice(&[pos.0, pos.1, time, 0.0]),
        );
    }

    fn update_colors(&mut self, queue: &wgpu::Queue, colors: &[[f32; 3]; 4]) {
        // Convert to vec4 format for GPU alignment (xyz = rgb, w = 1.0)
        let colors_vec4: [[f32; 4]; 4] = [
            [colors[0][0], colors[0][1], colors[0][2], 1.0],
            [colors[1][0], colors[1][1], colors[1][2], 1.0],
            [colors[2][0], colors[2][1], colors[2][2], 1.0],
            [colors[3][0], colors[3][1], colors[3][2], 1.0],
        ];

        queue.write_buffer(&self.icolors, 0, bytemuck::cast_slice(&colors_vec4));
    }
}
