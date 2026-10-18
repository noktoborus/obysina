//! Tests of the window (`RULE test.window`): `App` driven by `egui_kittest`
//! with egui events, what it draws found through the tree of accesskit.

use egui::{Event, Modifiers, PointerButton, Pos2, TouchDeviceId, TouchId, TouchPhase, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use obusina_pattern::{Cell, Grid};
use rust_i18n::t;

use crate::app::App;

/// The size of the window in every test.
const SIZE: egui::Vec2 = egui::vec2(1000.0, 700.0);
/// A bead far from the panels.
const FREE: Cell = Cell {
    column: 10,
    row: 20,
};

/// The window and the scratch directory its pattern is saved in.
struct Window {
    harness: Harness<'static, App>,
    directory: tempfile::TempDir,
}

/// The window over a new pattern, to be saved in a scratch directory.
fn window() -> Window {
    window_in(tempfile::tempdir().expect("a scratch directory"), SIZE)
}

/// The window of `size` over the pattern and the settings kept in
/// `directory`.
fn window_in(directory: tempfile::TempDir, size: egui::Vec2) -> Window {
    let path = directory.path().join("pattern.json");
    let settings = directory.path().join(crate::settings::FILE);
    let mut harness = Harness::builder()
        .with_size(size)
        .build_eframe(move |_| App::open(Some(path), Some(settings)));
    harness.run();
    Window { harness, directory }
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

    /// Drags the grip named `grip` to `target` with the first button.
    fn drag(&mut self, grip: &str, target: Pos2) {
        let start = self.harness.get_by_label(grip).rect().center();
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

    /// True when the widget named `first` stands above `second`, in a
    /// column; false when it stands beside it, in a row.
    fn stacked(&self, first: &str, second: &str) -> bool {
        let first = self.harness.get_by_label(first).rect().center();
        let second = self.harness.get_by_label(second).rect().center();
        (first.x - second.x).abs() < 1.0 && second.y > first.y
    }

    /// Clicks the widget named `name`.
    fn click(&mut self, name: &str) {
        self.harness.get_by_label(name).click();
        self.harness.run();
    }
}

/// The name of the swatch of a colour, as the panel of beads says it.
fn swatch(app: &App, index: usize) -> String {
    let bead = &app.pattern.palette()[index];
    t!(
        "palette.bead",
        key = bead.key,
        rgb = bead.rgb,
        count = app.pattern.counts()[index]
    )
    .to_string()
}

#[test]
fn a_press_paints_a_bead_and_the_panel_counts_it() {
    let mut window = window();
    let red = swatch(window.harness.state(), 2);
    window.click(&red);

    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(FREE), Some(2));
    let counted = swatch(window.harness.state(), 2);
    assert!(counted.contains('1'));
    window.harness.get_by_label(&counted);
}

#[test]
fn a_second_press_empties_the_bead() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    assert_eq!(window.harness.state().pattern.get(FREE), Some(0));

    window.press(FREE, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(FREE), None);
}

#[test]
fn a_press_of_the_second_button_paints_the_second_colour() {
    let mut window = window();
    let red = swatch(window.harness.state(), 2);
    window.harness.get_by_label(&red).click_secondary();
    window.harness.run();

    window.press(FREE, PointerButton::Secondary);

    assert_eq!(window.harness.state().pattern.get(FREE), Some(2));
}

#[test]
fn a_drag_moves_the_field_and_paints_nothing() {
    let mut window = window();
    let before = window.harness.state().view.offset;

    window.drag_field(FREE, egui::vec2(120.0, 60.0));

    let app = window.harness.state();
    assert_eq!(app.view.offset - before, egui::vec2(120.0, 60.0));
    assert!(app.pattern.counts().iter().all(|count| *count == 0));
}

#[test]
fn a_press_is_undone_and_redone() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);

    window.click(&t!("edit.undo"));
    assert_eq!(window.harness.state().pattern.get(FREE), None);

    window.click(&t!("edit.redo"));
    assert_eq!(window.harness.state().pattern.get(FREE), Some(0));
}

#[test]
fn the_fill_paints_the_whole_region() {
    let mut window = window();
    window.click(&t!("tool.fill"));

    window.press(FREE, PointerButton::Primary);

    let pattern = &window.harness.state().pattern;
    assert_eq!(pattern.counts()[0], pattern.width() * pattern.height());
}

#[test]
fn the_second_colour_is_reached_by_a_button() {
    let mut window = window();
    assert_eq!(window.harness.state().colours, [Some(0), None]);

    window.click(&t!("palette.swap"));

    assert_eq!(window.harness.state().colours, [None, Some(0)]);
}

#[test]
fn a_change_of_grid_is_undone() {
    let mut window = window();
    window.click(&t!("grid.kind"));
    window.click(&t!("grid.peyote"));
    assert_eq!(window.harness.state().pattern.grid(), Grid::Peyote);

    window.click(&t!("edit.undo"));

    assert_eq!(window.harness.state().pattern.grid(), Grid::Loom);
}

