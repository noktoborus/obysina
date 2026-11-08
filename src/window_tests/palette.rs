//! Tests of the palettes in the window: the window of the palettes, opened
//! by the menu of the colour in hand or by `P`, and the changes it makes.

use egui::{Key, PointerButton};
use egui_kittest::kittest::Queryable;
use rust_i18n::t;

use super::{FREE, SIZE, id_at, swatch, window, window_in};
use crate::settings::{PaletteView, PanelView};
use crate::ui::palette_window::delete_beads;

/// The window of the palettes opened by `P`.
fn opened() -> super::Window {
    let mut window = window();
    window.harness.key_press(Key::P);
    window.harness.run();
    assert!(window.harness.state().palette_window.open);
    window
}

/// The name of a colour in the window.
fn colour(number: usize) -> String {
    t!("palette.colour", number = number).to_string()
}

/// Closes the window of the palettes by Escape.
fn close(window: &mut super::Window) {
    window.harness.key_press(Key::Escape);
    window.harness.run();
    assert!(!window.harness.state().palette_window.open);
}

/// Taps a place of the window, then an entry of its menu.
fn choose(window: &mut super::Window, place: &str, entry: &str) {
    window.click(place);
    window.click(entry);
}

/// A second tap on the colour in hand opens nothing; the button of the
/// palette opens the window of the palettes.
#[test]
fn a_second_tap_opens_nothing_and_the_palette_button_opens_the_window() {
    let mut window = window();
    let first = swatch(window.harness.state(), 0);

    window.click(&first);
    window.click(&first);
    assert!(!window.harness.state().menu.is_open());
    assert!(!window.harness.state().palette_window.open);
    assert_eq!(window.harness.state().colour, Some(id_at(0)));

    window.click(&t!("palette.settings"));
    assert!(window.harness.state().palette_window.open);
}

#[test]
fn a_new_colour_stays_when_accepted_and_is_undone() {
    let mut window = opened();
    let place = t!("palette.place", number = 6).to_string();

    choose(&mut window, &place, &t!("palette.accept"));
    let places =
        |window: &super::Window| window.harness.state().pattern.palettes()[0].places()[5].is_some();
    assert!(places(&window));
    close(&mut window);

    let app = window.harness.state_mut();
    app.history.undo(&mut app.pattern);
    window.harness.run();
    assert!(!places(&window));
}

#[test]
fn a_new_colour_not_accepted_is_gone() {
    let mut window = opened();
    let place = t!("palette.place", number = 6).to_string();

    window.click(&place);
    assert!(window.harness.state().pattern.palettes()[0].places()[5].is_some());
    window.harness.key_press(Key::Escape);
    window.harness.run();

    assert!(window.harness.state().pattern.palettes()[0].places()[5].is_none());
    close(&mut window);
    assert!(!window.harness.state().history.can_undo());
}

#[test]
fn cancel_at_the_end_of_the_menu_puts_the_colour_back() {
    let mut window = opened();
    let place = t!("palette.place", number = 6).to_string();

    window.click(&place);
    assert!(window.harness.state().pattern.palettes()[0].places()[5].is_some());
    window.click(&t!("palette.cancel"));

    let app = window.harness.state();
    assert!(app.pattern.palettes()[0].places()[5].is_none());
    assert!(app.palette_window.open, "the window stays");
    assert!(
        window
            .harness
            .query_by_label(&t!("palette.cancel"))
            .is_none()
    );
}

#[test]
fn the_changes_of_the_window_are_one_step_made_when_it_closes() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    let before = window.harness.state().pattern.palettes().to_vec();
    window.harness.key_press(Key::P);
    window.harness.run();
    let place = t!("palette.place", number = 6).to_string();
    choose(&mut window, &place, &t!("palette.accept"));
    choose(&mut window, &colour(1), &t!("palette.remove"));
    let removed = t!("palette.removed", number = 1).to_string();
    choose(&mut window, &removed, &delete_beads(1));
    assert!(!window.harness.state().history.can_undo(), "no step yet");

    close(&mut window);
    let app = window.harness.state_mut();
    assert_eq!(app.pattern.get(FREE), None);
    app.history.undo(&mut app.pattern);

    assert_eq!(app.pattern.palettes(), before.as_slice());
    assert_eq!(app.pattern.get(FREE), Some(id_at(0)), "the bead is back");
    app.history.undo(&mut app.pattern);
    assert_eq!(app.pattern.get(FREE), None, "the next step is the bead");
}

