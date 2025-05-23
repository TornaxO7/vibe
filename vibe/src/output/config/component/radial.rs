use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::StandardEasing;

use super::Rgba;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
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
pub enum RadialVariantConfig {
    Color(Rgba),
}
