use crate::output::config::component::ComponentConfig;

use super::{FreqRange, Rgba};
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::{fetcher::Fetcher, BarProcessorConfig};
use vibe_renderer::components::{Circle, CircleDescriptor, CircleVariant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleConfig {
    pub audio_conf: CircleAudioConfig,
    pub variant: CircleVariantConfig,
    pub radius: f32,
    pub rotation: cgmath::Deg<f32>,
    pub position: (f32, f32),
}

impl ComponentConfig for CircleConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        let variant = match &self.variant {
            CircleVariantConfig::Graph {
                spike_sensitivity,
                color,
            } => CircleVariant::Graph {
                spike_sensitivity: *spike_sensitivity,
                color: color.as_f32()?,
            },
        };

        Ok(Box::new(Circle::new(&CircleDescriptor {
            renderer,
            sample_processor: processor,
            audio_conf: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            texture_format,
            variant,
            radius: self.radius,
            rotation: self.rotation,
            position: self.position,
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<CircleAudioConfig> for BarProcessorConfig {
    fn from(conf: CircleAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            down: conf.sensitivity,

            ..Default::default()
        }
    }
}

impl From<&CircleAudioConfig> for BarProcessorConfig {
    fn from(conf: &CircleAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircleVariantConfig {
    Graph { spike_sensitivity: f32, color: Rgba },
}
