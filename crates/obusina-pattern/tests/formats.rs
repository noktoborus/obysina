//! Every format ever written is read (`RULE pattern.version`).
//!
//! `tests/patterns/<format>.json` is a file of that format. It is never
//! changed once the format is replaced.

use obusina_pattern::{
    Bounds, Cell, ColourId, DEFAULT_PAPER, FORMAT, Grid, Pattern, PatternError, PatternId, Rgb,
    Ruling,
};

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
    assert_eq!(pattern.turn(), 90);
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
    let first = pattern.colour(ColourId(1)).expect("the first colour");
    assert_eq!(first.code.as_deref(), Some("DB-0010"));
    assert_eq!(pattern.get(Cell { column: -1, row: 2 }), Some(ColourId(2)));
    assert_eq!(pattern.get(Cell { column: 0, row: 2 }), None);
    assert_eq!(pattern.count(ColourId(1)), 5);
    assert_eq!(pattern.count(ColourId(2)), 5);
    assert_eq!(pattern.count(ColourId(3)), 1);
}

#[test]
fn format_1_reads_its_palettes() {
    let pattern = Pattern::from_json(&file(1)).expect("format 1 reads");

    let palettes = pattern.palettes();
    assert_eq!(palettes.len(), 2);
    assert!(palettes[1].places()[0].is_none(), "an empty place");
    let removed = pattern.colour(ColourId(3)).expect("a removed colour");
    assert_eq!((removed.rgb, removed.seen()), (None, None));
    let hidden = pattern.colour(ColourId(4)).expect("a hidden colour");
    assert!(hidden.rgb.is_some());
    assert_eq!(hidden.seen(), None);
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

    assert!(text.contains("\n    [1, 1, 2, 0, 2, 2],\n"));
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
    let text = file(1).replace("[1, 2, 1, 2, 1, 3]", "[1, 2, 1, 2, 1, 7]");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::UnknownId {
            row: 1,
            column: 5,
            id: 7
        })
    ));
}

#[test]
fn a_row_shorter_than_the_first_is_refused() {
    let text = file(1).replace("[1, 2, 1, 2, 1, 3]", "[1, 2, 1, 2, 1]");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::RowWidth {
            row: 1,
            expected: 6,
            found: 5
        })
    ));
}

#[test]
fn a_file_with_no_palette_is_refused() {
    let value: serde_json::Value = serde_json::from_str(&file(1)).expect("JSON");
    let mut value = value;
    value["palettes"] = serde_json::json!([]);

    assert!(matches!(
        Pattern::from_json(&value.to_string()),
        Err(PatternError::NoPalette)
    ));
}

#[test]
fn a_palette_with_too_many_places_is_refused() {
    let mut value: serde_json::Value = serde_json::from_str(&file(1)).expect("JSON");
    value["palettes"][1] = serde_json::Value::Array(vec![serde_json::Value::Null; 10]);

    assert!(matches!(
        Pattern::from_json(&value.to_string()),
        Err(PatternError::Palette { places: 10, .. })
    ));
}

/// The rows are the active area, and `rows_origin` its place on the field:
/// a pattern opens where it was, and a peyote column keeps its parity.
#[test]
fn a_bead_moved_out_moves_the_origin() {
    let mut pattern = Pattern::from_json(&file(FORMAT)).expect("the file reads");
    pattern.set(Cell { column: -4, row: 1 }, Some(ColourId(1)));

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
    let end = file(1).find("  \"ruling\"").expect("and a ruling");
    let mut text = file(1);
    text.replace_range(start..end, "");

    let pattern = Pattern::from_json(&text).expect("it reads");

    assert_eq!(pattern.sheet(), None);
    assert_eq!(pattern.count(ColourId(1)), 5);
}

#[test]
fn a_turn_that_is_not_a_quarter_is_refused() {
    let text = file(1).replace("\"turn\": 90", "\"turn\": 45");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::Turn { degrees: 45 })
    ));
}

