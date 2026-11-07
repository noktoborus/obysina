//! The screenshots of the README: the window drawn by lavapipe over patterns
//! made here, in English and the dark theme. `SCREENSHOTS=1 cargo test -p
//! obusina screenshots` writes them to `img/`; without it they are drawn and
//! dropped. They are no reference: nothing compares them (`RULE
//! test.picture`, its EXC).

use egui::Vec2;
use egui_kittest::Harness;
use obusina_pattern::{Bounds, Cell, ColourId, Grid, Pattern, PatternId};

use super::places;
use crate::app::App;

/// A pattern of the screenshots: a name, a grid, the place of the first
/// palette of each bead, by its cell.
struct Design {
    name: &'static str,
    grid: Grid,
    beads: Vec<(Cell, usize)>,
}

/// Places of `NEW_PALETTE`, the colour `ColourId(place + 1)`: black, cream,
/// red, blue, yellow.
const BLACK: usize = 0;
const CREAM: usize = 1;
const RED: usize = 2;
const BLUE: usize = 3;
const YELLOW: usize = 4;

fn cell(column: i32, row: i32) -> Cell {
    Cell { column, row }
}

/// A red heart outlined in black, on cream.
fn heart() -> Design {
    let (width, height) = (25, 23);
    let mut beads = Vec::new();
    let inside = |column: i32, row: i32| {
        let x = (column as f32 - 12.0) / 9.5;
        let y = (11.0 - row as f32) / 9.5 + 0.15;
        (x * x + y * y - 1.0).powi(3) - x * x * y.powi(3) <= 0.0
    };
    for row in 0..height {
        for column in 0..width {
            let place = if inside(column, row) {
                let edge = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .any(|(dc, dr)| !inside(column + dc, row + dr));
                if edge { BLACK } else { RED }
            } else {
                CREAM
            };
            beads.push((cell(column, row), place));
        }
    }
    Design {
        name: "Heart",
        grid: Grid::Loom,
        beads,
    }
}

/// Rings of four colours around a yellow middle.
fn rings() -> Design {
    let mut beads = Vec::new();
    for row in 0..21 {
        for column in 0..21 {
            let (x, y) = (column as f32 - 10.0, row as f32 - 10.0);
            let ring = (x * x + y * y).sqrt().round() as usize;
            let place = [
                YELLOW, YELLOW, RED, RED, BLACK, BLUE, BLUE, CREAM, CREAM, BLACK, BLUE,
            ]
            .get(ring)
            .copied();
            if let Some(place) = place {
                beads.push((cell(column, row), place));
            }
        }
    }
    Design {
        name: "Rings",
        grid: Grid::Peyote,
        beads,
    }
}

/// A band of diamonds, for a bracelet.
fn band() -> Design {
    let mut beads = Vec::new();
    for row in 0..11_i32 {
        for column in 0..36_i32 {
            let distance = (column % 12 - 6).abs() + (row - 5).abs();
            let place = match distance {
                0..=1 => YELLOW,
                2..=3 => BLUE,
                4 => BLACK,
                _ if row == 0 || row == 10 => BLACK,
                _ => CREAM,
            };
            beads.push((cell(column, row), place));
        }
    }
    Design {
        name: "Bracelet band",
        grid: Grid::Loom,
        beads,
    }
}

/// Zigzags of red and blue.
fn zigzag() -> Design {
    let mut beads = Vec::new();
    for row in 0..20 {
        for column in 0..24_i32 {
            let step = (row + (column % 8 - 4).abs()) % 8;
            let place = match step {
                0 | 1 => RED,
                4 | 5 => BLUE,
                _ => CREAM,
            };
            beads.push((cell(column, row), place));
        }
    }
    Design {
        name: "Zigzag",
        grid: Grid::Peyote,
        beads,
    }
}

/// The pattern of `design`, its sheet the box of its beads.
fn pattern(design: &Design) -> Pattern {
    let mut pattern = crate::app::new_pattern();
    pattern.set_id(PatternId::new());
    pattern.set_name(design.name);
    pattern.set_grid(design.grid);
    for (cell, place) in &design.beads {
        pattern.set(*cell, Some(ColourId(*place as u32 + 1)));
    }
    let max = design.beads.iter().fold(cell(0, 0), |max, (cell, _)| Cell {
        column: max.column.max(cell.column),
        row: max.row.max(cell.row),
    });
    pattern.set_sheet(Some(Bounds {
        min: cell(0, 0),
        max,
    }));
    pattern
}

/// The window at `size` on the list of the designs, the heart locked; the
/// volume of the beads on.
fn window(directory: &std::path::Path, size: Vec2) -> (Harness<'static, App>, Vec<PatternId>) {
    let places = places(directory);
    let mut harness = Harness::builder()
        .with_size(size)
        .wgpu()
        .build_eframe(move |_| App::start(places));
    let app = harness.state_mut();
    let library = app.library.as_ref().expect("a library");
    let mut ids = Vec::new();
    for design in [zigzag(), band(), rings(), heart()] {
        let mut pattern = pattern(&design);
        pattern.set_locked(design.name == "Zigzag");
        let history = crate::edit::History::default();
        futures::executor::block_on(library.save(&pattern, &history)).expect("saved");
        ids.push(pattern.id());
    }
    app.keep_glow(true);
    app.projects.refresh();
    settle(&mut harness);
    (harness, ids)
}

/// Runs until the pictures of the cards are drawn.
fn settle(harness: &mut Harness<'static, App>) {
    for _ in 0..16 {
        harness.run_steps(2);
        let _ = harness.render();
    }
}

/// Draws the window, and writes it to `img/<name>.png` when SCREENSHOTS is
/// set.
fn shoot(harness: &mut Harness<'static, App>, name: &str) {
    settle(harness);
    let image = harness.render().expect("drawn");
    if std::env::var_os("SCREENSHOTS").is_some() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("img")
            .join(format!("{name}.png"));
        image.save(path).expect("written");
    }
}

#[test]
fn screenshots() {
    rust_i18n::set_locale("en");
    let directory = tempfile::tempdir().expect("a scratch directory");
    let (mut harness, ids) = window(directory.path(), egui::vec2(1000.0, 700.0));
    shoot(&mut harness, "list");

    harness.state_mut().open_by_id(ids[3], 0.0);
    shoot(&mut harness, "editor");

    harness.state_mut().palette_window.open = true;
    shoot(&mut harness, "palettes");

    let directory = tempfile::tempdir().expect("a scratch directory");
    let (mut harness, ids) = window(directory.path(), egui::vec2(400.0, 820.0));
    shoot(&mut harness, "phone-list");
    harness.state_mut().open_by_id(ids[3], 0.0);
    shoot(&mut harness, "phone-editor");
}
