//! The mockups, one module each.

mod palette_editor;

/// A mockup: a part of the window drawn with made up data.
pub trait Mockup {
    /// The name in the gallery and in the address, `#name`.
    fn name(&self) -> &'static str;

    /// Draws the mockup in the space of `ui`. A panel or a window of it is
    /// shown on the context of `ui`.
    fn show(&mut self, ui: &mut egui::Ui);
}

/// Every mockup, in the order of the gallery.
pub fn all() -> Vec<Box<dyn Mockup>> {
    vec![Box::new(palette_editor::PaletteEditor::default())]
}

/// The size of a button of a panel, `button::SIZE` of the window.
pub const BUTTON: egui::Vec2 = egui::vec2(44.0, 36.0);
/// The size of the text on a button, `TEXT` of `button.rs` of the window.
pub const TEXT: f32 = 18.0;

/// A button of a panel, as the window makes it.
pub fn button(label: &str) -> obusina_button::SquareButton<'_> {
    obusina_button::SquareButton::new(label, BUTTON).text_size(TEXT)
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;

    /// Every mockup draws a few frames without a panic.
    #[test]
    fn every_mockup_draws() {
        for mut mock in super::all() {
            let mut harness = Harness::new_ui(move |ui| mock.show(ui));
            harness.run();
        }
    }
}
