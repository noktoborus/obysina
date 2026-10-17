//! Tests of the window (`RULE test.window`): `App` driven by `egui_kittest`
//! with egui events, what it draws found through the tree of accesskit.

use egui::{Event, Modifiers, PointerButton, Pos2, TouchDeviceId, TouchId, TouchPhase};
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
    let directory = tempfile::tempdir().expect("a scratch directory");
    let path = directory.path().join("pattern.json");
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .build_eframe(move |_| App::open(Some(path)));
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

    /// The first button pressed at one bead, moved over the others and let go.
    fn stroke(&mut self, cells: &[Cell]) {
        let points: Vec<Pos2> = cells.iter().map(|cell| self.bead(*cell)).collect();
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.harness.event(Event::PointerMoved(points[0]));
        self.harness.event(button(points[0], true));
        self.harness.step();
        for point in &points[1..] {
            self.harness.event(Event::PointerMoved(*point));
            self.harness.step();
        }
        self.harness.event(button(*points.last().unwrap(), false));
        self.harness.run();
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

    window.stroke(&[FREE]);

    assert_eq!(window.harness.state().pattern.get(FREE), Some(2));
    let counted = swatch(window.harness.state(), 2);
    assert!(counted.contains('1'));
    window.harness.get_by_label(&counted);
}

#[test]
fn a_stroke_is_undone_and_redone_as_one_step() {
    let mut window = window();
    let cells = [
        FREE,
        Cell {
            column: 11,
            row: 20,
        },
        Cell {
            column: 12,
            row: 20,
        },
    ];

    window.stroke(&cells);
    assert!(
        cells
            .iter()
            .all(|cell| window.harness.state().pattern.get(*cell) == Some(0))
    );

    window.click(&t!("edit.undo"));
    assert!(
        cells
            .iter()
            .all(|cell| window.harness.state().pattern.get(*cell).is_none())
    );

    window.click(&t!("edit.redo"));
    assert!(
        cells
            .iter()
            .all(|cell| window.harness.state().pattern.get(*cell) == Some(0))
    );
}

#[test]
fn the_fill_paints_the_whole_region() {
    let mut window = window();
    window.click(&t!("tool.fill"));

    window.stroke(&[FREE]);

    let pattern = &window.harness.state().pattern;
    assert_eq!(pattern.counts()[0], pattern.width() * pattern.height());
}

/// Two fingers, sent as winit sends them: the first finger is also the
/// pointer, pressed a frame before the second finger comes.
#[test]
fn a_pinch_zooms_and_paints_nothing() {
    let mut window = window();
    let before = window.harness.state().view.bead;
    let centre = window.bead(FREE);
    let touch = |id, phase, pos| Event::Touch {
        device_id: TouchDeviceId(1),
        id: TouchId(id),
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
    let spread = |step: usize| egui::vec2(20.0 + 10.0 * step as f32, 0.0);

    let first = centre - spread(0);
    window.harness.event(touch(1, TouchPhase::Start, first));
    window.harness.event(Event::PointerMoved(first));
    window.harness.event(button(first, true));
    window.harness.step();
    window
        .harness
        .event(touch(2, TouchPhase::Start, centre + spread(0)));
    window.harness.step();
    for step in 1..=5 {
        window
            .harness
            .event(touch(1, TouchPhase::Move, centre - spread(step)));
        window
            .harness
            .event(Event::PointerMoved(centre - spread(step)));
        window
            .harness
            .event(touch(2, TouchPhase::Move, centre + spread(step)));
        window.harness.step();
    }
    window
        .harness
        .event(touch(2, TouchPhase::End, centre + spread(5)));
    window.harness.step();
    window
        .harness
        .event(touch(1, TouchPhase::Move, centre - spread(6)));
    window
        .harness
        .event(Event::PointerMoved(centre - spread(6)));
    window.harness.step();
    window
        .harness
        .event(touch(1, TouchPhase::End, centre - spread(6)));
    window.harness.event(button(centre - spread(6), false));
    window.harness.run();

    let app = window.harness.state();
    assert!(app.view.bead > before, "{} > {before}", app.view.bead);
    assert!(app.pattern.counts().iter().all(|count| *count == 0));
    assert!(!app.history.can_undo());
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
    window.stroke(&[FREE]);

    window.click(&t!("file.save"));

    let path = window.directory.path().join("pattern.json");
    let again = App::open(Some(path));
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

    let app = App::open(Some(path.clone()));

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
