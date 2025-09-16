mod aurodio;
mod bars;
mod chessy;
mod circle;
mod fragment_canvas;
mod graph;
mod radial;

use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::{
    fetcher::{Fetcher, SystemAudioFetcher},
    SampleProcessor,
};
use vibe_renderer::{
    components::{
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, BarsFormat,
        BarsPlacement, Chessy, ChessyDescriptor, Circle, CircleDescriptor, CircleVariant,
        Component, FragmentCanvas, FragmentCanvasDescriptor, Graph, GraphDescriptor,
        GraphPlacement, GraphVariant, Radial, RadialDescriptor, RadialFormat, RadialVariant,
        SdfPattern, ShaderCode, ShaderCodeError,
    },
    Renderer,
};

pub use aurodio::{AurodioAudioConfig, AurodioLayerConfig};
pub use bars::{BarsAudioConfig, BarsFormatConfig, BarsPlacementConfig, BarsVariantConfig};
pub use chessy::ChessyAudioConfig;
pub use circle::{CircleAudioConfig, CircleVariantConfig};
pub use fragment_canvas::FragmentCanvasAudioConfig;
pub use graph::{GraphAudioConfig, GraphFormatConfig, GraphPlacementConfig, GraphVariantConfig};
pub use radial::{RadialAudioConfig, RadialFormatConfig, RadialVariantConfig};

const GAMMA: f32 = 2.2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentConfig {
    Bars {
        audio_conf: BarsAudioConfig,
        max_height: f32,
        variant: BarsVariantConfig,
        placement: BarsPlacementConfig,
        format: BarsFormatConfig,
    },
    FragmentCanvas {
        audio_conf: FragmentCanvasAudioConfig,
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
        placement: GraphPlacementConfig,
        format: GraphFormatConfig,
    },
    Circle {
        audio_conf: CircleAudioConfig,
        variant: CircleVariantConfig,
        radius: f32,
        rotation: cgmath::Deg<f32>,
        position: (f32, f32),
    },
    Radial {
        audio_conf: RadialAudioConfig,
        variant: RadialVariantConfig,
        format: RadialFormatConfig,

        init_rotation: cgmath::Deg<f32>,
        circle_radius: f32,
        bar_height_sensitivity: f32,
        bar_width: f32,
        position: (f32, f32),
    },
    Chessy {
        movement_speed: f32,
        pattern: SdfPattern,
        zoom_factor: f32,
        audio_conf: ChessyAudioConfig,
    },
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars {
            audio_conf: BarsAudioConfig {
                amount_bars: NonZero::new(60).unwrap(),
                freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                sensitivity: 0.2,
            },
            max_height: 0.75,
            variant: BarsVariantConfig::Color(Rgba::TURQUOISE),
            placement: BarsPlacementConfig::Bottom,
            format: BarsFormatConfig::BassTreble,
        }
    }
}

impl ComponentConfig {
    pub fn to_component<F: Fetcher>(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn Component<F>>, ShaderCodeError> {
        match self {
            ComponentConfig::Bars {
                audio_conf,
                max_height,
                variant,
                placement,
                format,
            } => {
                let variant = match variant {
                    BarsVariantConfig::Color(rgba) => BarVariant::Color(rgba.as_f32()),
                    BarsVariantConfig::PresenceGradient {
                        high_presence,
                        low_presence,
                    } => BarVariant::PresenceGradient {
                        high: high_presence.as_f32(),
                        low: low_presence.as_f32(),
                    },
                    BarsVariantConfig::FragmentCode(code) => BarVariant::FragmentCode(code.clone()),
                };

                Bars::new(&vibe_renderer::components::BarsDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    max_height: *max_height,
                    variant,
                    placement: BarsPlacement::from(placement),
                    format: BarsFormat::from(format),
                })
                .map(|bars| Box::new(bars) as Box<dyn Component<F>>)
            }
            ComponentConfig::FragmentCanvas {
                audio_conf,
                fragment_code,
            } => FragmentCanvas::new(&FragmentCanvasDescriptor {
                sample_processor: processor,
                audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                device: renderer.device(),
                format: texture_format,
                fragment_code: fragment_code.clone(),
            })
            .map(|canvas| Box::new(canvas) as Box<dyn Component<F>>),
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
                    base_color: base_color.as_f32(),
                    movement_speed: *movement_speed,
                    sensitivity: audio_conf.sensitivity,
                    layers: &layers,
                })) as Box<dyn Component<F>>)
            }
            ComponentConfig::Graph {
                audio_conf,
                max_height,
                variant,
                placement,
                format,
            } => {
                let variant = GraphVariant::from(variant);
                let placement = GraphPlacement::from(placement);

                Ok(Box::new(Graph::new(&GraphDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    max_height: *max_height,
                    placement,
                    format: format.into(),
                })) as Box<dyn Component<F>>)
            }
            ComponentConfig::Circle {
                audio_conf,
                variant,
                radius,
                rotation,
                position,
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
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    variant,
                    radius: *radius,
                    rotation: *rotation,
                    position: *position,
                })))
            }
            ComponentConfig::Radial {
                audio_conf,
                variant,
                init_rotation,
                circle_radius,
                bar_height_sensitivity,
                bar_width,
                position,
                format,
            } => {
                let variant = match variant {
                    RadialVariantConfig::Color(rgba) => RadialVariant::Color(rgba.as_f32()),
                    RadialVariantConfig::HeightGradient { inner, outer } => {
                        RadialVariant::HeightGradient {
                            inner: inner.as_f32(),
                            outer: outer.as_f32(),
                        }
                    }
                };

                Ok(Box::new(Radial::new(&RadialDescriptor {
                    device: renderer.device(),
                    processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    init_rotation: *init_rotation,
                    circle_radius: *circle_radius,
                    bar_height_sensitivity: *bar_height_sensitivity,
                    bar_width: *bar_width,
                    position: *position,
                    format: RadialFormat::from(format),
                })))
            }
            ComponentConfig::Chessy {
                movement_speed,
                pattern,
                zoom_factor,
                audio_conf,
            } => Ok(Box::new(Chessy::new(&ChessyDescriptor {
                renderer,
                sample_processor: processor,
                audio_config: vibe_audio::BarProcessorConfig::from(audio_conf),
                texture_format,
                movement_speed: *movement_speed,
                pattern: *pattern,
                zoom_factor: *zoom_factor,
            }))),
        }
    }

    /// Returns `true` if the given component needs two audio channels.
    pub fn uses_stereo_audio(&self) -> bool {
        match self {
            ComponentConfig::Bars { format, .. } => [
                BarsFormatConfig::TrebleBassTreble,
                BarsFormatConfig::BassTrebleBass,
            ]
            .contains(format),
            ComponentConfig::FragmentCanvas { .. } => false,
            ComponentConfig::Aurodio { .. } => false,
            ComponentConfig::Graph { format, .. } => [
                GraphFormatConfig::BassTrebleBass,
                GraphFormatConfig::TrebleBassTreble,
            ]
            .contains(format),
            ComponentConfig::Circle { .. } => false,
            ComponentConfig::Radial { format, .. } => [
                RadialFormatConfig::BassTrebleBass,
                RadialFormatConfig::TrebleBassTreble,
            ]
            .contains(format),
            ComponentConfig::Chessy { .. } => false,
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
    pub fn as_f32(&self) -> [f32; 3] {
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
