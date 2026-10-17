//! Pictures of the field, one for each kind of grid, compared with the ones in
//! `img/`. `UPDATE_SNAPSHOTS=1 cargo test -p obusina-canvas --test snapshots`
//! draws them again.

use egui_kittest::{Harness, SnapshotOptions};
use obusina_canvas::{Canvas, Turn, View};
use obusina_pattern::{Bead, Cell, Grid, Pattern, Rgb};

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";
/// A window that holds the whole pattern.
const SIZE: egui::Vec2 = egui::vec2(260.0, 200.0);

/// Three colours.
fn palette() -> Vec<Bead> {
    [
        ('A', [0x1b, 0x1b, 0x1b]),
        ('B', [0xf2, 0xef, 0xe6]),
        ('C', [0xc0, 0x30, 0x30]),
    ]
    .into_iter()
    .map(|(key, rgb)| Bead {
        key,
        rgb: Rgb(rgb),
        catalog: None,
        code: None,
    })
    .collect()
}

/// A diamond of three colours on a grid of `grid`, with empty places left.
fn pattern(grid: Grid) -> Pattern {
    let mut pattern = Pattern::new(grid, 9, 7, 0.8, palette()).expect("a pattern");
    for cell in pattern.cells().collect::<Vec<_>>() {
        let distance = cell.column.abs_diff(4) + cell.row.abs_diff(3);
        let colour = match distance {
            0 => Some(2),
            1..=2 => Some(0),
            3 => Some(1),
            4 => None,
            _ => Some(1),
        };
        pattern.set(cell, colour);
    }
    pattern.set(Cell { column: 0, row: 0 }, None);
    pattern
}

/// Draws the field of `grid`, turned by `turns`, and compares it with
/// `img/<name>.png`.
fn picture(grid: Grid, turns: &[Turn], name: &str) {
    let pattern = pattern(grid);
    let mut view = View::default();
    let extent = obusina_canvas::extent(&pattern, view.bead);
    for turn in turns {
        view.turn(*turn, extent);
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

#[test]
fn brick() {
    picture(Grid::Brick, &[], "brick");
}

/// Peyote turned a quarter clockwise: its columns stand as rows.
#[test]
fn peyote_turned() {
    picture(Grid::Peyote, &[Turn::Clockwise], "peyote-turned");
}