#[test]
fn a_saved_pattern_opens_again() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);

    window.click(&t!("file.save"));

    let path = window.directory.path().join("pattern.json");
    let again = App::open(Some(path), None);
    assert_eq!(again.pattern, window.harness.state().pattern);
    assert_eq!(
        window.harness.state().message.as_deref(),
        Some(&*t!("file.saved"))
    );
}

#[test]
fn a_newer_file_is_never_written_over() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let path = directory.path().join("pattern.json");
    std::fs::write(&path, "{\"format\": 99}").expect("the file is written");

    let app = App::open(Some(path.clone()), None);

    assert_eq!(app.path, None);
    assert_eq!(app.message.as_deref(), Some(&*t!("error.pattern.newer")));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"format\": 99}");
}

#[test]
fn every_widget_has_a_name() {
    let window = window();

    for name in [
        t!("field"),
        t!("edit.undo"),
        t!("edit.redo"),
        t!("tool.pencil"),
        t!("tool.fill"),
        t!("tool.pick"),
        t!("grid.kind"),
        t!("file.save"),
        t!("palette.empty"),
        t!("palette.first"),
        t!("palette.second"),
        t!("palette.swap"),
    ] {
        window.harness.get_by_label(&name);
    }
}

#[test]
fn a_panel_has_a_grip_and_no_title() {
    let window = window();

    let grips = window
        .harness
        .query_all_by_label(&t!("panel.tools"))
        .count();

    assert_eq!(grips, 1, "the grip is the one thing named for the panel");
}

#[test]
fn a_panel_at_a_side_stands_in_a_column() {
    let mut window = window();
    assert!(!window.stacked(&t!("edit.undo"), &t!("edit.redo")));

    window.drag(&t!("panel.tools"), pos2(8.0, SIZE.y / 2.0));
    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));

    window.drag(&t!("panel.tools"), pos2(100.0, 8.0));
    assert!(!window.stacked(&t!("edit.undo"), &t!("edit.redo")));
}

#[test]
fn a_panel_turns_while_it_is_dragged() {
    let mut window = window();
    let start = window
        .harness
        .get_by_label(&t!("panel.tools"))
        .rect()
        .center();
    let target = pos2(8.0, SIZE.y / 2.0);
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
    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));

    window.harness.event(button(target, false));
    window.harness.run();
    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    let undo = window.harness.get_by_label(&t!("edit.undo")).rect();
    assert!(undo.min.x < 3.0 * obusina_panel::MARGIN, "{undo:?}");
}

#[test]
fn a_panel_that_runs_out_of_the_window_turns() {
    let mut window = window();

    window.drag(&t!("panel.tools"), pos2(SIZE.x * 0.7, 150.0));

    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    let grip = window.harness.get_by_label(&t!("panel.tools")).rect();
    assert!(grip.contains(pos2(SIZE.x * 0.7, 150.0)), "{grip:?}");
}

#[test]
fn a_window_too_narrow_for_a_row_turns_the_tools() {
    let narrow = egui::vec2(400.0, 700.0);
    let window = window_in(tempfile::tempdir().expect("a scratch directory"), narrow);

    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
}

#[test]
fn a_panel_against_an_edge_stays_there_when_the_window_shrinks() {
    let mut window = window();
    let narrow = egui::vec2(700.0, 600.0);

    window.harness.set_size(narrow);
    window.harness.run();

    let swap = window.harness.get_by_label(&t!("palette.swap")).rect();
    assert!(swap.max.x <= narrow.x, "{swap:?}");
    assert!(
        swap.max.x > narrow.x - 3.0 * crate::ui::button::SIZE.x,
        "{swap:?}"
    );
}

#[test]
fn the_place_of_a_panel_is_kept_for_the_next_start() {
    let mut window = window();
    window.drag(&t!("panel.tools"), pos2(8.0, SIZE.y / 2.0));
    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    let Window { harness, directory } = window;
    drop(harness);

    let window = window_in(directory, SIZE);

    assert!(window.stacked(&t!("edit.undo"), &t!("edit.redo")));
    let grip = window.harness.get_by_label(&t!("panel.tools")).rect();
    assert!(grip.min.x < SIZE.x / 4.0, "{grip:?}");
}

#[test]
fn the_beads_fit_a_short_window() {
    let short = egui::vec2(1000.0, 260.0);
    let window = window_in(tempfile::tempdir().expect("a scratch directory"), short);
    let screen = egui::Rect::from_min_size(Pos2::ZERO, short);
    let app = window.harness.state();

    let mut names: Vec<String> = (0..app.pattern.palette().len())
        .map(|index| swatch(app, index))
        .collect();
    names.push(t!("palette.empty").to_string());
    for name in names {
        let rect = window.harness.get_by_label(&name).rect();
        assert!(screen.contains_rect(rect), "{name}: {rect:?}");
    }
}

#[test]
fn a_panel_in_the_centre_keeps_its_shape() {
    let mut window = window();
    window.drag(&t!("panel.palette"), pos2(SIZE.x - 8.0, SIZE.y / 2.0));
    assert!(window.stacked(&t!("palette.first"), &t!("palette.second")));

    window.drag(&t!("panel.palette"), pos2(SIZE.x / 2.0, SIZE.y / 2.0));

    assert!(window.stacked(&t!("palette.first"), &t!("palette.second")));
}

