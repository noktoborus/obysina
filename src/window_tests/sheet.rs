//! Tests of the sheet and the work area: its size in the tools, a bead
//! past it, a bead taken away.

use egui::PointerButton;
use egui_kittest::kittest::Queryable;
use obusina_pattern::Cell;
use rust_i18n::t;

use super::{SIZE, id_at, window};
use crate::app::App;

/// The sheet starts as the cells of the screen, and the two fields of the
/// tools show its size.
#[test]
fn the_sheet_starts_as_the_screen() {
    let window = window();
    let app = window.harness.state();
    let sheet = app.pattern.sheet().expect("the field was shown");
    let columns = SIZE.x / app.view.bead;
    let rows = SIZE.y / app.view.bead;

    assert!((sheet.width() as f32 - columns).abs() <= 2.0, "{sheet:?}");
    assert!((sheet.height() as f32 - rows).abs() <= 2.0, "{sheet:?}");
    window.harness.get_by_label(&t!("sheet.width"));
    window.harness.get_by_label(&t!("sheet.height"));
}

/// A number typed in a field gives the sheet that size, around its middle.
#[test]
fn a_size_typed_in_the_tools_resizes_the_sheet() {
    let mut window = window();
    let before = window.harness.state().pattern.sheet().expect("a sheet");

    window.harness.get_by_label(&t!("sheet.width")).click();
    window.harness.run();
    window
        .harness
        .get_by_label(&t!("sheet.width"))
        .type_text("12");
    window.harness.key_press(egui::Key::Enter);
    window.harness.run();

    let after = window.harness.state().pattern.sheet().expect("a sheet");
    assert_eq!(after.width(), 12);
    assert_eq!(after.height(), before.height());
    let middle = |sheet: obusina_pattern::Bounds| sheet.min.column + sheet.max.column;
    assert!(
        (middle(after) - middle(before)).abs() <= 1,
        "{before:?} {after:?}"
    );

    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.sheet(), Some(before));
    assert!(!window.harness.state().history.can_undo(), "one step");
}

/// A size smaller than the box around the beads is not taken: the sheet is
/// as small as the beads let it be, and over them.
#[test]
fn the_sheet_is_never_smaller_than_the_beads() {
    let mut window = window();
    let (left, right) = (Cell { column: 2, row: 4 }, Cell { column: 9, row: 6 });
    window.press(left, PointerButton::Primary);
    window.press(right, PointerButton::Primary);

    window.harness.get_by_label(&t!("sheet.width")).click();
    window.harness.run();
    window
        .harness
        .get_by_label(&t!("sheet.width"))
        .type_text("3");
    window.harness.key_press(egui::Key::Enter);
    window.harness.run();

    let sheet = window.harness.state().pattern.sheet().expect("a sheet");
    assert_eq!(sheet.width(), 8, "columns 2 to 9");
    assert!(sheet.contains(left) && sheet.contains(right), "{sheet:?}");
}

/// A bead put past the sheet grows the sheet to hold it, and one undo takes
/// both back.
#[test]
fn a_bead_past_the_sheet_grows_it_in_one_step() {
    let mut window = window();
    let small = obusina_pattern::Bounds {
        min: Cell { column: 2, row: 2 },
        max: Cell { column: 7, row: 7 },
    };
    window.harness.state_mut().pattern.set_sheet(Some(small));
    window.harness.run();
    let past = Cell { column: 9, row: 4 };

    window.press(past, PointerButton::Primary);

    let app = window.harness.state_mut();
    assert!(app.pattern.get(past).is_some());
    let grown = app.pattern.sheet().expect("a sheet");
    assert_eq!(grown, small.with(past));
    app.history.undo(&mut app.pattern);
    assert_eq!(app.pattern.get(past), None);
    assert_eq!(app.pattern.sheet(), Some(small));
    assert!(!app.history.can_undo());
}

/// The sheet is part of the pattern: it is saved and opens again.
#[test]
fn the_sheet_is_saved_with_the_pattern() {
    let mut window = window();
    let sheet = window.harness.state().pattern.sheet().expect("a sheet");
    window.click(&t!("file.save"));

    let path = window.directory.path().join("pattern.json");
    let again = App::open(Some(path), None);

    assert_eq!(again.pattern.sheet(), Some(sheet));
}

/// The field drawn is the work area doubled: the sheet of 51 columns,
/// -25 to 25, reaches to column 51. A press past it does nothing.
#[test]
fn a_press_past_the_field_does_nothing() {
    let mut window = window();
    window.harness.state_mut().view.offset.x -= 1200.0;
    window.harness.run();
    let cell = Cell { column: 70, row: 0 };

    window.press(cell, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(cell), None);
}

/// A bead put in the shade, past the sheet, grows the work area, and the
/// field with it: a cell past the field before is in it after.
#[test]
fn a_bead_in_the_shade_grows_the_work_area() {
    let mut window = window();
    window.harness.state_mut().view.offset.x -= 600.0;
    window.harness.run();
    let shade = Cell { column: 40, row: 0 };
    window.press(shade, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(shade), Some(id_at(0)));

    window.harness.state_mut().view.offset.x -= 600.0;
    window.harness.run();
    let further = Cell { column: 70, row: 0 };
    window.press(further, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(further), Some(id_at(0)));
}

/// The sheet grown by a bead shrinks when the bead goes, to the first screen
/// when no bead is past it, in the same step. Undo gives the bead and the
/// grown sheet back.
#[test]
fn a_bead_taken_away_shrinks_the_sheet_to_the_first_screen() {
    let mut window = window();
    let first = window.harness.state().view.sheet.expect("the first screen");
    window.harness.state_mut().view.offset.x -= 600.0;
    window.harness.run();
    let shade = Cell { column: 40, row: 0 };
    window.press(shade, PointerButton::Primary);
    let grown = window.harness.state().pattern.sheet();
    window.press(shade, PointerButton::Primary);

    let app = window.harness.state_mut();
    assert_eq!(app.pattern.get(shade), None);
    let sheet = app.pattern.sheet().expect("a sheet");
    assert_eq!(
        (sheet.width(), sheet.height()),
        (first.width(), first.height())
    );
    app.history.undo(&mut app.pattern);
    assert_eq!(app.pattern.get(shade), Some(id_at(0)));
    assert_eq!(app.pattern.sheet(), grown);
}

/// The sheet shrinks to the box of the beads left when it is larger than
/// the first screen.
#[test]
fn a_bead_taken_away_shrinks_the_sheet_to_the_beads() {
    let mut window = window();
    let first = window.harness.state().view.sheet.expect("the first screen");
    let left = Cell {
        column: first.min.column,
        row: 0,
    };
    window.press(left, PointerButton::Primary);
    window.harness.state_mut().view.offset.x -= 600.0;
    window.harness.run();
    let near = Cell { column: 40, row: 0 };
    window.press(near, PointerButton::Primary);
    window.harness.state_mut().view.offset.x -= 600.0;
    window.harness.run();
    let far = Cell { column: 70, row: 0 };
    window.press(far, PointerButton::Primary);
    window.press(far, PointerButton::Primary);

    let app = window.harness.state();
    let sheet = app.pattern.sheet().expect("a sheet");
    assert!(sheet.contains(near) && !sheet.contains(far), "{sheet:?}");
    assert_eq!(
        (sheet.min.column, sheet.max.column),
        (left.column, near.column)
    );
    assert_eq!(sheet.height(), first.height());
}
