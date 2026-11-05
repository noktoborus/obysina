//! Pictures of the field, one for each kind of grid, compared with the ones in
//! `img/`. `UPDATE_SNAPSHOTS=1 cargo test -p obusina-canvas --test snapshots`
//! draws them again.

use egui_kittest::{Harness, SnapshotOptions};
use obusina_canvas::{Canvas, Turn, View};
use obusina_pattern::{Bounds, Cell, Colour, ColourId, Grid, Pattern, Rgb};

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";
/// A window that holds the whole pattern and some of the field around it.
const SIZE: egui::Vec2 = egui::vec2(260.0, 200.0);

const BLACK: ColourId = ColourId(1);
const PALE: ColourId = ColourId(2);
const RED: ColourId = ColourId(3);

/// Three colours.
fn palette() -> Vec<Colour> {
    [
        (BLACK, [0x1b, 0x1b, 0x1b]),
        (PALE, [0xf2, 0xef, 0xe6]),
        (RED, [0xc0, 0x30, 0x30]),
    ]
    .into_iter()
    .map(|(id, rgb)| Colour::new(id, Rgb(rgb)))
    .collect()
}

/// A diamond of three colours on a grid of `grid`, with empty places left
/// in its active area, columns 0 to 8 and rows 0 to 6.
fn pattern(grid: Grid) -> Pattern {
    let mut pattern = Pattern::new(grid, 0.8, palette()).expect("a pattern");
    let area = Bounds {
        min: Cell { column: 0, row: 0 },
        max: Cell { column: 8, row: 6 },
    };
    for cell in area.cells() {
        let distance = cell.column.abs_diff(4) + cell.row.abs_diff(3);
        let colour = match distance {
            0 => Some(RED),
            1..=2 => Some(BLACK),
            3 => Some(PALE),
            4 => None,
            _ => Some(PALE),
        };
        pattern.set(cell, colour);
    }
    pattern.set(Cell { column: 0, row: 0 }, None);
    pattern
}

/// Draws the field of `grid`, turned by `turns`, and compares it with
/// `img/<name>.png`. The sheet is one cell, so the work area is the diamond
/// and the shade around it is seen.
fn picture(grid: Grid, turns: &[Turn], name: &str) {
    let pattern = pattern(grid);
    let mut view = View::default();
    view.sheet = Some(Bounds::of(Cell { column: 4, row: 3 }));
    for turn in turns {
        view.turn(*turn);
    }
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .with_theme(egui::Theme::Dark)
        .build_ui_state(
            |ui, (pattern, view): &mut (Pattern, View)| {
                Canvas::new(pattern, view).show(ui);
            },
            (pattern, view),
        );
    harness.run();
    harness.snapshot_options(name, &SnapshotOptions::new().output_path(PICTURES));
}

#[test]
fn loom() {
    picture(Grid::Loom, &[], "loom");
}

#[test]
fn peyote() {
    picture(Grid::Peyote, &[], "peyote");
}

/// Peyote turned a quarter clockwise: its columns stand as rows.
#[test]
fn peyote_turned() {
    picture(Grid::Peyote, &[Turn::Clockwise], "peyote-turned");
}

/// A colour removed with its beads: they are hatched.
#[test]
fn hatched() {
    let mut pattern = pattern(Grid::Loom);
    pattern.remove_colour(0, 0).expect("the black colour goes");
    let mut view = View::default();
    view.sheet = Some(Bounds::of(Cell { column: 4, row: 3 }));
    let mut harness = Harness::builder()
        .with_size(SIZE)
        .with_theme(egui::Theme::Dark)
        .build_ui_state(
            |ui, (pattern, view): &mut (Pattern, View)| {
                Canvas::new(pattern, view).show(ui);
            },
            (pattern, view),
        );
    harness.run();
    harness.snapshot_options("hatched", &SnapshotOptions::new().output_path(PICTURES));
}
