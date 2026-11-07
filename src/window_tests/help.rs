//! Tests of the plate of the keys: F1 on any layer shows the keys of that
//! layer; a tap, F1 or Escape closes it, and nothing under it takes a key.

use egui::Key;
use egui_kittest::kittest::Queryable;
use rust_i18n::t;

use super::{SIZE, window, window_in};
use crate::keys::Layer;

/// F1 on the field shows the keys of the field; a tap on the plate closes it.
#[test]
fn f1_shows_the_keys_of_the_field() {
    let mut window = window();
    window.harness.key_press(Key::F1);
    window.harness.run();
    assert_eq!(window.harness.state().help, Some(Layer::Editor));
    window.harness.get_by_label(&t!("keys.layer.editor"));

    window.harness.get_by_label(&t!("keys.save")).click();
    window.harness.run();
    assert_eq!(window.harness.state().help, None);
}

/// The plate takes the keys: Escape closes it and leaves the field open.
#[test]
fn escape_closes_the_plate_and_nothing_under_it() {
    let mut window = window();
    window.harness.key_press(Key::F1);
    window.harness.run();

    window.harness.key_press(Key::Escape);
    window.harness.run();

    let app = window.harness.state();
    assert_eq!(app.help, None);
    assert_eq!(app.screen, crate::session::Screen::Editor);
}

/// F1 over an open menu shows the keys of a menu; F1 again closes the plate
/// and leaves the menu open.
#[test]
fn f1_over_a_menu_shows_the_keys_of_a_menu() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.harness.key_press(Key::F1);
    window.harness.run();
    assert_eq!(window.harness.state().help, Some(Layer::Menu));

    window.harness.key_press(Key::F1);
    window.harness.run();
    let app = window.harness.state();
    assert_eq!(app.help, None);
    assert!(app.menu.is_open());
}

/// A plate taller than the window still opens, and a tap beside it closes
/// it.
#[test]
fn a_plate_in_a_short_window_closes_by_a_tap_beside_it() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut window = window_in(directory, egui::vec2(SIZE.x, 200.0));
    window.click(&t!("settings.open"));
    window.harness.key_press(Key::F1);
    window.harness.run();
    assert_eq!(window.harness.state().help, Some(Layer::Menu));

    window.harness.drag_at(egui::pos2(5.0, 5.0));
    window.harness.drop_at(egui::pos2(5.0, 5.0));
    window.harness.run();
    assert_eq!(window.harness.state().help, None);
}
