pub mod components;

use std::ops::Deref;

use components::Component;
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct RendererDescriptor {
    /// Decide which kind of gpu should be used.
    ///
    /// See <https://docs.rs/wgpu/latest/wgpu/enum.PowerPreference.html#variants>
    /// for the available options
    pub power_preference: wgpu::PowerPreference,

    /// Set the backend which should be used.
    pub backend: wgpu::Backends,

    /// Enforce software rendering if wgpu can't find a gpu.
    pub fallback_to_software_rendering: bool,
}

impl Default for RendererDescriptor {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::LowPower,
            backend: wgpu::Backends::VULKAN,
            fallback_to_software_rendering: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    pub fn new(desc: &RendererDescriptor) -> Self {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: desc.backend,

                ..Default::default()
            }
            .with_env(),
        );

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: desc.power_preference,
                force_fallback_adapter: desc.fallback_to_software_rendering,
                ..Default::default()
            })
            .block_on()
            .expect("Couldn't find GPU device.");

        info!("Choosing for rendering: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on()
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn render<'a, C: Deref<Target: Component>>(
        &self,
        view: &'a wgpu::TextureView,
        components: impl IntoIterator<Item = C>,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            for component in components {
                component.render_with_renderpass(&mut render_pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

/// Getter functions
impl Renderer {
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
