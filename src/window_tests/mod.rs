//! Tests of the window (`RULE test.window`): `App` driven by `egui_kittest`
//! with egui events, what it draws found through the tree of accesskit.

use egui::{Event, Modifiers, PointerButton, Pos2, TouchDeviceId, TouchId, TouchPhase, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use obusina_pattern::{Cell, ColourId, Grid};
use rust_i18n::t;

use crate::app::App;
use crate::settings::PanelView;

mod backdrop;
mod field;
mod help;
mod locked;
mod palette;
mod projects;
mod screenshots;
mod sheet;

/// The size of the window in every test.
const SIZE: egui::Vec2 = egui::vec2(1000.0, 700.0);
/// How far a pointer moves before a press is no click, as egui has it.
const CLICK: f32 = 6.0;
/// A bead far from the panels.
const FREE: Cell = Cell { column: 5, row: 5 };

/// The window and the scratch directory its pattern is saved in.
struct Window {
    harness: Harness<'static, App>,
    directory: tempfile::TempDir,
}

/// The window over a new pattern, to be saved in a scratch directory.
fn window() -> Window {
    window_in(tempfile::tempdir().expect("a scratch directory"), SIZE)
}

/// Where the window of a test keeps what it keeps, in `directory`.
fn places(directory: &std::path::Path) -> crate::app::Places {
    crate::app::Places {
        data: Some(unifs::Files::at(directory.join("data"))),
        settings: Some(unifs::Files::at(directory)),
        pictures: Some(unifs::Files::at(directory.join(crate::backdrop::DIRECTORY))),
        file: None,
    }
}

/// The window of `size` over a new pattern, opened from the list kept in
/// `directory` with the settings kept there.
fn window_in(directory: tempfile::TempDir, size: egui::Vec2) -> Window {
    let places = places(directory.path());
    let mut harness = Harness::builder().with_size(size).build_eframe(move |_| {
        let mut app = App::start(places);
        app.create_pattern(0.0);
        app
    });
    harness.run();
    Window { harness, directory }
}

/// Goes back to the list, which saves the pattern, and opens it again in a
/// window of its own, as the next start would.
fn reopened(window: &mut Window) -> App {
    let id = window.harness.state().pattern.id();
    window.click(&t!("projects.back"));
    assert_eq!(window.harness.state().screen, crate::session::Screen::List);
    let mut again = App::start(places(window.directory.path()));
    again.open_by_id(id, 0.0);
    again.settle();
    assert_eq!(again.screen, crate::session::Screen::Editor);
    again
}

impl Window {
    /// The point at the centre of a bead.
    fn bead(&self, cell: Cell) -> Pos2 {
        let field = self.harness.get_by_label(&t!("field")).rect();
        let app = self.harness.state();
        obusina_canvas::bead_centre(&app.pattern, &app.view, field, cell)
    }

    /// `button` pressed and let go in place at a bead.
    fn press(&mut self, cell: Cell, button: PointerButton) {
        let point = self.bead(cell);
        self.harness.event(Event::PointerMoved(point));
        for pressed in [true, false] {
            self.harness.event(Event::PointerButton {
                pos: point,
                button,
                pressed,
                modifiers: Modifiers::NONE,
            });
            self.harness.step();
        }
        self.harness.run();
    }

    /// The first button pressed at a bead and dragged by `by`.
    fn drag_field(&mut self, cell: Cell, by: egui::Vec2) {
        let start = self.bead(cell);
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.harness.event(Event::PointerMoved(start));
        self.harness.event(button(start, true));
        self.harness.step();
        for step in 1..=10 {
            self.harness
                .event(Event::PointerMoved(start + by * (step as f32 / 10.0)));
            self.harness.step();
        }
        self.harness.event(button(start + by, false));
        self.harness.run();
    }

    /// Drags the panel named `name` by its middle to `target` with the
    /// first button.
    fn drag(&mut self, name: &str, target: Pos2) {
        let start = self.panel(name).center();
        self.drag_from(start, target);
    }

    /// The rect of the panel named `name`.
    fn panel(&self, name: &str) -> egui::Rect {
        self.harness.get_by_label(name).rect()
    }

    /// Gives the panel `name` the view `view` in the posture of the window.
    fn view(&mut self, name: &str, view: PanelView) {
        let context = self.harness.ctx.clone();
        crate::ui::panels::set_view(self.harness.state_mut(), &context, name, view);
        self.harness.run();
    }

    /// Clicks the last widget named `name`: the one drawn on top.
    fn click_last(&mut self, name: &str) {
        let node = self
            .harness
            .query_all_by_label(name)
            .last()
            .expect("a widget of that name");
        node.click();
        self.harness.run();
    }

    /// The first button pressed at `start`, moved to `target` and let go.
    fn drag_from(&mut self, start: Pos2, target: Pos2) {
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.harness.event(Event::PointerMoved(start));
        self.harness.event(button(start, true));
        self.harness.step();
        for step in 1..=10 {
            let point = start + (target - start) * (step as f32 / 10.0);
            self.harness.event(Event::PointerMoved(point));
            self.harness.step();
        }
        self.harness.event(button(target, false));
        self.harness.run();
    }

    /// A finger put at `start`, held still a second first when `hold`,
    /// moved to `target` and lifted, sent as winit sends it: the touch, and
    /// the pointer for the first finger.
    fn touch_drag(&mut self, start: Pos2, target: Pos2, hold: bool) {
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
        self.harness.event(touch(TouchPhase::Start, start));
        self.harness.event(Event::PointerMoved(start));
        self.harness.event(button(start, true));
        self.harness.step();
        if hold {
            for _ in 0..4 {
                self.harness.step();
            }
        }
        for step in 1..=10 {
            let point = start + (target - start) * (step as f32 / 10.0);
            self.harness.event(touch(TouchPhase::Move, point));
            self.harness.event(Event::PointerMoved(point));
            self.harness.step();
        }
        self.harness.event(touch(TouchPhase::End, target));
        self.harness.event(button(target, false));
        self.harness.event(Event::PointerGone);
        self.harness.run();
    }

    /// True when the widget named `first` stands above `second`, in a
    /// column; false when it stands beside it, in a row.
    fn stacked(&self, first: &str, second: &str) -> bool {
        let first = self.harness.get_by_label(first).rect().center();
        let second = self.harness.get_by_label(second).rect().center();
        (first.x - second.x).abs() < 1.0 && second.y > first.y
    }

    /// Opens the menu of the card named `name` on the list: a long press
    /// chooses it, the menu of the bar on top opens.
    fn card_menu(&mut self, name: &str) {
        self.harness.get_by_label(name).click_secondary();
        self.harness.run_steps(5);
        self.harness.get_by_label(&t!("projects.menu")).click();
        self.harness.run_steps(5);
    }

    /// Ends the choice of the list.
    fn end_choice(&mut self) {
        self.harness
            .get_by_label(&t!("projects.end_choice"))
            .click();
        self.harness.run_steps(5);
    }

    /// Turns the field to `degrees` by the menu of the settings, and closes
    /// the menu.
    fn turn_to(&mut self, degrees: u16) {
        self.click(&t!("settings.open"));
        self.click(&t!("turn.open"));
        let name = match degrees {
            0 => t!("turn.none").to_string(),
            degrees => format!("{degrees}\u{b0}"),
        };
        self.click(&name);
        self.close_menu();
    }

    /// Closes the menu open, by Escape out of each level.
    fn close_menu(&mut self) {
        for _ in 0..4 {
            if !self.harness.state().menu.is_open() {
                return;
            }
            self.harness.key_press(egui::Key::Escape);
            self.harness.run();
        }
        assert!(!self.harness.state().menu.is_open(), "the menu closes");
    }

    /// Clicks the widget named `name`.
    fn click(&mut self, name: &str) {
        self.harness.get_by_label(name).click();
        self.harness.run();
    }
}

/// The id of the colour in a place of the first palette of a new pattern.
const fn id_at(place: usize) -> ColourId {
    ColourId(place as u32 + 1)
}

/// The name of the swatch of a colour, as the panel of beads says it.
fn swatch(app: &App, place: usize) -> String {
    let colour = app.pattern.palettes()[0].places()[place]
        .as_ref()
        .expect("a colour in that place");
    t!(
        "palette.bead",
        key = place + 1,
        rgb = colour.rgb.expect("a colour with its rgb"),
        count = app.pattern.count(colour.id)
    )
    .to_string()
}

#[test]
fn a_press_paints_a_bead_and_the_panel_counts_it() {
    let mut window = window();
    let red = swatch(window.harness.state(), 2);
    window.click(&red);

    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(2)));
    let counted = swatch(window.harness.state(), 2);
    assert!(counted.contains('1'));
    window.harness.get_by_label(&counted);
}

