mod bars;
mod fragment_canvas;

pub use bars::{Bars, BarsDescriptor};
pub use fragment_canvas::{FragmentCanvas, FragmentCanvasDescriptor};
use serde::{Deserialize, Serialize};
use shady_audio::SampleProcessor;

pub type ParseErrorMsg = String;

pub trait Component {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass);

    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor);

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32);

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderCode {
    Wgsl(String),
    Glsl(String),
}

fn bind_group_layout_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BufferBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bind_group_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn parse_wgsl_fragment_code(
    preamble: &'static str,
    code: &str,
) -> Result<wgpu::naga::Module, ParseErrorMsg> {
    let mut full_code = preamble.to_string();
    full_code.push_str(code);

    wgpu::naga::front::wgsl::parse_str(&full_code).map_err(|err| err.emit_to_string(&full_code))
}

fn parse_glsl_fragment_code(
    preamble: &'static str,
    code: &str,
) -> Result<wgpu::naga::Module, ParseErrorMsg> {
    let mut full_code = preamble.to_string();
    full_code.push_str(code);

    let mut frontend = wgpu::naga::front::glsl::Frontend::default();
    let options = wgpu::naga::front::glsl::Options::from(wgpu::naga::ShaderStage::Fragment);

    frontend
        .parse(&options, &full_code)
        .map_err(|err| err.emit_to_string(&full_code))
}

impl<T: Component + ?Sized> Component for Box<T> {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        self.as_ref().render_with_renderpass(pass);
    }

    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor) {
        self.as_mut().update_audio(queue, processor);
    }

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32) {
        self.as_mut().update_time(queue, new_time);
    }

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]) {
        self.as_mut().update_resolution(queue, new_resolution);
    }
}
