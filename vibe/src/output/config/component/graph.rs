use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::StandardEasing;

use super::Rgba;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAudioConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
    pub easing: StandardEasing,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphVariantConfig {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}
