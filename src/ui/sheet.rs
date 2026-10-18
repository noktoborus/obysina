//! The size of the sheet: two fields of the tools, its columns and its rows.
//!
//! The canvas takes the sheet from the cells of the first screen; these
//! fields show that size and change it, keeping its middle.

use rust_i18n::t;

use crate::app::App;
use crate::ui::access::Named;
use crate::ui::button;

/// The most columns or rows a sheet can have.
pub const MAX_SIDE: usize = 1000;

/// The width and the height of the sheet. Before the field is first shown
/// there is no sheet, and nothing is shown.
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let Some(sheet) = app.view.sheet else {
        return;
    };
    let mut width = sheet.width();
    let mut height = sheet.height();
    let width_changed = side(ui, &mut width, &t!("sheet.width"));
    let height_changed = side(ui, &mut height, &t!("sheet.height"));
    if width_changed || height_changed {
        app.view.sheet = Some(sheet.resized(width, height));
    }
}

/// One field: a number dragged or typed, the size of a button.
fn side(ui: &mut egui::Ui, value: &mut usize, name: &str) -> bool {
    let field = egui::DragValue::new(value).range(1..=MAX_SIDE);
    ui.add_sized(button::SIZE, field).tip(name).changed()
}
