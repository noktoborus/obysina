//! Tests of the size of the field: by the beads, or set by hand.

use egui::PointerButton;
use egui_kittest::kittest::Queryable;
use obusina_pattern::{Bounds, Cell};
use rust_i18n::t;

use super::{Window, id_at, window};

/// Opens the level of the field of the menu of the settings.
fn open_field(window: &mut Window) {
    window.click(&t!("settings.open"));
    window.click(&t!("field.open"));
}

/// Types `text` into the number field named `name` and takes it.
fn type_number(window: &mut Window, name: &str, text: &str) {
    window.harness.get_by_label(name).click();
    window.harness.run();
    window.harness.get_by_label(name).type_text(text);
    window.harness.key_press(egui::Key::Enter);
    window.harness.run();
}

/// A field of columns 2 to 7 and rows 2 to 7, set by hand.
const SMALL: Bounds = Bounds {
    min: Cell { column: 2, row: 2 },
    max: Cell { column: 7, row: 7 },
};

/// Sets the field by hand, past the history, as a file opened with it.
fn set_by_hand(window: &mut Window, field: Bounds) {
    window.harness.state_mut().pattern.set_field(Some(field));
    window.harness.run();
}

/// A new pattern has its size by the beads: no field, no work area.
#[test]
fn a_new_pattern_is_sized_by_the_beads() {
    let window = window();
    let app = window.harness.state();

    assert_eq!(app.pattern.field(), None);
    assert_eq!(app.pattern.work_area(), None);
}

/// By the beads a bead goes anywhere, and each grows the work area, their
/// box: step by step a bead goes far from the first.
#[test]
fn a_bead_goes_anywhere_by_the_beads() {
    let mut window = window();
    let near = Cell { column: 0, row: 0 };
    window.press(near, PointerButton::Primary);
    let far = Cell { column: 70, row: 0 };
    for column in (10..=far.column).step_by(10) {
        window.harness.state_mut().view.offset.x -= 200.0;
        window.harness.run();
        window.press(Cell { column, row: 0 }, PointerButton::Primary);
    }

    let app = window.harness.state();
    assert_eq!(app.pattern.get(far), Some(id_at(0)));
    assert_eq!(app.pattern.work_area(), Some(Bounds::of(near).with(far)));
}

/// By hand a press past the field puts no bead, and keeps no step.
#[test]
fn a_press_past_a_field_set_by_hand_does_nothing() {
    let mut window = window();
    set_by_hand(&mut window, SMALL);
    let past = Cell { column: 9, row: 4 };

    window.press(past, PointerButton::Primary);

    let app = window.harness.state();
    assert_eq!(app.pattern.get(past), None);
    assert_eq!(app.pattern.field(), Some(SMALL));
    assert!(!app.history.can_undo());
}

/// A field made smaller than the beads keeps the beads past it. The panel
/// counts the beads in the field; the entry that deletes them names those
/// past it.
#[test]
fn beads_past_a_smaller_field_stay_and_are_counted_apart() {
    let mut window = window();
    let inside = Cell { column: 3, row: 3 };
    let past = Cell { column: 9, row: 4 };
    window.press(inside, PointerButton::Primary);
    window.press(past, PointerButton::Primary);
    set_by_hand(&mut window, SMALL);

    let app = window.harness.state();
    assert_eq!(app.pattern.get(past), Some(id_at(0)));
    assert_eq!(app.pattern.count_in_field(id_at(0)), 1);
    let entry = crate::ui::palette_window::delete_beads(2, 1);
    assert!(entry.contains(&t!("palette.past_field", count = 1).to_string()));
}

/// The field set by hand is part of the pattern: it is saved and opens
/// again.
#[test]
fn the_field_is_saved_with_the_pattern() {
    let mut window = window();
    set_by_hand(&mut window, SMALL);
    let again = super::reopened(&mut window);

    assert_eq!(again.pattern.field(), Some(SMALL));
}

/// A number typed in the menu gives the field set by hand that size, around
/// its middle; it is kept as one step of undo when the menu closes.
#[test]
fn a_size_typed_in_the_menu_resizes_the_field() {
    let mut window = window();
    set_by_hand(&mut window, SMALL);

    open_field(&mut window);
    type_number(&mut window, &t!("field.width"), "12");
    window.close_menu();

    let after = window.harness.state().pattern.field().expect("a field");
    assert_eq!((after.width(), after.height()), (12, SMALL.height()));
    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.field(), Some(SMALL));
    assert!(!window.harness.state().history.can_undo(), "one step");
}

/// "Put the old size back" puts back the field the level was entered with,
/// and no step is kept.
#[test]
fn the_old_size_is_put_back() {
    let mut window = window();
    set_by_hand(&mut window, SMALL);

    open_field(&mut window);
    type_number(&mut window, &t!("field.width"), "12");
    window.click(&t!("field.revert"));
    window.close_menu();

    let app = window.harness.state();
    assert_eq!(app.pattern.field(), Some(SMALL));
    assert!(!app.history.can_undo());
}

/// "Set" on a pattern sized by the beads takes the first screen grown to
/// hold the beads, and shows the numbers; "By the beads" takes the field
/// away. Left, the level keeps one step of undo.
#[test]
fn set_takes_the_first_screen_grown_to_hold_the_beads() {
    let mut window = window();
    let far = Cell { column: 3, row: 2 };
    window.press(far, PointerButton::Primary);
    let app = window.harness.state();
    let first = app
        .view
        .first_screen(app.pattern.grid(), app.pattern.ratio());
    assert!(window.harness.query_by_label(&t!("field.width")).is_none());

    open_field(&mut window);
    window.click(&t!("field.set"));

    let field = window.harness.state().pattern.field().expect("a field");
    assert_eq!(field, first.union(Bounds::of(far)));
    assert!(window.harness.query_by_label(&t!("field.width")).is_some());

    window.click(&t!("field.by_beads"));
    assert_eq!(window.harness.state().pattern.field(), None);
    window.click(&t!("field.set"));
    window.close_menu();

    let app = window.harness.state_mut();
    assert_eq!(app.pattern.field(), Some(field));
    app.history.undo(&mut app.pattern);
    assert_eq!(app.pattern.field(), None, "one step back to the beads");
}

/// A size smaller than the beads is taken: the beads past it stay.
#[test]
fn a_field_made_smaller_than_the_beads_keeps_them() {
    let mut window = window();
    let past = Cell { column: 9, row: 4 };
    window.press(past, PointerButton::Primary);
    set_by_hand(&mut window, SMALL);

    open_field(&mut window);
    type_number(&mut window, &t!("field.width"), "2");
    window.close_menu();

    let app = window.harness.state();
    assert_eq!(app.pattern.field().map(|field| field.width()), Some(2));
    assert_eq!(app.pattern.get(past), Some(id_at(0)));
}

/// "As it was" puts back a size by the beads after "Set".
#[test]
fn as_it_was_puts_back_the_size_by_the_beads() {
    let mut window = window();

    open_field(&mut window);
    window.click(&t!("field.set"));
    window.click(&t!("field.revert"));
    window.close_menu();

    let app = window.harness.state();
    assert_eq!(app.pattern.field(), None);
    assert!(!app.history.can_undo());
}
