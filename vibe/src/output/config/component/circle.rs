use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::{BarProcessorConfig, StandardEasing};

use super::Rgba;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
}

impl From<CircleAudioConfig> for BarProcessorConfig {
    fn from(conf: CircleAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,
            easer: conf.easing,
            ..Default::default()
        }
    }
}

impl From<&CircleAudioConfig> for BarProcessorConfig {
    fn from(conf: &CircleAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircleVariantConfig {
    Graph { spike_sensitivity: f32, color: Rgba },
}
