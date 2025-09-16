mod descriptor;

use std::collections::HashMap;

pub use descriptor::*;
use vibe_audio::{fetcher::Fetcher, BarProcessor};
use wgpu::util::DeviceExt;

use crate::{
    components::{Component, SdfPattern},
    resource_manager::ResourceManager,
    Renderable,
};

type VertexPosition = [f32; 2];

// this texture size seems good enough for a 1920x1080 screen.
const DEFAULT_SDF_TEXTURE_SIZE: u32 = 512;

#[rustfmt::skip]
const VERTICES: [VertexPosition; 3] = [
    [-3., -1.], // bottom left
    [1., -1.], // bottom right
    [1., 3.] // top right
];

mod bindings0 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const RESOLUTION: u32 = 0;
    pub const MOVEMENT_SPEED: u32 = 1;
    pub const ZOOM_FACTOR: u32 = 2;

    pub const GRID_TEXTURE: u32 = 3;
    pub const GRID_SAMPLER: u32 = 4;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Resolution   , crate::util::buffer(RESOLUTION    , wgpu::ShaderStages::FRAGMENT  , wgpu::BufferBindingType::Uniform)),
            (ResourceID::MovementSpeed, crate::util::buffer(MOVEMENT_SPEED, wgpu::ShaderStages::FRAGMENT  , wgpu::BufferBindingType::Uniform)),
            (ResourceID::ZoomFactor   , crate::util::buffer(ZOOM_FACTOR   , wgpu::ShaderStages::FRAGMENT  , wgpu::BufferBindingType::Uniform)),

            (ResourceID::GridTexture  , crate::util::texture(GRID_TEXTURE , wgpu::ShaderStages::FRAGMENT)),
            (ResourceID::GridSampler  , crate::util::sampler(GRID_SAMPLER , wgpu::ShaderStages::FRAGMENT)),
        ])
    }
}

mod bindings1 {
    use super::ResourceID;
    use std::collections::HashMap;

    pub const TIME: u32 = 0;
    pub const FREQS: u32 = 1;

    #[rustfmt::skip]
    pub fn init_mapping() -> HashMap<ResourceID, wgpu::BindGroupLayoutEntry> {
        HashMap::from([
            (ResourceID::Time , crate::util::buffer(TIME , wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Uniform))                    ,
            (ResourceID::Freqs, crate::util::buffer(FREQS, wgpu::ShaderStages::FRAGMENT, wgpu::BufferBindingType::Storage { read_only: true })),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceID {
    Resolution,
    MovementSpeed,
    ZoomFactor,
    GridTexture,
    GridSampler,

    Time,
    Freqs,
}

pub struct Chessy {
    bar_processor: BarProcessor,

    resource_manager: ResourceManager<ResourceID>,

    bind_group0: wgpu::BindGroup,
    bind_group1: wgpu::BindGroup,

    bind_group0_mapping: HashMap<ResourceID, wgpu::BindGroupLayoutEntry>,

    vbuffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,

    // data to recreate the grid texture
    pattern: SdfPattern,
}

impl Chessy {
    pub fn new<F: Fetcher>(desc: &ChessyDescriptor<F>) -> Self {
        let renderer = desc.renderer;
        let device = renderer.device();
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_config.clone());

        let mut resource_manager = ResourceManager::new();
        let bind_group0_mapping = bindings0::init_mapping();
        let bind_group1_mapping = bindings1::init_mapping();

        resource_manager.extend_buffers([
            (
                ResourceID::Resolution,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Chessy: `iResolution` buffer"),
                    size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::MovementSpeed,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chessy: `movement_speed` buffer"),
                    contents: bytemuck::bytes_of(&desc.movement_speed),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::ZoomFactor,
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chessy: `zoom_factor` buffer"),
                    contents: bytemuck::bytes_of(&desc.zoom_factor),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
            ),
            (
                ResourceID::Time,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Chessy: `iTime` buffer"),
                    size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
            (
                ResourceID::Freqs,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Chessy: `freqs` buffer"),
                    size: (std::mem::size_of::<f32>()
                        * desc.audio_config.amount_bars.get() as usize)
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            ),
        ]);

        {
            // arbitrary size for the beginning
            let grid_texture = desc.renderer.create_sdf_mask(50, desc.pattern);

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Chessy: Grid texture sampler"),

                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mipmap_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mag_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            resource_manager.insert_texture(ResourceID::GridTexture, grid_texture);
            resource_manager.insert_sampler(ResourceID::GridSampler, sampler);
        }

        let vbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chessy: Vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (bind_group0, bind_group0_layout) =
            resource_manager.build_bind_group("Chessy: Bind group 0", device, &bind_group0_mapping);

        let (bind_group1, bind_group1_layout) =
            resource_manager.build_bind_group("Chessy: Bind group 1", device, &bind_group1_mapping);

        let pipeline = {
            let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Chessy: Vertex module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/vertex.wgsl").into()),
            });

            let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Chessy: Fragment shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("./shaders/fragment_shader.wgsl").into(),
                ),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Chessy: Pipeline layout descriptor"),
                bind_group_layouts: &[&bind_group0_layout, &bind_group1_layout],
                push_constant_ranges: &[],
            });

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Chessy: Render pipeline",
                    layout: &pipeline_layout,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<VertexPosition>()
                                as wgpu::BufferAddress,
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
                        entry_point: Some("main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        Self {
            bar_processor,
            resource_manager,
            bind_group0,
            bind_group1,

            bind_group0_mapping,

            vbuffer,
            pipeline,

            pattern: desc.pattern,
        }
    }
}

impl Renderable for Chessy {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        pass.set_bind_group(1, &self.bind_group1, &[]);

        pass.set_vertex_buffer(0, self.vbuffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..VERTICES.len() as u32, 0..1);
    }
}

impl<F: Fetcher> Component<F> for Chessy {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        let bar_values = self.bar_processor.process_bars(processor);

        let buffer = self.resource_manager.get_buffer(ResourceID::Freqs).unwrap();

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        let buffer = self.resource_manager.get_buffer(ResourceID::Time).unwrap();
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&new_time));
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let device = renderer.device();

        {
            // idea: If someone has a 4k screen for example, we'd need to double the sdf texture to
            // avoid blurring of each cell.
            // Why doubling if 4k:
            //
            // The longest side of a typical 4k (3840x2160) screen is 3840 ...
            let max_length = new_resolution.iter().max().unwrap();
            // ... so it's `3840 / 1920 = 2` twice is big as the texture size for a full hd screen...
            let factor = *max_length as f32 / 1920f32;
            // ... so we double it ~~and give it to the next person~~
            let new_size = DEFAULT_SDF_TEXTURE_SIZE as f32 * factor;

            let grid_texture = renderer.create_sdf_mask(new_size.ceil() as u32, self.pattern);

            self.resource_manager
                .replace_texture(ResourceID::GridTexture, grid_texture);

            let (bind_group, _layout) = self.resource_manager.build_bind_group(
                "Chessy: Bind group 0",
                device,
                &self.bind_group0_mapping,
            );

            self.bind_group0 = bind_group;
        }

        {
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
    }

    fn update_sample_processor(&mut self, processor: &vibe_audio::SampleProcessor<F>) {
        self.bar_processor.update_sample_processor(processor);
    }
}
