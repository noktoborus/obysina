//! The colours of a pattern: each has an id, unique in the pattern, and stands
//! in a place of a palette.

use crate::error::{PatternError, Result};

/// The most places a palette has, one for each digit of a keyboard.
pub const MAX_COLOURS: usize = 9;

/// The number in a row of the file for a cell with no bead. No colour has it.
pub(crate) const NO_BEAD: u32 = 0;

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

/// What a colour is known by. A bead holds it, so a colour moves between
/// places and palettes and no bead changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColourId(pub u32);

impl std::fmt::Display for ColourId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colour {
    /// What its beads hold.
    pub id: ColourId,
    /// What it looks like. With none it was removed, and its beads are
    /// hatched until a colour is put in its place again.
    pub rgb: Option<Rgb>,
    /// It keeps its `rgb`, but it and its beads are hatched.
    pub hidden: bool,
    /// The catalogue it comes from, if any.
    pub catalog: Option<String>,
    /// Its number in that catalogue.
    pub code: Option<String>,
}

impl Colour {
    /// A colour shown, from no catalogue.
    pub fn new(id: ColourId, rgb: Rgb) -> Colour {
        Colour {
            id,
            rgb: Some(rgb),
            hidden: false,
            catalog: None,
            code: None,
        }
    }

    /// What it is drawn with; `None` means a hatch.
    pub fn seen(&self) -> Option<Rgb> {
        self.rgb.filter(|_| !self.hidden)
    }
}

/// A row of [`MAX_COLOURS`] places, each with a colour or empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    places: Vec<Option<Colour>>,
}

impl Palette {
    /// A palette of these places; the ones not given are empty.
    pub fn new(mut places: Vec<Option<Colour>>) -> Result<Palette> {
        if places.len() > MAX_COLOURS {
            return Err(PatternError::Palette {
                places: places.len(),
                max: MAX_COLOURS,
            });
        }
        places.resize(MAX_COLOURS, None);
        Ok(Palette { places })
    }

    /// Its places, always [`MAX_COLOURS`].
    pub fn places(&self) -> &[Option<Colour>] {
        &self.places
    }

    /// The first place with nothing in it.
    pub fn first_free(&self) -> Option<usize> {
        self.places.iter().position(Option::is_none)
    }

    /// True when nothing stands in any place.
    pub fn is_empty(&self) -> bool {
        self.places.iter().all(Option::is_none)
    }

    /// Its places, to change.
    pub(crate) fn places_mut(&mut self) -> &mut [Option<Colour>] {
        &mut self.places
    }
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

    #[test]
    fn a_palette_has_its_places_and_no_more() {
        let colour = Colour::new(ColourId(1), Rgb([0, 0, 0]));
        let palette = Palette::new(vec![Some(colour.clone())]).expect("a palette");

        assert_eq!(palette.places().len(), MAX_COLOURS);
        assert_eq!(palette.first_free(), Some(1));
        assert!(matches!(
            Palette::new(vec![None; MAX_COLOURS + 1]),
            Err(PatternError::Palette { .. })
        ));
    }

    #[test]
    fn a_hidden_colour_and_a_removed_one_are_not_seen() {
        let mut colour = Colour::new(ColourId(1), Rgb([1, 2, 3]));
        assert_eq!(colour.seen(), Some(Rgb([1, 2, 3])));

        colour.hidden = true;
        assert_eq!(colour.seen(), None);

        colour.hidden = false;
        colour.rgb = None;
        assert_eq!(colour.seen(), None);
    }
}
