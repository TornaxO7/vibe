use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::{BarProcessorConfig, StandardEasing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCanvasAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
}

impl Default for FragmentCanvasAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 0.2,
            easing: StandardEasing::OutCubic,
        }
    }
}

impl From<FragmentCanvasAudioConfig> for BarProcessorConfig {
    fn from(conf: FragmentCanvasAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            sensitivity: conf.sensitivity,
            ..Default::default()
        }
    }
}

impl From<&FragmentCanvasAudioConfig> for BarProcessorConfig {
    fn from(conf: &FragmentCanvasAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}
