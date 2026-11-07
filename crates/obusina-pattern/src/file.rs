//! The pattern file: JSON, written in the current format only.
//!
//! Reading goes through [`crate::load`], one loader for each format ever
//! written. Writing knows only [`FORMAT`]: an older shape is never written.

use std::io::Write as _;
use std::path::Path;

use serde::Serialize;
use serde_json::value::RawValue;

use crate::error::{PatternError, Result};
use crate::grid::{Cell, Grid};
use crate::palette::{Colour, NO_BEAD};
use crate::pattern::Pattern;

/// The format this program writes.
pub const FORMAT: u64 = 1;

/// The file as it is written.
#[derive(Serialize)]
struct Document<'a> {
    format: u64,
    id: String,
    #[serde(skip_serializing_if = "str::is_empty")]
    name: &'a str,
    grid: Grid,
    turn: u16,
    #[serde(skip_serializing_if = "is_false")]
    locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    sheet: Option<Sheet>,
    ruling: WrittenRuling,
    paper: String,
    bead_ratio: f32,
    palettes: Vec<Vec<Option<Written<'a>>>>,
    rows_origin: [i32; 2],
    rows: Vec<Box<RawValue>>,
}

/// The sheet as it is written: its top left cell and its size.
#[derive(Serialize)]
struct Sheet {
    origin: [i32; 2],
    size: [usize; 2],
}

/// The ruling as it is written: the columns and rows between two lines,
/// and where a line stands.
#[derive(Serialize)]
struct WrittenRuling {
    step: [u32; 2],
    offset: [i32; 2],
}

/// One colour as it is written.
#[derive(Serialize)]
struct Written<'a> {
    id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rgb: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'a str>,
}

/// True for a value that is not written.
fn is_false(value: &bool) -> bool {
    !*value
}

/// The places of a palette as they are written: the empty ones at its end
/// are left out.
fn written(places: &[Option<Colour>]) -> Vec<Option<Written<'_>>> {
    let end = places
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |last| last + 1);
    places[..end]
        .iter()
        .map(|place| {
            place.as_ref().map(|colour| Written {
                id: colour.id.0,
                rgb: colour.rgb.map(|rgb| rgb.to_string()),
                hidden: colour.hidden,
                catalog: colour.catalog.as_deref(),
                code: colour.code.as_deref(),
            })
        })
        .collect()
}

impl Pattern {
    /// The pattern as the text of a file: pretty printed, one row of beads on
    /// one line. The rows are the active area; `rows_origin` is the cell of
    /// the field of its top left corner, `[0, 0]` for a pattern with no
    /// beads. The sheet is written when one is set.
    pub fn to_json(&self) -> Result<String> {
        let origin = self
            .bounds()
            .map_or(Cell { column: 0, row: 0 }, |bounds| bounds.min);
        let document = Document {
            format: FORMAT,
            id: self.id().to_string(),
            name: self.name(),
            grid: self.grid(),
            turn: self.turn(),
            locked: self.locked(),
            sheet: self.sheet().map(|sheet| Sheet {
                origin: [sheet.min.column, sheet.min.row],
                size: [sheet.width(), sheet.height()],
            }),
            ruling: WrittenRuling {
                step: self.ruling().step,
                offset: self.ruling().offset,
            },
            paper: self.paper().to_string(),
            bead_ratio: self.ratio(),
            palettes: self
                .palettes()
                .iter()
                .map(|palette| written(palette.places()))
                .collect(),
            rows_origin: [origin.column, origin.row],
            rows: self.rows()?,
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

    /// Each row of the active area as the ids of its beads, as text, so that
    /// a row stands on one line of the file (`RULE pattern.diff`).
    fn rows(&self) -> Result<Vec<Box<RawValue>>> {
        let Some(bounds) = self.bounds() else {
            return Ok(Vec::new());
        };
        (bounds.min.row..=bounds.max.row)
            .map(|row| {
                let ids: Vec<String> = (bounds.min.column..=bounds.max.column)
                    .map(|column| {
                        self.get(Cell { column, row })
                            .map_or(NO_BEAD, |id| id.0)
                            .to_string()
                    })
                    .collect();
                RawValue::from_string(format!("[{}]", ids.join(", ")))
                    .map_err(|source| PatternError::Json { source })
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
