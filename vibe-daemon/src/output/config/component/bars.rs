use serde::{Deserialize, Serialize};
use vibe_renderer::components::ShaderCode;

use super::AudioConfig;

const DEFAULT_WGSL_FRAGMENT_CODE: &str = "
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var color = sin(vec3<f32>(2., 4., 8.) * iTime * .25) * .2 + .6;

    // apply gamma correction
    const GAMMA: f32 = 2.2;
    color.r = pow(color.r, GAMMA);
    color.g = pow(color.g, GAMMA);
    color.b = pow(color.b, GAMMA);
    return vec4<f32>(color, pos.y);
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
            fragment_code: ShaderCode::Wgsl(DEFAULT_WGSL_FRAGMENT_CODE.into()),
        }
    }
}
