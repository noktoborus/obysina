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
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: String,
    grid: Grid,
    #[serde(default)]
    turn: u16,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    field: Option<Field>,
    #[serde(default)]
    ruling: Option<Ruling>,
    #[serde(default)]
    paper: Option<String>,
    bead_ratio: f32,
    palettes: Vec<Vec<Option<Colour>>>,
    rows_origin: [i32; 2],
    rows: Vec<Vec<u32>>,
}

/// The field set by hand of format 1: its top left cell and its size;
/// left out, the size is by the beads.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
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
    if let Some(id) = document.id {
        pattern.set_id(id.parse()?);
    }
    pattern.set_name(document.name);
    pattern.set_turn(document.turn)?;
    pattern.set_locked(document.locked);
    let [column, row] = document.rows_origin;
    super::put(&mut pattern, Cell { column, row }, &document.rows)?;
    if let Some(field) = document.field {
        pattern.set_field(Some(field.bounds()?));
    }
    if let Some(Ruling { step, offset }) = document.ruling {
        pattern.set_ruling(Lines { step, offset });
    }
    if let Some(paper) = document.paper {
        pattern.set_paper(Rgb::parse(&paper)?);
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

impl Field {
    /// The cells of the field; a field of no cells, or past what a cell can
    /// name, is refused.
    fn bounds(&self) -> Result<Bounds> {
        let [width, height] = self.size;
        let [column, row] = self.origin;
        let refused = || PatternError::Field { width, height };
        if width == 0 || height == 0 {
            return Err(refused());
        }
        let min = Cell { column, row };
        let max = super::offset(min, width - 1, height - 1).ok_or_else(refused)?;
        Ok(Bounds { min, max })
    }
}
