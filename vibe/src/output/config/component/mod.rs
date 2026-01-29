mod aurodio;
mod bars;
mod chessy;
mod circle;
mod encrust_wallpaper;
mod fragment_canvas;
mod graph;
mod light_sources;
mod radial;

use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::{num::NonZero, path::PathBuf};
use vibe_audio::{fetcher::Fetcher, SampleProcessor};
use vibe_renderer::{
    components::{
        live_wallpaper::{
            self,
            light_sources::{LightSourceData, LightSources, LightSourcesDescriptor},
        },
        Aurodio, AurodioDescriptor, AurodioLayerDescriptor, BarVariant, Bars, BarsFormat,
        BarsPlacement, Chessy, ChessyDescriptor, Circle, CircleDescriptor, CircleVariant,
        Component, FragmentCanvas, FragmentCanvasDescriptor, Graph, GraphDescriptor,
        GraphPlacement, GraphVariant, Radial, RadialDescriptor, RadialFormat, RadialVariant,
        ShaderCode,
    },
    texture_generation::SdfPattern,
    Renderer,
};

pub use aurodio::{AurodioAudioConfig, AurodioLayerConfig};
pub use bars::{BarsAudioConfig, BarsFormatConfig, BarsPlacementConfig, BarsVariantConfig};
pub use chessy::ChessyAudioConfig;
pub use circle::{CircleAudioConfig, CircleVariantConfig};
pub use encrust_wallpaper::WallpaperPulseEdgeAudioConfig;
pub use fragment_canvas::FragmentCanvasAudioConfig;
pub use graph::{GraphAudioConfig, GraphFormatConfig, GraphPlacementConfig, GraphVariantConfig};
pub use radial::{RadialAudioConfig, RadialFormatConfig, RadialVariantConfig};

use crate::output::config::component::light_sources::LightSourcesError;

use {
    encrust_wallpaper::{WallpaperPulseEdgeGaussianBlur, WallpaperPulseEdgeThresholds},
    light_sources::{LightSourcesAudioConfig, LightSourcesSource},
};

