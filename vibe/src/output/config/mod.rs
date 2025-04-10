pub mod component;

use std::{ffi::OsStr, io, path::PathBuf};

use anyhow::Context;
use component::ComponentConfig;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::output::OutputInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub enable: bool,
    pub components: Vec<ComponentConfig>,
}

impl OutputConfig {
    pub fn new(info: &OutputInfo) -> anyhow::Result<Self> {
        let name = info.name.as_ref().unwrap();

        let new = Self {
            enable: true,
            components: vec![ComponentConfig::default()],
        };

        new.save(name)?;
        Ok(new)
    }

    pub fn save(&self, name: impl AsRef<str>) -> io::Result<()> {
        let string = toml::to_string(self).unwrap();
        let save_path = {
            let mut path = crate::get_output_config_dir();
            path.push(format!("{}.toml", name.as_ref()));
            path
        };

        std::fs::write(save_path, string)?;

        Ok(())
    }

    // Return all paths to relevant files for hot reloading.
    pub fn hot_reloading_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for component in self.components.iter() {
            match component {
                ComponentConfig::Bars { variant, .. } => match variant {
                    component::BarVariantConfig::FragmentCode(code) => match &code.source {
                        vibe_renderer::components::ShaderSource::Path(path) => {
                            paths.push(path.clone());
                        }
                        _ => {}
                    },
                    _ => {}
                },
                ComponentConfig::FragmentCanvas { fragment_code, .. } => {
                    match &fragment_code.source {
                        vibe_renderer::components::ShaderSource::Path(path) => {
                            paths.push(path.clone());
                        }
                        _ => {}
                    }
                }
                _ => {}
            };
        }

        paths
    }
}

pub fn load<S: AsRef<str>>(output_name: S) -> Option<(PathBuf, anyhow::Result<OutputConfig>)> {
    let iterator = std::fs::read_dir(crate::get_output_config_dir()).unwrap();

    for entry in iterator {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.file_stem().unwrap() == OsStr::new(output_name.as_ref()) {
            let content = std::fs::read_to_string(&path).unwrap();
            return Some((path, toml::from_str(&content).context("")));
        }
    }

    None
}
