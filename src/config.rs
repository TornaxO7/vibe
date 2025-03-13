use std::io;

use serde::{Deserialize, Serialize};
use vibe_daemon::renderer::GraphicsConfig;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("The config file format is invalid: {0}")]
    Serde(#[from] toml::de::Error),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub graphics_config: GraphicsConfig,
}

impl Config {
    pub fn save(&self) -> io::Result<()> {
        std::fs::write(
            vibe_daemon::get_config_path(),
            toml::to_string(self).unwrap(),
        )
    }
}

pub fn load() -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(vibe_daemon::get_config_path())?;
    toml::from_str(&content).map_err(|err| err.into())
}
