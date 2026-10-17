//! The buttons of the panels: one size for all of them, the size of a swatch
//! of the palette, which is large enough for a finger.

use egui::{Button, Color32, Response, RichText, Stroke, Ui, Vec2, vec2};

use crate::ui::access::Named;

/// The size of every button of a panel.
pub const SIZE: Vec2 = vec2(44.0, 36.0);
/// The size of the text and of the icon on a button.
const TEXT: f32 = 18.0;
/// The width of the outline of the swatch in use.
const SELECTED_LINE: f32 = 3.0;
/// The width of the outline of any other swatch.
const LINE: f32 = 1.0;

/// A button that carries an icon; `hint` is shown on hover and read by a
/// screen reader.
pub fn icon(ui: &mut Ui, icon: &str, hint: &str, enabled: bool) -> Response {
    ui.add_enabled(enabled, Button::new(text(icon)).min_size(SIZE))
        .tip(hint)
}

/// A button with an icon that is on or off.
pub fn toggle(ui: &mut Ui, on: bool, icon: &str, hint: &str) -> Response {
    ui.add(Button::new(text(icon)).selected(on).min_size(SIZE))
        .tip(hint)
}

/// A button filled with a colour, `label` on it in black or white, whichever
/// reads on the colour; `name` is what a screen reader reads.
pub fn swatch(ui: &mut Ui, fill: Color32, label: &str, selected: bool, name: &str) -> Response {
    let stroke = if selected {
        Stroke::new(SELECTED_LINE, ui.visuals().selection.stroke.color)
    } else {
        Stroke::new(LINE, ui.visuals().widgets.inactive.bg_stroke.color)
    };
    ui.add(
        Button::new(text(label).color(contrast(fill)).strong())
            .fill(fill)
            .stroke(stroke)
            .min_size(SIZE),
    )
    .named(name)
}

/// The text of a button, at the size of every button.
fn text(text: &str) -> RichText {
    RichText::new(text).size(TEXT)
}

/// Black or white, whichever reads on `fill`.
fn contrast(fill: Color32) -> Color32 {
    let light =
        0.299 * f32::from(fill.r()) + 0.587 * f32::from(fill.g()) + 0.114 * f32::from(fill.b());
    if light > 140.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
