use std::num::NonZero;

use crate::output::config::component::ComponentConfig;
use serde::{Deserialize, Serialize};
use vibe_audio::BarProcessorConfig;
use vibe_renderer::components::{RisingBlocks, RisingBlocksDescriptor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisingBlocksConfig {}

impl ComponentConfig for RisingBlocksConfig {
    fn create_component<F: vibe_audio::fetcher::Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::ComponentAudio<F>>, super::ConfigError> {
        Ok(Box::new(RisingBlocks::new(&RisingBlocksDescriptor {
            renderer,
            sample_processor: processor,
            audio_conf: BarProcessorConfig {
                amount_bars: NonZero::new(5).unwrap(),
                down: 5.0,
                freq_range: NonZero::new(50).unwrap()..NonZero::new(5_000).unwrap(),
                ..Default::default()
            },
            format: texture_format,
            threshold: 0.5,
        })))
    }

    fn external_paths(&self) -> Vec<std::path::PathBuf> {
        vec![]
    }
}
