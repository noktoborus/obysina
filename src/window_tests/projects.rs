//! Tests of the list of patterns: the window opens on it, a pattern opens
//! from it and comes back to it saved, the trash and the export.

use egui::PointerButton;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use rust_i18n::t;

use super::{FREE, SIZE, Window, places, window};
use crate::app::App;
use crate::session::Screen;

/// Clicks the widget named `name` while toasts move, which never settle.
fn tap(window: &mut Window, name: &str) {
    window.harness.get_by_label(name).click();
    window.harness.run_steps(5);
}

/// The window on the list kept in a scratch directory.
fn list() -> Window {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let places = places(directory.path());
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .wgpu()
        .build_eframe(move |_| App::start(places));
    harness.run();
    Window { harness, directory }
}

#[test]
fn the_window_opens_on_the_list() {
    let window = list();

    assert_eq!(window.harness.state().screen, Screen::List);
    window.harness.get_by_label(&t!("projects.new"));
    window.harness.get_by_label(&t!("projects.import"));
}

/// "New pattern" opens a pattern named by the time; back on the list, its
/// card is there, and a tap opens it again.
#[test]
fn a_new_pattern_comes_back_to_the_list() {
    let mut window = list();
    window.click(&t!("projects.new"));
    assert_eq!(window.harness.state().screen, Screen::Editor);
    let name = window.harness.state().pattern.name().to_owned();
    assert!(!name.is_empty(), "named by the time");
    window.press(FREE, PointerButton::Primary);

    window.click(&t!("projects.back"));
    window.harness.run();
    window.click(&name);

    let app = window.harness.state();
    assert_eq!(app.screen, Screen::Editor);
    assert_eq!(app.pattern.count(super::id_at(0)), 1);
    assert!(app.history.can_undo(), "with its history");
}

/// Escape goes back to the list when nothing else takes it.
#[test]
fn escape_goes_back_to_the_list() {
    let mut window = window();

    window.harness.key_press(egui::Key::Escape);
    window.harness.run();

    assert_eq!(window.harness.state().screen, Screen::List);
}

/// The menu of a card moves it to the trash; "Undo" of the toast puts it
/// back.
#[test]
fn a_pattern_moved_to_the_trash_comes_back_by_undo() {
    let mut window = window();
    let name = window.harness.state().pattern.name().to_owned();
    window.click(&t!("projects.back"));
    window.harness.run();

    window.harness.get_by_label(&name).click_secondary();
    window.harness.run();
    tap(&mut window, &t!("projects.to_trash"));
    assert!(window.harness.state().projects.entries.is_empty());
    assert_eq!(window.harness.state().projects.trash.len(), 1);

    tap(&mut window, &t!("projects.undo"));
    assert_eq!(window.harness.state().projects.entries.len(), 1);
    assert!(window.harness.state().projects.trash.is_empty());
}

/// "Export all" saves one file of every pattern, which imports back.
#[test]
fn export_all_saves_a_file_that_imports_back() {
    let mut window = window();
    window.click(&t!("projects.back"));
    window.harness.run();
    window.harness.state_mut().projects.saver.hold();
    window
        .harness
        .get_by_label_contains(&t!("projects.export_all"))
        .click();
    window.harness.run_steps(5);

    let kept = window.harness.state().projects.saver.kept().to_vec();
    assert_eq!(kept.len(), 1);
    assert!(kept[0].0.ends_with(".obusiny"), "{}", kept[0].0);
    let patterns = crate::library::archive::read(&kept[0].1).expect("it reads");
    assert_eq!(patterns.len(), 1);
}

/// The picture of a card is drawn by the renderer of the window once every
/// texture of its beads is on the chip, and kept.
#[test]
fn a_card_gets_its_picture() {
    let mut window = list();
    window.click(&t!("projects.new"));
    window.press(FREE, PointerButton::Primary);
    window.click(&t!("projects.back"));

    for _ in 0..3 {
        window.harness.run_steps(2);
        let _ = window.harness.render();
    }

    assert_eq!(window.harness.state().projects.thumbs.drawn(), 1);
}

/// While patterns are chosen, the row of the trash and the export goes;
/// the bar on top exports the chosen.
#[test]
fn the_chosen_are_exported_from_the_bar() {
    let mut window = list();
    for _ in 0..2 {
        window.click(&t!("projects.new"));
        window.click(&t!("projects.back"));
    }
    let first = window.harness.state().projects.entries[0].pattern.id();
    window.harness.state_mut().projects.chosen = Some([first].into());
    window.harness.state_mut().projects.saver.hold();
    window.harness.run();
    let all = t!("projects.export_all");
    assert!(window.harness.query_by_label_contains(&all).is_none());

    tap(&mut window, &t!("projects.export"));

    let kept = window.harness.state().projects.saver.kept().to_vec();
    assert!(
        kept[0].0.ends_with(".obusina"),
        "one pattern: {}",
        kept[0].0
    );
    let patterns = crate::library::archive::read(&kept[0].1).expect("it reads");
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].id(), first);
}

/// The arrows walk the cards, from the first: "New pattern", then
/// "Import"; the card the keys stand on is drawn filled.
#[test]
fn the_arrows_walk_the_cards() {
    let mut window = list();

    window.harness.key_press(egui::Key::ArrowRight);
    window.harness.run();
    assert_eq!(window.harness.state().projects.focus, Some(0));

    window.harness.key_press(egui::Key::ArrowRight);
    window.harness.run();
    assert_eq!(window.harness.state().projects.focus, Some(1));
}

/// F1 on the list shows the keys of the list.
#[test]
fn f1_shows_the_keys_of_the_list() {
    let mut window = list();
    window.harness.key_press(egui::Key::F1);
    window.harness.run();
    assert_eq!(window.harness.state().help, Some(crate::keys::Layer::List));
}
