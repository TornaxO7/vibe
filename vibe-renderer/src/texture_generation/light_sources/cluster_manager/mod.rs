use std::{num::NonZero, ops::Range};

use wgpu::util::DeviceExt;

type Vector2 = [u32; 2];

pub struct ClusterManagerDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,

    pub src: wgpu::TextureView,

    // TODO: Extend to this one
    // /// Set the amount of centers which should be used.
    // /// `start`: The intial amount of centers
    // /// `end`: The maximal amount of centers to test
    // pub amount_centers_range: Range<NonZero<u32>>,
    pub amount_centers_max: NonZero<u32>,
}

pub struct ClusterManager {
    amount_centers: u32,
    amount_centers_max: u32,
    amount_centers_buffer: wgpu::Buffer,

    cluster_centers_buffer: wgpu::Buffer,

    possible_light_coords: wgpu::Buffer,
}

impl ClusterManager {
    pub fn new(desc: ClusterManagerDescriptor) -> Self {
        let ClusterManagerDescriptor {
            device,
            queue,
            src,
            amount_centers_max,
        } = desc;

        let amount_centers = 1;
        let amount_centers_max = amount_centers_max.get();
        let amount_centers_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cluster manager: Amount centers buffer"),
            contents: bytemuck::bytes_of(&amount_centers),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let cluster_centers_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster manager: Cluster centers"),
            size: (std::mem::size_of::<Vector2>() * amount_centers_max as usize)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        Self {
            amount_centers,
            amount_centers_max,
            amount_centers_buffer,

            cluster_centers_buffer,
        }
    }

    pub fn init(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        todo!()
    }

    pub fn converge(&self, iterations: usize) {
        todo!()
    }

    pub fn compute_err(&self) -> f32 {
        todo!()
    }

    pub fn add_center(&mut self, queue: &wgpu::Queue) -> bool {
        let is_within_valid_range = self.amount_centers + 1 <= self.amount_centers_max;
        if is_within_valid_range {
            self.amount_centers += 1;
            queue.write_buffer(
                &self.amount_centers_buffer,
                0,
                bytemuck::bytes_of(&self.amount_centers),
            );
        }

        is_within_valid_range
    }

    pub fn get_distance_map(&self) -> wgpu::Texture {
        todo!()
    }
}

// extracts all texels in the filtered
// fn filter_points() -> wgpu:Buffer {

// }
