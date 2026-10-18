//! The pattern file: JSON, written in the current format only.
//!
//! Reading goes through [`crate::load`], one loader for each format ever
//! written. Writing knows only [`FORMAT`]: an older shape is never written.

use std::io::Write as _;
use std::path::Path;

use serde::Serialize;

use crate::error::{PatternError, Result};
use crate::grid::{Cell, Grid};
use crate::palette::EMPTY_KEY;
use crate::pattern::Pattern;

/// The format this program writes.
pub const FORMAT: u64 = 1;

/// The file as it is written.
#[derive(Serialize)]
struct Document<'a> {
    format: u64,
    grid: Grid,
    origin: [i32; 2],
    bead_ratio: f32,
    palette: Vec<Colour<'a>>,
    rows: Vec<String>,
}

/// One colour as it is written.
#[derive(Serialize)]
struct Colour<'a> {
    key: char,
    rgb: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'a str>,
}

impl Pattern {
    /// The pattern as the text of a file: pretty printed, one row of beads on
    /// one line. The rows are the active area; `origin` is the cell of the
    /// field of its top left corner, `[0, 0]` for a pattern with no beads.
    pub fn to_json(&self) -> Result<String> {
        let origin = self
            .bounds()
            .map_or(Cell { column: 0, row: 0 }, |bounds| bounds.min);
        let document = Document {
            format: FORMAT,
            grid: self.grid(),
            origin: [origin.column, origin.row],
            bead_ratio: self.ratio(),
            palette: self
                .palette()
                .iter()
                .map(|bead| Colour {
                    key: bead.key,
                    rgb: bead.rgb.to_string(),
                    catalog: bead.catalog.as_deref(),
                    code: bead.code.as_deref(),
                })
                .collect(),
            rows: self.rows(),
        };
        let mut text = serde_json::to_string_pretty(&document)
            .map_err(|source| PatternError::Json { source })?;
        text.push('\n');
        Ok(text)
    }

    /// Reads the text of a file of any format this program ever wrote.
    pub fn from_json(text: &str) -> Result<Pattern> {
        crate::load::load(text)
    }

    /// Each row of the active area as the keys of its beads.
    fn rows(&self) -> Vec<String> {
        let Some(bounds) = self.bounds() else {
            return Vec::new();
        };
        (bounds.min.row..=bounds.max.row)
            .map(|row| {
                (bounds.min.column..=bounds.max.column)
                    .map(|column| {
                        self.get(Cell { column, row })
                            .map_or(EMPTY_KEY, |index| self.palette()[index].key)
                    })
                    .collect()
            })
            .collect()
    }
}

/// Reads a pattern file.
pub fn read(path: &Path) -> Result<Pattern> {
    let text = std::fs::read_to_string(path).map_err(|source| PatternError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Pattern::from_json(&text)
}

/// Writes a pattern file: to a temporary file beside it, then a rename.
pub fn write(path: &Path, pattern: &Pattern) -> Result<()> {
    let text = pattern.to_json()?;
    let failed = |source| PatternError::Write {
        path: path.to_path_buf(),
        source,
    };
    let directory = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let mut file = tempfile::NamedTempFile::new_in(directory).map_err(failed)?;
    file.write_all(text.as_bytes()).map_err(failed)?;
    file.as_file().sync_all().map_err(failed)?;
    file.persist(path).map_err(|error| failed(error.error))?;
    Ok(())
}