#[test]
fn a_removed_colour_keeps_its_beads_and_comes_back_by_undo() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.harness.key_press(Key::P);
    window.harness.run();

    choose(&mut window, &colour(1), &t!("palette.remove"));

    let app = window.harness.state();
    assert!(
        app.pattern
            .colour(id_at(0))
            .and_then(|found| found.rgb)
            .is_none()
    );
    assert_eq!(app.pattern.get(FREE), Some(id_at(0)));
    assert!(
        window
            .harness
            .query_by_label(&t!("palette.removed", number = 1))
            .is_some()
    );
    close(&mut window);

    let app = window.harness.state_mut();
    app.history.undo(&mut app.pattern);
    assert!(
        app.pattern
            .colour(id_at(0))
            .and_then(|found| found.rgb)
            .is_some()
    );
}

#[test]
fn a_removed_colour_deletes_only_its_beads() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.harness.key_press(Key::P);
    window.harness.run();
    choose(&mut window, &colour(1), &t!("palette.remove"));

    let removed = t!("palette.removed", number = 1).to_string();
    choose(&mut window, &removed, &delete_beads(1));

    let app = window.harness.state();
    assert_eq!(app.pattern.count(id_at(0)), 0);
    assert!(app.pattern.colour(id_at(0)).is_some(), "the place stays");
}

#[test]
fn the_first_move_makes_a_palette_and_the_keys_switch_it() {
    let mut window = opened();

    choose(&mut window, &colour(2), &t!("palette.move"));

    assert_eq!(window.harness.state().pattern.palettes().len(), 2);
    window
        .harness
        .get_by_label(&t!("palette.number", number = 1, count = 2));
    window.harness.key_press(Key::Plus);
    window.harness.run();
    assert_eq!(window.harness.state().palette, 1);
    window.harness.key_press(Key::Minus);
    window.harness.run();
    assert_eq!(window.harness.state().palette, 0);
    let other = t!("palette.switch").to_string();
    let panel = window.panel(&t!("panel.palette"));
    let in_window = window
        .harness
        .query_all_by_label(&other)
        .find(|node| !panel.contains_rect(node.rect()))
        .expect("the switch of the window, out of the panel");
    in_window.click();
    window.harness.run();
    assert_eq!(window.harness.state().palette, 1);
}

#[test]
fn a_later_move_asks_where() {
    let mut window = opened();
    choose(&mut window, &colour(2), &t!("palette.move"));

    choose(&mut window, &colour(1), &t!("palette.move"));
    window
        .harness
        .get_by_label(&t!("palette.where", number = 1));
    window.click(&t!("palette.to", number = 2));

    let palettes = window.harness.state().pattern.palettes();
    assert_eq!(palettes.len(), 2);
    assert_eq!(
        palettes[1].places()[1].as_ref().map(|found| found.id),
        Some(id_at(0))
    );
}

#[test]
fn a_colour_dragged_onto_another_swaps_with_it() {
    let mut window = opened();
    let to = window.harness.get_by_label(&colour(3)).rect().center();

    window.drag(&colour(1), to);

    let places = window.harness.state().pattern.palettes()[0].places();
    assert_eq!(places[2].as_ref().map(|found| found.id), Some(id_at(0)));
    assert_eq!(places[0].as_ref().map(|found| found.id), Some(id_at(2)));
    close(&mut window);
    assert!(window.harness.state().history.can_undo());
}

#[test]
fn the_tab_of_the_block_shows_every_place() {
    let mut window = opened();
    let seventh = t!("palette.place", number = 7).to_string();
    assert!(window.harness.query_by_label(&seventh).is_none());

    window.click(&t!("palette.block"));

    for number in 6..=9 {
        window
            .harness
            .get_by_label(&t!("palette.place", number = number));
    }
}

/// The menu of a place of the window hides a colour, shows it again and
/// deletes its beads.
#[test]
fn the_menu_of_a_place_hides_a_colour_and_deletes_its_beads() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.harness.key_press(Key::P);
    window.harness.run();
    let hidden = |window: &super::Window| {
        window
            .harness
            .state()
            .pattern
            .colour(id_at(0))
            .is_some_and(|found| found.hidden)
    };

    choose(&mut window, &colour(1), &t!("palette.hide"));
    assert!(hidden(&window));
    let hidden_one = t!("palette.hidden_colour", number = 1).to_string();
    choose(&mut window, &hidden_one, &t!("palette.show"));
    assert!(!hidden(&window));

    choose(&mut window, &colour(1), &delete_beads(1));
    assert_eq!(window.harness.state().pattern.count(id_at(0)), 0);
}

