use serde::{Deserialize, Serialize};

pub struct SdfMaskDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub format: wgpu::TextureFormat,
    pub pattern: SdfPattern,

    pub texture_size: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SdfPattern {
    Box = 0,
    Circle = 1,
    Heart = 2,
}

impl SdfPattern {
    pub fn id(&self) -> u32 {
        *self as u32
    }
}
