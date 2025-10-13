use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEdgeAudioConfig {
    pub sensitivity: f32,
    pub freq_range: Range<NonZero<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEdgeThresholds {
    pub high: f32,
    pub low: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseEdgeGaussianBlur {
    pub sigma: f32,
    pub kernel_size: usize,
}
