//! Format 1, the first one.

use serde::Deserialize;

use crate::error::{PatternError, Result};
use crate::grid::Grid;
use crate::palette::{Bead, Rgb};
use crate::pattern::Pattern;

/// A file of format 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(rename = "format")]
    _format: u64,
    grid: Grid,
    width: usize,
    height: usize,
    bead_ratio: f32,
    palette: Vec<Colour>,
    rows: Vec<String>,
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
    let mut pattern = Pattern::new(
        document.grid,
        document.width,
        document.height,
        document.bead_ratio,
        palette,
    )?;
    super::fill(&mut pattern, &document.rows)?;
    Ok(pattern)
}
