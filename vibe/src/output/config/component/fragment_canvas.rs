use std::{num::NonZero, ops::Range, path::PathBuf};

use image::{DynamicImage, ImageReader};
use serde::{Deserialize, Serialize};
use vibe_audio::BarProcessorConfig;

#[derive(thiserror::Error, Debug)]
pub enum FragmentCanvasLoadTexture {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Error which occured from `image` crate while trying to decode the image.
    #[error(transparent)]
    Decode(#[from] image::error::ImageError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCanvasTexture {
    pub path: PathBuf,
}

impl FragmentCanvasTexture {
    pub fn load(&self) -> Result<DynamicImage, FragmentCanvasLoadTexture> {
        ImageReader::open(&self.path)
            .map_err(FragmentCanvasLoadTexture::IO)?
            .decode()
            .map_err(FragmentCanvasLoadTexture::Decode)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCanvasAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
}

impl Default for FragmentCanvasAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
            sensitivity: 0.2,
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
