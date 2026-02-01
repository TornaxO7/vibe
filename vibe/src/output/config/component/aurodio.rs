use super::{ComponentConfig, FreqRange, Rgb};
use serde::{Deserialize, Serialize};
use vibe_audio::{fetcher::Fetcher, SampleProcessor};
use vibe_renderer::{
    components::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor},
    Renderer,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioConfig {
    pub base_color: Rgb,
    pub movement_speed: f32,
    pub audio_conf: AurodioAudioConfig,
    pub layers: Vec<AurodioLayerConfig>,
}

impl<F: Fetcher> ComponentConfig<F> for AurodioConfig {
    fn create_component(
        &self,
        renderer: &Renderer,
        processor: &SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::Component>, super::ConfigError> {
        let layers: Vec<AurodioLayerDescriptor> = self
            .layers
            .iter()
            .map(|layer| AurodioLayerDescriptor {
                freq_range: layer.freq_range.range(),
                zoom_factor: layer.zoom_factor,
            })
            .collect();

        Ok(Box::new(Aurodio::new(&AurodioDescriptor {
            renderer,
            sample_processor: processor,
            texture_format,
            base_color: self.base_color.as_f32().into(),
            movement_speed: self.movement_speed,
            sensitivity: self.audio_conf.sensitivity,
            layers: &layers,
        })))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioAudioConfig {
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurodioLayerConfig {
    pub freq_range: FreqRange,
    pub zoom_factor: f32,
}
