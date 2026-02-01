mod aurodio;
mod bars;
mod chessy;
mod circle;
mod encrust_wallpaper;
mod fragment_canvas;
mod graph;
mod light_sources;
mod radial;

use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range};
use vibe_audio::{fetcher::Fetcher, SampleProcessor};
use vibe_renderer::{components::Component, Renderer};

pub use aurodio::*;
pub use bars::*;
pub use chessy::*;
pub use circle::*;
pub use encrust_wallpaper::*;
pub use fragment_canvas::*;
pub use graph::*;
pub use light_sources::*;
pub use radial::*;

const GAMMA: f32 = 2.2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FreqRange {
    Bass,
    Mid,
    Treble,
    Custom(Range<NonZero<u16>>),
}

impl FreqRange {
    pub fn range(&self) -> Range<NonZero<u16>> {
        match self {
            Self::Bass => NonZero::new(20).unwrap()..NonZero::new(150).unwrap(),
            Self::Mid => NonZero::new(500).unwrap()..NonZero::new(2_000).unwrap(),
            Self::Treble => NonZero::new(6_000).unwrap()..NonZero::new(20_000).unwrap(),
            Self::Custom(range) => range.clone(),
        }
    }
}

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

trait ToComponent<F: Fetcher> {
    fn to_component(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn Component>, ConfigError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentConfig {
    Bars(BarsConfig),
    FragmentCanvas(FragmentCanvasConfig),
    Aurodio(AurodioConfig),
    Graph(GraphConfig),
    Circle(CircleConfig),
    Radial(RadialConfig),
    Chessy(ChessyConfig),
    WallpaperPulseEdges(WallpaperPulseEdgesConfig),
    WallpaperLightSources(LightSourcesConfig),
}

impl Default for ComponentConfig {
    fn default() -> Self {
        Self::Bars(BarsConfig::default())
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
            Self::Bars(config) => config.to_component(renderer, processor, texture_format),
            Self::FragmentCanvas(config) => {
                config.to_component(renderer, processor, texture_format)
            }
            Self::Aurodio(config) => config.to_component(renderer, processor, texture_format),
            Self::Graph(config) => config.to_component(renderer, processor, texture_format),
            Self::Circle(circle_config) => {
                circle_config.to_component(renderer, processor, texture_format)
            }
            Self::Radial(config) => config.to_component(renderer, processor, texture_format),
            Self::Chessy(chessy_config) => {
                chessy_config.to_component(renderer, processor, texture_format)
            }
            Self::WallpaperPulseEdges(config) => {
                config.to_component(renderer, processor, texture_format)
            }
            Self::WallpaperLightSources(config) => {
                config.to_component(renderer, processor, texture_format)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const TURQUOISE: Self = Self([0, 255, 255, 255]);

    pub fn as_f32(&self) -> vibe_renderer::components::Rgba {
        let mut rgba_f32 = [0f32; 4];
        for (idx, value) in self.0.iter().enumerate() {
            rgba_f32[idx] = (*value as f32) / 255f32;
        }

        // apply gamma correction
        for value in rgba_f32[0..3].iter_mut() {
            *value = value.powf(GAMMA);
        }

        rgba_f32.into()
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

    mod fragment_canvas_texture_path {
        use vibe_renderer::components::ShaderCode;

        use super::*;

        #[test]
        fn fragment_canvas_wgsl_with_missing_texture_path() {
            let renderer = Renderer::default();
            let processor = SampleProcessor::new(DummyFetcher::new(1));

            let config = ComponentConfig::FragmentCanvas(FragmentCanvasConfig {
            audio_conf: FragmentCanvasAudioConfig {
                amount_bars: NonZero::new(10).unwrap(),
                freq_range: FreqRange::Custom(NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap()),
                sensitivity: 4.0,
            },
            fragment_code: ShaderCode {
                language: ShaderLanguage::Wgsl,
                source: ShaderSource::Code("@fragment\nfn main(@builtin(position) pos: vec4f) -> @location(0) { return textureSample(iTexture, iSampler, pos.xy/iResolution.xy); }".to_string()),
            },
            texture: None,
        });

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

            let config = ComponentConfig::FragmentCanvas (FragmentCanvasConfig{
            audio_conf: FragmentCanvasAudioConfig {
                amount_bars: NonZero::new(10).unwrap(),
                freq_range: FreqRange::Custom(NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap()),
                sensitivity: 4.0,
            },
            fragment_code: ShaderCode {
                language: ShaderLanguage::Glsl,
                source: ShaderSource::Code("void main() { fragColor = texture(sampler2D(iTexture, iSampler), gl_FragCoord.xy/iResolution.xy); }".to_string()),
            },
            texture: None,
        });

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
}
