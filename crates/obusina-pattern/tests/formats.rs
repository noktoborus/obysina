//! Every format ever written is read (`RULE pattern.version`).
//!
//! `tests/patterns/<format>.json` is a file of that format. It is never
//! changed once the format is replaced.

use obusina_pattern::{Bounds, Cell, FORMAT, Grid, Pattern, PatternError};

/// The file of a format, as text.
fn file(format: u64) -> String {
    let path = format!("tests/patterns/{format}.json");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn there_is_a_file_of_every_format() {
    for format in 1..=FORMAT {
        Pattern::from_json(&file(format)).expect("the file reads");
    }
}

#[test]
fn format_1_reads_its_beads() {
    let pattern = Pattern::from_json(&file(1)).expect("format 1 reads");

    assert_eq!(pattern.grid(), Grid::Peyote);
    assert_eq!(
        pattern.bounds(),
        Some(Bounds {
            min: Cell { column: -3, row: 2 },
            max: Cell { column: 2, row: 3 },
        })
    );
    assert_eq!(
        pattern.sheet(),
        Some(Bounds {
            min: Cell { column: -4, row: 1 },
            max: Cell { column: 3, row: 4 },
        })
    );
    assert_eq!(pattern.palette()[0].code.as_deref(), Some("DB-0010"));
    assert_eq!(pattern.get(Cell { column: -1, row: 2 }), Some(1));
    assert_eq!(pattern.get(Cell { column: 0, row: 2 }), None);
    assert_eq!(pattern.counts(), [5, 5]);
}

#[test]
fn the_current_format_reads_back_what_it_wrote() {
    let pattern = Pattern::from_json(&file(FORMAT)).expect("the file reads");

    let text = pattern.to_json().expect("the pattern writes");

    assert_eq!(Pattern::from_json(&text).expect("it reads back"), pattern);
    assert_eq!(text, file(FORMAT), "the file is written the way it is kept");
}

#[test]
fn a_row_is_one_line_of_the_file() {
    let text = file(FORMAT);

    assert!(text.contains("\n    \"AAB.BB\",\n"));
}

#[test]
fn a_newer_format_is_refused() {
    let text = file(FORMAT).replacen(
        &format!("\"format\": {FORMAT}"),
        &format!("\"format\": {}", FORMAT + 1),
        1,
    );

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::Newer { .. })
    ));
}

#[test]
fn a_file_with_no_format_is_refused() {
    assert!(matches!(
        Pattern::from_json("{}"),
        Err(PatternError::NoFormat)
    ));
}

#[test]
fn a_bead_of_no_colour_is_refused() {
    let text = file(1).replace("\"ABABA.\"", "\"ABABAC\"");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::UnknownKey {
            row: 1,
            column: 5,
            key: 'C'
        })
    ));
}

#[test]
fn a_row_shorter_than_the_first_is_refused() {
    let text = file(1).replace("\"ABABA.\"", "\"ABABA\"");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::RowWidth {
            row: 1,
            expected: 6,
            found: 5
        })
    ));
}

/// The rows are the active area, and `rows_origin` its place on the field:
/// a pattern opens where it was, and a peyote column keeps its parity.
#[test]
fn a_bead_moved_out_moves_the_origin() {
    let mut pattern = Pattern::from_json(&file(FORMAT)).expect("the file reads");
    pattern.set(Cell { column: -4, row: 1 }, Some(0));

    let text = pattern.to_json().expect("the pattern writes");

    assert!(
        text.contains("\"rows_origin\": [\n    -4,\n    1\n  ]"),
        "{text}"
    );
    assert_eq!(Pattern::from_json(&text).expect("it reads back"), pattern);
}

#[test]
fn a_pattern_with_no_beads_has_no_rows() {
    let mut pattern = Pattern::from_json(&file(FORMAT)).expect("the file reads");
    for (cell, _) in pattern.beads().collect::<Vec<_>>() {
        pattern.set(cell, None);
    }

    let text = pattern.to_json().expect("the pattern writes");

    assert!(text.contains("\"rows\": []"), "{text}");
    let again = Pattern::from_json(&text).expect("it reads back");
    assert_eq!(again.bounds(), None);
}

/// The sheet is the size set by hand; a file may carry none, and a pattern
/// opened from it has none until the window sets one.
#[test]
fn a_file_with_no_sheet_reads_with_none() {
    let start = file(1).find("  \"sheet\"").expect("the file has a sheet");
    let end = file(1).find("  \"bead_ratio\"").expect("and a bead ratio");
    let mut text = file(1);
    text.replace_range(start..end, "");

    let pattern = Pattern::from_json(&text).expect("it reads");

    assert_eq!(pattern.sheet(), None);
    assert_eq!(pattern.counts(), [5, 5]);
}

#[test]
fn a_sheet_of_no_cells_is_refused() {
    let text = file(1).replace("      8,\n      4", "      0,\n      4");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::Sheet {
            width: 0,
            height: 4
        })
    ));
}

#[test]
fn a_write_leaves_a_whole_file() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let path = directory.path().join("pattern.json");
    let pattern = Pattern::from_json(&file(FORMAT)).expect("the file reads");

    obusina_pattern::write(&path, &pattern).expect("the pattern writes");

    assert_eq!(obusina_pattern::read(&path).expect("it reads"), pattern);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}
