//! The floating panel of tools. While the corners of the picture under the
//! pattern are edited, it holds only the two buttons that end the editing.

use obusina_canvas::Turn;
use obusina_panel::{FloatingPanel, Orientation, Placement};
use rust_i18n::t;

use crate::app::{App, Tool};
use crate::ui::{button, icons};

/// The name the place of the panel is kept by.
const NAME: &str = "tools";

/// Shows the panel of tools.
pub fn show(app: &mut App, context: &egui::Context) {
    let placement = app
        .settings
        .panel(NAME)
        .unwrap_or(Placement::new(egui::Vec2::ZERO, Orientation::Horizontal));
    let editing = app.backdrop.editing();
    let shown = FloatingPanel::new(NAME, t!("panel.tools"))
        .placement(placement)
        .content(editing)
        .show(context, |ui, room| {
            if editing {
                crate::ui::backdrop_menu::editing(app, ui);
            } else {
                match room.orientation {
                    Orientation::Horizontal => row(app, ui),
                    Orientation::Vertical => column(app, ui),
                }
            }
            if let Some(message) = &app.message {
                ui.label(message);
            }
        });
    if shown.changed {
        app.keep_panel(NAME, shown.placement);
    }
}

/// The tools in one row, a group after another.
fn row(app: &mut App, ui: &mut egui::Ui) {
    let orientation = Orientation::Horizontal;
    history(app, ui);
    gap(ui, orientation);
    turns(app, ui);
    gap(ui, orientation);
    tools(app, ui);
    gap(ui, orientation);
    grid(app, ui);
    gap(ui, orientation);
    crate::ui::backdrop_menu::button(app, ui);
    gap(ui, orientation);
    back(app, ui);
}

/// The tools in a column two buttons wide, like the swatches of the beads:
/// a group of two on a line, the picture and the way back on the last one.
///
/// The lines are one block, so the block stands in the middle across the
/// panel.
fn column(app: &mut App, ui: &mut egui::Ui) {
    let orientation = Orientation::Vertical;
    ui.vertical(|ui| {
        ui.horizontal(|ui| history(app, ui));
        gap(ui, orientation);
        ui.horizontal(|ui| turns(app, ui));
        gap(ui, orientation);
        ui.horizontal(|ui| tools(app, ui));
        gap(ui, orientation);
        ui.horizontal(|ui| grid(app, ui));
        gap(ui, orientation);
        ui.horizontal(|ui| {
            crate::ui::backdrop_menu::button(app, ui);
            back(app, ui);
        });
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
        (Tool::Pick, icons::PICK, t!("tool.pick")),
    ] {
        if button::toggle(ui, app.tool == tool, icon, &name).clicked() {
            app.tool = tool;
        }
    }
}

/// The settings: a button that opens their menu
/// ([`crate::ui::settings_menu`]); and whether the empty places show their
/// shadows.
fn grid(app: &mut App, ui: &mut egui::Ui) {
    if button::icon(ui, icons::SETTINGS, &t!("settings.open"), true).clicked() {
        crate::ui::settings_menu::open(app);
    }
    let shadows = app.settings.shadows();
    if button::toggle(ui, shadows, icons::SHADOW, &t!("view.shadows")).clicked() {
        app.keep_shadows(!shadows);
    }
}

/// The way back to the list of patterns, which saves the pattern; none
/// when there is no list.
fn back(app: &mut App, ui: &mut egui::Ui) {
    let listed = app.library.is_some();
    if button::icon(ui, icons::BACK, &t!("projects.back"), listed).clicked() {
        let time = ui.input(|input| input.time);
        app.back_to_list(ui.ctx(), time);
    }
}
