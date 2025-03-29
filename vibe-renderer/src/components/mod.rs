mod bars;

pub use bars::{Bars, BarsDescriptor};

pub trait Component {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass);
}
