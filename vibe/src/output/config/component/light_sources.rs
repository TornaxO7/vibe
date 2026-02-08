use super::ConfigError;
use crate::output::config::component::ComponentConfig;
use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range, path::PathBuf};
use vibe_audio::fetcher::Fetcher;
use vibe_renderer::components::live_wallpaper::light_sources::{
    LightSourceData, LightSources, LightSourcesDescriptor,
};

#[derive(thiserror::Error, Debug)]
pub enum LightSourcesError {
    // If radius is <= 0.
    #[error("Light source with center {center:?} must be > 0")]
    InvalidRadius { center: [f32; 2] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourcesConfig {
    pub wallpaper_path: PathBuf,

    pub audio_conf: LightSourcesAudioConfig,

    pub sources: Vec<LightSourcesSource>,
    pub uniform_pulse: bool,
    pub debug_sources: bool,
}

impl ComponentConfig for LightSourcesConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, ConfigError> {
        let img = ImageReader::open(&self.wallpaper_path)
            .map_err(|err| ConfigError::OpenFile {
                path: self.wallpaper_path.to_string_lossy().to_string(),
                reason: err,
            })?
            .decode()?;

        let sources = self
            .sources
            .iter()
            .map(LightSourceData::try_from)
            .collect::<Result<Vec<LightSourceData>, LightSourcesError>>()?;

        let light_sources = LightSources::new(&LightSourcesDescriptor {
            renderer,
            format: texture_format,

            processor,
            freq_range: self.audio_conf.freq_range.clone(),
            sensitivity: self.audio_conf.sensitivity,

            wallpaper: img,
            sources: &sources,
            uniform_pulse: self.uniform_pulse,
            debug_sources: self.debug_sources,
        });

        Ok(Box::new(light_sources))
    }

    fn external_paths(&self) -> Vec<PathBuf> {
        vec![self.wallpaper_path.clone()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourcesAudioConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourcesSource {
    pub center: [f32; 2],
    pub radius: f32,
}

impl<'a> TryFrom<&'a LightSourcesSource> for LightSourceData {
    type Error = LightSourcesError;

    fn try_from(source: &'a LightSourcesSource) -> Result<Self, Self::Error> {
        if source.radius <= 0. {
            return Err(LightSourcesError::InvalidRadius {
                center: source.center,
            });
        }

        Ok(Self {
            center: source.center,
            // invert the radius because the user expects: The higher the value => the higher the radius
            radius: 1. / source.radius,
        })
    }
}
