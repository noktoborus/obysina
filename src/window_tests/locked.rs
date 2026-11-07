//! Tests of a locked pattern: "Forbid changes" of the menu of a card, and a
//! pattern that opens only to be looked at.

use egui::PointerButton;
use egui_kittest::kittest::Queryable;
use rust_i18n::t;

use super::{FREE, Window, window};

/// Back on the list, locks the pattern by the menu of its card and opens it
/// again.
fn locked() -> Window {
    let mut window = window();
    let name = window.harness.state().pattern.name().to_owned();
    window.click(&t!("projects.back"));
    window.harness.run();
    window.harness.get_by_label(&name).click_secondary();
    window.harness.run_steps(5);
    window.harness.get_by_label(&t!("projects.lock")).click();
    window.harness.run_steps(5);
    window.harness.get_by_label(&name).click();
    window.harness.run_steps(5);
    window
}

#[test]
fn a_locked_pattern_opens_only_to_be_looked_at() {
    let mut window = locked();
    let app = window.harness.state();
    assert!(app.pattern.locked());
    assert_eq!(app.screen, crate::session::Screen::Editor);

    for gone in [
        t!("palette.empty"),
        t!("edit.undo"),
        t!("tool.pencil"),
        t!("view.turn_left"),
    ] {
        assert!(window.harness.query_by_label(&gone).is_none(), "{gone}");
    }
    for kept in [
        t!("view.shadows"),
        t!("backdrop.button"),
        t!("settings.open"),
        t!("projects.back"),
    ] {
        window.harness.get_by_label(&kept);
    }

    window.press(FREE, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(FREE), None, "no bead");
}

#[test]
fn the_settings_of_a_locked_pattern_hold_only_the_volume() {
    let mut window = locked();
    window.click(&t!("settings.open"));

    window.harness.get_by_label(&t!("view.glow"));
    for gone in [t!("grid.kind"), t!("ruling.open"), t!("paper.open")] {
        assert!(window.harness.query_by_label(&gone).is_none(), "{gone}");
    }
}

#[test]
fn escape_leaves_a_locked_pattern_and_unlock_lets_it_change() {
    let mut window = locked();
    let name = window.harness.state().pattern.name().to_owned();
    window.harness.key_press(egui::Key::Escape);
    window.harness.run();
    assert_eq!(window.harness.state().screen, crate::session::Screen::List);

    window.harness.get_by_label(&name).click_secondary();
    window.harness.run_steps(5);
    window.harness.get_by_label(&t!("projects.unlock")).click();
    window.harness.run_steps(5);
    window.harness.get_by_label(&name).click();
    window.harness.run_steps(5);
    assert!(!window.harness.state().pattern.locked());
    window.harness.get_by_label(&t!("palette.empty"));
}