#[test]
fn a_second_press_empties_the_bead() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(0)));

    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(FREE), None);
}

#[test]
fn the_second_button_does_nothing() {
    let mut window = window();
    let red = swatch(window.harness.state(), 2);
    window.harness.get_by_label(&red).click_secondary();
    window.harness.run();
    assert_eq!(window.harness.state().colour, Some(id_at(0)));

    window.press(FREE, PointerButton::Secondary);

    assert_eq!(window.harness.state().pattern.get(FREE), None);
}

#[test]
fn a_drag_moves_the_field_and_paints_nothing() {
    let mut window = window();
    let before = window.harness.state().view.offset;

    window.drag_field(FREE, egui::vec2(120.0, 60.0));

    let app = window.harness.state();
    assert_eq!(app.view.offset - before, egui::vec2(120.0, 60.0));
    assert_eq!(app.pattern.beads().count(), 0);
}

#[test]
fn a_press_is_undone_and_redone() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);

    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.get(FREE), None);

    window.click(&t!("edit.redo"));
    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(0)));
}

#[test]
fn a_press_is_undone_and_redone_by_the_keys() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);

    window
        .harness
        .key_combination_modifiers(Modifiers::COMMAND, &[egui::Key::Z]);
    window.harness.run();
    assert_eq!(window.harness.state().pattern.get(FREE), None);

    window
        .harness
        .key_combination_modifiers(Modifiers::COMMAND.plus(Modifiers::SHIFT), &[egui::Key::Z]);
    window.harness.run();
    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(0)));
}

