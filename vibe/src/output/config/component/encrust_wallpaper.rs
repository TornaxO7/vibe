use super::FreqRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgeAudioConfig {
    pub sensitivity: f32,
    pub freq_range: FreqRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgeThresholds {
    pub high: f32,
    pub low: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgeGaussianBlur {
    pub sigma: f32,
    pub kernel_size: usize,
}
