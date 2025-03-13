use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

type Code = String;
type DirName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderCode {
    Glsl(Code),
    Wgsl(Code),
    VibeShader(DirName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConf {
    pub amount_bars: NonZeroUsize,
}

impl Default for AudioConf {
    fn default() -> Self {
        Self {
            amount_bars: NonZeroUsize::new(60).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderConf {
    pub audio: AudioConf,
    pub code: ShaderCode,
}

impl Default for ShaderConf {
    fn default() -> Self {
        Self {
            audio: AudioConf::default(),
            code: ShaderCode::Glsl(include_str!("../default_shaders/default.glsl").to_string()),
        }
    }
}

impl AsRef<ShaderConf> for ShaderConf {
    fn as_ref(&self) -> &ShaderConf {
        self
    }
}
