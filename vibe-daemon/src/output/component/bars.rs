use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BarsConfig {
    pub audio_conf: shady_audio::Config,
}

impl Default for BarsConfig {
    fn default() -> Self {
        Self {
            audio_conf: shady_audio::Config {
                amount_bars: 60,
                ..Default::default()
            },
        }
    }
}