/// The keys of the field act only on the field: the window of the palettes
/// over it takes them (`docs/keys.md`).
#[test]
fn the_window_of_the_palettes_takes_the_undo_from_the_field() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.harness.key_press(egui::Key::P);
    window.harness.run();

    window
        .harness
        .key_combination_modifiers(Modifiers::COMMAND, &[egui::Key::Z]);
    window.harness.run();

    let app = window.harness.state();
    assert!(app.palette_window.open);
    assert_eq!(app.pattern.get(FREE), Some(id_at(0)));
}

#[test]
fn a_tap_on_a_swatch_takes_the_colour_in_hand() {
    let mut window = window();
    let red = swatch(window.harness.state(), 2);

    window.click(&red);
    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().colour, Some(id_at(2)));
    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(2)));
}

#[test]
fn a_change_of_grid_is_undone() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("grid.kind"));
    window.click(&t!("grid.peyote"));
    assert_eq!(window.harness.state().pattern.grid(), Grid::Peyote);
    assert!(
        window.harness.state().menu.is_open(),
        "the menu stays in the level"
    );
    window.close_menu();

    window.click(&t!("edit.undo"));

    assert_eq!(window.harness.state().pattern.grid(), Grid::Loom);
}

#[test]
fn a_saved_pattern_opens_again() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    let pattern = window.harness.state().pattern.clone();

    let again = reopened(&mut window);

    assert_eq!(again.pattern, pattern);
    assert!(again.history.can_undo(), "the history comes with it");
}

/// A file named on the command line that this version cannot read is told
/// about, never written over, and the list shows.
#[test]
fn a_newer_file_is_never_written_over() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let path = directory.path().join("pattern.json");
    std::fs::write(&path, "{\"format\": 99}").expect("the file is written");

    let mut app = App::start(crate::app::Places {
        file: Some(path.clone()),
        ..places(directory.path())
    });
    app.settle();

    assert_eq!(app.screen, crate::session::Screen::List);
    assert_eq!(app.projects.told(), [t!("error.pattern.newer").to_string()]);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"format\": 99}");
}

#[test]
fn every_widget_has_a_name() {
    let window = window();

    for name in [
        t!("field"),
        t!("edit.undo"),
        t!("edit.redo"),
        t!("settings.open"),
        t!("projects.back"),
        t!("palette.empty"),
    ] {
        window.harness.get_by_label(&name);
    }
}

#[test]
fn a_panel_has_a_name_and_no_title() {
    let window = window();

    let named = window
        .harness
        .query_all_by_label(&t!("panel.tools"))
        .count();

    assert_eq!(named, 1, "the panel is the one thing named for it");
}

/// The first start stands the panels apart, inside the window, and keeps
/// their places.
#[test]
fn the_first_start_stands_the_panels_apart() {
    for size in [SIZE, egui::vec2(420.0, 900.0), egui::vec2(1000.0, 260.0)] {
        let window = window_in(tempfile::tempdir().expect("a scratch directory"), size);
        let screen = egui::Rect::from_min_size(Pos2::ZERO, size);
        let tools = window.panel(&t!("panel.tools"));
        let beads = window.panel(&t!("panel.palette"));

        assert!(!tools.intersects(beads), "{size:?}: {tools:?} {beads:?}");
        assert!(screen.contains_rect(tools), "{size:?}: {tools:?}");
        assert!(screen.contains_rect(beads), "{size:?}: {beads:?}");
        let posture = crate::settings::Posture::of(size);
        let settings = &window.harness.state().settings;
        assert!(settings.panel(posture, crate::ui::tools::NAME).is_some());
        assert!(settings.panel(posture, crate::ui::palette::NAME).is_some());
    }
}

