//! While a menu is open, nothing else of the window answers: a modal layer
//! with nothing to show lies over everything, and the menu above it. A press
//! beside the menu closes it and reaches nothing under the layer.

use egui::{Color32, Frame, Id, Modal};
use plate_menu::PlateMenu;

/// Lays a layer that takes every press under the menu of `menu`, and puts
/// the menu on top, for this frame. Called in every frame the menu is open.
pub fn under(context: &egui::Context, id: &str, menu: &PlateMenu) {
    let menu = menu.layer_id();
    Modal::new(Id::unique(("menu-blocker", id)))
        .backdrop_color(Color32::TRANSPARENT)
        .frame(Frame::NONE)
        .show(context, |_| {});
    context.move_to_top(menu);
}
