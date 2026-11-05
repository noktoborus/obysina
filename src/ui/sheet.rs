//! The size of the sheet: two fields of the tools, its columns and its rows.
//!
//! The sheet is the size of the work area set by hand, kept in the pattern
//! and its file. A new pattern takes the cells of the first screen; these
//! fields show the size and change it, keeping its middle, as a step of
//! undo. A size is never smaller than the box around the beads, and the
//! sheet moves as little as it takes to hold them.

use rust_i18n::t;

use crate::app::App;
use crate::ui::access::Named;
use crate::ui::button;

/// The most columns or rows a sheet can have.
pub const MAX_SIDE: usize = 1000;

/// The width and the height of the sheet. Before the field is first shown
/// a new pattern has no sheet, and nothing is shown.
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let Some(sheet) = app.pattern.sheet() else {
        return;
    };
    let beads = app.pattern.bounds();
    let least =
        |side: fn(&obusina_pattern::Bounds) -> usize| beads.as_ref().map_or(1, side).min(MAX_SIDE);
    let mut width = sheet.width();
    let mut height = sheet.height();
    let width_changed = side(
        ui,
        &mut width,
        least(|box_| box_.width()),
        &t!("sheet.width"),
    );
    let height_changed = side(
        ui,
        &mut height,
        least(|box_| box_.height()),
        &t!("sheet.height"),
    );
    if width_changed || height_changed {
        let resized = sheet.resized_over(width, height, beads);
        app.history.resize_sheet(&mut app.pattern, resized);
    }
}

/// One field: a number dragged or typed, the size of a button, from `least`.
fn side(ui: &mut egui::Ui, value: &mut usize, least: usize, name: &str) -> bool {
    let field = egui::DragValue::new(value).range(least..=MAX_SIDE);
    ui.add_sized(button::SIZE, field).tip(name).changed()
}
