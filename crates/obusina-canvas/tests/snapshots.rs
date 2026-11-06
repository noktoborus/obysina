//! Pictures of the field, one for each kind of grid, compared with the ones in
//! `img/`. `UPDATE_SNAPSHOTS=1 cargo test -p obusina-canvas --test snapshots`
//! draws them again.

use egui_kittest::{Harness, SnapshotOptions};
use obusina_canvas::{Canvas, Picture, Turn, View};
use obusina_pattern::{Bounds, Cell, Colour, ColourId, Grid, Pattern, Rgb, Ruling};

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

/// Zoomed out, the field moved so the middle of the screen is at the top
/// left corner of the paper, the pointer in the shade at the bottom left:
/// the shadows in the frame of the screen at zoom 1 and in the round under
/// the pointer, each fading to its edge.
#[test]
fn shadows_zoomed_out() {
    let pattern = pattern(Grid::Peyote);
    let mut view = View::default();
    view.bead = 6.0;
    view.sheet = Some(Bounds {
        min: Cell {
            column: -20,
            row: -15,
        },
        max: Cell {
            column: 28,
            row: 21,
        },
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .with_theme(egui::Theme::Dark)
        .build_ui_state(
            |ui, (pattern, view): &mut (Pattern, View)| {
                Canvas::new(pattern, view).show(ui);
            },
            (pattern, view),
        );
    harness.run();
    harness.state_mut().1.offset += egui::vec2(150.0, 130.0);
    harness.hover_at(egui::pos2(170.0, 240.0));
    harness.run();
    harness.snapshot_options(
        "shadows-zoomed-out",
        &SnapshotOptions::new().output_path(PICTURES),
    );
}

/// The field moved far off to the left: ten beads of it stay at the left
/// side of the screen.
#[test]
fn kept_at_the_side() {
    let pattern = pattern(Grid::Peyote);
    let mut view = View::default();
    view.bead = 10.0;
    view.sheet = Some(Bounds::of(Cell { column: 4, row: 3 }));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 300.0))
        .with_theme(egui::Theme::Dark)
        .build_ui_state(
            |ui, (pattern, view): &mut (Pattern, View)| {
                Canvas::new(pattern, view).show(ui);
            },
            (pattern, view),
        );
    harness.run();
    harness.state_mut().1.offset.x -= 2000.0;
    harness.run();
    harness.snapshot_options(
        "kept-at-the-side",
        &SnapshotOptions::new().output_path(PICTURES),
    );
}

/// Loom with a line every ten columns and four rows, the first at column 2
/// and row 1: the shadows on a line are darker than the rest.
#[test]
fn ruling() {
    let mut pattern = pattern(Grid::Loom);
    pattern.set_ruling(Ruling {
        step: [10, 4],
        offset: [2, 1],
    });
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
    harness.snapshot_options("ruling", &SnapshotOptions::new().output_path(PICTURES));
}

/// A made up picture: a grid of lines on two colours, which shows whether a
/// line stays straight.
fn lines() -> egui::ColorImage {
    let size = [96, 72];
    let pixels = (0..size[1])
        .flat_map(|y| (0..size[0]).map(move |x| (x, y)))
        .map(|(x, y)| {
            if x % 12 == 0 || y % 12 == 0 {
                egui::Color32::from_rgb(0x30, 0x30, 0x30)
            } else if y < 36 {
                egui::Color32::from_rgb(0x70, 0xa0, 0xe0)
            } else {
                egui::Color32::from_rgb(0x50, 0x80, 0x40)
            }
        })
        .collect();
    egui::ColorImage::new(size, pixels)
}

/// A picture under the beads over corners that make no rectangle; while
/// `editing`, with the handles of its corners; while `loading`, the panel
/// that shimmers in its place.
fn picture_shot(editing: bool, loading: bool, name: &str) {
    {
        let pattern = pattern(Grid::Loom);
        let mut view = View::default();
        view.sheet = Some(Bounds::of(Cell { column: 4, row: 3 }));
        let corners = [
            egui::vec2(1.0, -1.0),
            egui::vec2(7.0, -0.5),
            egui::vec2(10.0, 8.0),
            egui::vec2(-2.0, 7.0),
        ];
        let mut harness = Harness::builder()
            .with_size(SIZE)
            .with_theme(egui::Theme::Dark)
            .build_ui_state(
                move |ui,
                      (pattern, view, texture, corners): &mut (
                    Pattern,
                    View,
                    Option<egui::TextureHandle>,
                    [egui::Vec2; 4],
                )| {
                    let texture = texture.get_or_insert_with(|| {
                        ui.ctx()
                            .load_texture("lines", lines(), egui::TextureOptions::LINEAR)
                    });
                    let picture = Picture {
                        texture: (!loading).then(|| texture.id()),
                        corners,
                        opacity: 0.7,
                        editing,
                    };
                    Canvas::new(pattern, view).picture(picture).show(ui);
                },
                (pattern, view, None, corners),
            );
        harness.run_steps(4);
        harness.snapshot_options(name, &SnapshotOptions::new().output_path(PICTURES));
    }
}

#[test]
fn a_picture_under_the_beads() {
    picture_shot(false, false, "picture");
}

/// The same picture while its corners are edited: a handle on each, over
/// the beads.
#[test]
fn a_picture_edited() {
    picture_shot(true, false, "picture-editing");
}

/// The same picture while it loads: a panel over its corners, a light band
/// across it.
#[test]
fn a_picture_that_loads() {
    picture_shot(false, true, "picture-loading");
}