/// A window that stands keeps its panels apart from one that lies: a panel
/// moved while it stands is where it was when it stands again.
#[test]
fn a_window_that_stands_keeps_its_own_places() {
    let tall = egui::vec2(500.0, 900.0);
    let mut window = window_in(tempfile::tempdir().expect("a scratch directory"), tall);
    window.drag(&t!("panel.tools"), pos2(250.0, 450.0));
    let moved = window.panel(&t!("panel.tools"));

    window.harness.set_size(SIZE);
    window.harness.run();
    let lying = window.panel(&t!("panel.tools"));
    window.harness.set_size(tall);
    window.harness.run();

    assert!(
        (lying.center() - moved.center()).length() > 50.0,
        "{lying:?}"
    );
    let again = window.panel(&t!("panel.tools"));
    assert!(
        (again.center() - moved.center()).length() < 1.0,
        "{again:?}"
    );
}

/// A row pulled to a side turns into a column while it is dragged, the
/// button taken still under the pointer along the panel; let go, the column
/// stands against the side and is kept.
#[test]
fn a_row_pulled_to_a_side_turns_into_a_column() {
    let mut window = window();
    window.view(crate::ui::tools::NAME, PanelView::Row);
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let start = window
        .harness
        .get_by_label(&t!("projects.back"))
        .rect()
        .center();
    let target = pos2(20.0, SIZE.y / 2.0);
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    window.harness.event(Event::PointerMoved(start));
    window.harness.event(button(start, true));
    window.harness.step();
    for step in 1..=10 {
        let point = start + (target - start) * (step as f32 / 10.0);
        window.harness.event(Event::PointerMoved(point));
        window.harness.step();
    }
    window.harness.step();
    window.harness.step();

    assert!(window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let back = window.harness.get_by_label(&t!("projects.back")).rect();
    assert!(
        (back.center().y - target.y).abs() < back.height(),
        "the button taken, {back:?}, stays at the pointer {target:?} along the panel"
    );

    window.harness.event(button(target, false));
    window.harness.run();
    assert!(window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let undo = window.harness.get_by_label(&t!("edit.undo")).rect();
    assert!(undo.min.x < 3.0 * obusina_panel::MARGIN, "{undo:?}");
    let posture = window.harness.state().posture;
    let kept = window
        .harness
        .state()
        .settings
        .panel(posture, crate::ui::tools::NAME);
    assert_eq!(kept.map(|(_, view)| view), Some(PanelView::Column));
}

/// A column pulled to the top by a finger turns into a row.
#[test]
fn a_column_pulled_up_by_a_finger_turns_into_a_row() {
    let mut window = window();
    window.view(crate::ui::tools::NAME, PanelView::Column);
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let start = window
        .harness
        .get_by_label(&t!("settings.open"))
        .rect()
        .center();

    window.touch_drag(start, pos2(SIZE.x / 2.0, 20.0), true);

    assert!(!window.stacked(&t!("edit.undo"), &t!("settings.open")));
    assert!(window.panel(&t!("panel.tools")).min.y < 3.0 * obusina_panel::MARGIN);
}

/// A row whose column would not fit the height of the window stays a row at
/// a side.
#[test]
fn a_panel_that_would_not_fit_turned_stays_as_it_is() {
    let short = egui::vec2(1000.0, 200.0);
    let mut window = window_in(tempfile::tempdir().expect("a scratch directory"), short);
    window.view(crate::ui::tools::NAME, PanelView::Row);

    window.drag(&t!("panel.tools"), pos2(20.0, short.y / 2.0));

    assert!(!window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let screen = egui::Rect::from_min_size(Pos2::ZERO, short);
    assert!(screen.contains_rect(window.panel(&t!("panel.tools"))));
}

/// The block of the beads stays a block at any edge.
#[test]
fn a_block_stays_a_block_at_an_edge() {
    let mut window = window();
    window.view(crate::ui::palette::NAME, PanelView::Block);

    window.drag(&t!("panel.palette"), pos2(SIZE.x / 2.0, 20.0));

    let posture = window.harness.state().posture;
    let kept = window
        .harness
        .state()
        .settings
        .panel(posture, crate::ui::palette::NAME);
    assert_eq!(kept.map(|(_, view)| view), Some(PanelView::Block));
}

#[test]
fn a_line_too_long_for_the_window_goes_on_in_the_next_line() {
    let narrow = egui::vec2(260.0, 700.0);
    let mut window = window_in(tempfile::tempdir().expect("a scratch directory"), narrow);
    window.view(crate::ui::tools::NAME, PanelView::Row);

    let undo = window.harness.get_by_label(&t!("edit.undo")).rect();
    let back = window.harness.get_by_label(&t!("projects.back")).rect();
    assert!(back.min.y > undo.max.y, "{undo:?} {back:?}");
    let screen = egui::Rect::from_min_size(Pos2::ZERO, narrow);
    assert!(screen.contains_rect(back), "{back:?}");
}

#[test]
fn a_panel_against_an_edge_stays_there_when_the_window_shrinks() {
    let mut window = window();
    window.drag(&t!("panel.palette"), pos2(SIZE.x - 8.0, SIZE.y / 2.0));
    let narrow = egui::vec2(700.0, 600.0);

    window.harness.set_size(narrow);
    window.harness.run();

    let empty = window.harness.get_by_label(&t!("palette.empty")).rect();
    assert!(empty.max.x <= narrow.x, "{empty:?}");
    assert!(
        empty.max.x > narrow.x - 3.0 * crate::ui::button::SIZE.x,
        "{empty:?}"
    );
}

#[test]
fn the_place_of_a_panel_is_kept_for_the_next_start() {
    let mut window = window();
    window.view(crate::ui::tools::NAME, PanelView::TwoColumns);
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let moved = window.panel(&t!("panel.tools"));
    let Window { harness, directory } = window;
    drop(harness);

    let window = window_in(directory, SIZE);

    assert!(window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let again = window.panel(&t!("panel.tools"));
    assert!(
        (again.center() - moved.center()).length() < 1.0,
        "{again:?}"
    );
}

#[test]
fn the_beads_fit_a_short_window() {
    let short = egui::vec2(1000.0, 260.0);
    let window = window_in(tempfile::tempdir().expect("a scratch directory"), short);
    let screen = egui::Rect::from_min_size(Pos2::ZERO, short);
    let app = window.harness.state();

    let mut names: Vec<String> = (0..5).map(|index| swatch(app, index)).collect();
    names.push(t!("palette.empty").to_string());
    for name in names {
        let rect = window.harness.get_by_label(&name).rect();
        assert!(screen.contains_rect(rect), "{name}: {rect:?}");
    }
}

#[test]
fn a_panel_in_the_centre_keeps_its_shape() {
    let mut window = window();
    window.view(crate::ui::palette::NAME, PanelView::Column);
    let first = swatch(window.harness.state(), 0);
    let third = swatch(window.harness.state(), 2);

    window.drag(&t!("panel.palette"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));

    assert!(window.stacked(&first, &third));
}

/// A mouse pressed on a button and moved moves the panel, and the button
/// is not pressed.
#[test]
fn a_press_on_a_button_moved_takes_the_panel_and_not_the_button() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 3.0));
    let before = window.panel(&t!("panel.tools"));
    let undo = window
        .harness
        .get_by_label(&t!("edit.undo"))
        .rect()
        .center();

    window.drag_from(undo, undo + egui::vec2(120.0, 140.0));

    let after = window.panel(&t!("panel.tools"));
    assert!((after.min - before.min - egui::vec2(120.0, 140.0)).length() < 1.0);
    assert_eq!(window.harness.state().pattern.beads().count(), 1);
}

/// A finger held still takes the panel; one moved at once does not, and the
/// panel stays.
#[test]
fn a_finger_takes_a_panel_only_when_held() {
    let mut window = window();
    let before = window.panel(&t!("panel.tools"));
    let settings = window.harness.get_by_label(&t!("settings.open")).rect();
    let start = settings.center();
    let by = egui::vec2(100.0, 120.0);

    window.touch_drag(start, start + by, false);
    assert_eq!(window.panel(&t!("panel.tools")), before, "a quick swipe");

    window.touch_drag(start, start + by, true);
    let after = window.panel(&t!("panel.tools"));
    assert!((after.min - before.min - by).length() < 1.0, "{after:?}");
    assert!(!window.harness.state().menu.is_open(), "no button pressed");
}

/// In the frame it is let go, and in every frame after, the panel stands
/// where it was let go: it does not go back to its old place first.
#[test]
fn a_panel_let_go_stands_there_at_once() {
    let mut window = window();
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let start = window.panel(&t!("panel.tools")).center();
    let target = start + egui::vec2(-120.0, 90.0);
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    window.harness.event(Event::PointerMoved(start));
    window.harness.event(button(start, true));
    window.harness.step();
    for step in 1..=10 {
        let point = start + (target - start) * (step as f32 / 10.0);
        window.harness.event(Event::PointerMoved(point));
        window.harness.step();
    }

    window.harness.event(button(target, false));
    for frame in 0..5 {
        window.harness.step();
        let centre = window.panel(&t!("panel.tools")).center();
        assert!(
            (centre - target).length() < 1.0,
            "frame {frame} after the release: {centre:?}, let go at {target:?}"
        );
    }
}

#[test]
fn a_panel_stays_inside_the_window() {
    let mut window = window();

    window.drag(&t!("panel.tools"), pos2(SIZE.x + 300.0, SIZE.y + 300.0));

    let screen = egui::Rect::from_min_size(Pos2::ZERO, SIZE);
    let panel = window.panel(&t!("panel.tools"));
    assert!(screen.contains_rect(panel), "{panel:?} in {screen:?}");
}

/// The view of a panel is chosen in the menu of the settings, and kept.
#[test]
fn the_view_of_a_panel_is_chosen_in_the_menu() {
    let mut window = window();
    window.click(&t!("settings.open"));
    window.click(&t!("panels.open"));
    window.click_last(&t!("panel.tools"));
    window.click(&t!("panels.two_rows"));
    window.harness.key_press(egui::Key::Escape);
    window.harness.run();
    window.harness.key_press(egui::Key::Escape);
    window.harness.run();

    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    assert!(!window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let posture = window.harness.state().posture;
    let kept = window
        .harness
        .state()
        .settings
        .panel(posture, crate::ui::tools::NAME);
    assert_eq!(kept.map(|(_, view)| view), Some(PanelView::TwoRows));
}

impl Window {
    /// Two fingers on the field, from `from` points apart to `to`, sent as
    /// winit sends them.
    fn pinch(&mut self, from: f32, to: f32) {
        let centre = self.bead(FREE);
        let touch = |id, phase, pos| Event::Touch {
            device_id: TouchDeviceId(1),
            id: TouchId(id),
            phase,
            pos,
            force: None,
        };
        let half = |distance: f32| egui::vec2(distance / 2.0, 0.0);
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.harness
            .event(touch(1, TouchPhase::Start, centre - half(from)));
        self.harness.event(Event::PointerMoved(centre - half(from)));
        self.harness.event(button(centre - half(from), true));
        self.harness.step();
        self.harness
            .event(touch(2, TouchPhase::Start, centre + half(from)));
        self.harness.step();
        for step in 1..=5 {
            let distance = from + (to - from) * step as f32 / 5.0;
            self.harness
                .event(touch(1, TouchPhase::Move, centre - half(distance)));
            self.harness
                .event(Event::PointerMoved(centre - half(distance)));
            self.harness
                .event(touch(2, TouchPhase::Move, centre + half(distance)));
            self.harness.step();
        }
        self.harness
            .event(touch(2, TouchPhase::End, centre + half(to)));
        self.harness
            .event(touch(1, TouchPhase::End, centre - half(to)));
        self.harness.event(button(centre - half(to), false));
        self.harness.run();
    }

    /// The wheel turned by `lines` over the field with `modifiers` held.
    fn wheel(&mut self, lines: f32, modifiers: Modifiers) {
        let point = self.bead(FREE);
        self.harness.event(Event::PointerMoved(point));
        self.harness.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, lines),
            phase: egui::TouchPhase::Move,
            modifiers,
            source: egui::MouseWheelSource::Wheel,
        });
        self.harness.run();
    }

    /// A pinch of the touchpad, as winit gives it: a factor of zoom.
    fn touchpad(&mut self, factor: f32) {
        let point = self.bead(FREE);
        self.harness.event(Event::PointerMoved(point));
        self.harness.event(Event::Zoom(factor));
        self.harness.run();
    }

    /// The height of a bead now.
    fn bead_size(&self) -> f32 {
        self.harness.state().view.bead
    }
}

