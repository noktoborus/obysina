//! The history of a pattern as a file, kept beside the pattern so a pattern
//! opens with its undo and redo.
//!
//! ```text
//! { "format": 1, "pattern": <a hash of the text of the pattern>,
//!   "done": [<edit>, ...], "undone": [<edit>, ...] }
//! ```
//!
//! The history is of the pattern it was written with: its `pattern` is a
//! hash of the text of that pattern file. A pattern whose text has another
//! hash, written alone after the history, opens with no history. A file this
//! version cannot read is no history either: the history is a help, not the
//! work (`RULE pattern.version` is of the pattern only).

#[cfg(test)]
use obusina_pattern::Pattern;
use obusina_pattern::{Bounds, Cell, Colour, ColourId, Grid, Palette, Rgb, Ruling};
use serde::{Deserialize, Serialize};

use crate::edit::{Change, Edit, History};

/// The format of the file this version writes and reads.
const FORMAT: u64 = 1;

/// The file.
#[derive(Serialize, Deserialize)]
struct Document {
    format: u64,
    pattern: u64,
    done: Vec<Step>,
    undone: Vec<Step>,
}

/// An edit as the file holds it.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Step {
    Beads {
        changes: Vec<[i64; 4]>,
    },
    Grid {
        before: Grid,
        after: Grid,
    },
    Palettes {
        before: Vec<Vec<Option<Place>>>,
        after: Vec<Vec<Option<Place>>>,
    },
    Sheet {
        before: Option<[i32; 4]>,
        after: Option<[i32; 4]>,
    },
    Ruling {
        before: [i32; 4],
        after: [i32; 4],
    },
    Paper {
        before: [u8; 3],
        after: [u8; 3],
    },
    Many {
        steps: Vec<Step>,
    },
}

/// A colour of a palette as the file holds it.
#[derive(Serialize, Deserialize)]
struct Place {
    id: u32,
    rgb: Option<[u8; 3]>,
    hidden: bool,
    catalog: Option<String>,
    code: Option<String>,
}

/// The text of the history of `pattern`, whose file is `text`.
pub fn write(history: &History, text: &str) -> serde_json::Result<String> {
    let document = Document {
        format: FORMAT,
        pattern: hash(text),
        done: history.done().iter().map(Step::from).collect(),
        undone: history.undone().iter().map(Step::from).collect(),
    };
    serde_json::to_string(&document)
}

/// The history in `file`, if it is of the pattern whose file is `text`;
/// `None` when it is of another one or cannot be read.
pub fn read(file: &str, text: &str) -> Option<History> {
    let document: Document = serde_json::from_str(file)
        .inspect_err(|error| log::warn!("a history is not read: {error}"))
        .ok()?;
    if document.format != FORMAT || document.pattern != hash(text) {
        log::info!("a history of another pattern is left");
        return None;
    }
    let edits = |steps: Vec<Step>| {
        steps
            .into_iter()
            .map(Step::edit)
            .collect::<Option<Vec<_>>>()
    };
    Some(History::restored(
        edits(document.done)?,
        edits(document.undone)?,
    ))
}

/// The hash of the text of a pattern file: FNV-1a, which is the same on
/// every platform and in every version.
pub fn hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The text of a pattern, which its history is hashed by.
#[cfg(test)]
pub fn text_of(pattern: &Pattern) -> Option<String> {
    pattern
        .to_json()
        .inspect_err(|error| log::error!("a pattern does not turn into text: {error}"))
        .ok()
}

impl From<&Edit> for Step {
    fn from(edit: &Edit) -> Step {
        match edit {
            Edit::Beads(changes) => Step::Beads {
                changes: changes
                    .iter()
                    .map(|change| {
                        [
                            i64::from(change.cell.column),
                            i64::from(change.cell.row),
                            change.before.map_or(0, |id| i64::from(id.0)),
                            change.after.map_or(0, |id| i64::from(id.0)),
                        ]
                    })
                    .collect(),
            },
            Edit::Grid { before, after } => Step::Grid {
                before: *before,
                after: *after,
            },
            Edit::Palettes { before, after } => Step::Palettes {
                before: palettes(before),
                after: palettes(after),
            },
            Edit::Sheet { before, after } => Step::Sheet {
                before: before.map(sheet),
                after: after.map(sheet),
            },
            Edit::Ruling { before, after } => Step::Ruling {
                before: ruling(*before),
                after: ruling(*after),
            },
            Edit::Paper { before, after } => Step::Paper {
                before: before.0,
                after: after.0,
            },
            Edit::Many(edits) => Step::Many {
                steps: edits.iter().map(Step::from).collect(),
            },
        }
    }
}

