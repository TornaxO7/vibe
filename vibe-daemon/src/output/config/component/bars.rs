use serde::{Deserialize, Serialize};

use super::{AudioConfig, ShaderCode};

const DEFAULT_FRAGMENT_CODE: &str = "
@group(1) @binding(1)
var<uniform> iTime: f32;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let bottom_color = sin(vec3<f32>(2., 4., 8.) * iTime * .25) * .2 + .6;
    return vec4<f32>(bottom_color, pos.y);
}
";

#[derive(Debug, Serialize, Deserialize)]
pub struct BarsConfig {
    pub audio_conf: AudioConfig,
    pub fragment_code: ShaderCode,
}

impl Default for BarsConfig {
    fn default() -> Self {
        Self {
            audio_conf: AudioConfig::default(),
            fragment_code: ShaderCode::Wgsl(DEFAULT_FRAGMENT_CODE.into()),
        }
    }
}
