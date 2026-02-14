mod rgb;
mod rgba;

use std::num::ParseIntError;

pub use rgb::Rgb;
pub use rgba::Rgba;

type HexFormat = String;
type Color = String;
type Channel = String;

/// Gamma-correction constant for the color correction from u8 => f32
const GAMMA: f32 = 2.2;

const ENV_FORMAT: &str = "$ENV_NAME";

#[derive(thiserror::Error, Debug)]
pub enum ColorFormatError {
    #[error("Your color string '{color}' seems to be wrong. It can either be '{hex_format}' or '{ENV_FORMAT}' (the env variable should also store a string with the format '{hex_format}').")]
    InvalidFormat { color: Color, hex_format: HexFormat },

    #[error("Couldn't parse the color '{channel}' in '{color}': {err}")]
    InvalidChannelFormat {
        color: HexFormat,
        channel: Channel,
        err: ParseIntError,
    },

    #[error("Couldn't read color value from environment variable '{var_name}': {err}")]
    EnvVar {
        var_name: String,
        err: std::env::VarError,
    },
}
