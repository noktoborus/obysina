//! The buttons of the panels: one size for all of them, the size of a swatch
//! of the palette, which is large enough for a finger. Each is an
//! `obusina_button::SquareButton`, its label in the middle.

use egui::{Color32, Response, Ui, Vec2, vec2};
use obusina_button::SquareButton;

use crate::ui::access::Named;

/// The size of every button of a panel.
pub const SIZE: Vec2 = vec2(44.0, 36.0);
/// The size of the text and of the icon on a button.
pub const TEXT: f32 = 18.0;

/// A button that carries an icon; `hint` is shown on hover and read by a
/// screen reader.
pub fn icon(ui: &mut Ui, icon: &str, hint: &str, enabled: bool) -> Response {
    ui.add_enabled(enabled, square(icon).name(hint)).tip(hint)
}

/// A button with an icon that is on or off.
pub fn toggle(ui: &mut Ui, on: bool, icon: &str, hint: &str) -> Response {
    ui.add(square(icon).selected(on).name(hint)).tip(hint)
}

/// A button filled with a colour, `label` on it; `name` is what a screen
/// reader reads.
pub fn swatch(ui: &mut Ui, fill: Color32, label: &str, selected: bool, name: &str) -> Response {
    ui.add(square(label).fill(fill).selected(selected).name(name))
}

/// A button of the panels with `label` on it.
fn square(label: &str) -> SquareButton<'_> {
    SquareButton::new(label, SIZE).text_size(TEXT)
}
