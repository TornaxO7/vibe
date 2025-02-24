use std::ops::Range;

use tracing::instrument;
use wgpu::util::DeviceExt;

type VertexCoord = [f32; 2];

const TOP_LEFT_CORNER: VertexCoord = [-1.0, 1.0];
const BOTTOM_LEFT_CORNER: VertexCoord = [-1.0, -1.0];
const BOTTOM_RIGHT_CORNER: VertexCoord = [1.0, -1.0];
const TOP_RIGHT_CORNER: VertexCoord = [1.0, 1.0];

const VERTICES: &[VertexCoord] = &[
    TOP_LEFT_CORNER,
    BOTTOM_LEFT_CORNER,
    BOTTOM_RIGHT_CORNER,
    TOP_RIGHT_CORNER,
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    // left
    0, 1, 2,
    // right
    0, 2, 3,
];

/// Constains the vertex buffer layout for the vertex buffer for the shaders of `Shady`.
pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<VertexCoord>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0 as wgpu::BufferAddress,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    }],
};

#[instrument(level = "trace")]
pub fn vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shady Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

#[instrument(level = "trace")]
pub fn index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shady Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub const fn index_buffer_range() -> Range<u32> {
    0..INDICES.len() as u32
}
