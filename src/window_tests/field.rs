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

/// Opens the menu of the ruling from the menu of the settings.
fn open_ruling(window: &mut Window) {
    window.click(&t!("settings.open"));
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

/// The numbers change the ruling at once and it stays; leaving the level
/// keeps it as one step of undo, and the menu stays, on its top.
#[test]
fn a_changed_ruling_is_one_step_when_its_level_is_left() {
    let mut window = window();
    open_ruling(&mut window);

    type_number(&mut window, &t!("ruling.step_x"), "10");
    type_number(&mut window, &t!("ruling.offset_y"), "-2");
    let shown = Ruling {
        step: [10, 5],
        offset: [0, -2],
    };
    assert_eq!(window.harness.state().pattern.ruling(), shown);
    window.harness.key_press(egui::Key::Escape);
    window.harness.run();

    let app = window.harness.state();
    assert!(app.menu.is_open(), "Escape left the level only");
    assert!(app.menu.level().is_empty());
    assert_eq!(app.pattern.ruling(), shown);
    window.close_menu();
    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.ruling(), Ruling::default());
    assert!(!window.harness.state().history.can_undo(), "one step");
}

/// A press beside the menu keeps the ruling of the level stood in.
#[test]
fn a_ruling_stays_when_the_menu_closes_in_its_level() {
    let mut window = window();
    open_ruling(&mut window);
    type_number(&mut window, &t!("ruling.step_y"), "3");

    let corner = egui::pos2(SIZE.x - 5.0, SIZE.y - 5.0);
    super::palette::press_at(&mut window, corner);

    let app = window.harness.state();
    assert!(!app.menu.is_open());
    assert_eq!(app.pattern.ruling().step, [5, 3]);
    assert!(app.history.can_undo(), "one step");
}

/// "Put the old ruling back" is offered once the ruling changed; it puts
/// back the one the level was entered with, and the menu stays in the
/// level. Nothing is kept then.
#[test]
fn the_old_ruling_is_put_back() {
    let mut window = window();
    open_ruling(&mut window);
    let revert = t!("ruling.revert").to_string();
    let disabled = |window: &Window| {
        use egui_kittest::kittest::NodeT;
        let node = window.harness.get_by_label(&revert);
        node.accesskit_node().is_disabled()
    };
    assert!(disabled(&window), "nothing to put back");

    type_number(&mut window, &t!("ruling.step_y"), "3");
    assert!(!disabled(&window), "offered once it changed");
    window.click(&revert);

    let app = window.harness.state();
    assert_eq!(app.pattern.ruling(), Ruling::default());
    assert_eq!(app.menu.level(), ["ruling"], "the menu stays in the level");
    window.close_menu();
    assert!(!window.harness.state().history.can_undo(), "no step");
}

/// A swatch under the picker changes the background at once; the way back
/// out of its level keeps it as one step of undo.
#[test]
fn a_changed_background_is_one_step() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("paper.open"));

    window.click(&t!("paper.black"));
    let black = obusina_pattern::Rgb([0x1e, 0x1e, 0x1e]);
    assert_eq!(window.harness.state().pattern.paper(), black);
    window.close_menu();

    assert_eq!(window.harness.state().pattern.paper(), black);
    window.click(&t!("edit.undo"));
    let app = window.harness.state();
    assert_eq!(app.pattern.paper(), obusina_pattern::DEFAULT_PAPER);
    assert!(!app.history.can_undo(), "one step");
}

/// "Put the old colour back" puts back the background the level was
/// entered with.
#[test]
fn the_old_background_is_put_back() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("paper.open"));
    window.click(&t!("paper.linen"));

    window.click(&t!("paper.revert"));
    window.close_menu();

    let app = window.harness.state();
    assert_eq!(app.pattern.paper(), obusina_pattern::DEFAULT_PAPER);
    assert!(!app.history.can_undo());
}

/// The glow is on at first; its entry turns it off, the menu stays open on
/// it, and the next start keeps it off.
#[test]
fn the_glow_is_kept_until_the_next_start() {
    let mut window = window();
    assert!(window.harness.state().settings.glow());

    window.click(&t!("settings.open"));
    window.click(&t!("view.glow"));
    assert!(!window.harness.state().settings.glow());
    assert!(window.harness.state().menu.is_open(), "it opens again");
    let Window { harness, directory } = window;
    drop(harness);

    let window = window_in(directory, SIZE);
    assert!(!window.harness.state().settings.glow());
}

/// "Palette settings" closes the settings and opens the window of the
/// palettes; "Return to the list" goes back to the list.
#[test]
fn the_settings_lead_to_the_palettes_and_to_the_list() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("palette.settings"));
    let app = window.harness.state();
    assert!(!app.menu.is_open());
    assert!(app.palette_window.open);

    let mut window = super::window();
    window.click(&t!("settings.open"));
    window.click(&t!("settings.to_list"));
    assert_eq!(window.harness.state().screen, crate::session::Screen::List);
}

/// "Close the settings" closes them, and nothing changes.
#[test]
fn the_settings_close_by_their_entry() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("settings.close"));

    let app = window.harness.state();
    assert!(!app.menu.is_open());
    assert!(!app.history.can_undo());
}