#[test]
fn two_fingers_zoom_in_and_out() {
    let mut window = window();
    let start = window.bead_size();

    window.pinch(40.0, 120.0);
    let larger = window.bead_size();
    assert!(larger > start, "{larger} > {start}");

    window.pinch(120.0, 40.0);
    let smaller = window.bead_size();
    assert!(smaller < larger, "{smaller} < {larger}");
    assert_eq!(window.harness.state().pattern.beads().count(), 0);
}

#[test]
fn the_wheel_with_ctrl_zooms_in_and_out() {
    let mut window = window();
    let start = window.bead_size();

    window.wheel(3.0, Modifiers::COMMAND);
    let larger = window.bead_size();
    assert!(larger > start, "{larger} > {start}");

    window.wheel(-3.0, Modifiers::COMMAND);
    assert!(window.bead_size() < larger);
}

#[test]
fn the_touchpad_zooms_in_and_out() {
    let mut window = window();
    let start = window.bead_size();

    window.touchpad(1.5);
    let larger = window.bead_size();
    assert!(larger > start, "{larger} > {start}");

    window.touchpad(0.5);
    assert!(window.bead_size() < larger);
}

#[test]
fn the_wheel_alone_moves_the_field() {
    let mut window = window();
    let (offset, bead) = (window.harness.state().view.offset, window.bead_size());

    window.wheel(-3.0, Modifiers::NONE);

    assert_eq!(window.bead_size(), bead);
    assert!(window.harness.state().view.offset.y < offset.y);
}

