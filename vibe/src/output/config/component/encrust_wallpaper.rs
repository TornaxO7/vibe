use std::path::PathBuf;

use crate::output::config::component::ComponentConfig;

use super::FreqRange;
use image::ImageReader;
use serde::{Deserialize, Serialize};
use vibe_audio::fetcher::Fetcher;
use vibe_renderer::components::live_wallpaper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgesConfig {
    pub wallpaper_path: PathBuf,
    pub audio_conf: WallpaperPulseEdgesAudioConfig,

    pub thresholds: WallpaperPulseEdgesThresholds,
    pub wallpaper_brightness: f32,
    pub edge_width: f32,
    pub pulse_brightness: f32,

    pub gaussian_blur: WallpaperPulseEdgesGaussianBlur,
}

impl ComponentConfig for WallpaperPulseEdgesConfig {
    fn create_component<F: Fetcher>(
        &self,
        renderer: &vibe_renderer::Renderer,
        processor: &vibe_audio::SampleProcessor<F>,
        texture_format: wgpu::TextureFormat,
    ) -> Result<Box<dyn vibe_renderer::Component>, super::ConfigError> {
        let img = ImageReader::open(&self.wallpaper_path)
            .map_err(|err| super::ConfigError::OpenFile {
                path: self.wallpaper_path.to_string_lossy().to_string(),
                reason: err,
            })?
            .decode()?;

        let high_threshold_ratio = self.thresholds.high.clamp(0.0, 1.0);
        let low_threshold_ratio = self.thresholds.low.min(high_threshold_ratio);

        let pulse_edges = live_wallpaper::pulse_edges::PulseEdges::new(
            &live_wallpaper::pulse_edges::PulseEdgesDescriptor {
                renderer,
                sample_processor: processor,
                texture_format,

                img,
                freq_range: self.audio_conf.freq_range.range(),
                audio_sensitivity: self.audio_conf.sensitivity,
                high_threshold_ratio,
                low_threshold_ratio,
                wallpaper_brightness: self.wallpaper_brightness,
                edge_width: self.edge_width,
                pulse_brightness: self.pulse_brightness,

                sigma: self.gaussian_blur.sigma,
                kernel_size: self.gaussian_blur.kernel_size,
            },
        )?;

        Ok(Box::new(pulse_edges))
    }

    fn external_paths(&self) -> Vec<PathBuf> {
        vec![self.wallpaper_path.clone()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgesAudioConfig {
    pub sensitivity: f32,
    pub freq_range: FreqRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgesThresholds {
    pub high: f32,
    pub low: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperPulseEdgesGaussianBlur {
    pub sigma: f32,
    pub kernel_size: usize,
}
