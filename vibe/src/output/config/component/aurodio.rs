use std::{num::NonZero, ops::Range};

use serde::{Deserialize, Serialize};
use shady_audio::StandardEasing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioAudioConfig {
    pub easing: StandardEasing,
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioLayerConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub zoom_factor: f32,
}
