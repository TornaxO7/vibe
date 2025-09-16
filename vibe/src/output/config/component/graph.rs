use std::{num::NonZero, ops::Range};

use cgmath::Deg;
use serde::{Deserialize, Serialize};
use vibe_renderer::components::{GraphFormat, GraphPlacement, GraphVariant};

use super::Rgba;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAudioConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
}

impl From<GraphAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: GraphAudioConfig) -> Self {
        Self {
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,

            ..Default::default()
        }
    }
}

impl From<&GraphAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: &GraphAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphVariantConfig {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}

impl From<GraphVariantConfig> for GraphVariant {
    fn from(conf: GraphVariantConfig) -> Self {
        match conf {
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
        }
    }
}

impl From<&GraphVariantConfig> for GraphVariant {
    fn from(conf: &GraphVariantConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphPlacementConfig {
    Bottom,
    Top,
    Right,
    Left,
    Custom {
        offset: [f32; 2],
        rotation: Deg<f32>,
        amount_bars: NonZero<u16>,
    },
}

impl From<GraphPlacementConfig> for GraphPlacement {
    fn from(conf: GraphPlacementConfig) -> Self {
        match conf {
            GraphPlacementConfig::Bottom => Self::Bottom,
            GraphPlacementConfig::Top => Self::Top,
            GraphPlacementConfig::Right => Self::Right,
            GraphPlacementConfig::Left => Self::Left,
            GraphPlacementConfig::Custom {
                offset,
                rotation,
                amount_bars,
            } => Self::Custom {
                bottom_left_corner: offset,
                rotation,
                amount_bars,
            },
        }
    }
}

impl From<&GraphPlacementConfig> for GraphPlacement {
    fn from(conf: &GraphPlacementConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphFormatConfig {
    BassTreble,
    TrebleBass,
    BassTrebleBass,
    TrebleBassTreble,
}

impl From<GraphFormatConfig> for GraphFormat {
    fn from(conf: GraphFormatConfig) -> Self {
        match conf {
            GraphFormatConfig::BassTreble => Self::BassTreble,
            GraphFormatConfig::TrebleBass => Self::TrebleBass,
            GraphFormatConfig::BassTrebleBass => Self::BassTrebleBass,
            GraphFormatConfig::TrebleBassTreble => Self::TrebleBassTreble,
        }
    }
}

impl From<&GraphFormatConfig> for GraphFormat {
    fn from(conf: &GraphFormatConfig) -> Self {
        Self::from(conf.clone())
    }
}
