use std::str::FromStr;

use css_color::Srgb;
use thiserror::Error;

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// Wrap definition to give it an `Error` implementation
#[derive(Debug, Error)]
#[error("Failed to find color by name \"{0}\"")]
pub struct ParseColorError(String);

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Color, Self::Err> {
        let srgb = s.parse::<Srgb>().map_err(|_| ParseColorError(s.to_owned()))?;

        Ok(Color {
            r: (srgb.red * srgb.alpha * 255.) as u8,
            g: (srgb.green * srgb.alpha * 255.) as u8,
            b: (srgb.blue * srgb.alpha * 255.) as u8,
        })
    }
}
