use shady_audio::SampleProcessor;
use vibe_daemon::resources::{Audio, AudioDesc, Resource, ResourceCollection};

use super::config::ShaderConf;

const LABEL: &str = "Shader resources";

#[repr(u32)]
enum Bindings {
    Audio,
}

pub struct ShaderResources {
    pub audio: Audio,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl ShaderResources {
    pub fn new(device: &wgpu::Device, processor: &SampleProcessor, conf: &ShaderConf) -> Self {
        let audio = Audio::new(AudioDesc {
            device,
            processor,
            amount_bars: conf.audio.amount_bars,
            binding: Bindings::Audio as u32,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(LABEL),
            entries: &[audio.bind_group_layout_entry()],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(LABEL),
            layout: &bind_group_layout,
            entries: &[audio.bind_group_entry()],
        });

        Self {
            audio,

            bind_group_layout,
            bind_group,
        }
    }
}

impl ResourceCollection for ShaderResources {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn update_ressource_buffers(&self, queue: &wgpu::Queue) {
        self.audio.update_buffer(queue);
    }
}
