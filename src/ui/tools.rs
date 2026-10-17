//! The floating panel of tools.

use obusina_pattern::Grid;
use rust_i18n::t;

use crate::app::{App, Tool};
use crate::ui::access::Named;
use crate::ui::icons;

/// Where the panel stands the first time.
const PLACE: egui::Pos2 = egui::pos2(16.0, 16.0);

/// Shows the panel of tools.
pub fn show(app: &mut App, context: &egui::Context) {
    egui::Window::new(t!("panel.tools"))
        .id(egui::Id::unique("tools"))
        .default_pos(PLACE)
        .resizable(false)
        .constrain(true)
        .show(context, |ui| {
            ui.horizontal(|ui| {
                history(app, ui);
                ui.separator();
                tools(app, ui);
                ui.separator();
                grid(app, ui);
                ui.separator();
                save(app, ui);
            });
            if let Some(message) = &app.message {
                ui.label(message);
            }
        });
}

/// Undo and redo.
fn history(app: &mut App, ui: &mut egui::Ui) {
    let undo = ui.add_enabled(app.history.can_undo(), egui::Button::new(icons::UNDO));
    if undo.tip(&t!("edit.undo")).clicked() {
        app.undo();
    }
    let redo = ui.add_enabled(app.history.can_redo(), egui::Button::new(icons::REDO));
    if redo.tip(&t!("edit.redo")).clicked() {
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
        if ui
            .selectable_label(app.tool == tool, icon)
            .tip(&name)
            .clicked()
        {
            app.tool = tool;
        }
    }
}

/// The kind of grid.
fn grid(app: &mut App, ui: &mut egui::Ui) {
    let mut chosen = app.pattern.grid();
    egui::ComboBox::from_id_salt("grid")
        .selected_text(grid_name(chosen))
        .show_ui(ui, |ui| {
            for grid in Grid::ALL {
                ui.selectable_value(&mut chosen, grid, grid_name(grid));
            }
        })
        .response
        .named(&t!("grid.kind"));
    app.set_grid(chosen);
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
    let button = ui.add_enabled(app.path.is_some(), egui::Button::new(icons::SAVE));
    if button.tip(&hint).clicked() {
        app.save();
    }
}
