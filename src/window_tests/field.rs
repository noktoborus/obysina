//! Tests of what the field shows: the shadows of the empty places and the
//! ruling of the pattern.

use egui_kittest::kittest::Queryable;
use obusina_pattern::Ruling;
use rust_i18n::t;

use super::{SIZE, Window, window, window_in};

/// Types `text` into the number field named `name` and takes it.
fn type_number(window: &mut Window, name: &str, text: &str) {
    window.harness.get_by_label(name).click();
    window.harness.run();
    window.harness.get_by_label(name).type_text(text);
    window.harness.key_press(egui::Key::Enter);
    window.harness.run();
}

/// Opens the menu of the ruling from the menu of grids.
fn open_ruling(window: &mut Window) {
    window.click(&t!("grid.kind"));
    window.click(&t!("ruling.open"));
}

#[test]
fn the_button_hides_the_shadows_until_the_next_start() {
    let mut window = window();
    assert!(window.harness.state().settings.shadows());

    window.click(&t!("view.shadows"));
    assert!(!window.harness.state().settings.shadows());
    let Window { harness, directory } = window;
    drop(harness);

    let mut window = window_in(directory, SIZE);
    assert!(!window.harness.state().settings.shadows());
    window.click(&t!("view.shadows"));
    assert!(window.harness.state().settings.shadows());
}

/// The numbers change the ruling at once; "Accept" keeps it as one step of
/// undo.
#[test]
fn an_accepted_ruling_is_one_step() {
    let mut window = window();
    open_ruling(&mut window);

    type_number(&mut window, &t!("ruling.step_x"), "10");
    type_number(&mut window, &t!("ruling.offset_y"), "-2");
    let shown = Ruling {
        step: [10, 5],
        offset: [0, -2],
    };
    assert_eq!(window.harness.state().pattern.ruling(), shown);
    window.click(&t!("ruling.accept"));

    assert_eq!(window.harness.state().pattern.ruling(), shown);
    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.ruling(), Ruling::default());
    assert!(!window.harness.state().history.can_undo(), "one step");
}

/// "Cancel", Escape and a press beside the menu put the ruling back.
#[test]
fn a_ruling_not_accepted_goes_back() {
    for leave in ["cancel", "escape", "beside"] {
        let mut window = window();
        open_ruling(&mut window);
        type_number(&mut window, &t!("ruling.step_y"), "3");
        assert_eq!(window.harness.state().pattern.ruling().step, [5, 3]);

        match leave {
            "cancel" => window.click(&t!("ruling.cancel")),
            "escape" => {
                window.harness.key_press(egui::Key::Escape);
                window.harness.run();
            }
            _ => {
                let corner = egui::pos2(SIZE.x - 5.0, SIZE.y - 5.0);
                super::palette::press_at(&mut window, corner);
            }
        }

        let app = window.harness.state();
        assert!(!app.menu.is_open(), "{leave}");
        assert_eq!(app.pattern.ruling(), Ruling::default(), "{leave}");
        assert!(!app.history.can_undo(), "{leave}");
    }
}
