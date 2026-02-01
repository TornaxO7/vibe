use crate::output::config::component::{ConfigError, ToComponent};

use super::FreqRange;
use image::{DynamicImage, ImageReader};
use serde::{Deserialize, Serialize};
use std::{num::NonZero, path::PathBuf};
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig};
use vibe_renderer::components::{FragmentCanvas, FragmentCanvasDescriptor, ShaderCode};

#[derive(thiserror::Error, Debug)]
pub enum FragmentCanvasLoadTexture {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Error which occured from `image` crate while trying to decode the image.
    #[error(transparent)]
    Decode(#[from] image::error::ImageError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCanvasConfig {
    pub audio_conf: FragmentCanvasAudioConfig,
    pub fragment_code: ShaderCode,

    pub texture: Option<FragmentCanvasTexture>,
}

impl<F: Fetcher> ToComponent<F> for FragmentCanvasConfig {
    fn to_component(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::Component>, ConfigError> {
        let img = match &self.texture {
            Some(texture) => match texture.load() {
                Ok(img) => Some(img),
                Err(FragmentCanvasLoadTexture::IO(err)) => {
                    return Err(ConfigError::OpenFile {
                        path: texture.path.to_string_lossy().to_string(),
                        reason: err,
                    })
                }
                Err(FragmentCanvasLoadTexture::Decode(err)) => return Err(ConfigError::Image(err)),
            },
            None => None,
        };

        // Check: Is `texture_path` set if it's used in the shader-code?
        {
            let code = self.fragment_code.source()?;
            if (code.contains("iSampler") || code.contains("iTexture")) && img.is_none() {
                return Err(ConfigError::MissingTexture);
            }
        }

        let fragment_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
            sample_processor: processor,
            audio_conf: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            renderer,
            format: texture_format,
            fragment_code: self.fragment_code.clone(),
            img,
        })?;

        Ok(Box::new(fragment_canvas))
    }
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
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl Default for FragmentCanvasAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: FreqRange::Custom(NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap()),
            sensitivity: 0.2,
        }
    }
}

impl From<FragmentCanvasAudioConfig> for BarProcessorConfig {
    fn from(conf: FragmentCanvasAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
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
