use super::AudioConfig;
use serde::{Deserialize, Serialize};
use vibe_renderer::components::ShaderCode;

const DEFAULT_WGSL_FRAGMENT_CODE: &str = "
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
            fragment_code: ShaderCode::Wgsl(DEFAULT_WGSL_FRAGMENT_CODE.into()),
        }
    }
}
