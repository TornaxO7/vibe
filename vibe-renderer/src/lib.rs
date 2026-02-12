#[cfg(not(target_arch = "wasm32"))]
pub mod cache;
pub mod components;
pub mod texture_generation;
pub mod util;

pub use components::{Component, ComponentAudio};

use crate::texture_generation::TextureGenerator;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tracing::{error, info};
#[cfg(not(target_arch = "wasm32"))]
use xdg::BaseDirectories;

#[cfg(not(target_arch = "wasm32"))]
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

/// Simply contains the name of the application.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// A trait which marks a struct as something which can be rendered by the [Renderer]
/// or to be more specific: By the [Renderer::render] method.
pub trait Renderable {
    /// The renderer will call this function on the renderable object
    /// and it can starts its preparations (for example `pass.set_vertex_buffer` etc.)
    /// and call the draw command (`pass.draw(...)`).
    ///
    /// # Example
    /// ```
    /// use vibe_renderer::Renderable;
    ///
    /// /// Your struct which should create its own pipeline etc.
    /// struct Triangle {
    ///     pipeline: wgpu::RenderPipeline,
    ///     // other things which you need
    /// }
    ///
    /// impl Renderable for Triangle {
    ///     fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
    ///          // // if you have any bind groups for example
    ///          // pass.set_bind_group(0, &self.bind_group, &[]);
    ///          pass.set_pipeline(&self.pipeline);
    ///          pass.draw(0..4, 0..1);
    ///     }
    /// }
    /// ```
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
            #[cfg(not(target_arch = "wasm32"))]
            backend: wgpu::Backends::VULKAN,
            #[cfg(target_arch = "wasm32")]
            backend: wgpu::Backends::BROWSER_WEBGPU,
            fallback_to_software_rendering: false,
            adapter_name: None,
        }
    }
}

/// The main renderer which renders the effects.
///
/// # Example
/// ```
/// use vibe_renderer::Renderer;
///
/// let renderer = Renderer::default();
///
/// // go wild!
/// ```
#[derive(Debug, Clone)]
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Renderer {
    /// Core async initialization logic shared between native and WASM.
    async fn init_renderer(desc: &RendererDescriptor) -> Self {
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
            let adapters = instance.enumerate_adapters(desc.backend).await;

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
                .await
                .expect("Couldn't find GPU device.")
        };

        info!("Choosing for rendering: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features,
                ..Default::default()
            })
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    /// Create a new instance of this struct (native only, uses pollster to block).
    ///
    /// # Example
    /// ```rust
    /// use vibe_renderer::{Renderer, RendererDescriptor};
    ///
    /// let renderer = Renderer::new(&RendererDescriptor::default());
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(desc: &RendererDescriptor) -> Self {
        pollster::block_on(Self::init_renderer(desc))
    }

    /// Create a new instance of this struct (WASM, async).
    #[cfg(target_arch = "wasm32")]
    pub async fn new_async(desc: &RendererDescriptor) -> Self {
        Self::init_renderer(desc).await
    }

    /// Start rendering multiple (or one) [`Renderable`]s onto `output_texture`.
    pub fn render<'a, 'r, R: Deref<Target: Renderable> + 'r>(
        &self,
        output_texture: &'a wgpu::TextureView,
        renderables: impl IntoIterator<Item = &'r R>,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_texture,
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

    /// Renders the given [TextureGenerator] into a new [wgpu::Texture] which gets returned.
    ///
    /// See the list of Implementors of [TextureGenerator] to see which kind of textures
    /// you are able to generate.
    pub fn generate<G: TextureGenerator>(&self, gen: &G) -> wgpu::Texture {
        let device = self.device();
        let queue = self.queue();

        gen.generate(device, queue)
    }
}

/// Getter functions
impl Renderer {
    /// Returns the internal [wgpu::Instance].
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Returns the internal [wgpu::Adapter].
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Returns the internal [wgpu::Device].
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the internal [wgpu::Queue].
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Renderer {
    fn default() -> Self {
        Self::new(&RendererDescriptor::default())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_cache_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    get_xdg().create_cache_directory(path).unwrap()
}
