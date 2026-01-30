use super::FreqRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioAudioConfig {
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioLayerConfig {
    pub freq_range: FreqRange,
    pub zoom_factor: f32,
}
