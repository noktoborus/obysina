//! The floating panel of tools.

use obusina_canvas::Turn;
use obusina_panel::{Edge, FloatingPanel, Orientation, Placement};
use rust_i18n::t;

use crate::app::{App, Tool};
use crate::ui::{button, icons, lines};

/// The name the place of the panel is kept by.
const NAME: &str = "tools";

/// A group of buttons, kept on one line.
#[derive(Debug, Clone, Copy)]
enum Group {
    /// Undo and redo.
    History,
    /// The quarter turns of the field.
    Turns,
    /// The pencil, the fill, the picker.
    Tools,
    /// The kind of grid.
    Grid,
    /// Save.
    Save,
}

/// The groups, in the order they stand.
const GROUPS: [Group; 5] = [
    Group::History,
    Group::Turns,
    Group::Tools,
    Group::Grid,
    Group::Save,
];

impl Group {
    /// How many buttons the group has.
    fn count(self) -> usize {
        match self {
            Group::History | Group::Turns => 2,
            Group::Tools => 3,
            Group::Grid | Group::Save => 1,
        }
    }

    /// Shows the buttons of the group.
    fn show(self, app: &mut App, ui: &mut egui::Ui) {
        match self {
            Group::History => history(app, ui),
            Group::Turns => turns(app, ui),
            Group::Tools => tools(app, ui),
            Group::Grid => grid(app, ui),
            Group::Save => save(app, ui),
        }
    }
}

/// Shows the panel of tools.
pub fn show(app: &mut App, context: &egui::Context) {
    let placement = app
        .settings
        .panel(NAME)
        .unwrap_or(Placement::docked(Edge::Top, 0.0));
    let shown = FloatingPanel::new(NAME, t!("panel.tools"))
        .placement(placement)
        .show(context, |ui, room| {
            let spacing = ui.spacing().item_spacing;
            let split = lines::split(room, &GROUPS.map(Group::count), spacing);
            ui.with_layout(lines::across(room.orientation), |ui| {
                for line in split {
                    ui.with_layout(lines::along_layout(room.orientation), |ui| {
                        for (index, group) in GROUPS[line].iter().enumerate() {
                            if index > 0 {
                                gap(ui, room.orientation);
                            }
                            group.show(app, ui);
                        }
                    });
                }
            });
            if let Some(message) = &app.message {
                ui.label(message);
            }
        });
    if shown.changed {
        app.keep_panel(NAME, shown.placement);
    }
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
