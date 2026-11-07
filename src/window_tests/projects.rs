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
    // The list comes a frame after it is asked for, its pictures two
    // frames after that.
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .with_max_steps(8)
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

    window.card_menu(&name);
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

    tap(&mut window, &t!("projects.export_chosen"));

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

/// A long press chooses a card, it opens no menu; a tap then marks another,
/// and the menu has no "Select".
#[test]
fn a_long_press_starts_the_choice() {
    let mut window = list();
    for _ in 0..2 {
        window.click(&t!("projects.new"));
        window.click(&t!("projects.back"));
    }
    // Both are named by the same minute: the cards are told by their order.
    let name = window.harness.state().projects.entries[0]
        .pattern
        .name()
        .to_owned();

    window
        .harness
        .get_all_by_label(&name)
        .next()
        .expect("a card")
        .click_secondary();
    window.harness.run_steps(5);
    let projects = &window.harness.state().projects;
    assert_eq!(projects.chosen.as_ref().map(|chosen| chosen.len()), Some(1));
    assert!(!projects.menu_open(), "no menu");

    window
        .harness
        .get_all_by_label(&name)
        .nth(1)
        .expect("a card")
        .click();
    window.harness.run_steps(5);
    let projects = &window.harness.state().projects;
    assert_eq!(projects.chosen.as_ref().map(|chosen| chosen.len()), Some(2));
    assert_eq!(window.harness.state().screen, Screen::List, "nothing opens");
}

/// The menu of two chosen has no rename and no duplicate; its items say the
/// count; its export is one .obusiny; its lock locks both.
#[test]
fn the_menu_of_the_chosen_says_their_count() {
    let mut window = list();
    for _ in 0..2 {
        window.click(&t!("projects.new"));
        window.click(&t!("projects.back"));
    }
    let ids: std::collections::BTreeSet<_> = window
        .harness
        .state()
        .projects
        .entries
        .iter()
        .map(|entry| entry.pattern.id())
        .collect();
    window.harness.state_mut().projects.chosen = Some(ids);
    window.harness.state_mut().projects.saver.hold();
    window.harness.run();
    tap(&mut window, &t!("projects.menu"));

    for gone in [t!("projects.rename"), t!("projects.duplicate")] {
        assert!(window.harness.query_by_label(&gone).is_none(), "{gone}");
    }
    let what = crate::ui::projects::plural(2, crate::ui::projects::PATTERNS_OF);
    window.harness.get_by_label(&t!(
        "projects.to_trash_many",
        count = 2,
        what = what.clone()
    ));
    tap(
        &mut window,
        &t!("projects.export_many", count = 2, what = what.clone()),
    );
    let kept = window.harness.state().projects.saver.kept().to_vec();
    assert!(kept[0].0.ends_with(".obusiny"), "{}", kept[0].0);
    assert_eq!(
        crate::library::archive::read(&kept[0].1)
            .expect("it reads")
            .len(),
        2
    );

    tap(&mut window, &t!("projects.menu"));
    tap(
        &mut window,
        &t!("projects.lock_many", count = 2, what = what),
    );
    let entries = &window.harness.state().projects.entries;
    assert!(entries.iter().all(|entry| entry.pattern.locked()));
}

