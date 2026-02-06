mod aurodio;
mod bars;
mod chessy;
mod circle;
mod encrust_wallpaper;
mod fragment_canvas;
mod graph;
mod light_sources;
mod radial;
mod util;

use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range, path::PathBuf};
use util::Rgba;
use vibe_audio::{fetcher::Fetcher, SampleProcessor};
use vibe_renderer::{components::ComponentAudio, Renderer};

pub use aurodio::*;
pub use bars::*;
pub use chessy::*;
pub use circle::*;
pub use encrust_wallpaper::*;
pub use fragment_canvas::*;
pub use graph::*;
pub use light_sources::*;
pub use radial::*;

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

    #[error(transparent)]
    ColorFormat(#[from] util::ColorFormatError),
}

pub trait ComponentConfig {
    /// Creates a component with the given config.
    fn create_component<F: Fetcher>(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn ComponentAudio<F>>, ConfigError>;

    /// Returns a `vec` of paths which are stored in this component config.
    ///
    /// # Example
    /// Each live-wallpaper config needs a path to an image which will be used as a base
    /// to apply the effect on it.
    /// The returned vector of this function will include this path to this image.
    fn external_paths(&self) -> Vec<PathBuf>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Config {
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

impl Default for Config {
    fn default() -> Self {
        Self::Bars(BarsConfig::default())
    }
}

impl ComponentConfig for Config {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn ComponentAudio<F>>, ConfigError> {
        match self {
            Self::Bars(config) => config.create_component(renderer, processor, texture_format),
            Self::FragmentCanvas(config) => {
                config.create_component(renderer, processor, texture_format)
            }
            Self::Aurodio(config) => config.create_component(renderer, processor, texture_format),
            Self::Graph(config) => config.create_component(renderer, processor, texture_format),
            Self::Circle(circle_config) => {
                circle_config.create_component(renderer, processor, texture_format)
            }
            Self::Radial(config) => config.create_component(renderer, processor, texture_format),
            Self::Chessy(chessy_config) => {
                chessy_config.create_component(renderer, processor, texture_format)
            }
            Self::WallpaperPulseEdges(config) => {
                config.create_component(renderer, processor, texture_format)
            }
            Self::WallpaperLightSources(config) => {
                config.create_component(renderer, processor, texture_format)
            }
        }
    }

    fn external_paths(&self) -> Vec<PathBuf> {
        match self {
            Config::Bars(config) => config.external_paths(),
            Config::FragmentCanvas(config) => config.external_paths(),
            Config::Aurodio(config) => config.external_paths(),
            Config::Graph(config) => config.external_paths(),
            Config::Circle(config) => config.external_paths(),
            Config::Radial(config) => config.external_paths(),
            Config::Chessy(config) => config.external_paths(),
            Config::WallpaperPulseEdges(config) => config.external_paths(),
            Config::WallpaperLightSources(config) => config.external_paths(),
        }
    }
}
