//! The mockups of obusina, one module each, shown by the gallery of
//! `showcase`. A mockup goes when the window has what it drew; none waits
//! now.

pub use showcase::Mockup;

/// Every mockup, in the order of the gallery.
pub fn all() -> Vec<Box<dyn Mockup>> {
    Vec::new()
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