const GAMMA: f32 = 2.2;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    ShaderCode(#[from] vibe_renderer::components::ShaderCodeError),

    #[error(transparent)]
    PulseError(#[from] vibe_renderer::components::live_wallpaper::pulse_edges::PulseEdgesError),

    #[error(transparent)]
    LightSource(#[from] light_sources::LightSourcesError),

    #[error("Couldn't open '{path}': {reason}")]
    OpenFile {
        path: String,
        reason: std::io::Error,
    },

    #[error(transparent)]
    Image(#[from] image::error::ImageError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("It looks like as if you've tried to access `iSampler` or `iTexture` in your shader code but you didn't set `texture_path` in the 'FragmentCanvas' config.")]
    MissingTexture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentConfig {
    Bars {
        audio_conf: BarsAudioConfig,
        max_height: f32,
        variant: BarsVariantConfig,
        placement: BarsPlacementConfig,
        format: BarsFormatConfig,
    },
    FragmentCanvas {
        audio_conf: FragmentCanvasAudioConfig,
        fragment_code: ShaderCode,

        texture_path: Option<PathBuf>,
    },
    Aurodio {
        base_color: Rgb,
        movement_speed: f32,
        audio_conf: AurodioAudioConfig,
        layers: Vec<AurodioLayerConfig>,
    },
    Graph {
        audio_conf: GraphAudioConfig,
        max_height: f32,
        variant: GraphVariantConfig,
        placement: GraphPlacementConfig,
        format: GraphFormatConfig,
    },
    Circle {
        audio_conf: CircleAudioConfig,
        variant: CircleVariantConfig,
        radius: f32,
        rotation: cgmath::Deg<f32>,
        position: (f32, f32),
    },
    Radial {
        audio_conf: RadialAudioConfig,
        variant: RadialVariantConfig,
        format: RadialFormatConfig,

        init_rotation: cgmath::Deg<f32>,
        circle_radius: f32,
        bar_height_sensitivity: f32,
        bar_width: f32,
        position: (f32, f32),
    },
    Chessy {
        movement_speed: f32,
        pattern: SdfPattern,
        zoom_factor: f32,
        audio_conf: ChessyAudioConfig,
    },
    WallpaperPulseEdges {
        wallpaper_path: PathBuf,
        audio_conf: WallpaperPulseEdgeAudioConfig,

        thresholds: WallpaperPulseEdgeThresholds,
        wallpaper_brightness: f32,
        edge_width: f32,
        pulse_brightness: f32,

        gaussian_blur: WallpaperPulseEdgeGaussianBlur,
    },
    WallpaperLightSources {
        wallpaper_path: PathBuf,

        audio_conf: LightSourcesAudioConfig,

        sources: Vec<LightSourcesSource>,
        uniform_pulse: bool,
        debug_sources: bool,
    },
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars {
            audio_conf: BarsAudioConfig {
                amount_bars: NonZero::new(60).unwrap(),
                freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                sensitivity: 4.0,
            },
            max_height: 0.75,
            variant: BarsVariantConfig::Color(Rgba::TURQUOISE),
            placement: BarsPlacementConfig::Bottom,
            format: BarsFormatConfig::BassTreble,
        }
    }
}

impl ComponentConfig {
    pub fn to_component<F: Fetcher>(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn Component>, ConfigError> {
        match self {
            Self::Bars {
                audio_conf,
                max_height,
                variant,
                placement,
                format,
            } => {
                let variant = match variant {
                    BarsVariantConfig::Color(rgba) => BarVariant::Color(rgba.as_f32()),
                    BarsVariantConfig::PresenceGradient {
                        high_presence,
                        low_presence,
                    } => BarVariant::PresenceGradient {
                        high: high_presence.as_f32(),
                        low: low_presence.as_f32(),
                    },
                    BarsVariantConfig::HorizontalGradient { left, right } => {
                        BarVariant::HorizontalGradient {
                            left: left.as_f32(),
                            right: right.as_f32(),
                        }
                    }
                    BarsVariantConfig::VerticalGradient { bottom, top } => {
                        BarVariant::VerticalGradient {
                            top: top.as_f32(),
                            bottom: bottom.as_f32(),
                        }
                    }
                    BarsVariantConfig::FragmentCode(code) => BarVariant::FragmentCode(code.clone()),
                };

                let bars = Bars::new(&vibe_renderer::components::BarsDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    max_height: *max_height,
                    variant,
                    placement: BarsPlacement::from(placement),
                    format: BarsFormat::from(format),
                })?;

                Ok(Box::new(bars) as Box<dyn Component>)
            }
            Self::FragmentCanvas {
                audio_conf,
                fragment_code,
                texture_path,
            } => {
                let img = match texture_path {
                    Some(path) => Some(
                        ImageReader::open(path)
                            .map_err(|err| ConfigError::OpenFile {
                                path: path.to_string_lossy().to_string(),
                                reason: err,
                            })?
                            .decode()?,
                    ),
                    None => None,
                };

                // Check: Is `texture_path` set if it's used in the shader-code?
                {
                    let code = fragment_code.source()?;
                    if (code.contains("iSampler") || code.contains("iTexture")) && img.is_none() {
                        return Err(ConfigError::MissingTexture);
                    }
                }

                let fragment_canvas = FragmentCanvas::new(&FragmentCanvasDescriptor {
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    renderer,
                    format: texture_format,
                    fragment_code: fragment_code.clone(),
                    img,
                })?;

                Ok(Box::new(fragment_canvas) as Box<dyn Component>)
            }
            Self::Aurodio {
                base_color,
                movement_speed,
                audio_conf,
                layers,
            } => {
                let layers: Vec<AurodioLayerDescriptor> = layers
                    .iter()
                    .map(|layer| AurodioLayerDescriptor {
                        freq_range: layer.freq_range.clone(),
                        zoom_factor: layer.zoom_factor,
                    })
                    .collect();

                Ok(Box::new(Aurodio::new(&AurodioDescriptor {
                    renderer,
                    sample_processor: processor,
                    texture_format,
                    base_color: base_color.as_f32(),
                    movement_speed: *movement_speed,
                    sensitivity: audio_conf.sensitivity,
                    layers: &layers,
                })) as Box<dyn Component>)
            }
            Self::Graph {
                audio_conf,
                max_height,
                variant,
                placement,
                format,
            } => {
                let variant = GraphVariant::from(variant);
                let placement = GraphPlacement::from(placement);

                Ok(Box::new(Graph::new(&GraphDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    max_height: *max_height,
                    placement,
                    format: format.into(),
                })) as Box<dyn Component>)
            }
            Self::Circle {
                audio_conf,
                variant,
                radius,
                rotation,
                position,
            } => {
                let variant = match variant {
                    CircleVariantConfig::Graph {
                        spike_sensitivity,
                        color,
                    } => CircleVariant::Graph {
                        spike_sensitivity: *spike_sensitivity,
                        color: color.as_f32(),
                    },
                };

                Ok(Box::new(Circle::new(&CircleDescriptor {
                    device: renderer.device(),
                    sample_processor: processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    texture_format,
                    variant,
                    radius: *radius,
                    rotation: *rotation,
                    position: *position,
                })))
            }
            Self::Radial {
                audio_conf,
                variant,
                init_rotation,
                circle_radius,
                bar_height_sensitivity,
                bar_width,
                position,
                format,
            } => {
                let variant = match variant {
                    RadialVariantConfig::Color(rgba) => RadialVariant::Color(rgba.as_f32()),
                    RadialVariantConfig::HeightGradient { inner, outer } => {
                        RadialVariant::HeightGradient {
                            inner: inner.as_f32(),
                            outer: outer.as_f32(),
                        }
                    }
                };

                Ok(Box::new(Radial::new(&RadialDescriptor {
                    device: renderer.device(),
                    processor,
                    audio_conf: vibe_audio::BarProcessorConfig::from(audio_conf),
                    output_texture_format: texture_format,
                    variant,
                    init_rotation: *init_rotation,
                    circle_radius: *circle_radius,
                    bar_height_sensitivity: *bar_height_sensitivity,
                    bar_width: *bar_width,
                    position: *position,
                    format: RadialFormat::from(format),
                })))
            }
            Self::Chessy {
                movement_speed,
                pattern,
                zoom_factor,
                audio_conf,
            } => Ok(Box::new(Chessy::new(&ChessyDescriptor {
                renderer,
                sample_processor: processor,
                audio_config: vibe_audio::BarProcessorConfig::from(audio_conf),
                texture_format,
                movement_speed: *movement_speed,
                pattern: *pattern,
                zoom_factor: *zoom_factor,
            }))),
            Self::WallpaperPulseEdges {
                wallpaper_path,
                audio_conf,
                thresholds,
                wallpaper_brightness,
                edge_width,
                pulse_brightness,

                gaussian_blur,
            } => {
                let img = ImageReader::open(wallpaper_path)
                    .map_err(|err| ConfigError::OpenFile {
                        path: wallpaper_path.to_string_lossy().to_string(),
                        reason: err,
                    })?
                    .decode()?;

                let high_threshold_ratio = thresholds.high.clamp(0.0, 1.0);
                let low_threshold_ratio = thresholds.low.min(high_threshold_ratio);

                let pulse_edges = live_wallpaper::pulse_edges::PulseEdges::new(
                    &live_wallpaper::pulse_edges::PulseEdgesDescriptor {
                        renderer,
                        sample_processor: processor,
                        texture_format,

                        img,
                        freq_range: audio_conf.freq_range.clone(),
                        audio_sensitivity: audio_conf.sensitivity,
                        high_threshold_ratio,
                        low_threshold_ratio,
                        wallpaper_brightness: *wallpaper_brightness,
                        edge_width: *edge_width,
                        pulse_brightness: *pulse_brightness,

                        sigma: gaussian_blur.sigma,
                        kernel_size: gaussian_blur.kernel_size,
                    },
                )?;

                Ok(Box::new(pulse_edges) as Box<dyn Component>)
            }
            Self::WallpaperLightSources {
                wallpaper_path,
                sources,
                audio_conf,
                uniform_pulse,
                debug_sources,
            } => {
                let img = ImageReader::open(wallpaper_path)
                    .map_err(|err| ConfigError::OpenFile {
                        path: wallpaper_path.to_string_lossy().to_string(),
                        reason: err,
                    })?
                    .decode()?;

                let sources = sources
                    .iter()
                    .map(LightSourceData::try_from)
                    .collect::<Result<Vec<LightSourceData>, LightSourcesError>>()?;

                let light_sources = LightSources::new(&LightSourcesDescriptor {
                    renderer,
                    format: texture_format,

                    processor,
                    freq_range: audio_conf.freq_range.clone(),
                    sensitivity: audio_conf.sensitivity,

                    wallpaper: img,
                    sources: &sources,
                    uniform_pulse: *uniform_pulse,
                    debug_sources: *debug_sources,
                });

                Ok(Box::new(light_sources) as Box<dyn Component>)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const TURQUOISE: Self = Self([0, 255, 255, 255]);

    pub fn as_f32(&self) -> [f32; 4] {
        let mut rgba_f32 = [0f32; 4];
        for (idx, value) in self.0.iter().enumerate() {
            rgba_f32[idx] = (*value as f32) / 255f32;
        }

        // apply gamma correction
        for value in rgba_f32[0..3].iter_mut() {
            *value = value.powf(GAMMA);
        }

        rgba_f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    pub fn as_f32(&self) -> [f32; 3] {
        let mut rgba_f32 = [0f32; 3];
        for (idx, value) in self.0.iter().enumerate() {
            rgba_f32[idx] = (*value as f32) / 255f32;
        }

        // apply gamma correction
        for value in rgba_f32[0..2].iter_mut() {
            *value = value.powf(GAMMA);
        }

        rgba_f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use vibe_audio::fetcher::DummyFetcher;
    use vibe_renderer::components::{ShaderLanguage, ShaderSource};

    #[test]
    fn fragment_canvas_wgsl_with_missing_texture_path() {
        let renderer = Renderer::default();
        let processor = SampleProcessor::new(DummyFetcher::new(1));

        let config = ComponentConfig::FragmentCanvas {
            audio_conf: FragmentCanvasAudioConfig {
                amount_bars: NonZero::new(10).unwrap(),
                freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                sensitivity: 4.0,
            },
            fragment_code: ShaderCode {
                language: ShaderLanguage::Wgsl,
                source: ShaderSource::Code("@fragment\nfn main(@builtin(position) pos: vec4f) -> @location(0) { return textureSample(iTexture, iSampler, pos.xy/iResolution.xy); }".to_string()),
            },
            texture_path: None,
        };

        let err = config
            .to_component(&renderer, &processor, wgpu::TextureFormat::Rgba8Unorm)
            .err()
            .unwrap();

        match err {
            ConfigError::MissingTexture => {}
            _ => unreachable!("Weird: {}", err),
        }
    }

    #[test]
    fn fragment_canvas_glsl_with_missing_texture_path() {
        let renderer = Renderer::default();
        let processor = SampleProcessor::new(DummyFetcher::new(1));

        let config = ComponentConfig::FragmentCanvas {
            audio_conf: FragmentCanvasAudioConfig {
                amount_bars: NonZero::new(10).unwrap(),
                freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                sensitivity: 4.0,
            },
            fragment_code: ShaderCode {
                language: ShaderLanguage::Glsl,
                source: ShaderSource::Code("void main() { fragColor = texture(sampler2D(iTexture, iSampler), gl_FragCoord.xy/iResolution.xy); }".to_string()),
            },
            texture_path: None,
        };

        let err = config
            .to_component(&renderer, &processor, wgpu::TextureFormat::Rgba8Unorm)
            .err()
            .unwrap();

        match err {
            ConfigError::MissingTexture => {}
            _ => unreachable!("Weird: {}", err),
        }
    }
}