impl Step {
    /// The edit of the model; `None` for one that names what cannot be.
    fn edit(self) -> Option<Edit> {
        Some(match self {
            Step::Beads { changes } => Edit::Beads(
                changes
                    .into_iter()
                    .map(|[column, row, before, after]| {
                        Some(Change {
                            cell: Cell {
                                column: i32::try_from(column).ok()?,
                                row: i32::try_from(row).ok()?,
                            },
                            before: colour(before)?,
                            after: colour(after)?,
                        })
                    })
                    .collect::<Option<_>>()?,
            ),
            Step::Grid { before, after } => Edit::Grid { before, after },
            Step::Palettes { before, after } => Edit::Palettes {
                before: read_palettes(before)?,
                after: read_palettes(after)?,
            },
            Step::Sheet { before, after } => Edit::Sheet {
                before: before.map(bounds),
                after: after.map(bounds),
            },
            Step::Ruling { before, after } => Edit::Ruling {
                before: read_ruling(before)?,
                after: read_ruling(after)?,
            },
            Step::Paper { before, after } => Edit::Paper {
                before: Rgb(before),
                after: Rgb(after),
            },
            Step::Many { steps } => {
                Edit::Many(steps.into_iter().map(Step::edit).collect::<Option<_>>()?)
            }
        })
    }
}

/// The colour of a bead in the file: 0 for none.
fn colour(id: i64) -> Option<Option<ColourId>> {
    match id {
        0 => Some(None),
        id => Some(Some(ColourId(u32::try_from(id).ok()?))),
    }
}

fn palettes(palettes: &[Palette]) -> Vec<Vec<Option<Place>>> {
    palettes
        .iter()
        .map(|palette| {
            palette
                .places()
                .iter()
                .map(|place| {
                    place.as_ref().map(|colour| Place {
                        id: colour.id.0,
                        rgb: colour.rgb.map(|rgb| rgb.0),
                        hidden: colour.hidden,
                        catalog: colour.catalog.clone(),
                        code: colour.code.clone(),
                    })
                })
                .collect()
        })
        .collect()
}

fn read_palettes(palettes: Vec<Vec<Option<Place>>>) -> Option<Vec<Palette>> {
    palettes
        .into_iter()
        .map(|places| {
            let places = places
                .into_iter()
                .map(|place| {
                    place.map(|place| Colour {
                        id: ColourId(place.id),
                        rgb: place.rgb.map(Rgb),
                        hidden: place.hidden,
                        catalog: place.catalog,
                        code: place.code,
                    })
                })
                .collect();
            Palette::new(places).ok()
        })
        .collect()
}

fn sheet(bounds: Bounds) -> [i32; 4] {
    [
        bounds.min.column,
        bounds.min.row,
        bounds.max.column,
        bounds.max.row,
    ]
}

fn bounds([column, row, last_column, last_row]: [i32; 4]) -> Bounds {
    Bounds {
        min: Cell { column, row },
        max: Cell {
            column: last_column,
            row: last_row,
        },
    }
}

fn ruling(ruling: Ruling) -> [i32; 4] {
    let [columns, rows] = ruling.step.map(|step| step as i32);
    let [column, row] = ruling.offset;
    [columns, rows, column, row]
}

fn read_ruling([columns, rows, column, row]: [i32; 4]) -> Option<Ruling> {
    Some(Ruling {
        step: [u32::try_from(columns).ok()?, u32::try_from(rows).ok()?],
        offset: [column, row],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A history written and read back undoes and redoes as it did.
    #[test]
    fn a_history_reads_back_and_undoes() {
        let mut pattern = crate::app::new_pattern();
        let mut history = History::default();
        let cell = Cell { column: 2, row: 3 };
        history.paint(&mut pattern, &[cell], Some(ColourId(1)));
        let before = pattern.paper();
        let paper = Edit::Paper {
            before,
            after: Rgb([0, 0, 0]),
        };
        history.apply(&mut pattern, paper);
        history.undo(&mut pattern);
        let text = text_of(&pattern).expect("text");

        let file = write(&history, &text).expect("it writes");
        let mut again = read(&file, &text).expect("it reads");

        again.redo(&mut pattern);
        assert_eq!(pattern.paper(), Rgb([0, 0, 0]));
        again.undo(&mut pattern);
        again.undo(&mut pattern);
        assert_eq!(pattern.get(cell), None);
        assert!(!again.can_undo());
    }

    /// A history of another pattern is left.
    #[test]
    fn a_history_of_another_pattern_is_not_read() {
        let history = History::default();
        let file = write(&history, "one").expect("it writes");

        assert!(read(&file, "two").is_none());
        assert!(read("not json", "one").is_none());
    }
}
