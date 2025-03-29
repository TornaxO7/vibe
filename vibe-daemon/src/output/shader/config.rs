use std::{
    num::{NonZero, NonZeroUsize},
    ops::Range,
};

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
    pub frequency_range: Range<NonZero<u16>>,
}

impl Default for AudioConf {
    fn default() -> Self {
        Self {
            amount_bars: NonZeroUsize::new(60).unwrap(),
            frequency_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
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
