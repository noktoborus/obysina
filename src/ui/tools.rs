//! The floating panel of tools.

use obusina_panel::{FloatingPanel, Orientation};
use obusina_pattern::Grid;
use rust_i18n::t;

use crate::app::{App, Tool};
use crate::ui::{button, icons};

/// Where the panel stands the first time, from the top left of the window.
const PLACE: egui::Vec2 = egui::vec2(16.0, 16.0);

/// Shows the panel of tools.
pub fn show(app: &mut App, context: &egui::Context) {
    FloatingPanel::new("tools", t!("panel.tools"))
        .place(egui::Align2::LEFT_TOP, PLACE)
        .show(context, |ui, orientation| {
            history(app, ui);
            gap(ui, orientation);
            tools(app, ui);
            gap(ui, orientation);
            grid(app, ui);
            gap(ui, orientation);
            save(app, ui);
            if let Some(message) = &app.message {
                ui.label(message);
            }
        });
}

/// A space between two groups of buttons. A separator is not used: in a
/// column it would take all the width it is offered.
pub fn gap(ui: &mut egui::Ui, orientation: Orientation) {
    let space = ui.spacing().item_spacing * 2.0;
    match orientation {
        Orientation::Horizontal => ui.add_space(space.x),
        Orientation::Vertical => ui.add_space(space.y),
    }
}

/// Undo and redo.
fn history(app: &mut App, ui: &mut egui::Ui) {
    if button::icon(ui, icons::UNDO, &t!("edit.undo"), app.history.can_undo()).clicked() {
        app.undo();
    }
    if button::icon(ui, icons::REDO, &t!("edit.redo"), app.history.can_redo()).clicked() {
        app.redo();
    }
}

/// What a press on the field does.
fn tools(app: &mut App, ui: &mut egui::Ui) {
    for (tool, icon, name) in [
        (Tool::Pencil, icons::PENCIL, t!("tool.pencil")),
        (Tool::Fill, icons::FILL, t!("tool.fill")),
        (Tool::Pick, icons::PICK, t!("tool.pick")),
    ] {
        if button::toggle(ui, app.tool == tool, icon, &name).clicked() {
            app.tool = tool;
        }
    }
}

/// The kind of grid: a button that opens the list of kinds.
fn grid(app: &mut App, ui: &mut egui::Ui) {
    let response = button::icon(ui, icons::GRID, &t!("grid.kind"), true);
    egui::Popup::menu(&response).show(|ui| {
        for grid in Grid::ALL {
            if ui
                .selectable_label(app.pattern.grid() == grid, grid_name(grid))
                .clicked()
            {
                app.set_grid(grid);
            }
        }
    });
}

/// The name of a kind of grid.
fn grid_name(grid: Grid) -> String {
    match grid {
        Grid::Loom => t!("grid.loom"),
        Grid::Peyote => t!("grid.peyote"),
        Grid::Brick => t!("grid.brick"),
    }
    .to_string()
}

/// Save, when there is a file to save to.
fn save(app: &mut App, ui: &mut egui::Ui) {
    let hint = match app.path {
        Some(_) => t!("file.save"),
        None => t!("file.nowhere"),
    };
    if button::icon(ui, icons::SAVE, &hint, app.path.is_some()).clicked() {
        app.save();
    }
}
