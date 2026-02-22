use crate::output::config::component::ComponentConfig;

use super::FreqRange;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig};
use vibe_renderer::{
    components::{Chessy, ChessyDescriptor},
    texture_generation::SdfPattern,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessyConfig {
    pub movement_speed: f32,
    pub pattern: SdfPattern,
    pub zoom_factor: f32,
    pub audio_conf: ChessyAudioConfig,
}

impl ComponentConfig for ChessyConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        Ok(Box::new(Chessy::new(&ChessyDescriptor {
            renderer,
            sample_processor: processor,
            audio_config: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            texture_format,
            movement_speed: self.movement_speed,
            pattern: self.pattern,
            zoom_factor: self.zoom_factor,
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChessyAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<ChessyAudioConfig> for BarProcessorConfig {
    fn from(conf: ChessyAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            down: conf.sensitivity,
            ..Default::default()
        }
    }
}

impl From<&ChessyAudioConfig> for BarProcessorConfig {
    fn from(conf: &ChessyAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}
