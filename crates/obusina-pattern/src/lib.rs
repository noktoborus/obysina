#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
mod file;
mod grid;
mod load;
mod palette;
mod pattern;

pub use error::{PatternError, Result};
pub use file::{FORMAT, read, write};
pub use grid::{Bounds, Cell, Grid};
pub use palette::{Bead, EMPTY_KEY, Rgb};
pub use pattern::{DEFAULT_BEAD_RATIO, Pattern, TURNS};