#[test]
fn a_file_with_no_turn_is_shown_unturned() {
    let text = file(1).replace("  \"turn\": 90,\n", "");

    assert_eq!(Pattern::from_json(&text).expect("it reads").turn(), 0);
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
fn format_1_reads_its_ruling() {
    let pattern = Pattern::from_json(&file(1)).expect("format 1 reads");

    assert_eq!(
        pattern.ruling(),
        Ruling {
            step: [10, 4],
            offset: [2, -1],
        }
    );
}

/// A file written before the ruling reads with the default one.
#[test]
fn a_file_with_no_ruling_reads_with_the_default() {
    let start = file(1).find("  \"ruling\"").expect("the file has a ruling");
    let end = file(1).find("  \"paper\"").expect("and a paper");
    let mut text = file(1);
    text.replace_range(start..end, "");

    let pattern = Pattern::from_json(&text).expect("it reads");

    assert_eq!(pattern.ruling(), Ruling::default());
}

#[test]
fn format_1_reads_its_id() {
    let pattern = Pattern::from_json(&file(1)).expect("format 1 reads");

    let id: PatternId = "6f0d2c7e-9a41-4b8e-8d3a-2f5b1c9e7a10"
        .parse()
        .expect("an identifier");
    assert_eq!(pattern.id(), id);
}

/// A file written before the id reads with a new one, written at the next
/// save.
#[test]
fn a_file_with_no_id_reads_with_a_new_one() {
    let text = file(1).replace("  \"id\": \"6f0d2c7e-9a41-4b8e-8d3a-2f5b1c9e7a10\",\n", "");

    let first = Pattern::from_json(&text).expect("it reads");
    let second = Pattern::from_json(&text).expect("it reads");

    assert_ne!(first.id(), second.id());
    assert!(
        first
            .to_json()
            .expect("it writes")
            .contains(&first.id().to_string())
    );
}

#[test]
fn an_id_that_is_no_identifier_is_refused() {
    let text = file(1).replace("6f0d2c7e-9a41-4b8e-8d3a-2f5b1c9e7a10", "twelve");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::Id { .. })
    ));
}

#[test]
fn format_1_reads_its_paper() {
    let pattern = Pattern::from_json(&file(1)).expect("format 1 reads");

    assert_eq!(pattern.paper(), Rgb([0xe6, 0xdc, 0xc8]));
}

/// A file written before the paper reads with the paper of a new pattern.
#[test]
fn a_file_with_no_paper_reads_with_the_default() {
    let text = file(1).replace("  \"paper\": \"#e6dcc8\",\n", "");

    assert_eq!(
        Pattern::from_json(&text).expect("it reads").paper(),
        DEFAULT_PAPER
    );
}

#[test]
fn a_paper_that_is_no_colour_is_refused() {
    let text = file(1).replace("\"#e6dcc8\"", "\"linen\"");

    assert!(matches!(
        Pattern::from_json(&text),
        Err(PatternError::Colour { .. })
    ));
}

#[test]
fn format_1_reads_its_name() {
    assert_eq!(
        Pattern::from_json(&file(1)).expect("it reads").name(),
        "Ромашки"
    );
}

/// A pattern renamed is the same work, and no longer equal.
#[test]
fn a_renamed_pattern_is_the_same_work() {
    let pattern = Pattern::from_json(&file(1)).expect("it reads");
    let mut renamed = pattern.clone();
    renamed.set_name("Васильки");

    assert!(renamed.same_work(&pattern));
    assert_ne!(renamed, pattern);
    renamed.set_paper(Rgb([0, 0, 0]));
    assert!(!renamed.same_work(&pattern));
}

#[test]
fn a_locked_pattern_is_written_and_read_locked() {
    let mut pattern = Pattern::from_json(&file(1)).expect("format 1 reads");
    assert!(!pattern.locked(), "a file with no lock is not locked");

    pattern.set_locked(true);
    let text = pattern.to_json().expect("it is written");
    assert!(text.contains("\"locked\": true"), "{text}");
    assert!(Pattern::from_json(&text).expect("it reads").locked());

    pattern.set_locked(false);
    let text = pattern.to_json().expect("it is written");
    assert!(!text.contains("locked"), "an unlocked pattern says nothing");
}
