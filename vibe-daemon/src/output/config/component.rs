use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use vibe_renderer::components::ShaderCode;

const GAMMA: f32 = 2.2;

const DEFAULT_BARS_WGSL_FRAGMENT_CODE: &str = "
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var color = sin(vec3<f32>(2., 4., 8.) * iTime * .25) * .2 + .6;

    // apply gamma correction
    const GAMMA: f32 = 2.2;
    color.r = pow(color.r, GAMMA);
    color.g = pow(color.g, GAMMA);
    color.b = pow(color.b, GAMMA);
    return vec4<f32>(color, 1. - pos.y / iResolution.y);
}
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub fn gamma_corrected(&self) -> [f32; 4] {
        let mut rgba_f32 = [0f32; 4];
        for (idx, value) in self.0.iter().enumerate() {
            rgba_f32[idx] = (*value as f32) / 255f32;
        }

        // apply gamma correction
        for value in rgba_f32[0..3].iter_mut() {
            *value = value.powf(GAMMA);
        }

        rgba_f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarVariantConfig {
    Color(Rgba),
    PresenceGradient {
        high_presence: Rgba,
        low_presence: Rgba,
    },
    FragmentCode(ShaderCode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentConfig {
    Bars {
        audio_conf: AudioConfig,
        max_height: f32,
        variant: BarVariantConfig,
    },
    FragmentCanvas {
        audio_conf: AudioConfig,
        fragment_code: ShaderCode,
    },
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars {
            audio_conf: AudioConfig::default(),
            max_height: 0.75,
            variant: BarVariantConfig::FragmentCode(ShaderCode::Wgsl(
                DEFAULT_BARS_WGSL_FRAGMENT_CODE.into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensitivity {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: Sensitivity,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: Sensitivity {
                min: 0.05,
                max: 0.2,
            },
        }
    }
}

impl From<AudioConfig> for shady_audio::Config {
    fn from(conf: AudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: shady_audio::Sensitivity {
                min: conf.sensitivity.min,
                max: conf.sensitivity.max,
            },
            ..Default::default()
        }
    }
}
