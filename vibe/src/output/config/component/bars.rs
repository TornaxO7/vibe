use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use vibe_renderer::components::{BarsFormat, BarsPlacement, ShaderCode};

use super::Rgba;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarsAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
}

impl Default for BarsAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 0.2,
        }
    }
}

impl From<BarsAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: BarsAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,

            ..Default::default()
        }
    }
}

impl From<&BarsAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(value: &BarsAudioConfig) -> Self {
        Self::from(value.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsVariantConfig {
    Color(Rgba),
    PresenceGradient {
        high_presence: Rgba,
        low_presence: Rgba,
    },
    FragmentCode(ShaderCode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsPlacementConfig {
    Bottom,
    Top,
    Left,
    Right,
    Custom {
        bottom_left_corner: (f32, f32),
        width_factor: f32,
        rotation: cgmath::Deg<f32>,
    },
}

impl From<BarsPlacementConfig> for BarsPlacement {
    fn from(value: BarsPlacementConfig) -> Self {
        match value {
            BarsPlacementConfig::Bottom => BarsPlacement::Bottom,
            BarsPlacementConfig::Top => BarsPlacement::Top,
            BarsPlacementConfig::Left => BarsPlacement::Left,
            BarsPlacementConfig::Right => BarsPlacement::Right,
            BarsPlacementConfig::Custom {
                bottom_left_corner,
                width_factor,
                rotation,
            } => BarsPlacement::Custom {
                bottom_left_corner,
                width_factor,
                rotation,
            },
        }
    }
}

impl From<&BarsPlacementConfig> for BarsPlacement {
    fn from(value: &BarsPlacementConfig) -> Self {
        match *value {
            BarsPlacementConfig::Bottom => BarsPlacement::Bottom,
            BarsPlacementConfig::Top => BarsPlacement::Top,
            BarsPlacementConfig::Left => BarsPlacement::Left,
            BarsPlacementConfig::Right => BarsPlacement::Right,
            BarsPlacementConfig::Custom {
                bottom_left_corner,
                width_factor,
                rotation,
            } => BarsPlacement::Custom {
                bottom_left_corner,
                width_factor,
                rotation,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsFormatConfig {
    BassTreble,
    TrebleBass,
    TrebleBassTreble,
    BassTrebleBass,
}

impl From<BarsFormatConfig> for BarsFormat {
    fn from(config: BarsFormatConfig) -> Self {
        match config {
            BarsFormatConfig::BassTreble => Self::BassTreble,
            BarsFormatConfig::TrebleBass => Self::TrebleBass,
            BarsFormatConfig::BassTrebleBass => Self::BassTrebleBass,
            BarsFormatConfig::TrebleBassTreble => Self::TrebleBassTreble,
        }
    }
}

impl From<&BarsFormatConfig> for BarsFormat {
    fn from(config: &BarsFormatConfig) -> Self {
        Self::from(config.clone())
    }
}