#[test]
fn a_panel_follows_the_pointer_exactly() {
    let mut window = window();
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let start = window
        .harness
        .get_by_label(&t!("panel.tools"))
        .rect()
        .center();
    let by = egui::vec2(-150.0, -60.0);

    window.drag(&t!("panel.tools"), start + by);

    let end = window
        .harness
        .get_by_label(&t!("panel.tools"))
        .rect()
        .center();
    assert!(
        (end - (start + by)).length() < 1.0,
        "{end:?} != {:?}",
        start + by
    );
}

#[test]
fn a_panel_dragged_slowly_follows_the_pointer() {
    let mut window = window();
    window.drag(&t!("panel.tools"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));
    let start = window
        .harness
        .get_by_label(&t!("panel.tools"))
        .rect()
        .center();
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };

    window.harness.event(Event::PointerMoved(start));
    window.harness.event(button(start, true));
    window.harness.step();
    let mut point = start;
    for _ in 0..40 {
        point += egui::vec2(1.0, 0.5);
        window.harness.event(Event::PointerMoved(point));
        window.harness.step();
        if (point - start).length() <= CLICK {
            continue;
        }
        let centre = window.panel(&t!("panel.tools")).center();
        let behind = (point - centre).length();
        assert!(behind < 1.0, "the panel is {behind} behind the pointer");
    }
    window.harness.event(button(point, false));
    window.harness.run();
}

