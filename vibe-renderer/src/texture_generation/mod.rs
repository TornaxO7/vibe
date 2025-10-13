mod edge_distance_map;
mod gaussian_blur;
mod sdf_mask;
mod value_noise;

pub use edge_distance_map::EdgeDistanceMap;
pub use gaussian_blur::GaussianBlur;
pub use sdf_mask::{SdfMask, SdfPattern};
pub use value_noise::ValueNoise;

pub trait TextureGenerator {
    fn generate(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture;
}
