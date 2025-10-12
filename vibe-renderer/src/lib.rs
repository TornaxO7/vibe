mod resource_manager;

pub mod cache;
pub mod components;
pub mod texture_generation;
pub mod util;

use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tracing::{error, info};
use xdg::BaseDirectories;

pub use components::Component;
pub use resource_manager::ResourceManager;

use crate::texture_generation::TextureGenerator;

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

const APP_NAME: &str = env!("CARGO_PKG_NAME");
// const DISTANCE_MAP_DIR: &str = "distance-maps";

/// A trait which marks a struct as something which can be rendered by the [Renderer].
pub trait Renderable {
    /// The renderer will call this function on the renderable object
    /// and it can starts its preparations (for example `pass.set_vertex_buffer` etc.)
    /// and call the draw command (`pass.draw(...)`).
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass);
}

/// The descriptor to configure and create a new renderer.
///
/// See [Renderer::new] for more information.
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
    ///
    /// # Example
    /// ```rust
    /// use vibe_renderer::{Renderer, RendererDescriptor};
    ///
    /// let renderer = Renderer::new(&RendererDescriptor::default());
    /// ```
    pub fn new(desc: &RendererDescriptor) -> Self {
        let required_features =
            wgpu::Features::FLOAT32_FILTERABLE | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;

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
                .find(|adapter| {
                    &adapter.get_info().name == adapter_name
                        && adapter.features().contains(required_features)
                })
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
            .request_device(&wgpu::DeviceDescriptor {
                required_features,
                ..Default::default()
            })
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
                    depth_slice: None,
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

    pub fn generate<G: TextureGenerator>(&self, gen: &G) -> wgpu::Texture {
        let device = self.device();
        let queue = self.queue();

        gen.generate(device, queue)
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

impl Default for Renderer {
    fn default() -> Self {
        Self::new(&RendererDescriptor::default())
    }
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn get_cache_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    get_xdg().create_cache_directory(path).unwrap()
}
