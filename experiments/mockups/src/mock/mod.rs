//! The mockups of obusina, one module each, shown by the gallery of
//! `showcase`.

mod palette_editor;

pub use showcase::{Lang, Mockup, named, named_rect};

/// Every mockup, in the order of the gallery.
pub fn all() -> Vec<Box<dyn Mockup>> {
    vec![
        Box::new(palette_editor::PaletteEditor::new(
            palette_editor::Reorder::Menu,
        )),
        Box::new(palette_editor::PaletteEditor::new(
            palette_editor::Reorder::Drag,
        )),
    ]
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
            for lang in super::Lang::ALL {
                let mut harness = Harness::new_ui(|ui| mock.show(ui, lang));
                harness.run();
            }
        }
    }
}
