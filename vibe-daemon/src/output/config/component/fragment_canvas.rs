use serde::{Deserialize, Serialize};

use super::{AudioConfig, ShaderCode};

const DEFAULT_FRAGMENT_CODE: &str = "
@group(0) @binding(0)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(0)
var<uniform> iTime: f32;

@group(0) @binding(1)
var<storage, read> freqs: array<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / iResolution.xy;
    return vec4<f32>(sin(uv + iTime + freqs[3]) * .5 + .5, 0., 1.0);
}
";

#[derive(Debug, Serialize, Deserialize)]
pub struct FragmentCanvasConfig {
    pub audio_conf: AudioConfig,
    pub fragment_code: ShaderCode,
}

impl Default for FragmentCanvasConfig {
    fn default() -> Self {
        Self {
            audio_conf: AudioConfig::default(),
            fragment_code: ShaderCode::Glsl(),
        }
    }
}
