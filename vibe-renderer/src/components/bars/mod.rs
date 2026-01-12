mod descriptor;

pub use descriptor::*;

use super::{Component, ShaderCodeError};
use crate::{
    components::{Pixels, Rgba},
    resource_manager::ResourceManager,
    Renderable,
};
use cgmath::{Deg, Matrix2, Vector2};
use pollster::FutureExt;
use std::{borrow::Cow, num::NonZero};
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    BarProcessor, SampleProcessor,
};
use wgpu::util::DeviceExt;

/// The x coords goes from -1 to 1.
const VERTEX_SURFACE_WIDTH: f32 = 2.;

// The actual column direction needs to be computed first after we know
// the size of the screen.
const INIT_COLUMN_DIRECTION: Vector2<f32> = Vector2::new(1.0, 0.0);

const TRUE: u32 = 1;
const FALSE: u32 = 0;

type Vec2f = [f32; 2];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct VertexParams {
    bottom_left_corner: Vec2f,
    up_direction: Vec2f,
    column_direction: Vec2f,
    max_height: f32,
    // should be a boolean, but... you know, it's not possible due to `bytemuck::Pod`.
    // So, it's meaning is: 1 = TRUE, 0 = False
    height_mirrored: u32,
    amount_bars: u32,

    // memory padding
    _padding1: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct FragmentParams {
    color1: Rgba,
    color2: Rgba,
}

impl FragmentParams {
    const LABEL: &str = "Bars: `fragment_params` buffer";
}

#[derive(Debug, Clone, Copy)]
enum FragmentEntrypoint {
    Color,
    Presence,
    CustomFragmentShader,
    HorizontalGradient,
    VerticalGradient,
}

impl FragmentEntrypoint {
    fn as_str(&self) -> &'static str {
        match self {
            FragmentEntrypoint::Color => "fs_color",
            FragmentEntrypoint::Presence => "fs_presence",
            FragmentEntrypoint::CustomFragmentShader => "main",
            FragmentEntrypoint::HorizontalGradient => "fs_horizontal_gradient",
            FragmentEntrypoint::VerticalGradient => "fs_vertical_gradient",
        }
    }
}

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const VERTEX_PARAMS: u32 = 0;
    pub const FRAGMENT_PARAMS: u32 = 1;

    pub const RESOLUTION: u32 = 6;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::VertexParams, crate::util::buffer(VERTEX_PARAMS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform)),
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
            (ResourceID::Freqs1, crate::util::buffer( FREQS, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true }, ))
        ])
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum ResourceID {
    VertexParams,
    FragmentParams,

    Freqs1,
    Freqs2,

    Resolution,
    Time,
}

pub struct Bars {
    amount_bars: NonZero<u16>,
    bar_processor: BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1_left: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    // the bind group and render pipeline for the second audio channel
    right: Option<(wgpu::RenderPipeline, wgpu::BindGroup)>,

    // thing we need to update the column-direction-vector
    vparams: VertexParams,
    angle: Deg<f32>,
    width: BarsWidth,
}