#[test]
fn a_tap_on_the_grip_folds_the_panel() {
    let mut window = window();

    window.click(&t!("panel.tools"));
    assert!(window.harness.query_by_label(&t!("edit.undo")).is_none());

    window.click(&t!("panel.tools"));
    window.harness.get_by_label(&t!("edit.undo"));
}

#[test]
fn a_panel_stays_inside_the_window() {
    let mut window = window();

    window.drag(&t!("panel.tools"), pos2(SIZE.x + 300.0, SIZE.y + 300.0));

    let screen = egui::Rect::from_min_size(Pos2::ZERO, SIZE);
    let grip = window.harness.get_by_label(&t!("panel.tools")).rect();
    assert!(screen.contains_rect(grip), "{grip:?} in {screen:?}");
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
    assert!(
        window
            .harness
            .state()
            .pattern
            .counts()
            .iter()
            .all(|count| *count == 0)
    );
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
        let grip = window
            .harness
            .get_by_label(&t!("panel.tools"))
            .rect()
            .center();
        let behind = (point - grip).length();
        assert!(behind < 1.0, "the grip is {behind} behind the pointer");
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
        t!("tool.pencil"),
        t!("tool.fill"),
        t!("tool.pick"),
        t!("grid.kind"),
        t!("file.save"),
        t!("palette.empty"),
        t!("palette.first"),
        t!("palette.second"),
        t!("palette.swap"),
        first.into(),
    ] {
        let size = window.harness.get_by_label(&name).rect().size();
        assert_eq!(size, crate::ui::button::SIZE, "{name}");
    }
}

#[test]
fn the_pattern_opens_in_the_middle_of_the_window() {
    let window = window();
    let app = window.harness.state();
    let field = window.harness.get_by_label(&t!("field")).rect();
    let (width, height) = app.pattern.grid().extent(
        app.pattern.width(),
        app.pattern.height(),
        app.pattern.ratio(),
    );
    let middle = field.min + app.view.offset + egui::vec2(width, height) * app.view.bead / 2.0;

    assert!(
        (middle - field.center()).length() < 1.0,
        "{middle:?} != {:?}",
        field.center()
    );
}

#[test]
fn a_column_is_as_narrow_as_its_buttons() {
    let mut window = window();

    window.drag(&t!("panel.tools"), pos2(8.0, SIZE.y / 2.0));

    let grip = window.harness.get_by_label(&t!("panel.tools")).rect();
    let undo = window.harness.get_by_label(&t!("edit.undo")).rect();
    assert!(
        (undo.center().x - grip.center().x).abs() < 1.0,
        "the button {undo:?} stands under the grip {grip:?}"
    );
    assert!(undo.max.x < 100.0, "the column is {} wide", undo.max.x);
}

#[test]
fn the_buttons_of_a_row_stand_on_one_line() {
    let mut window = window();
    window.drag(&t!("panel.palette"), pos2(SIZE.x / 2.0, 8.0));
    let first = swatch(window.harness.state(), 0);
    let line = window.harness.get_by_label(&first).rect().center().y;

    for name in [
        t!("palette.empty"),
        t!("palette.first"),
        t!("palette.second"),
        t!("palette.swap"),
    ] {
        let centre = window.harness.get_by_label(&name).rect().center().y;
        assert!(
            (centre - line).abs() < 0.5,
            "{name} at {centre}, the row at {line}"
        );
    }
}

#[test]
fn the_field_turns_both_ways_and_the_pattern_stays() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    let pattern = window.harness.state().pattern.clone();
    let start = window.harness.state().view;

    for degrees in [90, 180, 270, 0] {
        window.click(&t!("view.turn_right"));
        assert_eq!(window.harness.state().view.degrees(), degrees);
    }
    assert_eq!(window.harness.state().view, start);

    window.click(&t!("view.turn_left"));
    assert_eq!(window.harness.state().view.degrees(), 270);
    assert_eq!(window.harness.state().pattern, pattern);
    assert!(!window.harness.state().history.can_redo());
}

#[test]
fn a_press_on_a_turned_field_reaches_its_bead() {
    let mut window = window();
    window.click(&t!("view.turn_right"));
    let cell = Cell { column: 3, row: 25 };

    window.press(cell, PointerButton::Primary);

    assert_eq!(window.harness.state().pattern.get(cell), Some(0));
    assert_eq!(window.harness.state().pattern.counts()[0], 1);
}

#[test]
fn a_moved_field_draws_its_beads_where_they_are_pressed() {
    let mut window = window();
    window.press(FREE, PointerButton::Primary);
    window.drag_field(Cell { column: 2, row: 2 }, egui::vec2(120.0, 60.0));

    let point = window.bead(FREE);
    let picture = window.harness.render().expect("the window renders");
    let scale = window.harness.ctx.pixels_per_point();
    let pixel = picture.get_pixel((point.x * scale) as u32, (point.y * scale) as u32);

    assert_eq!(&pixel.0[..3], &[0x1b, 0x1b, 0x1b], "the bead at {point:?}");
}
