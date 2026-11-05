//! What can go wrong with a pattern and its file.

use std::path::PathBuf;

/// One failure of a pattern or its file.
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    /// The file cannot be read.
    #[error("the pattern file {} cannot be read", path.display())]
    Read {
        /// The file.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },

    /// The file cannot be written.
    #[error("the pattern file {} cannot be written", path.display())]
    Write {
        /// The file.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },

    /// The text is not JSON, or not the shape its format says.
    #[error("the pattern is not valid JSON of its format")]
    Json {
        /// What the parser said.
        #[source]
        source: serde_json::Error,
    },

    /// The file has no `format` field.
    #[error("the pattern has no format number")]
    NoFormat,

    /// The file was written by a newer program.
    #[error("the pattern has format {found}, this program reads up to {known}")]
    Newer {
        /// The format of the file.
        found: u64,
        /// The newest format this program knows.
        known: u64,
    },

    /// The format number is not one this program ever wrote.
    #[error("the pattern has format {found}, which was never written")]
    UnknownFormat {
        /// The format of the file.
        found: u64,
    },

    /// A row is not as long as the first one.
    #[error("row {row} has {found} beads, the first has {expected}")]
    RowWidth {
        /// The row, from zero.
        row: usize,
        /// The beads in the first row.
        expected: usize,
        /// The beads in the row.
        found: usize,
    },

    /// A bead names an id no colour has.
    #[error("row {row}, column {column}: no colour has the id {id}")]
    UnknownId {
        /// The row, from zero.
        row: usize,
        /// The column, from zero.
        column: usize,
        /// The id.
        id: u32,
    },

    /// Two colours have one id, or a colour has the number of a cell with no
    /// bead.
    #[error("the colour id {id} is used twice or is reserved")]
    DuplicateId {
        /// The id.
        id: u32,
    },

    /// A palette has more places than it can.
    #[error("a palette has {places} places, at most {max} are allowed")]
    Palette {
        /// The places of the file.
        places: usize,
        /// The most it can have.
        max: usize,
    },

    /// A pattern has no palette.
    #[error("the pattern has no palette")]
    NoPalette,

    /// A place of a palette is not there, or holds nothing to act on.
    #[error("palette {palette} has no colour to act on at place {place}")]
    Place {
        /// The palette, from zero.
        palette: usize,
        /// The place, from zero.
        place: usize,
    },

    /// A colour cannot go to a palette with no free place.
    #[error("palette {palette} has no free place")]
    Full {
        /// The palette, from zero.
        palette: usize,
    },

    /// A colour is not written as `#rrggbb`.
    #[error("the colour {text:?} is not #rrggbb")]
    Colour {
        /// What the file says.
        text: String,
    },

    /// A bead has no size.
    #[error("the bead ratio {ratio} is not a size")]
    Ratio {
        /// The width of a bead to its height.
        ratio: f32,
    },

    /// A turn is not a quarter of a whole one.
    #[error("the turn {degrees} is not 0, 90, 180 or 270")]
    Turn {
        /// The turn, in degrees.
        degrees: u16,
    },

    /// The sheet of the file has no cells.
    #[error("the sheet is {width} by {height}")]
    Sheet {
        /// Columns.
        width: usize,
        /// Rows.
        height: usize,
    },

    /// A bead of the file lies outside the field a cell can name.
    #[error("row {row}, column {column} lies outside the field")]
    Outside {
        /// The row of the file, from zero.
        row: usize,
        /// The column of the file, from zero.
        column: usize,
    },
}

/// Result of this crate.
pub type Result<T> = std::result::Result<T, PatternError>;
