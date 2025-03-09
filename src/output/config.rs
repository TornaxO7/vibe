use anyhow::{anyhow, Context};
use std::{ffi::OsStr, io, num::NonZeroUsize};

use serde::{Deserialize, Serialize};
use shady::TemplateLang;
use smithay_client_toolkit::output::OutputInfo;
use wgpu::naga::{front::glsl, Module, ShaderStage};

type Code = String;
type DirName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderCode {
    Glsl(Code),
    Wgsl(Code),
    VibeShader(DirName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub enable: bool,
    pub amount_bars: NonZeroUsize,
    pub shader_code: Vec<ShaderCode>,
}

impl OutputConfig {
    pub fn new(info: &OutputInfo) -> anyhow::Result<Self> {
        let name = info.name.as_ref().unwrap();

        let new = Self {
            enable: true,
            amount_bars: crate::DEFAULT_AMOUNT_BARS,
            shader_code: vec![ShaderCode::Glsl(
                TemplateLang::Glsl
                    .generate_to_string(Some(include_str!("./shaders/default.glsl")))
                    .unwrap(),
            )],
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

pub fn load(output_info: &OutputInfo) -> Option<OutputConfig> {
    let name = output_info.name.as_ref().unwrap();
    let iterator = std::fs::read_dir(vibe_daemon::get_output_config_dir()).unwrap();

    for entry in iterator {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.file_stem().unwrap() == OsStr::new(&name) {
            let content = std::fs::read_to_string(&path).unwrap();

            return Some(toml::from_str(&content).unwrap());
        }
    }

    None
}
