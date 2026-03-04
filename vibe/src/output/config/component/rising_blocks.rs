use crate::output::config::component::ComponentConfig;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{RisingBlocks, RisingBlocksDescriptor, RisingBlocksEasing};

#[derive(thiserror::Error, Debug)]
pub enum RisingBlocksConfigError {
    #[error("The speed value {0} is invalid. Speed must be > 0!")]
    InvalidSpeed(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisingBlocksConfig {
    pub canvas_height: Option<f32>,
    pub spawn_random: Option<bool>,

    // > 1.0 => faster
    // < 1.0 => slower
    pub speed: Option<f32>,

    pub easing: Option<RisingBlockConfigEasing>,
}

impl ComponentConfig for RisingBlocksConfig {
    fn create_component<F: vibe_audio::fetcher::Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        if let Some(speed) = self.speed {
            if speed <= 0f32 {
                return Err(RisingBlocksConfigError::InvalidSpeed(speed).into());
            }
        }

        Ok(Box::new(RisingBlocks::new(&RisingBlocksDescriptor {
            renderer,
            sample_processor: processor,
            format: texture_format,
            audio_conf: BarProcessorConfig {
                amount_bars: NonZero::new(30).unwrap(),
                down: 5.0,
                correction_offset: 0.075,
                freq_range: NonZero::new(50).unwrap()..NonZero::new(5_000).unwrap(),
                ..Default::default()
            },

            canvas_height: self.canvas_height.unwrap_or(1.0),
            spawn_random: self.spawn_random.unwrap_or(false),
            speed: self.speed.unwrap_or(1f32),
            easing: self.easing.map(|conf| conf.into()),
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RisingBlockConfigEasing {
    InSine,
    OutSine,
    InOutSine,
}

impl From<RisingBlockConfigEasing> for RisingBlocksEasing {
    fn from(conf: RisingBlockConfigEasing) -> Self {
        match conf {
            RisingBlockConfigEasing::InSine => Self::InSine,
            RisingBlockConfigEasing::OutSine => Self::OutSine,
            RisingBlockConfigEasing::InOutSine => Self::InOutSine,
        }
    }
}