#[test]
fn every_button_of_the_panels_has_one_size() {
    let window = window();
    let first = swatch(window.harness.state(), 0);

    for name in [
        t!("edit.undo"),
        t!("edit.redo"),
        t!("settings.open"),
        t!("view.shadows"),
        t!("projects.back"),
        t!("palette.empty"),
        first.into(),
    ] {
        let size = window.harness.get_by_label(&name).rect().size();
        assert_eq!(size, crate::ui::button::SIZE, "{name}");
    }
}

#[test]
fn a_new_pattern_opens_on_the_middle_of_the_field() {
    let window = window();
    let app = window.harness.state();
    let field = window.harness.get_by_label(&t!("field")).rect();
    let origin = Cell { column: 0, row: 0 };

    let middle = obusina_canvas::bead_centre(&app.pattern, &app.view, field, origin);

    assert!(
        (middle - field.center()).length() < 1.0,
        "{middle:?} != {:?}",
        field.center()
    );
}

/// A new size of the window keeps the middle of the field where it is: the
/// bead in the middle stays in the middle.
#[test]
fn a_new_size_of_the_window_keeps_the_middle() {
    let mut window = window();
    let middle = Cell { column: 0, row: 0 };
    let smaller = egui::vec2(600.0, 450.0);

    window.harness.set_size(smaller);
    window.harness.run();

    let app = window.harness.state();
    let field = window.harness.get_by_label(&t!("field")).rect();
    let centre = obusina_canvas::bead_centre(&app.pattern, &app.view, field, middle);
    assert!(
        (centre - field.center()).length() < 1.0,
        "{centre:?} != {:?}",
        field.center()
    );
}

/// How many columns of the field stand left of the right side of the
/// screen, counted by the middles of the beads of a row of the sheet.
fn columns_on_screen(window: &Window) -> i32 {
    let app = window.harness.state();
    let screen = window.harness.get_by_label(&t!("field")).rect();
    let sheet = app.pattern.field().expect("a sheet");
    let field = app
        .pattern
        .bounds()
        .map_or(sheet, |beads| sheet.union(beads))
        .doubled();
    let row = (sheet.min.row + sheet.max.row) / 2;
    (field.min.column..=field.max.column)
        .filter(|column| {
            let cell = Cell {
                column: *column,
                row,
            };
            let centre = obusina_canvas::bead_centre(&app.pattern, &app.view, screen, cell);
            screen.x_range().contains(centre.x)
        })
        .count() as i32
}

/// The field dragged far to the right keeps `KEEP_BEADS` columns on the
/// screen, at its right side; a new size of the window keeps it at that
/// side, not in the middle.
#[test]
fn a_field_dragged_away_stays_at_the_side() {
    let keep = obusina_canvas::KEEP_BEADS as i32;
    let mut window = window();

    window.drag_field(FREE, egui::vec2(3000.0, 0.0));
    assert_eq!(columns_on_screen(&window), keep);

    window.harness.set_size(egui::vec2(600.0, 450.0));
    window.harness.run();
    assert_eq!(columns_on_screen(&window), keep);
}

