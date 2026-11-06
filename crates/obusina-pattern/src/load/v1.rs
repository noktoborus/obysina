//! Format 1, the first one.

use serde::Deserialize;

use crate::error::{PatternError, Result};
use crate::grid::{Bounds, Cell, Grid};
use crate::palette::{Colour as Bead, ColourId, Palette, Rgb};
use crate::pattern::Pattern;
use crate::ruling::Ruling as Lines;

/// A file of format 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(rename = "format")]
    _format: u64,
    grid: Grid,
    #[serde(default)]
    turn: u16,
    #[serde(default)]
    sheet: Option<Sheet>,
    #[serde(default)]
    ruling: Option<Ruling>,
    bead_ratio: f32,
    palettes: Vec<Vec<Option<Colour>>>,
    rows_origin: [i32; 2],
    rows: Vec<Vec<u32>>,
}

/// The sheet of format 1: its top left cell and its size.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sheet {
    origin: [i32; 2],
    size: [usize; 2],
}

/// The ruling of format 1; left out, the default one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ruling {
    step: [u32; 2],
    offset: [i32; 2],
}

/// A colour of format 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Colour {
    id: u32,
    rgb: Option<String>,
    #[serde(default)]
    hidden: bool,
    catalog: Option<String>,
    code: Option<String>,
}

/// Reads a file of format 1.
pub(super) fn load(value: serde_json::Value) -> Result<Pattern> {
    let document: Document =
        serde_json::from_value(value).map_err(|source| PatternError::Json { source })?;
    let palettes = document
        .palettes
        .into_iter()
        .map(|places| {
            let places = places
                .into_iter()
                .map(|place| place.map(Colour::read).transpose())
                .collect::<Result<Vec<_>>>()?;
            Palette::new(places)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut pattern = Pattern::with_palettes(document.grid, document.bead_ratio, palettes)?;
    pattern.set_turn(document.turn)?;
    let [column, row] = document.rows_origin;
    super::put(&mut pattern, Cell { column, row }, &document.rows)?;
    if let Some(sheet) = document.sheet {
        pattern.set_sheet(Some(sheet.bounds()?));
    }
    if let Some(Ruling { step, offset }) = document.ruling {
        pattern.set_ruling(Lines { step, offset });
    }
    Ok(pattern)
}

impl Colour {
    /// The colour of the model.
    fn read(self) -> Result<Bead> {
        Ok(Bead {
            id: ColourId(self.id),
            rgb: self.rgb.as_deref().map(Rgb::parse).transpose()?,
            hidden: self.hidden,
            catalog: self.catalog,
            code: self.code,
        })
    }
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
