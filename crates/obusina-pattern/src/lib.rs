#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
mod file;
mod grid;
mod id;
mod load;
mod palette;
mod pattern;
mod ruling;

pub use error::{PatternError, Result};
pub use file::{FORMAT, read, write};
pub use grid::{Bounds, Cell, Grid};
pub use id::PatternId;
pub use palette::{Colour, ColourId, MAX_COLOURS, Palette, Rgb};
pub use pattern::{DEFAULT_BEAD_RATIO, Pattern, TURNS};
pub use ruling::{DEFAULT_RULING_STEP, Ruling};
