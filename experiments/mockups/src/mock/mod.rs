//! The mockups, one module each.

mod palette_editor;

use crate::note::Widget;

/// A mockup: a part of the window drawn with made up data.
pub trait Mockup {
    /// The name in the gallery and in the address, `#name`.
    fn name(&self) -> &'static str;

    /// Draws the mockup in the space of `ui`, its text in `lang`. A panel or
    /// a window of it is shown on the context of `ui`.
    fn show(&mut self, ui: &mut egui::Ui, lang: Lang);

    /// What the mockup shows, for a note: enough to draw it again the way
    /// it was.
    fn state(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

/// A language of the window, `locales/app.yml`: a mockup shows its text in
/// each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Russian.
    Ru,
    /// English.
    En,
}

impl Lang {
    /// Every language, in the order of the strip.
    pub const ALL: [Lang; 2] = [Lang::Ru, Lang::En];

    /// The code of the language.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }

    /// The text in this language, of the two given.
    pub fn tr<'a>(self, en: &'a str, ru: &'a str) -> &'a str {
        match self {
            Lang::Ru => ru,
            Lang::En => en,
        }
    }
}

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

/// The key the named widgets of a frame are kept by.
fn named_id() -> egui::Id {
    egui::Id::unique("mockups-named")
}

/// Gives a part with no name of its own, such as a window, a name a note
/// can carry. A widget with a name is seen in the tree of accesskit
/// already.
pub fn named(context: &egui::Context, response: egui::Response, name: &str) -> egui::Response {
    let rect = context
        .layer_transform_to_global(response.layer_id)
        .map_or(response.rect, |transform| transform * response.rect);
    named_rect(context, rect, name);
    response
}

/// Names a part of the window by where it stands, such as a menu the
/// mockup knows only the layer of.
pub fn named_rect(context: &egui::Context, rect: egui::Rect, name: &str) {
    let widget = Widget {
        name: name.to_string(),
        rect: [rect.min.x, rect.min.y, rect.max.x, rect.max.y],
        path: Vec::new(),
    };
    context.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<Widget>>(named_id())
            .push(widget)
    });
}

/// The widgets named in this frame.
pub fn named_widgets(context: &egui::Context) -> Vec<Widget> {
    context.data(|data| data.get_temp(named_id()).unwrap_or_default())
}

/// Forgets the widgets named in the last frame, at the start of a frame.
pub fn forget_named(context: &egui::Context) {
    context.data_mut(|data| data.remove::<Vec<Widget>>(named_id()));
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
