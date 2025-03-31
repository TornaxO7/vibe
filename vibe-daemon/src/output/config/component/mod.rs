use std::{num::NonZero, ops::Range};

use bars::BarsConfig;
use fragment_canvas::FragmentCanvasConfig;
use serde::{Deserialize, Serialize};

pub mod bars;
pub mod fragment_canvas;

#[derive(Debug)]
pub enum ComponentConfig {
    Bars(BarsConfig),
    FragmentCanvas(FragmentCanvasConfig),
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars(BarsConfig::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
        }
    }
}

impl From<AudioConfig> for shady_audio::Config {
    fn from(conf: AudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ShaderCode {
    Wgsl(String),
    Glsl(String),
}
