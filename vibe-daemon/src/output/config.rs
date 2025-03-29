use std::{ffi::OsStr, io};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::output::OutputInfo;

use super::shader::config::ShaderConf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub enable: bool,
    pub shaders: Vec<ShaderConf>,
}

impl OutputConfig {
    pub fn new(info: &OutputInfo) -> anyhow::Result<Self> {
        let name = info.name.as_ref().unwrap();

        let new = Self {
            enable: true,
            shaders: vec![ShaderConf::default()],
        };

        new.save(name)?;
        Ok(new)
    }

    pub fn save(&self, name: impl AsRef<str>) -> io::Result<()> {
        let string = toml::to_string(self).unwrap();
        let save_path = {
            let mut path = vibe_daemon::get_output_config_dir();
            path.push(format!("{}.toml", name.as_ref()));
            path
        };

        std::fs::write(save_path, string)?;

        Ok(())
    }
}

pub fn load(output_info: &OutputInfo) -> Option<anyhow::Result<OutputConfig>> {
    let name = output_info.name.as_ref().unwrap();
    let iterator = std::fs::read_dir(vibe_daemon::get_output_config_dir()).unwrap();

    for entry in iterator {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.file_stem().unwrap() == OsStr::new(&name) {
            let content = std::fs::read_to_string(&path).unwrap();

            return Some(toml::from_str(&content).context(""));
        }
    }

    None
}
