mod aurodio;
mod bars;
mod circle;
mod fragment_canvas;
mod graph;
mod radial;
mod value_noise;

pub use aurodio::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor};
pub use bars::{BarVariant, Bars, BarsDescriptor, BarsPlacement};
pub use circle::{Circle, CircleDescriptor, CircleVariant};
pub use fragment_canvas::{FragmentCanvas, FragmentCanvasDescriptor};
pub use graph::{Graph, GraphDescriptor, GraphPlacement, GraphVariant};
pub use radial::{Radial, RadialDescriptor, RadialVariant};
pub use value_noise::{ValueNoise, ValueNoiseDescriptor};

use crate::{Renderable, Renderer};
use serde::{Deserialize, Serialize};
use shady_audio::SampleProcessor;
use std::path::PathBuf;

// rgba values are each directly set in the fragment shader
pub type Rgba = [f32; 4];
pub type Rgb = [f32; 3];

pub trait Component: Renderable {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor);

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32);

    fn update_resolution(&mut self, renderer: &Renderer, new_resolution: [u32; 2]);
}

impl Renderable for Box<dyn Component> {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        self.as_ref().render_with_renderpass(pass)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShaderCodeError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't parse shader code: {0}")]
    ParseError(#[from] wgpu::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShaderSource {
    Path(PathBuf),
    Code(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderCode {
    pub language: ShaderLanguage,
    #[serde(flatten)]
    pub source: ShaderSource,
}

impl ShaderCode {
    fn source(&self) -> std::io::Result<String> {
        match self.source.clone() {
            ShaderSource::Code(code) => Ok(code),
            ShaderSource::Path(path) => std::fs::read_to_string(path),
        }
    }
}
