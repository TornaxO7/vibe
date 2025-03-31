use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FragmentCanvasConfig {
    pub audio_conf: shady_audio::Config,
}
