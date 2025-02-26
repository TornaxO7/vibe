use serde::{Deserialize, Serialize};
use wgpu::{Backends, PowerPreference};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// Decide which kind of gpu should be used.
    ///
    /// See <https://docs.rs/wgpu/latest/wgpu/enum.PowerPreference.html#variants>
    /// for the available options
    pub power_preference: PowerPreference,

    /// Set the backend which should be used.
    pub backend: Backends,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            power_preference: PowerPreference::LowPower,
            backend: Backends::VULKAN,
        }
    }
}
