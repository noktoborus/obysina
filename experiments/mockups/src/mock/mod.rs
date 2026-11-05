//! The mockups of obusina, one module each, shown by the gallery of
//! `showcase`.

mod numpad;
mod palette_editor;
mod panel;

pub use showcase::{Lang, Mockup, named, named_rect};

/// Every mockup, in the order of the gallery.
pub fn all() -> Vec<Box<dyn Mockup>> {
    vec![
        Box::new(palette_editor::PaletteEditor::new()),
        Box::new(panel::Panel::new()),
        Box::new(numpad::Numpad),
    ]
}

/// The size of a button of a panel, `button::SIZE` of the window.
pub const BUTTON: egui::Vec2 = egui::vec2(44.0, 36.0);
/// The size of the text on a button, `TEXT` of `button.rs` of the window.
pub const TEXT: f32 = 18.0;

/// The height of the bar of the colour in hand.
const BAR: f32 = 4.0;
/// The space between the bar and the swatch.
const BAR_GAP: f32 = 2.0;

/// Draws the swatch of the colour in hand over the button in `rect`: lower,
/// with a bar under it, of one colour for every swatch. The bar lies on the
/// ground of the panel, so no colour hides it. With no `fill` the swatch is
/// hatched.
pub fn in_hand(ui: &egui::Ui, rect: egui::Rect, fill: Option<egui::Color32>, number: &str) {
    let inner =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - BAR - BAR_GAP));
    let corner = ui.visuals().widgets.inactive.corner_radius;
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, ui.visuals().panel_fill);
    match fill {
        Some(fill) => {
            painter.rect(
                inner,
                corner,
                fill,
                egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
                egui::StrokeKind::Inside,
            );
            painter.text(
                inner.center(),
                egui::Align2::CENTER_CENTER,
                number,
                egui::FontId::proportional(TEXT),
                obusina_button::contrast(fill),
            );
        }
        None => palette_editor::hatched_swatch(ui, inner, number),
    }
    let bar = egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - BAR), rect.max);
    painter.rect_filled(bar, 1.0, ui.visuals().strong_text_color());
}

/// The entry that deletes `count` beads.
pub fn delete_beads(lang: Lang, count: usize) -> String {
    let word = match (count % 10, count % 100) {
        (1, hundred) if hundred != 11 => "бусину",
        (2..=4, hundred) if !(12..=14).contains(&hundred) => "бусины",
        _ => "бусин",
    };
    match lang {
        Lang::Ru => format!("Удалить {count} {word}"),
        Lang::En => format!("Delete {count} beads"),
    }
}

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
