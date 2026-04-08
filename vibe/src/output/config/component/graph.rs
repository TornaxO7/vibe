use crate::output::config::component::ComponentConfig;

use super::{FreqRange, Rgba};
use cgmath::Deg;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::fetcher::Fetcher;
use vibe_renderer::components::{
    Graph, GraphBorder, GraphDescriptor, GraphFormat, GraphPlacement, GraphVariant,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    audio_conf: GraphAudioConfig,
    max_height: f32,
    variant: GraphVariantConfig,
    placement: GraphPlacementConfig,
    format: GraphFormatConfig,
    border: Option<GraphBorderConfig>,
}

impl ComponentConfig for GraphConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        let variant = match &self.variant {
            GraphVariantConfig::Color(color) => GraphVariant::Color(color.as_f32()?),
            GraphVariantConfig::HorizontalGradient { left, right } => {
                GraphVariant::HorizontalGradient {
                    left: left.as_f32()?,
                    right: right.as_f32()?,
                }
            }
            GraphVariantConfig::VerticalGradient { top, bottom } => {
                GraphVariant::VerticalGradient {
                    top: top.as_f32()?,
                    bottom: bottom.as_f32()?,
                }
            }
        };

        let placement = GraphPlacement::from(&self.placement);
        let border = match &self.border {
            Some(conf) => Some(GraphBorder {
                color: conf.color.as_f32()?,
                width: conf.width,
            }),
            None => None,
        };

        Ok(Box::new(Graph::new(&GraphDescriptor {
            renderer,
            sample_processor: processor,
            audio_conf: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            output_texture_format: texture_format,
            variant,
            max_height: self.max_height,
            placement,
            format: self.format.clone().into(),
            border,
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAudioConfig {
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl From<GraphAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: GraphAudioConfig) -> Self {
        Self {
            freq_range: conf.freq_range.range(),
            down: conf.sensitivity,
            ..Default::default()
        }
    }
}

impl From<&GraphAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: &GraphAudioConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphVariantConfig {
    Color(Rgba),
    HorizontalGradient { left: Rgba, right: Rgba },
    VerticalGradient { top: Rgba, bottom: Rgba },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphPlacementConfig {
    Bottom,
    Top,
    Right,
    Left,
    Custom {
        offset: [f32; 2],
        rotation: Deg<f32>,
        amount_bars: NonZero<u16>,
    },
}

impl From<GraphPlacementConfig> for GraphPlacement {
    fn from(conf: GraphPlacementConfig) -> Self {
        match conf {
            GraphPlacementConfig::Bottom => Self::Bottom,
            GraphPlacementConfig::Top => Self::Top,
            GraphPlacementConfig::Right => Self::Right,
            GraphPlacementConfig::Left => Self::Left,
            GraphPlacementConfig::Custom {
                offset,
                rotation,
                amount_bars,
            } => Self::Custom {
                bottom_left_corner: offset,
                rotation,
                amount_bars,
            },
        }
    }
}

impl From<&GraphPlacementConfig> for GraphPlacement {
    fn from(conf: &GraphPlacementConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphFormatConfig {
    BassTreble,
    TrebleBass,
    BassTrebleBass,
    TrebleBassTreble,
}

impl From<GraphFormatConfig> for GraphFormat {
    fn from(conf: GraphFormatConfig) -> Self {
        match conf {
            GraphFormatConfig::BassTreble => Self::BassTreble,
            GraphFormatConfig::TrebleBass => Self::TrebleBass,
            GraphFormatConfig::BassTrebleBass => Self::BassTrebleBass,
            GraphFormatConfig::TrebleBassTreble => Self::TrebleBassTreble,
        }
    }
}

impl From<&GraphFormatConfig> for GraphFormat {
    fn from(conf: &GraphFormatConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBorderConfig {
    pub width: f32,
    pub color: Rgba,
}
