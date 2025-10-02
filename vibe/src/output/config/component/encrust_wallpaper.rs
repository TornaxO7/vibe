use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgeAudioConfig {
    pub sensitivity: f32,
    pub freq_range: Range<NonZero<u16>>,
}
