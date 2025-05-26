use std::io;

use serde::{Deserialize, Serialize};
use shady_audio::{
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    SampleProcessor,
};
use vibe_renderer::RendererDescriptor;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("The config file format is invalid: {0}")]
    Serde(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub power_preference: wgpu::PowerPreference,
    pub backend: wgpu::Backends,
    pub gpu_name: Option<String>,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::LowPower,
            backend: wgpu::Backends::VULKAN,
            gpu_name: None,
        }
    }
}

impl From<GraphicsConfig> for RendererDescriptor {
    fn from(conf: GraphicsConfig) -> Self {
        Self {
            power_preference: conf.power_preference,
            backend: conf.backend,
            fallback_to_software_rendering: false,
            adapter_name: conf.gpu_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    pub output_device_name: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub graphics_config: GraphicsConfig,
    pub audio_config: Option<AudioConfig>,
}

impl Config {
    pub fn save(&self) -> io::Result<()> {
        std::fs::write(crate::get_config_path(), toml::to_string(self).unwrap())
    }

    pub fn sample_processor(&self) -> anyhow::Result<SampleProcessor> {
        let device = match self
            .audio_config
            .clone() // honestly... this doesn't hurt
            .unwrap_or_default()
            .output_device_name
        {
            Some(device_name) => {
                match shady_audio::util::get_device(&device_name, DeviceType::Output)? {
                    Some(device) => device,
                    None => {
                        anyhow::bail!(
                            concat![
                                "\n== Available output devices ==\n{:#?}\n",
                                "\nThere's no output device called \"{}\" as you've set in \"{}\"\n",
                                "Please choose one from the list and add it to your config."
                            ],
                            shady_audio::util::get_device_names(DeviceType::Output)?,
                            &device_name,
                            crate::get_config_path().to_string_lossy()
                        );
                    }
                }
            }
            None => match shady_audio::util::get_default_device(DeviceType::Output) {
                Some(device) => device,
                None => {
                    anyhow::bail!(
                        concat![
                            "\n== Available output devices ==\n{:#?}\n",
                            "\nCouldn't find the default output device on your system.\n",
                            "Please choose one from the list and add it to your config in \"{}\"."
                        ],
                        shady_audio::util::get_device_names(DeviceType::Output)?,
                        crate::get_config_path().to_string_lossy()
                    );
                }
            },
        };

        let system_audio_fetcher = SystemAudioFetcher::new(&SystemAudioFetcherDescriptor {
            device,
            ..Default::default()
        })?;

        Ok(SampleProcessor::new(system_audio_fetcher))
    }
}

pub fn load() -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(crate::get_config_path())?;
    toml::from_str(&content).map_err(|err| err.into())
}
