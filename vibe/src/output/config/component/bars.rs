use crate::output::config::component::ComponentConfig;

use super::{FreqRange, Rgba};
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::fetcher::Fetcher;
use vibe_renderer::components::{
    BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement, Pixels,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarsConfig {
    pub audio_conf: BarsAudioConfig,
    pub max_height: f32,
    pub variant: BarsVariantConfig,
    pub placement: BarsPlacementConfig,
    pub format: BarsFormatConfig,
}

impl ComponentConfig for BarsConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        let variant = match &self.variant {
            BarsVariantConfig::Color(rgba) => BarVariant::Color(rgba.as_f32()?),
            BarsVariantConfig::PresenceGradient {
                high_presence,
                low_presence,
            } => BarVariant::PresenceGradient {
                high: high_presence.as_f32()?,
                low: low_presence.as_f32()?,
            },
            BarsVariantConfig::HorizontalGradient { left, right } => {
                BarVariant::HorizontalGradient {
                    left: left.as_f32()?,
                    right: right.as_f32()?,
                }
            }
            BarsVariantConfig::VerticalGradient { bottom, top } => BarVariant::VerticalGradient {
                top: top.as_f32()?,
                bottom: bottom.as_f32()?,
            },
        };

        let bars = Bars::new(&BarsDescriptor {
            renderer,
            sample_processor: processor,
            audio_conf: vibe_audio::BarProcessorConfig::from(&self.audio_conf),
            texture_format,
            max_height: self.max_height,
            variant,
            placement: BarsPlacement::from(&self.placement),
            format: BarsFormat::from(&self.format),
        })?;

        Ok(Box::new(bars))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

impl Default for BarsConfig {
    fn default() -> Self {
        let turquoise = Rgba::new("#00FFFFFF");
        Self {
            audio_conf: BarsAudioConfig {
                amount_bars: NonZero::new(60).unwrap(),
                freq_range: FreqRange::Custom(
                    NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                ),
                sensitivity: 4.0,
            },
            max_height: 0.75,
            variant: BarsVariantConfig::Color(turquoise),
            placement: BarsPlacementConfig::Bottom,
            format: BarsFormatConfig::BassTreble,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarsAudioConfig {
    pub amount_bars: NonZero<u16>,
    pub freq_range: FreqRange,
    pub sensitivity: f32,
}

impl Default for BarsAudioConfig {
    fn default() -> Self {
        Self {
            amount_bars: NonZero::new(60).unwrap(),
            freq_range: FreqRange::Custom(NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap()),
            sensitivity: 0.2,
        }
    }
}

impl From<BarsAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(conf: BarsAudioConfig) -> Self {
        Self {
            amount_bars: conf.amount_bars,
            freq_range: conf.freq_range.range(),
            sensitivity: conf.sensitivity,

            ..Default::default()
        }
    }
}

impl From<&BarsAudioConfig> for vibe_audio::BarProcessorConfig {
    fn from(value: &BarsAudioConfig) -> Self {
        Self::from(value.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsVariantConfig {
    Color(Rgba),
    PresenceGradient {
        high_presence: Rgba,
        low_presence: Rgba,
    },
    HorizontalGradient {
        left: Rgba,
        right: Rgba,
    },
    VerticalGradient {
        bottom: Rgba,
        top: Rgba,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsPlacementConfig {
    Bottom,
    Top,
    Left,
    Right,
    Custom {
        bottom_left_corner: (f32, f32),
        width: Pixels<u16>,
        rotation: cgmath::Deg<f32>,
        height_mirrored: Option<bool>,
    },
}

impl From<BarsPlacementConfig> for BarsPlacement {
    fn from(value: BarsPlacementConfig) -> Self {
        match value {
            BarsPlacementConfig::Bottom => BarsPlacement::Bottom,
            BarsPlacementConfig::Top => BarsPlacement::Top,
            BarsPlacementConfig::Left => BarsPlacement::Left,
            BarsPlacementConfig::Right => BarsPlacement::Right,
            BarsPlacementConfig::Custom {
                bottom_left_corner,
                width,
                rotation,
                height_mirrored,
            } => BarsPlacement::Custom {
                bottom_left_corner,
                width,
                rotation,
                height_mirrored: height_mirrored.unwrap_or(false),
            },
        }
    }
}

impl From<&BarsPlacementConfig> for BarsPlacement {
    fn from(conf: &BarsPlacementConfig) -> Self {
        Self::from(conf.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BarsFormatConfig {
    BassTreble,
    TrebleBass,
    TrebleBassTreble,
    BassTrebleBass,
}

impl From<BarsFormatConfig> for BarsFormat {
    fn from(config: BarsFormatConfig) -> Self {
        match config {
            BarsFormatConfig::BassTreble => Self::BassTreble,
            BarsFormatConfig::TrebleBass => Self::TrebleBass,
            BarsFormatConfig::BassTrebleBass => Self::BassTrebleBass,
            BarsFormatConfig::TrebleBassTreble => Self::TrebleBassTreble,
        }
    }
}

impl From<&BarsFormatConfig> for BarsFormat {
    fn from(config: &BarsFormatConfig) -> Self {
        Self::from(config.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod audio_config {
        use super::*;

        mod freq_range {
            use super::*;

            #[test]
            fn with_preset() {
                let conf = "
                    amount_bars = 10
                    freq_range = \"Bass\"
                    sensitivity = 4.0
                ";

                let current: BarsAudioConfig = toml::from_str(conf).unwrap();

                assert_eq!(current.freq_range, FreqRange::Bass);
            }

            #[test]
            fn with_custom() {
                let conf = "
                    amount_bars = 42
                    freq_range.Custom = { start = 250, end = 1000 }
                    sensitivity = 4.0
                ";

                let current: BarsAudioConfig = toml::from_str(conf).unwrap();

                assert_eq!(
                    current.freq_range,
                    FreqRange::Custom(NonZero::new(250).unwrap()..NonZero::new(1_000).unwrap())
                );
            }
        }
    }
}
