//! Contains the implementors of the [TextureGenerator] trait which are there to... well, generate/create [wgpu::Texture]s which can then be
//! used by [crate::Component]s.
pub mod edge_distance_map;
mod gaussian_blur;
mod sdf_mask;
mod value_noise;

pub use gaussian_blur::GaussianBlur;
pub use sdf_mask::{SdfMask, SdfPattern};
pub use value_noise::ValueNoise;

/// Provides a method for structs which are there to generate (helper-)textures which can then be used
/// by [crate::Component]s.
pub trait TextureGenerator {
    /// The struct (which implements this trait) should:
    /// 1. Create the texture on its own
    /// 2. Renders on it
    /// 3. Returns it.
    ///
    /// # Example
    /// See the implementation of [value_noise::ValueNoise].
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture;
}

/// A simple type wrapper for size.
#[derive(Debug, Clone)]
pub struct Size {
    /// Width in pixel.
    pub width: u32,

    /// Height in pixel.
    pub height: u32,
}