/// The lock of the bar of the choice locks the chosen; then it unlocks them.
#[test]
fn the_lock_of_the_bar_locks_and_unlocks() {
    let mut window = list();
    window.click(&t!("projects.new"));
    window.click(&t!("projects.back"));
    let id = window.harness.state().projects.entries[0].pattern.id();
    window.harness.state_mut().projects.chosen = Some([id].into());
    window.harness.run();

    tap(&mut window, &t!("projects.lock_chosen"));
    assert!(window.harness.state().projects.entries[0].pattern.locked());
    tap(&mut window, &t!("projects.unlock_chosen"));
    assert!(!window.harness.state().projects.entries[0].pattern.locked());
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

/// Escape in the trash goes back to the list, as its "Back" does, and as the
/// way back of android does.
#[test]
fn escape_goes_back_from_the_trash() {
    let mut window = list();
    window.harness.state_mut().projects.view = crate::ui::projects::View::Trash;
    window.harness.run();

    window.harness.key_press(egui::Key::Escape);
    window.harness.run();

    assert_eq!(
        window.harness.state().projects.view,
        crate::ui::projects::View::List
    );
}

/// The list stands inside the safe area that android gives: the cut out of
/// the camera on top, the bar of the system at the bottom.
#[test]
fn the_list_keeps_out_of_the_cut_out() {
    let mut window = list();
    let insets = egui::SafeAreaInsets(egui::epaint::MarginF32 {
        left: 30.0,
        top: 40.0,
        right: 0.0,
        bottom: 0.0,
    });
    window.harness.input_mut().safe_area_insets = Some(insets);
    window.harness.run();

    let card = window.harness.get_by_label(&t!("projects.new")).rect();
    assert!(card.top() >= 40.0, "{card:?}");
    assert!(card.left() >= 30.0, "{card:?}");
}

/// A finger held on a card, moved down to each of `moves` in points, then
/// held still. Each event is a frame, a quarter of a second; the pointer
/// moves before the touch, as the hold reads the pointer. Gives whether the
/// card was chosen when the finger rose.
fn hold_a_finger(moves: &[f32]) -> bool {
    use egui::{Event, Modifiers, TouchDeviceId, TouchId, TouchPhase};
    let mut window = list();
    window.click(&t!("projects.new"));
    window.click(&t!("projects.back"));
    let name = window.harness.state().projects.entries[0]
        .pattern
        .name()
        .to_owned();
    let start = window.harness.get_by_label(&name).rect().center();
    let touch = |phase, pos| Event::Touch {
        device_id: TouchDeviceId(1),
        id: TouchId(1),
        phase,
        pos,
        force: None,
    };
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    window.harness.event(touch(TouchPhase::Start, start));
    window.harness.event(Event::PointerMoved(start));
    window.harness.event(button(start, true));
    window.harness.step();
    let mut at = start;
    for down in moves {
        at = start + egui::vec2(0.0, *down);
        window.harness.event(Event::PointerMoved(at));
        window.harness.event(touch(TouchPhase::Move, at));
        window.harness.step();
    }
    window.harness.run_steps(2);
    window.harness.event(touch(TouchPhase::End, at));
    window.harness.event(button(at, false));
    window.harness.run_steps(3);
    let app = window.harness.state();
    assert_eq!(app.screen, Screen::List, "nothing opens");
    app.projects
        .chosen
        .as_ref()
        .is_some_and(|chosen| chosen.len() == 1)
}

/// A finger that drifts a little as it holds a card still chooses it, and
/// its rise does not clear it again.
#[test]
fn a_finger_may_drift_as_it_holds_a_card() {
    assert!(hold_a_finger(&[6.0, 12.0, 18.0]));
}

/// A finger that moves far scrolls; it chooses nothing.
#[test]
fn a_finger_that_moves_far_chooses_nothing() {
    assert!(!hold_a_finger(&[60.0]));
}

/// Patterns of other sizes, whose cards draw beads of other sizes, all get
/// their pictures: they do not draw the textures of each other away.
#[test]
fn cards_of_patterns_of_other_sizes_all_get_pictures() {
    let mut window = list();
    for side in [5, 9, 14] {
        let mut pattern = crate::app::new_pattern();
        pattern.set_id(obusina_pattern::PatternId::new());
        let corner = obusina_pattern::Cell {
            column: side,
            row: side / 2,
        };
        pattern.set(corner, Some(super::id_at(0)));
        pattern.set_sheet(Some(obusina_pattern::Bounds {
            min: obusina_pattern::Cell { column: 0, row: 0 },
            max: corner,
        }));
        let library = window.harness.state().library.as_ref().expect("a library");
        let history = crate::edit::History::default();
        futures::executor::block_on(library.save(&pattern, &history)).expect("saved");
    }
    window.harness.state_mut().projects.refresh();

    for _ in 0..6 {
        window.harness.run_steps(2);
        let _ = window.harness.render();
    }

    assert_eq!(window.harness.state().projects.thumbs.drawn(), 3);
}