impl Bars {
    pub fn new<F: Fetcher>(desc: &BarsDescriptor<F>) -> Result<Self, ShaderCodeError> {
        let device = desc.device;
        let amount_bars = desc.audio_conf.amount_bars;
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        let (bottom_left_corner, angle, width) = match desc.placement {
            BarsPlacement::Bottom => (Vector2::from([-1., -1.]), Deg(0.), BarsWidth::ScreenWidth),
            BarsPlacement::Right => (Vector2::from([1., -1.]), Deg(90.), BarsWidth::ScreenHeight),
            BarsPlacement::Top => (Vector2::from([1., 1.]), Deg(180.), BarsWidth::ScreenWidth),
            BarsPlacement::Left => (Vector2::from([-1., 1.]), Deg(270.), BarsWidth::ScreenHeight),
            BarsPlacement::Custom {
                bottom_left_corner,
                width,
                rotation,
                ..
            } => {
                let bottom_left_corner = {
                    // remap [0, 1] x [0, 1] to [-1, 1] x [-1, 1]
                    let mut pos = {
                        let bottom_left_corner = Vector2::from(bottom_left_corner);

                        let x = 2. * bottom_left_corner.x - 1.0;
                        let y = -(2. * bottom_left_corner.y - 1.0);

                        Vector2::from((x, y))
                    };
                    pos.x = pos.x.clamp(-1., 1.);
                    pos.y = pos.y.clamp(-1., 1.);
                    pos
                };

                (bottom_left_corner, rotation, BarsWidth::Custom(width))
            }
        };

        let vparams = {
            let rotation = Matrix2::from_angle(angle);
            let up_direction = rotation * Vector2::unit_y();
            let column_direction = INIT_COLUMN_DIRECTION;
            let height_mirrored = match desc.placement {
                BarsPlacement::Custom {
                    height_mirrored, ..
                } => match height_mirrored {
                    true => TRUE,
                    false => FALSE,
                },
                _ => FALSE,
            };

            VertexParams {
                bottom_left_corner: bottom_left_corner.into(),
                up_direction: up_direction.into(),
                column_direction: column_direction.into(),
                max_height: desc.max_height * VERTEX_SURFACE_WIDTH,
                height_mirrored,
                amount_bars: amount_bars.get() as u32,
                _padding1: 0,
            }
        };

        // == create buffers ==
        let mut resource_manager = ResourceManager::new();

        let mut bind_group0_mapping = bindings0::init_mapping();
        let mut bind_group1_mapping = bindings1::init_mapping();

        resource_manager.extend_buffers([
            (
                ResourceID::VertexParams,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(FragmentParams::LABEL),
                    contents: bytemuck::bytes_of(&vparams),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
            ),
            (
                ResourceID::Freqs1,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Bar: main `freqs` buffer"),
                    size: (std::mem::size_of::<f32>() * usize::from(amount_bars.get())) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        let main_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bar: `main` module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into()),
        });

        let (fragment_module, fragment_entrypoint) = match &desc.variant {
            BarVariant::Color(rgba) => {
                resource_manager.insert_buffer(ResourceID::FragmentParams, {
                    let fragment_params = FragmentParams {
                        color1: *rgba,
                        color2: *rgba,
                    };

                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(FragmentParams::LABEL),
                        contents: bytemuck::bytes_of(&fragment_params),
                        usage: wgpu::BufferUsages::UNIFORM,
                    })
                });

                bind_group0_mapping.insert(
                    ResourceID::FragmentParams,
                    crate::util::buffer(
                        bindings0::FRAGMENT_PARAMS,
                        wgpu::ShaderStages::FRAGMENT,
                        wgpu::BufferBindingType::Uniform,
                    ),
                );

                (main_shader_module, FragmentEntrypoint::Color)
            }
            BarVariant::PresenceGradient { high, low } => {
                resource_manager.insert_buffer(ResourceID::FragmentParams, {
                    let fragment_params = FragmentParams {
                        color1: *low,
                        color2: *high,
                    };

                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Bar: `fragment_params` buffer"),
                        contents: bytemuck::bytes_of(&fragment_params),
                        usage: wgpu::BufferUsages::UNIFORM,
                    })
                });

                bind_group0_mapping.insert(
                    ResourceID::FragmentParams,
                    crate::util::buffer(
                        bindings0::FRAGMENT_PARAMS,
                        wgpu::ShaderStages::FRAGMENT,
                        wgpu::BufferBindingType::Uniform,
                    ),
                );

                (main_shader_module, FragmentEntrypoint::Presence)
            }

            BarVariant::HorizontalGradient { left, right } => {
                resource_manager.insert_buffer(ResourceID::FragmentParams, {
                    let fp = FragmentParams {
                        color1: *left,
                        color2: *right,
                    };

                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(FragmentParams::LABEL),
                        contents: bytemuck::bytes_of(&fp),
                        usage: wgpu::BufferUsages::UNIFORM,
                    })
                });

                bind_group0_mapping.insert(
                    ResourceID::FragmentParams,
                    crate::util::buffer(
                        bindings0::FRAGMENT_PARAMS,
                        wgpu::ShaderStages::FRAGMENT,
                        wgpu::BufferBindingType::Uniform,
                    ),
                );

                (main_shader_module, FragmentEntrypoint::HorizontalGradient)
            }
            BarVariant::VerticalGradient { top, bottom } => {
                resource_manager.insert_buffer(ResourceID::FragmentParams, {
                    let fp = FragmentParams {
                        color1: *bottom,
                        color2: *top,
                    };

                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(FragmentParams::LABEL),
                        contents: bytemuck::bytes_of(&fp),
                        usage: wgpu::BufferUsages::UNIFORM,
                    })
                });

                bind_group0_mapping.insert(
                    ResourceID::FragmentParams,
                    crate::util::buffer(
                        bindings0::FRAGMENT_PARAMS,
                        wgpu::ShaderStages::FRAGMENT,
                        wgpu::BufferBindingType::Uniform,
                    ),
                );

                (main_shader_module, FragmentEntrypoint::VerticalGradient)
            }
            BarVariant::FragmentCode(code) => {
                {
                    resource_manager.insert_buffer(
                        ResourceID::Resolution,
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Bar: iResolution buffer"),
                            size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }),
                    );

                    bind_group0_mapping.extend([(
                        ResourceID::Resolution,
                        crate::util::buffer(
                            bindings0::RESOLUTION,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    )]);
                }

                {
                    resource_manager.insert_buffer(
                        ResourceID::Time,
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Bar: iTime buffer"),
                            size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        }),
                    );

                    bind_group1_mapping.extend([(
                        ResourceID::Time,
                        crate::util::buffer(
                            bindings1::TIME,
                            wgpu::ShaderStages::FRAGMENT,
                            wgpu::BufferBindingType::Uniform,
                        ),
                    )]);
                }

                let fragment_module = {
                    let source = code.source().map_err(ShaderCodeError::from)?;

                    let shader_source = match code.language {
                        super::ShaderLanguage::Wgsl => {
                            const PREAMBLE: &str =
                                include_str!("./shaders/fragment_code_preamble.wgsl");
                            let full_code = format!("{}\n{}", PREAMBLE, &source);
                            wgpu::ShaderSource::Wgsl(Cow::Owned(full_code))
                        }
                        super::ShaderLanguage::Glsl => {
                            const PREAMBLE: &str =
                                include_str!("./shaders/fragment_code_preamble.glsl");
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

                (fragment_module, FragmentEntrypoint::CustomFragmentShader)
            }
        };

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Bars: Bind group 0", device, &bind_group0_mapping);

        let (bind_group1_left, bind_group1_left_layout) = resource_manager.build_bind_group(
            "Bars: Bind group 1 - 1",
            device,
            &bind_group1_mapping,
        );

        let pipeline = {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bars: Pipeline layout left"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_left_layout],
                ..Default::default()
            });

            let entry_point = match desc.format {
                BarsFormat::BassTreble | BarsFormat::BassTrebleBass => "bass_treble",
                BarsFormat::TrebleBass | BarsFormat::TrebleBassTreble => "treble_bass",
            };

            create_pipeline(
                device,
                desc.texture_format,
                pipeline_layout,
                entry_point,
                fragment_module.clone(),
                fragment_entrypoint,
            )
        };

        let right = match &desc.format {
            BarsFormat::TrebleBass | BarsFormat::BassTreble => None,
            f @ (BarsFormat::TrebleBassTreble | BarsFormat::BassTrebleBass) => {
                resource_manager.insert_buffer(
                    ResourceID::Freqs2,
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Bar: second `freqs` buffer"),
                        size: (std::mem::size_of::<f32>() * usize::from(amount_bars.get())) as u64,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                );

                let (bind_group1_right, bind_group1_right_layout) = {
                    let mut right_bind_group1_mapping = bind_group1_mapping.clone();
                    let buffer = right_bind_group1_mapping
                        .remove(&ResourceID::Freqs1)
                        .unwrap();
                    right_bind_group1_mapping.insert(ResourceID::Freqs2, buffer);

                    resource_manager.build_bind_group(
                        "Bars: Bind group 1 - 2",
                        device,
                        &right_bind_group1_mapping,
                    )
                };

                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Bars: Pipeline layout right"),
                        bind_group_layouts: &[&bind_group0_layout, &bind_group1_right_layout],
                        ..Default::default()
                    });

                let entry_point = match f {
                    BarsFormat::TrebleBassTreble => "bass_treble",
                    BarsFormat::BassTrebleBass => "treble_bass",
                    _ => unreachable!(),
                };

                let pipeline = create_pipeline(
                    device,
                    desc.texture_format,
                    pipeline_layout,
                    entry_point,
                    fragment_module,
                    fragment_entrypoint,
                );

                Some((pipeline, bind_group1_right))
            }
        };

        Ok(Self {
            amount_bars,
            bar_processor,

            resource_manager,

            bind_group0,
            bind_group1_left,
            pipeline,

            vparams,
            angle,
            width,
            right,
        })
    }
}

