//! Loaders, one for each format ever written.
//!
//! A loader turns a file of its format into the current [`Pattern`]. Once a
//! format is replaced, its loader is not changed again: files of that format
//! exist and must keep opening. Nothing else in the crate knows an old format.

mod v1;

use crate::error::{PatternError, Result};
use crate::file::FORMAT;
use crate::grid::Cell;
use crate::palette::EMPTY_KEY;
use crate::pattern::Pattern;

/// Reads a pattern of any known format.
pub fn load(text: &str) -> Result<Pattern> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|source| PatternError::Json { source })?;
    let format = value
        .get("format")
        .and_then(serde_json::Value::as_u64)
        .ok_or(PatternError::NoFormat)?;
    match format {
        1 => v1::load(value),
        found if found > FORMAT => Err(PatternError::Newer {
            found,
            known: FORMAT,
        }),
        found => Err(PatternError::UnknownFormat { found }),
    }
}

/// Puts the beads of rows of keys into an empty pattern, the first key of
/// the first row at `origin`. Every row is as long as the first.
fn put(pattern: &mut Pattern, origin: Cell, rows: &[String]) -> Result<()> {
    let expected = rows.first().map_or(0, |keys| keys.chars().count());
    for (row, keys) in rows.iter().enumerate() {
        let found = keys.chars().count();
        if found != expected {
            return Err(PatternError::RowWidth {
                row,
                expected,
                found,
            });
        }
        for (column, key) in keys.chars().enumerate() {
            if key == EMPTY_KEY {
                continue;
            }
            let index = pattern
                .palette()
                .iter()
                .position(|bead| bead.key == key)
                .ok_or(PatternError::UnknownKey { row, column, key })?;
            let cell = offset(origin, column, row).ok_or(PatternError::Outside { row, column })?;
            pattern.set(cell, Some(index));
        }
    }
    Ok(())
}

/// The cell `column` and `row` past `origin`, if the field can name it.
fn offset(origin: Cell, column: usize, row: usize) -> Option<Cell> {
    let column = origin.column.checked_add(i32::try_from(column).ok()?)?;
    let row = origin.row.checked_add(i32::try_from(row).ok()?)?;
    Some(Cell { column, row })
}
