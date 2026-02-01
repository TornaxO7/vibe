use crate::output::config::component::ComponentConfig;

use super::{FreqRange, Rgba};
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::fetcher::Fetcher;
use vibe_renderer::components::{Radial, RadialDescriptor, RadialFormat, RadialVariant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialConfig {
    pub audio_conf: RadialAudioConfig,
    pub variant: RadialVariantConfig,
    pub format: RadialFormatConfig,

    pub init_rotation: cgmath::Deg<f32>,
    pub circle_radius: f32,
    pub bar_height_sensitivity: f32,
    pub bar_width: f32,
    pub position: (f32, f32),
}

impl ComponentConfig for RadialConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::Component>, super::ConfigError> {
        let variant = match &self.variant {
            RadialVariantConfig::Color(rgba) => RadialVariant::Color(rgba.as_f32()),
            RadialVariantConfig::HeightGradient { inner, outer } => RadialVariant::HeightGradient {
                inner: inner.as_f32(),
                outer: outer.as_f32(),
            },
        };

        Ok(Box::new(Radial::new(&RadialDescriptor {
            renderer,
            processor,
            audio_conf: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            output_texture_format: texture_format,
            variant,
            init_rotation: self.init_rotation,
            circle_radius: self.circle_radius,
            bar_height_sensitivity: self.bar_height_sensitivity,
            bar_width: self.bar_width,
            position: self.position,
            format: RadialFormat::from(self.format.clone()),
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<RadialAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: RadialAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            sensitivity: conf.sensitivity,

            ..Default::default()
        }
    }
}

impl From<&RadialAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: &RadialAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RadialVariantConfig {
    Color(Rgba),
    HeightGradient { inner: Rgba, outer: Rgba },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RadialFormatConfig {
    BassTreble,
    TrebleBass,
    BassTrebleBass,
    TrebleBassTreble,
}

impl From<RadialFormatConfig> for RadialFormat {
    fn from(conf: RadialFormatConfig) -> Self {
        match conf {
            RadialFormatConfig::BassTreble => Self::BassTreble,
            RadialFormatConfig::TrebleBass => Self::TrebleBass,
            RadialFormatConfig::BassTrebleBass => Self::BassTrebleBass,
            RadialFormatConfig::TrebleBassTreble => Self::TrebleBassTreble,
        }
    }
}

impl From<&RadialFormatConfig> for RadialFormat {
    fn from(conf: &RadialFormatConfig) -> Self {
        Self::from(conf.clone())
    }
}
