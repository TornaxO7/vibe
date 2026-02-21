mod block_manager;
mod bounded_ring_buffer;
mod descriptor;

use crate::{
    components::{utils::wgsl_types::Vec2f, Rgba},
    Component, ComponentAudio, Renderable,
};
use cgmath::Vector2;
use vibe_audio::{fetcher::Fetcher, BarProcessor, CubicSplineInterpolation};
use wgpu::{include_wgsl, util::DeviceExt};

pub use descriptor::*;

// The actual column direction needs to be computed first after we know
// the size of the screen.
const INIT_COLUMN_DIRECTION: Vector2<f32> = Vector2::new(1.0, 0.0);

type ColumnDirection = Vec2f;
type BottomLeftCorner = Vec2f;
type UpDirection = Vec2f;
type Time = f32;
type AmountColumns = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct VertexParams {
    column_direction: ColumnDirection,
    bottom_left_corner: BottomLeftCorner,
    up_direction: UpDirection,
    time: Time,
    amount_columns: AmountColumns,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct FragmentParams {
    color1: Rgba,
}

pub struct RisingBlocks {
    bar_processor: BarProcessor<CubicSplineInterpolation>,

    // vp = "vertex params"
    vp_buffer: wgpu::Buffer,
    // fp = "fragment params"
    // fp_buffer: wgpu::Buffer,
    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    // block_datas_buffer: wgpu::Buffer,
}

impl RisingBlocks {
    pub fn new<F: Fetcher>(desc: &RisingBlocksDescriptor<F>) -> Self {
        let device = desc.renderer.device();
        let bar_processor = BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());

        // let block_datas = BlockDatas::new(BlockDatasDescriptor {
        //     threshold: desc.threshold,
        //     ..Default::default()
        // });

        // let block_datas_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Rising blocks: Block data buffer"),
        //     size: (std::mem::size_of::<BlockData>() * block_datas.max_amount_blocks())
        //         as wgpu::BufferAddress,
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        let vp_buffer = {
            // let rotation = Matrix2::from_angle(Deg(0.));
            // let up_direction = rotation * Vector2::unit_y();
            let up_direction = Vector2::new(0., 2.);
            let column_direction = INIT_COLUMN_DIRECTION;

            let params = VertexParams {
                bottom_left_corner: Vec2f::from([-1., -1.]),
                up_direction: up_direction.into(),
                column_direction: column_direction.into(),
                time: 0.,
                amount_columns: bar_processor.total_amount_bars() as AmountColumns,
            };

            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rising blocks: Vertex params buffer"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };

        // let fp_buffer = {
        //     let params = FragmentParams {
        //         color1: Vec4f::from([0., 0., 1., 1.]),
        //     };

        //     device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Rising blocks: Fragment params buffer"),
        //         contents: bytemuck::bytes_of(&params),
        //         usage: wgpu::BufferUsages::UNIFORM,
        //     })
        // };

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
                        // buffers: &[wgpu::VertexBufferLayout {
                        //     array_stride: std::mem::size_of::<BlockData>() as wgpu::BufferAddress,
                        //     step_mode: wgpu::VertexStepMode::Instance,
                        //     attributes: &[
                        //         wgpu::VertexAttribute {
                        //             format: wgpu::VertexFormat::Float32,
                        //             offset: 0,
                        //             shader_location: 0,
                        //         },
                        //         wgpu::VertexAttribute {
                        //             format: wgpu::VertexFormat::Float32,
                        //             offset: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                        //             shader_location: 1,
                        //         },
                        //     ],
                        // }],
                        buffers: &[],
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
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vp_buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 1,
                //     resource: fp_buffer.as_entire_binding(),
                // },
            ],
        });

        Self {
            bar_processor,

            vp_buffer,
            // fp_buffer,
            bind_group0,
            pipeline,
            // block_datas,
            // block_datas_buffer,
        }
    }
}

impl Renderable for RisingBlocks {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);
        // pass.set_vertex_buffer(
        //     0,
        //     self.block_datas_buffer
        //         .slice(0..(std::mem::size_of::<BlockData>() * self.block_datas.len()) as u64),
        // );
        // pass.set_pipeline(&self.pipeline);
        // pass.draw(0..4, 0..self.block_datas.len() as u32);
    }
}

impl Component for RisingBlocks {
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        // self.block_datas.discard_expired_blocks(new_time);

        let offset = std::mem::size_of::<ColumnDirection>()
            + std::mem::size_of::<BottomLeftCorner>()
            + std::mem::size_of::<UpDirection>();

        queue.write_buffer(
            &self.vp_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&new_time),
        );
    }

    fn update_resolution(&mut self, _renderer: &crate::Renderer, _new_resolution: [u32; 2]) {
        todo!()
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}

impl<F: Fetcher> ComponentAudio<F> for RisingBlocks {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &vibe_audio::SampleProcessor<F>) {
        self.bar_processor.process_bars(&processor);

        for channel in self.bar_processor.bars() {
            for bar in channel.iter().cloned() {}
        }

        // self.block_datas
        //     .update_blocks(self.bar_processor.bars(), queue);
    }
}