#[test]
fn a_new_colour_is_none_of_the_others() {
    let mut window = opened();
    let place = t!("palette.place", number = 6).to_string();

    choose(&mut window, &place, &t!("palette.accept"));

    let pattern = &window.harness.state().pattern;
    let colours: Vec<_> = pattern.palettes()[0]
        .places()
        .iter()
        .flatten()
        .filter_map(|found| found.rgb)
        .collect();
    let new = colours[5];
    assert_eq!(
        colours.iter().filter(|rgb| **rgb == new).count(),
        1,
        "{new}"
    );
}

/// A colour moved to a new palette, the window closed: two palettes.
fn two_palettes() -> super::Window {
    let mut window = opened();
    choose(&mut window, &colour(2), &t!("palette.move"));
    window.harness.state_mut().palette_window.open = false;
    window.harness.run();
    window
}

#[test]
fn a_digit_takes_the_colour_of_its_place_and_zero_the_empty_place() {
    let mut window = window();

    window.harness.key_press(Key::Num3);
    window.harness.run();
    assert_eq!(window.harness.state().colour, Some(id_at(2)));

    window.harness.key_press(Key::Num0);
    window.harness.run();
    assert_eq!(window.harness.state().colour, None);

    window.harness.key_press(Key::Num9);
    window.harness.run();
    assert_eq!(window.harness.state().colour, None, "place 9 is empty");
}

#[test]
fn the_keys_switch_the_palette_on_the_main_screen() {
    let mut window = two_palettes();

    window.harness.key_press(Key::Plus);
    window.harness.run();
    assert_eq!(window.harness.state().palette, 1);
    window.harness.key_press(Key::Num1);
    window.harness.run();
    assert_eq!(
        window.harness.state().colour,
        Some(id_at(1)),
        "a digit of that palette"
    );

    window.harness.key_press(Key::Minus);
    window.harness.run();
    assert_eq!(window.harness.state().palette, 0);
}

#[test]
fn the_panel_has_one_switch_for_two_palettes_and_two_for_more() {
    let mut window = window();
    assert!(
        window
            .harness
            .query_by_label(&t!("palette.switch"))
            .is_none()
    );

    let mut window2 = two_palettes();
    window2.click(&t!("palette.switch"));
    assert_eq!(window2.harness.state().palette, 1);
    assert!(
        window2
            .harness
            .query_by_label(&t!("palette.next"))
            .is_none()
    );

    window.harness.key_press(Key::P);
    window.harness.run();
    choose(&mut window, &colour(2), &t!("palette.move"));
    choose(&mut window, &colour(3), &t!("palette.move"));
    window.click(&t!("palette.new"));
    window.harness.state_mut().palette_window.open = false;
    window.harness.run();
    assert_eq!(window.harness.state().pattern.palettes().len(), 3);
    window.click(&t!("palette.next"));
    window.click(&t!("palette.next"));
    window.click(&t!("palette.next"));
    assert_eq!(window.harness.state().palette, 0, "a circle");
}

#[test]
fn the_block_stands_as_a_numpad_and_is_kept() {
    let mut window = window();
    window.view(crate::ui::palette::NAME, PanelView::Block);

    let first = swatch(window.harness.state(), 0);
    let fourth = swatch(window.harness.state(), 3);
    assert!(window.stacked(&fourth, &first), "4 stands over 1");
    let empty = window.harness.get_by_label(&t!("palette.empty")).rect();
    let one = window.harness.get_by_label(&first).rect();
    let palettes = window.harness.get_by_label(&t!("palette.settings")).rect();
    assert_eq!(empty.width(), one.width(), "the 0 is one swatch wide");
    assert!(empty.max.y < palettes.max.y + 1.0 && palettes.min.x > empty.max.x);
    assert!(
        (palettes.center().y - empty.center().y).abs() < 1.0,
        "🎨 beside 0"
    );

    let super::Window { harness, directory } = window;
    drop(harness);
    let again = window_in(directory, SIZE);
    let posture = crate::settings::Posture::of(SIZE);
    let kept = again
        .harness
        .state()
        .settings
        .panel(posture, crate::ui::palette::NAME);
    assert_eq!(kept.map(|(_, view)| view), Some(PanelView::Block));
}

