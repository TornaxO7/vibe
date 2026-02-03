use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{io, str::FromStr};
use vibe_audio::{
    cpal::DeviceId,
    fetcher::{SystemAudioFetcher, SystemAudioFetcherDescriptor},
    util::DeviceType,
    SampleProcessor,
};
use vibe_renderer::RendererDescriptor;

use crate::output::config::component;

const STEREO_AUDIO: u16 = 2;

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

impl From<&GraphicsConfig> for RendererDescriptor {
    fn from(conf: &GraphicsConfig) -> Self {
        Self {
            power_preference: conf.power_preference,
            backend: conf.backend,
            fallback_to_software_rendering: false,
            adapter_name: conf.gpu_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    pub output_device_id: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub graphics_config: GraphicsConfig,
    pub audio_config: Option<AudioConfig>,
    pub default_component: Option<component::Config>,
}

impl Config {
    pub fn save(&self) -> io::Result<()> {
        std::fs::write(crate::get_config_path(), toml::to_string(self).unwrap())
    }

    pub fn sample_processor(&self) -> anyhow::Result<SampleProcessor<SystemAudioFetcher>> {
        let device = match self
            .audio_config
            .clone()
            .unwrap_or_default()
            .output_device_id
        {
            Some(output_device_id) => {
                let device_id = DeviceId::from_str(&output_device_id).map_err(|err| {
                    anyhow!(
                        "Couldn't parse the device id from your config file (in '{}'):\n{}",
                        crate::get_config_path().to_string_lossy(),
                        err
                    )
                })?;

                match vibe_audio::util::get_device(device_id, DeviceType::Input)? {
                    Some(device) => device,
                    None => {
                        anyhow::bail!(
                        concat![
                            "Available output devices:\n\n{:#?}\n",
                            "\nThere's no output device with the id \"{}\" as you've set in \"{}\"\n",
                            "Please choose one from the list and add it to your config."
                        ],
                        vibe_audio::util::get_device_ids(DeviceType::Input)?,
                        &output_device_id,
                        crate::get_config_path().to_string_lossy()
                    );
                    }
                }
            }
            None => match vibe_audio::util::get_default_device(DeviceType::Input) {
                Some(device) => device,
                None => {
                    anyhow::bail!(
                        concat![
                            "Available output devices:\n\n{:#?}\n",
                            "\nCouldn't find the default output device on your system.\n",
                            "Please choose one from the list and add it to your config in \"{}\"."
                        ],
                        vibe_audio::util::get_device_ids(DeviceType::Input)?,
                        crate::get_config_path().to_string_lossy()
                    );
                }
            },
        };

        let system_audio_fetcher = SystemAudioFetcher::new(&SystemAudioFetcherDescriptor {
            device,
            amount_channels: Some(STEREO_AUDIO),
            ..Default::default()
        })?;

        Ok(SampleProcessor::new(system_audio_fetcher))
    }
}

pub fn load() -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(crate::get_config_path())?;
    toml::from_str(&content).map_err(|err| err.into())
}
