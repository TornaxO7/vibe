mod descriptor;

pub use descriptor::*;
use vibe_audio::{fetcher::Fetcher, CubicSplineInterpolation, SampleProcessor};

use super::{Component, Mat2x2, Rgba, Vec2f};
use crate::{components::ComponentAudio, util::SimpleRenderPipelineDescriptor, Renderable};
use cgmath::Matrix2;
use wgpu::{include_wgsl, util::DeviceExt};

type Resolution = Vec2f;
type PositionOffset = Vec2f;
type Color = Rgba;
type Rotation = Mat2x2;
type Radius = f32;
type SpikeSensitivity = f32;
type FreqRadiantStep = f32;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
struct Data {
    resolution: Resolution,
    position_offset: PositionOffset,
    color: Color,
    rotation: Rotation,
    radius: Radius,
    spike_sensitivity: SpikeSensitivity,
    freq_radiant_step: FreqRadiantStep,
    _padding: f32,
}

pub struct Circle {
    bar_processor: vibe_audio::BarProcessor<CubicSplineInterpolation>,

    data_buffer: wgpu::Buffer,
    freq_buffer: wgpu::Buffer,

    bind_group0: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
}

impl Circle {
    pub fn new<F: Fetcher>(desc: &CircleDescriptor<F>) -> Self {
        let device = desc.renderer.device();
        let bar_processor =
            vibe_audio::BarProcessor::new(desc.sample_processor, desc.audio_conf.clone());
        let total_amount_bars = bar_processor.total_amount_bars_per_channel();

        let data = {
            let (spike_sensitivity, color) = match &desc.variant {
                CircleVariant::Graph {
                    spike_sensitivity,
                    color,
                } => (*spike_sensitivity, *color),
            };

            let position_offset = {
                let rel_x_offset = desc.position.0.clamp(0., 1.);
                let rel_y_offset = desc.position.1.clamp(0., 1.);

                PositionOffset::from([rel_x_offset, rel_y_offset])
            };

            Data {
                radius: desc.radius,
                spike_sensitivity,
                freq_radiant_step: std::f32::consts::PI / (total_amount_bars as f32 + 0.99),
                resolution: Resolution::default(),
                position_offset,
                color,
                rotation: Matrix2::from_angle(desc.rotation).into(),
                ..Default::default()
            }
        };

        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle: `data` buffer"),
            contents: bytemuck::bytes_of(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let freq_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle: `freqs` buffer"),
            size: (std::mem::size_of::<f32>() * total_amount_bars) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline = {
            let fragment_module =
                device.create_shader_module(include_wgsl!("./fragment_graph.wgsl"));

            let vertex_module =
                device.create_shader_module(include_wgsl!("../utils/full_screen_vertex.wgsl"));

            device.create_render_pipeline(&crate::util::simple_pipeline_descriptor(
                SimpleRenderPipelineDescriptor {
                    label: "Circle: Render pipeline",
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &vertex_module,
                        entry_point: None,
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[],
                    },
                    fragment: wgpu::FragmentState {
                        module: &fragment_module,
                        entry_point: None,
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

        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Circle: Bind group 0"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: freq_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bar_processor,

            freq_buffer,
            data_buffer,

            bind_group0,
            pipeline,
        }
    }
}

impl Renderable for Circle {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(0, &self.bind_group0, &[]);

        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

impl<F: Fetcher> ComponentAudio<F> for Circle {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>) {
        let bar_values = self.bar_processor.process_bars(processor);

        queue.write_buffer(&self.freq_buffer, 0, bytemuck::cast_slice(&bar_values[0]));
    }
}

impl Component for Circle {
    fn update_time(&mut self, _queue: &wgpu::Queue, _new_time: f32) {}

    fn update_resolution(&mut self, renderer: &crate::Renderer, new_resolution: [u32; 2]) {
        let queue = renderer.queue();

        queue.write_buffer(
            &self.data_buffer,
            0,
            bytemuck::cast_slice(&[new_resolution[0] as f32, new_resolution[1] as f32]),
        );
    }

    fn update_mouse_position(&mut self, _queue: &wgpu::Queue, _new_pos: (f32, f32)) {}
}
