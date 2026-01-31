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
    pub fn new(info: &OutputInfo, default_component: ComponentConfig) -> anyhow::Result<Self> {
        let name = info.name.as_ref().unwrap();

        let new = Self {
            enable: true,
            components: vec![default_component],
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

    /// Returns all relevant paths which are occurring inside the config file.
    ///
    /// Only relevant for hot reloading to know which other files have to be watched as
    /// well.
    pub fn external_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for component in self.components.iter() {
            match component {
                ComponentConfig::FragmentCanvas {
                    fragment_code,
                    texture,
                    ..
                } => {
                    if let vibe_renderer::components::ShaderSource::Path(path) =
                        &fragment_code.source
                    {
                        paths.push(path.clone());
                    }

                    if let Some(t) = texture {
                        paths.push(t.path.clone());
                    }
                }
                ComponentConfig::WallpaperPulseEdges { wallpaper_path, .. } => {
                    paths.push(wallpaper_path.clone());
                }
                ComponentConfig::WallpaperLightSources { wallpaper_path, .. } => {
                    paths.push(wallpaper_path.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::config::component::{
        FragmentCanvasTexture, WallpaperPulseEdgeAudioConfig, WallpaperPulseEdgeGaussianBlur,
        WallpaperPulseEdgeThresholds,
    };
    use std::{collections::HashSet, num::NonZero};
    use vibe_renderer::components::{ShaderCode, ShaderLanguage, ShaderSource};

    #[test]
    fn external_paths() {
        let output_config = OutputConfig {
            enable: true,
            components: vec![
                ComponentConfig::FragmentCanvas {
                    audio_conf: component::FragmentCanvasAudioConfig::default(),
                    texture: Some(FragmentCanvasTexture {
                        path: "/dir/fragment_canvas_img.png".into(),
                    }),
                    fragment_code: ShaderCode {
                        language: ShaderLanguage::Wgsl,
                        source: ShaderSource::Path("/dir/fragment_canvas_code.wgsl".into()),
                    },
                },
                ComponentConfig::WallpaperPulseEdges {
                    wallpaper_path: "/tmp/wallpaper_palse_edges.png".into(),
                    audio_conf: WallpaperPulseEdgeAudioConfig {
                        sensitivity: 4.,
                        freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                    },
                    thresholds: WallpaperPulseEdgeThresholds {
                        high: 0.2,
                        low: 0.8,
                    },
                    wallpaper_brightness: 0.5,
                    edge_width: 0.2,
                    pulse_brightness: 0.5,
                    gaussian_blur: WallpaperPulseEdgeGaussianBlur {
                        sigma: 0.5,
                        kernel_size: 3,
                    },
                },
                ComponentConfig::WallpaperLightSources {
                    wallpaper_path: "/tmp/wallpaper_light_sources.png".into(),
                    audio_conf: component::LightSourcesAudioConfig {
                        freq_range: NonZero::new(50).unwrap()..NonZero::new(10_000).unwrap(),
                        sensitivity: 0.5,
                    },
                    sources: vec![],
                    uniform_pulse: true,
                    debug_sources: false,
                },
            ],
        };

        let expected = HashSet::from([
            "/dir/fragment_canvas_img.png".into(),
            "/dir/fragment_canvas_code.wgsl".into(),
            "/tmp/wallpaper_palse_edges.png".into(),
            "/tmp/wallpaper_light_sources.png".into(),
        ]);

        let current: HashSet<PathBuf> = output_config.external_paths().into_iter().collect();

        assert_eq!(expected, current);
    }

    #[test]
    fn accept_reference_config() -> Result<(), toml::de::Error> {
        let reference_config = include_str!("./reference-config.toml");

        if let Err(err) = toml::from_str::<OutputConfig>(reference_config).context("") {
            panic!("{:?}", err);
        }

        Ok(())
    }
}
