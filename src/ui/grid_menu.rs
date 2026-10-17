//! The menu of the kinds of grid, a `PlateMenu` in the middle of the window.

use obusina_pattern::Grid;
use plate_menu::MenuItem;
use rust_i18n::t;

use crate::app::App;

/// Opens the menu on the kind the pattern has.
pub fn open(app: &mut App) {
    let items = Grid::ALL
        .iter()
        .map(|grid| MenuItem::new(id(*grid), name(*grid)))
        .collect();
    if let Err(error) = app.menu.open_at(items, id(app.pattern.grid())) {
        log::error!("the menu of grids does not open: {error}");
    }
}

/// Draws the menu while it is open, and changes the kind of grid to the one
/// chosen.
pub fn show(app: &mut App, context: &egui::Context) {
    app.menu.nothing_found(t!("menu.nothing_found"));
    let Some(chosen) = app.menu.show(context) else {
        return;
    };
    if let Some(grid) = Grid::ALL.into_iter().find(|grid| id(*grid) == chosen.id) {
        app.set_grid(grid);
    }
}

/// The identifier of the entry of a kind.
fn id(grid: Grid) -> &'static str {
    match grid {
        Grid::Loom => "loom",
        Grid::Peyote => "peyote",
        Grid::Brick => "brick",
    }
}

/// The name of a kind of grid.
fn name(grid: Grid) -> String {
    match grid {
        Grid::Loom => t!("grid.loom"),
        Grid::Peyote => t!("grid.peyote"),
        Grid::Brick => t!("grid.brick"),
    }
    .to_string()
}
