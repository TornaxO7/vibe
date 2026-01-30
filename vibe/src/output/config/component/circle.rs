use super::{FreqRange, Rgba};
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<CircleAudioConfig> for BarProcessorConfig {
    fn from(conf: CircleAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            sensitivity: conf.sensitivity,

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
