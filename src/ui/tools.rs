//! The floating panel of tools. While the corners of the picture under the
//! pattern are edited, it holds only the two buttons that end the editing.
//! A locked pattern is only looked at: the panel holds the settings, the
//! shadows, the underlay and the way back.

use obusina_panel::{FloatingPanel, Orientation, Placement};
use rust_i18n::t;

use crate::app::App;
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
            } else if app.pattern.locked() {
                viewer(app, ui, room.orientation);
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

/// The tools in one row, a group after another: undo and redo, the
/// settings and the shadows, the underlay, the way back.
fn row(app: &mut App, ui: &mut egui::Ui) {
    history(app, ui);
    gap(ui, Orientation::Horizontal);
    viewer(app, ui, Orientation::Horizontal);
}

/// The tools in a column two buttons wide, like the swatches of the beads:
/// a group of two on a line.
///
/// The lines are one block, so the block stands in the middle across the
/// panel.
fn column(app: &mut App, ui: &mut egui::Ui) {
    let orientation = Orientation::Vertical;
    ui.vertical(|ui| {
        ui.horizontal(|ui| history(app, ui));
        gap(ui, orientation);
        viewer(app, ui, orientation);
    });
}

/// The tools of a locked pattern, the last ones of the others: the settings
/// and the shadows, the underlay and the way back; in a column two buttons
/// wide.
fn viewer(app: &mut App, ui: &mut egui::Ui, orientation: Orientation) {
    match orientation {
        Orientation::Horizontal => {
            grid(app, ui);
            gap(ui, orientation);
            crate::ui::backdrop_menu::button(app, ui);
            gap(ui, orientation);
            back(app, ui);
        }
        Orientation::Vertical => {
            ui.vertical(|ui| {
                ui.horizontal(|ui| grid(app, ui));
                gap(ui, orientation);
                ui.horizontal(|ui| {
                    crate::ui::backdrop_menu::button(app, ui);
                    back(app, ui);
                });
            });
        }
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
