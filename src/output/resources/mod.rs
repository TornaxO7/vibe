use vibe_daemon::{
    resources::{Resolution, Resource, ResourceCollection},
    types::size::Size,
};

const LABEL: &str = "Output resources";

#[repr(u32)]
enum BindingIndex {
    Resolution,
}

pub struct OutputResources {
    resolution: Resolution,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl OutputResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let resolution = Resolution::new(device, BindingIndex::Resolution as u32);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(LABEL),
            entries: &[resolution.bind_group_layout_entry()],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &bind_group_layout,
            entries: &[resolution.bind_group_entry()],
        });

        Self {
            resolution,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn set_resolution(&mut self, new_size: Size) {
        self.resolution.set(new_size);
    }
}

impl ResourceCollection for OutputResources {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn update_ressource_buffers(&self, queue: &wgpu::Queue) {
        self.resolution.update_buffer(queue);
    }
}
