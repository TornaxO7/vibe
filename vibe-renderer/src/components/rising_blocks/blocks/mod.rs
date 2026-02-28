mod descriptor;
mod manager;

use crate::{
    components::{utils::wgsl_types::Vec2f, Rgba},
    Component, ComponentAudio, Renderable,
};
use cgmath::{InnerSpace, Vector2};
use manager::{BlockData, BlockManager};
use vibe_audio::{fetcher::Fetcher, BarProcessor, BarProcessorConfig, LinearInterpolation};
use wgpu::{include_wgsl, util::DeviceExt};

pub use descriptor::*;

/// The `x` and `y` coords goes from -1 to 1.
const VERTEX_SPACE_SIZE: f32 = 2.;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct VertexParams {
    column_direction: Vec2f,
    bottom_left_corner: Vec2f,
    up_direction: Vec2f,
    block_height: Vec2f,
    time: f32,
    amount_columns: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    color1: Rgba,
}

/// Struct which renders each flying block.
pub struct BlocksRenderer {
    bar_processor: BarProcessor<LinearInterpolation>,

    // vp = "vertex params"
    vp_buffer: wgpu::Buffer,
    // fp = "fragment params"
    // fp_buffer: wgpu::Buffer,
    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    block_manager: BlockManager,
    blocks_buffer: wgpu::Buffer,

    block_height: Vector2<f32>,
}

impl BlocksRenderer {
    pub fn new<F: Fetcher>(desc: &BlocksDescriptor<F>) -> Self {
        let device = desc.renderer.device();
        let bar_processor = BarProcessor::new(
            desc.sample_processor,
            BarProcessorConfig {
                up: 0.,
                down: 100.,
                ..desc.audio_conf.clone()
            },
        );
        let total_amount_bars = bar_processor.amount_channels().get() as usize
            * bar_processor.total_amount_bars_per_channel();

        let block_manager = BlockManager::new(total_amount_bars);

        let blocks_buffer = block_manager.create_block_buffer(device);

        let (vp_buffer, block_height) = {
            // let up_direction = rotation * Vector2::unit_y();
            let up_direction =
                Vector2::new(0., desc.canvas_height.clamp(0., 1.) * VERTEX_SPACE_SIZE);
            let column_direction = Vector2::new(2. / total_amount_bars as f32, 0.);
            let block_height = up_direction.normalize() * column_direction.magnitude();

            let params = VertexParams {
                bottom_left_corner: Vec2f::from([-1., -1.]),
                up_direction: up_direction.into(),
                block_height: block_height.into(),
                column_direction: column_direction.into(),
                time: 0.,
                amount_columns: bar_processor.total_amount_bars_per_channel() as f32,
            };

            (
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Rising blocks: Vertex params buffer"),
                    contents: bytemuck::bytes_of(&params),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
                block_height,
            )
        };

        let pipeline = {
            let module = device.create_shader_module(include_wgsl!("./shader.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                crate::util::SimpleRenderPipelineDescriptor {
                    label: "Rising blocks: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<BlockData>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Uint32,
                                    offset: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        }],
                    },
                    fragment: wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("fs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: desc.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    },
                },
            ))
        };

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rising blocks: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vp_buffer.as_entire_binding(),
            }],
        });

        Self {
            bar_processor,

            vp_buffer,
            bind_group0,
            pipeline,

            block_manager,
            blocks_buffer,

            block_height,
        }
    }
}

impl Renderable for BlocksRenderer {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        let amount_blocks = self.block_manager.amount_active_blocks();

        pass.set_vertex_buffer(0, self.blocks_buffer.slice(..));
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..4, 0..amount_blocks as u32);
    }
}

impl Component for BlocksRenderer {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        self.block_manager.discard_expired_blocks(new_time);
        let offset = std::mem::offset_of!(VertexParams, time);

        queue.write_buffer(
            &self.vp_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();
        let aspect_ratio = new_resolution[0] as f32 / new_resolution[1] as f32;
        let block_height = self.block_height * aspect_ratio;
        let offset = std::mem::offset_of!(VertexParams, block_height);

        let block_height_bytes: [f32; 2] = block_height.into();

        queue.write_buffer(
            &self.vp_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&block_height_bytes),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

impl<F: Fetcher> ComponentAudio<F> for BlocksRenderer {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        self.bar_processor.process_bars(&processor);
        self.block_manager.process_bars(self.bar_processor.bars());
        self.block_manager
            .update_wgpu_buffer(queue, &self.blocks_buffer);
    }
}
