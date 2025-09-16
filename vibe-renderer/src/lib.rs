mod resource_manager;

pub mod components;
pub mod util;

use std::ops::Deref;

use components::{SdfMask, SdfMaskDescriptor, SdfPattern, ValueNoise, ValueNoiseDescriptor};
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

pub use components::Component;
pub use resource_manager::ResourceManager;

/// A trait which marks a struct as something which can be rendered by the [Renderer].
pub trait Renderable {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RendererDescriptor {
    /// Decide which kind of gpu should be used.
    ///
    /// See <https://docs.rs/wgpu/latest/wgpu/enum.PowerPreference.html#variants>
    /// for the available options
    pub power_preference: wgpu::PowerPreference,

    /// Set the backend which should be used.
    pub backend: wgpu::Backends,

    /// Optionally provide the name for the adapter to use.
    pub adapter_name: Option<String>,

    /// Enforce software rendering if wgpu can't find a gpu.
    pub fallback_to_software_rendering: bool,
}

impl Default for RendererDescriptor {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::LowPower,
            backend: wgpu::Backends::VULKAN,
            fallback_to_software_rendering: false,
            adapter_name: None,
        }
    }
}

/// The main renderer which renders the effects.
#[derive(Debug, Clone)]
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    /// Create a new instance of this struct.
    pub fn new(desc: &RendererDescriptor) -> Self {
        let instance = wgpu::Instance::new(
            &wgpu::InstanceDescriptor {
                backends: desc.backend,

                ..Default::default()
            }
            .with_env(),
        );

        let adapter = if let Some(adapter_name) = &desc.adapter_name {
            let adapters = instance.enumerate_adapters(desc.backend);

            let adapter_names: Vec<String> = adapters
                .iter()
                .map(|adapter| adapter.get_info().name)
                .collect();

            adapters
                .into_iter()
                .find(|adapter| &adapter.get_info().name == adapter_name)
                .clone()
                .unwrap_or_else(|| {
                    error!(
                        "Couldn't find the adapter '{}'. Available adapters are: {:?}",
                        adapter_name, adapter_names
                    );

                    panic!("Couldn't find adapter.");
                })
        } else {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: desc.power_preference,
                    force_fallback_adapter: desc.fallback_to_software_rendering,
                    ..Default::default()
                })
                .block_on()
                .expect("Couldn't find GPU device.")
        };

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

    /// Start rendering multiple (or one) [`Renderable`].
    pub fn render<'a, 'r, R: Deref<Target: Renderable> + 'r>(
        &self,
        view: &'a wgpu::TextureView,
        renderables: impl IntoIterator<Item = &'r R>,
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

            for renderable in renderables {
                renderable.render_with_renderpass(&mut render_pass);
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

impl Renderer {
    // `brightness`: should be within the range `0` and `1`
    pub fn create_value_noise_texture(&self, texture_size: u32, octaves: u32) -> wgpu::Texture {
        let device = self.device();
        let texture = self.create_texture("Value noise texture", texture_size);

        let renderable = ValueNoise::new(&ValueNoiseDescriptor {
            device,
            texture_size,
            format: texture.format(),
            octaves,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.render(&view, &[&renderable]);

        texture
    }

    pub fn create_sdf_mask(&self, texture_size: u32, pattern: SdfPattern) -> wgpu::Texture {
        let device = self.device();
        let texture = self.create_texture("Grid texture", texture_size);

        let renderable = SdfMask::new(&SdfMaskDescriptor {
            device,
            format: texture.format(),
            pattern,

            texture_size,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.render(&view, &[&renderable]);

        texture
    }

    fn create_texture(&self, label: &'static str, texture_size: u32) -> wgpu::Texture {
        let device = self.device();

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sdf_mask() {
        let renderer = Renderer::new(&RendererDescriptor::default());

        renderer.create_sdf_mask(10, SdfPattern::Box);
    }
}
