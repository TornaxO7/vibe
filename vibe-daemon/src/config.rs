use std::io;

use serde::{Deserialize, Serialize};
use vibe_renderer::RendererDescriptor;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("The config file format is invalid: {0}")]
    Serde(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub power_preference: wgpu::PowerPreference,
    pub backend: wgpu::Backends,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::LowPower,
            backend: wgpu::Backends::VULKAN,
        }
    }
}

impl From<GraphicsConfig> for RendererDescriptor {
    fn from(conf: GraphicsConfig) -> Self {
        Self {
            power_preference: conf.power_preference,
            backend: conf.backend,
            fallback_to_software_rendering: false,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub graphics_config: GraphicsConfig,
}

impl Config {
    pub fn save(&self) -> io::Result<()> {
        std::fs::write(crate::get_config_path(), toml::to_string(self).unwrap())
    }
}

pub fn load() -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(crate::get_config_path())?;
    toml::from_str(&content).map_err(|err| err.into())
}
