//! The floating panel of tools.

use obusina_canvas::Turn;
use obusina_panel::{FloatingPanel, Orientation};
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
            turns(app, ui);
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

/// The quarter turns of the field, each way.
fn turns(app: &mut App, ui: &mut egui::Ui) {
    for (turn, icon, name) in [
        (
            Turn::CounterClockwise,
            icons::TURN_LEFT,
            t!("view.turn_left"),
        ),
        (Turn::Clockwise, icons::TURN_RIGHT, t!("view.turn_right")),
    ] {
        if button::icon(ui, icon, &name, true).clicked() {
            app.turn(turn);
        }
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

/// The kind of grid: a button that opens the menu of kinds
/// ([`crate::ui::grid_menu`]).
fn grid(app: &mut App, ui: &mut egui::Ui) {
    if button::icon(ui, icons::GRID, &t!("grid.kind"), true).clicked() {
        crate::ui::grid_menu::open(app);
    }
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
