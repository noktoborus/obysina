#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use egui::{
    Color32, Response, RichText, Sense, Stroke, StrokeKind, TextWrapMode, Ui, Vec2, Widget,
    WidgetText,
};

/// The width of the outline of a filled button that is selected.
const SELECTED_LINE: f32 = 3.0;
/// The width of the outline of any other filled button, so that a fill like
/// the panel under it is still seen as a button.
const LINE: f32 = 1.0;
/// The lightness above which black reads on a fill, and white below.
const LIGHT: f32 = 140.0;

/// A button of a fixed size, its label in the middle.
///
/// It is plain, selected, or filled with a colour of its own; on a fill the
/// label is black or white, whichever reads, and a selected one is outlined.
pub struct SquareButton<'a> {
    label: &'a str,
    size: Vec2,
    text_size: Option<f32>,
    fill: Option<Color32>,
    selected: bool,
    name: Option<&'a str>,
}

impl<'a> SquareButton<'a> {
    /// A button of `size` with `label` on it: text or an icon.
    pub fn new(label: &'a str, size: Vec2) -> Self {
        SquareButton {
            label,
            size,
            text_size: None,
            fill: None,
            selected: false,
            name: None,
        }
    }

    /// The size of the label; the size of the button text of the style when
    /// not given.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Fills the button with a colour of its own.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Shows the button as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// What a screen reader reads; the label when not given.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }
}

impl Widget for SquareButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        let name = self.name.unwrap_or(self.label).to_owned();
        let selected = self.selected;
        let enabled = ui.is_enabled();
        response.widget_info(move || {
            let mut info = egui::WidgetInfo::labeled(egui::accesskit::Role::Button, enabled, &name);
            info.selected = Some(selected);
            info
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let visuals = ui.style().interact_selectable(&response, self.selected);
        let (fill, stroke, text) = match self.fill {
            Some(fill) => {
                let stroke = if self.selected {
                    Stroke::new(SELECTED_LINE, ui.visuals().selection.stroke.color)
                } else {
                    Stroke::new(LINE, ui.visuals().weak_text_color())
                };
                (fill, stroke, contrast(fill))
            }
            None => (
                visuals.weak_bg_fill,
                visuals.bg_stroke,
                visuals.text_color(),
            ),
        };
        let painter = ui.painter();
        painter.rect(
            rect,
            visuals.corner_radius,
            fill,
            stroke,
            StrokeKind::Inside,
        );

        let mut label = RichText::new(self.label).color(text);
        if let Some(size) = self.text_size {
            label = label.size(size);
        }
        let galley = WidgetText::from(label).into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let corner = rect.center() - galley.size() / 2.0;
        painter.galley(corner, galley, text);
        response
    }
}

/// Black or white, whichever reads on `fill`.
pub fn contrast(fill: Color32) -> Color32 {
    let light =
        0.299 * f32::from(fill.r()) + 0.587 * f32::from(fill.g()) + 0.114 * f32::from(fill.b());
    if light > LIGHT {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_reads_on_a_light_fill_and_white_on_a_dark_one() {
        assert_eq!(
            contrast(Color32::from_rgb(0xf2, 0xef, 0xe6)),
            Color32::BLACK
        );
        assert_eq!(
            contrast(Color32::from_rgb(0x1b, 0x1b, 0x1b)),
            Color32::WHITE
        );
    }
}
