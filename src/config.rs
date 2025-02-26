use std::io;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::gpu::GraphicsConfig;

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

pub fn load() -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(vibe_daemon::get_config_path())?;
    toml::from_str(&content).context("Your config file is invalid!")
}