/// A bead put left of and above cell (0, 0) is saved and opens where it
/// was: the field has no end either way.
#[test]
fn a_bead_left_of_the_middle_opens_where_it_was() {
    let mut window = window();
    let cell = Cell {
        column: -3,
        row: -2,
    };
    window.press(cell, PointerButton::Primary);
    let again = reopened(&mut window);

    assert_eq!(again.pattern.get(cell), Some(id_at(0)));
    assert_eq!(again.pattern.count(id_at(0)), 1);
}

#[test]
fn two_columns_of_tools_stand_in_pairs() {
    let mut window = window();

    window.view(crate::ui::tools::NAME, PanelView::TwoColumns);

    let undo = window.harness.get_by_label(&t!("edit.undo")).rect();
    let redo = window.harness.get_by_label(&t!("edit.redo")).rect();
    assert!(
        (undo.center().y - redo.center().y).abs() < 1.0,
        "{undo:?} {redo:?}"
    );
    assert!(window.stacked(&t!("edit.undo"), &t!("settings.open")));
    let panel = window.panel(&t!("panel.tools"));
    assert!(panel.width() < 150.0, "the panel is {} wide", panel.width());
}

#[test]
fn two_rows_of_tools_stand_in_pairs() {
    let mut window = window();

    window.view(crate::ui::tools::NAME, PanelView::TwoRows);

    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    assert!(window.stacked(&t!("settings.open"), &t!("view.shadows")));
    let panel = window.panel(&t!("panel.tools"));
    assert!(
        panel.height() < 120.0,
        "the panel is {} high",
        panel.height()
    );
}

#[test]
fn the_buttons_of_a_row_stand_on_one_line() {
    let mut window = window();
    window.view(crate::ui::palette::NAME, PanelView::Row);
    let first = swatch(window.harness.state(), 0);
    let line = window.harness.get_by_label(&first).rect().center().y;

    let centre = window
        .harness
        .get_by_label(&t!("palette.empty"))
        .rect()
        .center()
        .y;
    assert!(
        (centre - line).abs() < 0.5,
        "the empty place at {centre}, the row at {line}"
    );
}

#[test]
fn the_field_turns_from_the_settings_and_the_beads_stay() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    let pattern = window.harness.state().pattern.clone();
    let start = window.harness.state().view;

    for degrees in [90, 180, 270, 0] {
        window.turn_to(degrees);
        assert_eq!(window.harness.state().view.degrees(), degrees);
    }
    assert_eq!(window.harness.state().view, start);

    window.turn_to(270);
    let app = window.harness.state();
    assert_eq!(app.view.degrees(), 270);
    assert_eq!(app.pattern.turn(), 270);
    let beads = |pattern: &obusina_pattern::Pattern| pattern.beads().collect::<Vec<_>>();
    assert_eq!(beads(&app.pattern), beads(&pattern));
    assert!(app.history.can_undo(), "the press, and no turn");
    assert!(!app.history.can_redo());
}

/// The turn is written to the file, and the pattern opens turned as it was.
#[test]
fn a_turned_pattern_opens_turned() {
    let mut window = window();
    window.turn_to(90);
    let again = reopened(&mut window);

    assert_eq!(again.pattern.turn(), 90);
    assert_eq!(again.view.degrees(), 90);
}

#[test]
fn a_press_on_a_turned_field_reaches_its_bead() {
    let mut window = window();
    window.turn_to(90);
    let cell = Cell { column: 3, row: 8 };

    window.press(cell, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(cell), Some(id_at(0)));
    assert_eq!(window.harness.state().pattern.count(id_at(0)), 1);
}

/// How far the glow takes the middle of a bead from its colour.
const GLOW_SHADE: u8 = 4;

#[test]
fn a_moved_field_draws_its_beads_where_they_are_pressed() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.drag_field(Cell { column: 2, row: 2 }, egui::vec2(120.0, 60.0));

    let point = window.bead(FREE);
    let picture = window.harness.render().expect("the window renders");
    let scale = window.harness.ctx.pixels_per_point();
    let pixel = picture.get_pixel((point.x * scale) as u32, (point.y * scale) as u32);

    let near = |channel: u8| channel.abs_diff(0x1b) <= GLOW_SHADE;
    assert!(
        pixel.0[..3].iter().all(|channel| near(*channel)),
        "the bead at {point:?}: {pixel:?}"
    );
}

#[test]
fn the_empty_place_in_hand_takes_a_bead_away() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(FREE), Some(id_at(0)));
    window.click(&t!("palette.empty"));
    assert_eq!(window.harness.state().colour, None);

    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(FREE), None);
}

/// A menu of the window opens with no entry selected (`RULE ui.menu`).
#[test]
fn the_menu_of_the_settings_opens_with_no_entry_selected() {
    let mut window = window();
    window.click(&t!("settings.open"));

    let app = window.harness.state();
    assert!(app.menu.is_open());
    assert_eq!(app.menu.selected(), None);
}
