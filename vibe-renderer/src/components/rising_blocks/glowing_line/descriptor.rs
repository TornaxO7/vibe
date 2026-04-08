use crate::{components::Rgba, Renderer};

pub struct GlowingLineDescriptor<'a> {
    pub renderer: &'a Renderer,
    pub format: wgpu::TextureFormat,

    /// The canvas height.
    ///
    /// `0`: Well... zero height...
    /// `1`: The full screen height
    pub canvas_height: f32,

    pub color1: Rgba,
}