impl Renderable for Bars {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        let amount_bars = self.amount_bars.get() as u32;

        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1_left, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..amount_bars);

        if let Some((pipeline, bind_group)) = &self.right {
            pass.set_bind_group(1, bind_group, &[]);
            pass.set_pipeline(pipeline);
            pass.draw(0..4, amount_bars..(2 * amount_bars));
        }
    }
}

impl Component for Bars {
    fn update_audio(
        &mut self,
        queue: &wgpu::Queue,
        processor: &SampleProcessor<SystemAudioFetcher>,
    ) {
        let bar_values = self.bar_processor.process_bars(processor);

        let buffer = self
            .resource_manager
            .get_buffer(ResourceID::Freqs1)
            .unwrap();
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));

        if let Some(buffer) = self.resource_manager.get_buffer(ResourceID::Freqs2) {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[1]));
        }
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        if let Some(buffer) = self.resource_manager.get_buffer(ResourceID::Time) {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
        }
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        let new_width = new_resolution[0] as f32;
        let new_height = new_resolution[1] as f32;

        if let Some(buffer) = self.resource_manager.get_buffer(ResourceID::Resolution) {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[new_width, new_height]));
        }

        // update the column direction vector
        {
            let buffer = self
                .resource_manager
                .get_buffer(ResourceID::VertexParams)
                .unwrap();

            let rotation = Matrix2::from_angle(self.angle);
            let aspect_ratio = new_width / new_height;

            let component_width = match self.width {
                BarsWidth::ScreenWidth => new_width,
                BarsWidth::ScreenHeight => new_height,
                BarsWidth::Custom(custom) => custom.get() as f32,
            };

            let bar_len = component_width / self.amount_bars.get() as f32;
            let mut dir = rotation * Vector2::unit_x();

            dir = bar_len * dir;

            let is_mono_channel = self.right.is_none();
            if is_mono_channel {
                dir *= 2.;
            }

            // apply aspect ratio
            dir.y *= aspect_ratio;
            dir /= new_width;

            self.vparams.column_direction = dir.into();

            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&self.vparams));
        }
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

fn create_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    pipeline_layout: wgpu::PipelineLayout,
    vertex_entrypoint: &'static str,
    fragment_module: wgpu::ShaderModule,
    fragment_entrypoint: FragmentEntrypoint,
) -> wgpu::RenderPipeline {
    let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bar vertex module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into()),
    });

    let fragment_targets = [Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::all(),
    })];

    device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
        crate::util::SimpleRenderPipelineDescriptor {
            label: "Bar: Render pipeline",
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_module,
                entry_point: Some(vertex_entrypoint),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: wgpu::FragmentState {
                module: &fragment_module,
                entry_point: Some(fragment_entrypoint.as_str()),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &fragment_targets,
            },
        },
    ))
}

enum BarsWidth {
    ScreenWidth,
    ScreenHeight,
    Custom(Pixels<u16>),
}
