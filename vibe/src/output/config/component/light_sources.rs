use serde::{Deserialize, Serialize};
use std::{num::NonZero, ops::Range};
use vibe_renderer::components::live_wallpaper::light_sources::LightSourceData;

#[derive(thiserror::Error, Debug)]
pub enum LightSourcesError {
    // If radius is <= 0.
    #[error("Light source with center {center:?} must be > 0")]
    InvalidRadius { center: [f32; 2] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourcesAudioConfig {
    pub freq_range: Range<NonZero<u16>>,
    pub sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSourcesSource {
    pub center: [f32; 2],
    pub radius: f32,
}

impl<'a> TryFrom<&'a LightSourcesSource> for LightSourceData {
    type Error = LightSourcesError;

    fn try_from(source: &'a LightSourcesSource) -> Result<Self, Self::Error> {
        if source.radius <= 0. {
            return Err(LightSourcesError::InvalidRadius {
                center: source.center,
            });
        }

        Ok(Self {
            center: source.center,
            // invert the radius because the user expects: The higher the value => the higher the radius
            radius: 1. / source.radius,
        })
    }
}
