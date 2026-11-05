//! While a menu is open, nothing else of the window answers: a modal layer
//! with nothing to show lies over everything, and the menu above it. A press
//! beside the menu does not close it either; a key or an entry does.

use egui::{Color32, Frame, Id, Modal};
use plate_menu::{Beside, PlateMenu};

/// Lays a layer that takes every press under the menu of `menu`, and puts
/// the menu on top, for this frame. Called in every frame the menu is open.
pub fn under(context: &egui::Context, id: &str, menu: &mut PlateMenu) {
    menu.beside(Beside::Ignored);
    let menu = menu.layer_id();
    Modal::new(Id::unique(("menu-blocker", id)))
        .backdrop_color(Color32::TRANSPARENT)
        .frame(Frame::NONE)
        .show(context, |_| {});
    context.move_to_top(menu);
}
