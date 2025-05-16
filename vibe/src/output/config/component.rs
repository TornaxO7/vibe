use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::{SampleProcessor, StandardEasing};
use vibe_renderer::{
    components::{
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, Circle,
        CircleDescriptor, CircleVariant, Component, FragmentCanvas, FragmentCanvasDescriptor,
        Graph, GraphDescriptor, GraphVariant, Radial, RadialDescriptor, RadialVariant, ShaderCode,
        ShaderCodeError,
    },
    Renderer,
};

const GAMMA: f32 = 2.2;

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
    Graph {
        audio_conf: GraphAudioConfig,
        max_height: f32,
        variant: GraphVariantConfig,
        smoothness: f32,
    },
    Circle {
        audio_conf: BarAudioConfig,
        variant: CircleVariantConfig,
        radius: f32,
        rotation: cgmath::Deg<f32>,
    },
    Radial {
        audio_conf: RadialAudioConfig,
        variant: RadialVariantConfig,

        init_rotation: cgmath::Deg<f32>,
        circle_radius: f32,
        bar_height_sensitivity: f32,
        bar_width: f32,
    },
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars {
            audio_conf: BarAudioConfig::default(),
            max_height: 0.75,
            variant: BarVariantConfig::Color(Rgba::TURQUOISE),
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
                    BarVariantConfig::Color(rgba) => BarVariant::Color(rgba.as_f32()),
                    BarVariantConfig::PresenceGradient {
                        high_presence,
                        low_presence,
                    } => BarVariant::PresenceGradient {
                        high: high_presence.as_f32(),
                        low: low_presence.as_f32(),
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
            ComponentConfig::Graph {
                audio_conf,
                max_height,
                variant,
                smoothness,
            } => {
                let variant = match variant {
                    GraphVariantConfig::Color(rgba) => GraphVariant::Color(rgba.as_f32()),
                    GraphVariantConfig::HorizontalGradient { left, right } => {
                        GraphVariant::HorizontalGradient {
                            left: left.as_f32(),
                            right: right.as_f32(),
                        }
                    }
                    GraphVariantConfig::VerticalGradient { top, bottom } => {
                        GraphVariant::VerticalGradient {
                            top: top.as_f32(),
                            bottom: bottom.as_f32(),
                        }
                    }
                };

                Ok(Box::new(Graph::new(&GraphDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    max_height: *max_height,
                    smoothness: *smoothness,
                })) as Box<dyn Component>)
            }
            ComponentConfig::Circle {
                audio_conf,
                variant,
                radius,
                rotation,
            } => {
                let variant = match variant {
                    CircleVariantConfig::Graph {
                        spike_sensitivity,
                        color,
                    } => CircleVariant::Graph {
                        spike_sensitivity: *spike_sensitivity,
                        color: color.as_f32(),
                    },
                };

                Ok(Box::new(Circle::new(&CircleDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    variant,
                    radius: *radius,
                    rotation: *rotation,
                })))
            }
            ComponentConfig::Radial {
                audio_conf,
                variant,
                init_rotation,
                circle_radius,
                bar_height_sensitivity,
                bar_width,
            } => {
                let variant = match variant {
                    RadialVariantConfig::Color(rgba) => RadialVariant::Color(rgba.as_f32()),
                };

                Ok(Box::new(Radial::new(&RadialDescriptor {
                    device: renderer.device(),
                    processor,
                    audio_conf: shady_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    init_rotation: *init_rotation,
                    circle_radius: *circle_radius,
                    bar_height_sensitivity: *bar_height_sensitivity,
                    bar_width: *bar_width,
                })))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const TURQUOISE: Self = Self([0, 255, 255, 255]);

    pub fn as_f32(&self) -> [f32; 4] {
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
pub enum CircleVariantConfig {
    Graph { spike_sensitivity: f32, color: Rgba },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphVariantConfig {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
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
pub enum RadialVariantConfig {
    Color(Rgba),
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

impl From<GraphAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(conf: GraphAudioConfig) -> Self {
        Self {
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,
            easer: conf.easing,
            ..Default::default()
        }
    }
}

impl From<&GraphAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(conf: &GraphAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

impl From<RadialAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(conf: RadialAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,
            easer: conf.easing,
            ..Default::default()
        }
    }
}

impl From<&RadialAudioConfig> for shady_audio::BarProcessorConfig {
    fn from(conf: &RadialAudioConfig) -> Self {
        Self::from(conf.clone())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAudioConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
}
