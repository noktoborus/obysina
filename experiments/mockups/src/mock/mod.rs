//! The mockups of obusina, one module each, shown by the gallery of
//! `showcase`. A mockup goes when the window has what it drew.

mod glow;
mod projects;
mod settings;
mod wrap;

pub use showcase::{Lang, Mockup, named};

/// The size of a button of a panel, `button::SIZE` of the window.
pub const BUTTON: egui::Vec2 = egui::vec2(44.0, 36.0);

/// Every mockup, in the order of the gallery.
pub fn all() -> Vec<Box<dyn Mockup>> {
    vec![
        Box::new(projects::Projects::new()),
        Box::new(settings::Settings::new()),
        Box::new(glow::Glow::new()),
        Box::new(wrap::Wrap::new()),
    ]
}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;

    /// Every mockup draws a few frames without a panic.
    #[test]
    fn every_mockup_draws() {
        for mut mock in super::all() {
            for lang in showcase::Lang::ALL {
                let mut harness = Harness::new_ui(|ui| mock.show(ui, lang));
                harness.run();
            }
        }
    }
}
