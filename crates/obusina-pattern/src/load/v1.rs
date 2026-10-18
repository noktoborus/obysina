//! Format 1, the first one.

use serde::Deserialize;

use crate::error::{PatternError, Result};
use crate::grid::{Bounds, Cell, Grid};
use crate::palette::{Bead, Rgb};
use crate::pattern::Pattern;

/// A file of format 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(rename = "format")]
    _format: u64,
    grid: Grid,
    #[serde(default)]
    sheet: Option<Sheet>,
    bead_ratio: f32,
    palette: Vec<Colour>,
    rows_origin: [i32; 2],
    rows: Vec<String>,
}

/// The sheet of format 1: its top left cell and its size.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sheet {
    origin: [i32; 2],
    size: [usize; 2],
}

/// A colour of format 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Colour {
    key: char,
    rgb: String,
    catalog: Option<String>,
    code: Option<String>,
}

/// Reads a file of format 1.
pub(super) fn load(value: serde_json::Value) -> Result<Pattern> {
    let document: Document =
        serde_json::from_value(value).map_err(|source| PatternError::Json { source })?;
    let palette = document
        .palette
        .into_iter()
        .map(|colour| {
            Ok(Bead {
                key: colour.key,
                rgb: Rgb::parse(&colour.rgb)?,
                catalog: colour.catalog,
                code: colour.code,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut pattern = Pattern::new(document.grid, document.bead_ratio, palette)?;
    let [column, row] = document.rows_origin;
    super::put(&mut pattern, Cell { column, row }, &document.rows)?;
    if let Some(sheet) = document.sheet {
        pattern.set_sheet(Some(sheet.bounds()?));
    }
    Ok(pattern)
}

impl Sheet {
    /// The cells of the sheet; a sheet of no cells, or past the field a
    /// cell can name, is refused.
    fn bounds(&self) -> Result<Bounds> {
        let [width, height] = self.size;
        let [column, row] = self.origin;
        let refused = || PatternError::Sheet { width, height };
        if width == 0 || height == 0 {
            return Err(refused());
        }
        let min = Cell { column, row };
        let max = super::offset(min, width - 1, height - 1).ok_or_else(refused)?;
        Ok(Bounds { min, max })
    }
}
