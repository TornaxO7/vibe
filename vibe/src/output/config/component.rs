use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::{SampleProcessor, StandardEasing};
use vibe_renderer::{
    components::{
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, Component,
        FragmentCanvas, FragmentCanvasDescriptor, ShaderCode, ShaderCodeError, ShaderSource,
    },
    Renderer,
};

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
pub enum ComponentConfig {
    Bars {
        audio_conf: BarAudioConfig,
        max_height: f32,
        variant: BarVariantConfig,
    },
    FragmentCanvas {
        audio_conf: BarAudioConfig,
        fragment_code: ShaderCode,
    },
    Aurodio {
        base_color: Rgb,
        movement_speed: f32,
        audio_conf: AurodioAudioConfig,
        layers: Vec<AurodioLayerConfig>,
    },
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars {
            audio_conf: BarAudioConfig::default(),
            max_height: 0.75,
            variant: BarVariantConfig::FragmentCode(ShaderCode {
                language: vibe_renderer::components::ShaderLanguage::Wgsl,
                source: ShaderSource::Code(DEFAULT_BARS_WGSL_FRAGMENT_CODE.into()),
            }),
        }
    }
}

impl ComponentConfig {
    pub fn to_component(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn Component>, ShaderCodeError> {
        match self {
            ComponentConfig::Bars {
                audio_conf,
                max_height,
                variant,
            } => {
                let variant = match variant {
                    BarVariantConfig::Color(rgba) => BarVariant::Color(rgba.gamma_corrected()),
                    BarVariantConfig::PresenceGradient {
                        high_presence,
                        low_presence,
                    } => BarVariant::PresenceGradient {
                        high: high_presence.gamma_corrected(),
                        low: low_presence.gamma_corrected(),
                    },
                    BarVariantConfig::FragmentCode(code) => BarVariant::FragmentCode(code.clone()),
                };

                Bars::new(&vibe_renderer::components::BarsDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    max_height: *max_height,
                    variant,
                })
                .map(|bars| Box::new(bars) as Box<dyn Component>)
            }
            ComponentConfig::FragmentCanvas {
                audio_conf,
                fragment_code,
            } => FragmentCanvas::new(&FragmentCanvasDescriptor {
                sample_processor: processor,
                audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                device: renderer.device(),
                format: texture_format,
                fragment_code: fragment_code.clone(),
            })
            .map(|canvas| Box::new(canvas) as Box<dyn Component>),
            ComponentConfig::Aurodio {
                base_color,
                movement_speed,
                audio_conf,
                layers,
            } => {
                let layers: Vec<AurodioLayerDescriptor> = layers
                    .iter()
                    .map(|layer| AurodioLayerDescriptor {
                        freq_range: layer.freq_range.clone(),
                        zoom_factor: layer.zoom_factor,
                    })
                    .collect();

                Ok(Box::new(Aurodio::new(&AurodioDescriptor {
                    renderer,
                    sample_processor: processor,
                    texture_format,
                    base_color: base_color.gamma_corrected(),
                    movement_speed: *movement_speed,
                    easing: audio_conf.easing,
                    sensitivity: audio_conf.sensitivity,
                    layers: &layers,
                })) as Box<dyn Component>)
            }
        }
    }
}

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
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    pub fn gamma_corrected(&self) -> [f32; 3] {
        let mut rgba_f32 = [0f32; 3];
        for (idx, value) in self.0.iter().enumerate() {
            rgba_f32[idx] = (*value as f32) / 255f32;
        }

        // apply gamma correction
        for value in rgba_f32[0..2].iter_mut() {
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
pub struct BarAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
}

impl Default for BarAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 0.2,
            easing: StandardEasing::OutCubic,
        }
    }
}

impl From<BarAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(conf: BarAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,
            easer: conf.easing,
            ..Default::default()
        }
    }
}

impl From<&BarAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(value: &BarAudioConfig) -> Self {
        Self::from(value.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioAudioConfig {
    pub easing: StandardEasing,
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioLayerConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub zoom_factor: f32,
}
