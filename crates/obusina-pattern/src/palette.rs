//! The colours of a pattern.

use crate::error::{PatternError, Result};

/// The key of a place with no bead.
pub const EMPTY_KEY: char = '.';

/// A colour as red, green and blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub [u8; 3]);

impl Rgb {
    /// Reads `#rrggbb`.
    pub fn parse(text: &str) -> Result<Rgb> {
        let invalid = || PatternError::Colour {
            text: text.to_string(),
        };
        let digits = text.strip_prefix('#').ok_or_else(invalid)?;
        if digits.len() != 6 || !digits.is_ascii() {
            return Err(invalid());
        }
        let channel =
            |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).map_err(|_| invalid());
        Ok(Rgb([channel(0)?, channel(2)?, channel(4)?]))
    }
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [red, green, blue] = self.0;
        write!(formatter, "#{red:02x}{green:02x}{blue:02x}")
    }
}

/// One colour of the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bead {
    /// The character that stands for it in a row of the file.
    pub key: char,
    /// What it looks like.
    pub rgb: Rgb,
    /// The catalogue it comes from, if any.
    pub catalog: Option<String>,
    /// Its number in that catalogue.
    pub code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_reads_back_as_it_was_written() {
        let rgb = Rgb::parse("#1b0aF2").expect("a colour");

        assert_eq!(rgb, Rgb([0x1b, 0x0a, 0xf2]));
        assert_eq!(rgb.to_string(), "#1b0af2");
    }

    #[test]
    fn a_colour_without_its_hash_is_refused() {
        assert!(matches!(
            Rgb::parse("1b0af2"),
            Err(PatternError::Colour { .. })
        ));
        assert!(matches!(
            Rgb::parse("#1b0a"),
            Err(PatternError::Colour { .. })
        ));
    }
}
