use std::{
    io,
    num::{NonZeroU32, NonZeroUsize},
    ops::Range,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

pub const DIR_NAME: &str = "output_configs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderCode {
    Wgsl(String),
    Glsl(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub shader_code: ShaderCode,

    /// The amount of bars
    pub amount_bars: NonZeroUsize,

    /// The frequency range which should be used
    pub frequency_range: Range<NonZeroU32>,
}

impl OutputConfig {
    pub fn save(&self, wl_output_name: impl AsRef<str>) -> io::Result<()> {
        let file_path = get_file_path(wl_output_name);
        std::fs::write(file_path, toml::to_string(self).unwrap())
    }

    pub fn load(wl_output_name: impl AsRef<str>) -> io::Result<Self> {
        let file_path = get_file_path(wl_output_name);
        let content = std::fs::read_to_string(file_path)?;
        Ok(toml::from_str(&content).unwrap())
    }
}

pub fn load(output_name: impl AsRef<str>) -> io::Result<Option<OutputConfig>> {
    let config_dir = std::fs::read_dir(crate::config_directory())?;

    for entry in config_dir {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_stem() == Some(output_name.as_ref().as_ref()) {
            let content = std::fs::read_to_string(path)?;

            return Ok(toml::from_str(&content).unwrap());
        }
    }

    Ok(None)
}

pub fn load_all() -> io::Result<Vec<OutputConfig>> {
    let mut configs = Vec::new();
    let config_dir = std::fs::read_dir(crate::config_directory())?;

    for entry in config_dir {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            let config = toml::from_str(&content).unwrap();

            configs.push(config);
        }
    }

    Ok(configs)
}

fn get_file_path(wl_output_name: impl AsRef<str>) -> PathBuf {
    let mut file_path = crate::config_directory();
    file_path.push(format!("{}.toml", wl_output_name.as_ref()));
    file_path
}
