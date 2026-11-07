#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod volume;

use egui::{
    Color32, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, TextWrapMode, Ui, Vec2,
    Widget, WidgetText,
};

/// The width of the outline of a filled button that is selected.
const SELECTED_LINE: f32 = 3.0;
/// The width of the outline of any other filled button, so that a fill like
/// the panel under it is still seen as a button.
const LINE: f32 = 1.0;
/// The lightness above which black reads on a fill, and white below.
const LIGHT: f32 = 140.0;
/// The height of the bar under a marked button.
const BAR: f32 = 4.0;
/// The space between the bar and the button.
const BAR_GAP: f32 = 2.0;
/// The space between two lines of a hatch.
const HATCH_STEP: f32 = 5.0;
/// The room kept clear of the hatch around the label.
const LABEL_ROOM: Vec2 = Vec2::new(18.0, 22.0);

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
    marked: bool,
    hatched: bool,
    volume: bool,
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
            marked: false,
            hatched: false,
            volume: false,
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

    /// Marks the button with a bar under it, of one colour whatever its
    /// fill: the button gets lower, and the bar lies on the ground around it,
    /// so no fill hides it.
    pub fn marked(mut self, marked: bool) -> Self {
        self.marked = marked;
        self
    }

    /// Draws diagonal lines over the button, its label kept clear: a thing
    /// with no colour to show. A fill is not drawn.
    pub fn hatched(mut self, hatched: bool) -> Self {
        self.hatched = hatched;
        self
    }

    /// Draws a fill round in its light, as a round bead is: lighter in the
    /// middle, darker to the sides and the bottom. A button with no fill is
    /// not changed.
    pub fn volume(mut self, volume: bool) -> Self {
        self.volume = volume;
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
        let selected = self.selected || self.marked;
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
        let ground = ui.visuals().panel_fill;
        let bar_colour = ui.visuals().strong_text_color();
        let face = if self.marked {
            Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.max.y - BAR - BAR_GAP))
        } else {
            rect
        };
        let fill = if self.hatched { None } else { self.fill };
        let (fill, stroke, text) = match fill {
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
        if self.marked {
            painter.rect_filled(rect, 0.0, ground);
            let bar = Rect::from_min_max(Pos2::new(rect.min.x, rect.max.y - BAR), rect.max);
            painter.rect_filled(bar, 1.0, bar_colour);
        }
        let round = self.volume && !self.hatched && self.fill.is_some();
        if round {
            let radius = visuals.corner_radius.average();
            let texture = volume::texture(ui.ctx(), face.size(), radius);
            let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            painter.image(texture.id(), face, whole, fill);
            painter.rect_stroke(face, visuals.corner_radius, stroke, StrokeKind::Inside);
        } else {
            painter.rect(
                face,
                visuals.corner_radius,
                fill,
                stroke,
                StrokeKind::Inside,
            );
        }
        if self.hatched {
            hatch(painter, face, text);
            let room = Rect::from_center_size(face.center(), LABEL_ROOM);
            painter.rect_filled(room, 3.0, fill);
        }

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
        let corner = face.center() - galley.size() / 2.0;
        painter.galley(corner, galley, text);
        response
    }
}

/// Diagonal lines over `rect`, clipped to it.
fn hatch(painter: &egui::Painter, rect: Rect, colour: Color32) {
    let line = Stroke::new(1.0, colour.gamma_multiply(0.6));
    let clipped = painter.with_clip_rect(painter.clip_rect().intersect(rect.shrink(1.0)));
    let lines = ((rect.width() + rect.height()) / HATCH_STEP).ceil() as usize;
    for step in 0..lines {
        let along = step as f32 * HATCH_STEP;
        clipped.line_segment(
            [
                rect.left_top() + Vec2::new(along, 0.0),
                rect.left_top() + Vec2::new(0.0, along),
            ],
            line,
        );
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
