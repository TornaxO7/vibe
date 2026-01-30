use super::FreqRange;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessyAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<ChessyAudioConfig> for BarProcessorConfig {
    fn from(conf: ChessyAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            sensitivity: conf.sensitivity,
            ..Default::default()
        }
    }
}

impl From<&ChessyAudioConfig> for BarProcessorConfig {
    fn from(conf: &ChessyAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}