/// The tabs of the window of the palettes change the window, not the panel.
#[test]
fn the_tab_of_the_window_keeps_its_view_for_the_next_start() {
    let mut window = opened();
    window.click(&t!("palette.block"));

    let super::Window { harness, directory } = window;
    drop(harness);
    let again = window_in(directory, SIZE);
    assert_eq!(
        again.harness.state().settings.palette_view(),
        PaletteView::Block
    );
}

/// The panel measures itself again when its content changes: back from a
/// block to a line it is the size it was, and a colour taken away makes it
/// smaller.
#[test]
fn the_panel_fits_its_content_again() {
    let mut window = window();
    let panel = |window: &super::Window| {
        let id = egui::Id::unique(("floating-panel", "palette"));
        window
            .harness
            .ctx
            .memory(|memory| memory.area_rect(id))
            .expect("the panel of beads is drawn")
            .size()
    };
    let line = panel(&window);

    window.view(crate::ui::palette::NAME, PanelView::Block);
    let block = panel(&window);
    window.view(crate::ui::palette::NAME, PanelView::Column);

    assert_ne!(line, block);
    assert_eq!(panel(&window), line, "the size it had before the block");

    window.harness.key_press(Key::P);
    window.harness.run();
    choose(&mut window, &colour(5), &t!("palette.remove"));
    window.harness.state_mut().palette_window.open = false;
    window.harness.run();
    assert!(panel(&window).y < line.y, "one colour less");
}

/// The first button pressed and let go at a point, as a mouse does.
pub(super) fn press_at(window: &mut super::Window, at: egui::Pos2) {
    window.harness.event(egui::Event::PointerMoved(at));
    for pressed in [true, false] {
        window.harness.event(egui::Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        window.harness.step();
    }
    window.harness.run();
}

/// While the picker of a colour is open, a press on the window of the
/// palettes where the picker does not cover it closes the picker and reaches
/// nothing under it.
#[test]
fn a_press_beside_the_picker_closes_it_and_nothing_else() {
    let mut window = opened();
    let first = window.harness.get_by_label(&colour(1)).rect();
    window.click(&colour(2));
    let picker = window.harness.get_by_label(&t!("palette.accept")).rect();
    let uncovered = egui::pos2(first.left() + 3.0, first.center().y);
    assert!(
        uncovered.x < picker.left(),
        "the press is beside the picker"
    );

    press_at(&mut window, uncovered);

    assert!(!window.harness.state().palette_window.menu_open());
    assert!(
        window
            .harness
            .query_by_label(&t!("palette.accept"))
            .is_none()
    );
    let places = window.harness.state().pattern.palettes()[0].places();
    assert!(places[0].is_some(), "colour 1 was not tapped");
}

/// In the block with three palettes, `0` is two swatches wide and `🎨`
/// stands between `<` and `>` on the row under it; with two, `0 🎨 ↔` stand
/// on one row, `0` one swatch wide.
#[test]
fn the_block_puts_the_palette_button_between_the_arrows() {
    let mut window = two_palettes();
    window.view(crate::ui::palette::NAME, PanelView::Block);
    let rect = |window: &super::Window, name: &str| window.harness.get_by_label(name).rect();
    let first = swatch(window.harness.state(), 0);
    let one = rect(&window, &first);
    let empty = rect(&window, &t!("palette.empty"));
    let palettes = rect(&window, &t!("palette.settings"));
    let switch = rect(&window, &t!("palette.switch"));
    assert_eq!(empty.width(), one.width());
    for beside in [palettes, switch] {
        assert!(
            (beside.center().y - empty.center().y).abs() < 1.0,
            "{beside:?}"
        );
    }

    window.harness.key_press(Key::P);
    window.harness.run();
    choose(&mut window, &colour(3), &t!("palette.move"));
    window.click(&t!("palette.new"));
    window.harness.state_mut().palette_window.open = false;
    window.harness.run();
    assert_eq!(window.harness.state().pattern.palettes().len(), 3);

    let one = rect(&window, &swatch(window.harness.state(), 0));
    let empty = rect(&window, &t!("palette.empty"));
    let back = rect(&window, &t!("palette.previous"));
    let palettes = rect(&window, &t!("palette.settings"));
    let on = rect(&window, &t!("palette.next"));
    assert!(empty.width() > one.width() * 2.0, "the 0 is wide");
    assert!(back.min.y > empty.max.y, "the arrows under the 0");
    for beside in [palettes, on] {
        assert!(
            (beside.center().y - back.center().y).abs() < 1.0,
            "{beside:?}"
        );
    }
    assert!(back.max.x < palettes.min.x && palettes.max.x < on.min.x);
}
