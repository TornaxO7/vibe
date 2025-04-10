mod aurodio;
mod bars;
mod fragment_canvas;
mod value_noise;

use std::path::PathBuf;

pub use aurodio::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor};
pub use bars::{BarVariant, Bars, BarsDescriptor};
pub use fragment_canvas::{FragmentCanvas, FragmentCanvasDescriptor};
pub use value_noise::{ValueNoise, ValueNoiseDescriptor};

use serde::{Deserialize, Serialize};
use shady_audio::SampleProcessor;

use crate::Renderable;

pub trait Component: Renderable {
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor);

    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32);

    fn update_resolution(&mut self, queue: &wgpu::Queue, new_resolution: [u32; 2]);
}

type ParseErrorMsg = String;

#[derive(thiserror::Error, Debug)]
pub enum ShaderCodeError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't parse shader code: {0}")]
    ParseError(String),
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
