mod audio;
mod resolution;
mod time;

pub use audio::{Audio, AudioDesc};
pub use resolution::Resolution;
pub use time::Time;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BindingValue {
    Audio,
    Resolution,
    Time,
}

/// Represents a single buffer which can be then accessed in the shader.
pub trait Resource {
    fn bind_group_layout_entry(&self) -> wgpu::BindGroupLayoutEntry;

    fn bind_group_entry(&self) -> wgpu::BindGroupEntry;

    fn update_buffer(&self, queue: &wgpu::Queue);
}

/// Structs which contain different [Resource]s should implement this.
pub trait ResourceCollection {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;

    fn bind_group(&self) -> &wgpu::BindGroup;

    fn update_ressource_buffers(&self, queue: &wgpu::Queue);
}
