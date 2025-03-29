use vibe_daemon::resources::{Resource, ResourceCollection, Time};

const LABEL: &str = "Global resources";

#[repr(u32)]
enum Bindings {
    Time,
}

pub struct GlobalResources {
    time: Time,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl GlobalResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let time = Time::new(device, Bindings::Time as u32);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(LABEL),
            entries: &[time.bind_group_layout_entry()],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &bind_group_layout,
            entries: &[time.bind_group_entry()],
        });

        Self {
            time,

            bind_group_layout,
            bind_group,
        }
    }
}

impl ResourceCollection for GlobalResources {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn update_ressource_buffers(&self, queue: &wgpu::Queue) {
        self.time.update_buffer(queue);
    }
}
