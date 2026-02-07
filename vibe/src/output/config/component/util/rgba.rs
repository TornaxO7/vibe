use super::ColorFormatError;
use serde::{Deserialize, Serialize};

const HEX_FORMAT: &str = "#RRGGBBAA";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgba(String);

impl Rgba {
    /// Creates a new instance of this.
    pub fn new<S: ToString>(color: S) -> Self {
        Self(color.to_string())
    }

    /// Converts the internal hex-rgb color into the gamma-corrected color value which within the range `[0, 1]`.
    pub fn as_f32(&self) -> Result<vibe_renderer::components::Rgba, ColorFormatError> {
        let hex = if self.is_hex() {
            self.0.clone()
        } else if self.is_env() {
            std::env::var(&self.0[1..]).map_err(|err| ColorFormatError::EnvVar {
                var_name: self.0.clone(),
                err,
            })?
        } else {
            return Err(ColorFormatError::InvalidFormat {
                color: self.0.clone(),
                hex_format: HEX_FORMAT.into(),
            });
        };

        parse_hex(hex)
    }

    /// Returns `true` if the struct contains a hex color as a string, otherwise `false`.
    fn is_hex(&self) -> bool {
        self.0.starts_with('#')
    }

    /// Returns `true` if the struct contains an environment variable as a string, otherwise `false`.
    fn is_env(&self) -> bool {
        self.0.starts_with('$')
    }
}

fn parse_hex<S: AsRef<str>>(hex: S) -> Result<vibe_renderer::components::Rgba, ColorFormatError> {
    // == validation
    let hex = hex.as_ref();

    let has_correct_length = hex.len() == HEX_FORMAT.len();
    let has_hex_prefix = hex.starts_with('#');
    if !(has_correct_length && has_hex_prefix) {
        return Err(ColorFormatError::InvalidFormat {
            color: hex.into(),
            hex_format: HEX_FORMAT.into(),
        });
    }

    // == parsing
    let hex = &hex[1..];

    let mut rgba = vibe_renderer::components::Rgba::default();

    for (idx, channel) in hex.as_bytes().chunks_exact(2).enumerate() {
        let value = u8::from_str_radix(str::from_utf8(channel).unwrap(), 16).map_err(|err| {
            ColorFormatError::InvalidChannelFormat {
                color: hex.into(),
                channel: String::from_utf8(channel.into()).expect("Used to be utf8???"),
                err,
            }
        })?;

        rgba[idx] = (value as f32 / 255.).powf(super::GAMMA);
    }

    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod valid {
        use super::*;

        #[test]
        fn black() {
            parse_hex("#00000000").unwrap();
        }

        #[test]
        fn white() {
            parse_hex("#FFFFFFFF").unwrap();
        }

        #[test]
        fn lowercase() {
            parse_hex("#aaaaaaaa").unwrap();
        }
    }

    mod invalid {
        use super::*;

        #[test]
        #[should_panic]
        fn too_short() {
            parse_hex("#0000000").unwrap();
        }

        #[test]
        #[should_panic]
        fn too_long() {
            parse_hex("#000000000").unwrap();
        }

        #[test]
        #[should_panic]
        fn missing_prefix() {
            parse_hex("FF00FF00").unwrap();
        }

        #[test]
        #[should_panic]
        fn invalid_char() {
            parse_hex("FFXXFFYY").unwrap();
        }
    }
}
