//! The parts the list is drawn of: a card of a pattern with its picture, a
//! card of an action and a plate across the list.

use egui::{Color32, Rect, Sense, TextureId, Vec2, pos2, vec2};
use obusina_pattern::Rgb;

use crate::ui::access::Named;

/// The least width of a card.
pub const CARD_LEAST: f32 = 150.0;
/// The height of the picture of a card to its width.
pub const THUMB_RATIO: f32 = 110.0 / 138.0;
/// The margin of a card around its picture.
pub const CARD_MARGIN: f32 = 6.0;
/// The height of the lines of a card under its picture.
pub const CARD_TEXT: f32 = 62.0;
/// The space between two cards.
pub const GAP: f32 = 10.0;
/// The height of a plate across the list.
pub const PLATE: f32 = 56.0;
/// The mark of a card chosen.
const MARK: &str = "\u{2714}";

/// What a card of a pattern says under its picture.
pub struct Lines<'a> {
    /// The name.
    pub name: &'a str,
    /// The size and the beads.
    pub size: String,
    /// When it changed, or the days it has left in the trash.
    pub last: String,
    /// The paper of the pattern, the ground of its picture.
    pub paper: Color32,
}

/// One card: its picture (`thumbs.rs`), or the paper of the pattern until
/// it is drawn; its name, its size and `last`; the mark of a
/// choice when `mark` is given; when `focused`, the keys stand on it: it is
/// filled with the colour of the selection.
pub fn card(
    ui: &mut egui::Ui,
    size: Vec2,
    picture: Option<TextureId>,
    lines: &Lines<'_>,
    (mark, focused): (Option<bool>, bool),
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);
    let visuals = ui.style().interact(&response);
    let selection = ui.visuals().selection;
    let ground = match focused {
        true => selection.bg_fill,
        false => ui.visuals().extreme_bg_color,
    };
    painter.rect_filled(rect, 6.0, ground);
    painter.rect_stroke(rect, 6.0, visuals.bg_stroke, egui::StrokeKind::Inside);
    let inner = size.x - 2.0 * CARD_MARGIN;
    let frame = Rect::from_min_size(
        rect.min + Vec2::splat(CARD_MARGIN),
        vec2(inner, inner * THUMB_RATIO),
    );
    painter.rect_filled(frame, 4.0, lines.paper);
    if let Some(texture) = picture {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(texture, frame, uv, Color32::WHITE);
    }
    let (text, weak) = match focused {
        true => (selection.stroke.color, selection.stroke.color),
        false => (ui.visuals().text_color(), ui.visuals().weak_text_color()),
    };
    let mut line = frame.bottom() + 6.0;
    let room = rect.width() - 16.0;
    for (words, colour, height) in [
        (lines.name, text, 13.0),
        (lines.size.as_str(), weak, 11.0),
        (lines.last.as_str(), weak, 11.0),
    ] {
        let galley = painter.layout(
            words.to_owned(),
            egui::FontId::proportional(height),
            colour,
            room,
        );
        let mut job = (*galley.job).clone();
        job.wrap.max_rows = 1;
        job.wrap.overflow_character = Some('\u{2026}');
        let galley = painter.fonts_mut(|fonts| fonts.layout_job(job));
        painter.galley(pos2(rect.left() + 8.0, line), galley, colour);
        line += height + 5.0;
    }
    if let Some(chosen) = mark {
        let at = pos2(rect.right() - 20.0, rect.top() + 20.0);
        let accent = ui.visuals().selection.bg_fill;
        if chosen {
            painter.circle_filled(at, 13.0, accent);
            painter.text(
                at,
                egui::Align2::CENTER_CENTER,
                MARK,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        } else {
            let ring = egui::Stroke::new(2.0, Color32::WHITE);
            painter.circle(at, 13.0, Color32::from_black_alpha(90), ring);
        }
    }
    response.named(lines.name)
}

/// A card that does something rather than open a pattern: its icon over its
/// words; filled with the colour of the selection when the keys stand on it.
pub fn action_card(
    ui: &mut egui::Ui,
    size: Vec2,
    (icon, words): (&str, &str),
    focused: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let visuals = ui.style().interact(&response);
    let painter = ui.painter_at(rect);
    let selection = ui.visuals().selection;
    let ink = match focused {
        true => selection.stroke.color,
        false => visuals.fg_stroke.color,
    };
    if focused {
        painter.rect_filled(rect, 6.0, selection.bg_fill);
    }
    painter.rect_stroke(rect, 6.0, visuals.fg_stroke, egui::StrokeKind::Inside);
    painter.text(
        rect.center() - vec2(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(36.0),
        ink,
    );
    painter.text(
        rect.center() + vec2(0.0, 26.0),
        egui::Align2::CENTER_CENTER,
        words,
        egui::FontId::proportional(14.0),
        ink,
    );
    response.named(words)
}

/// A plate across the list: an icon, then its words. A plate too narrow for
/// its words on one line puts what follows " · ", a count, on a line of its
/// own, never cutting a word.
pub fn plate(ui: &mut egui::Ui, width: f32, icon: &str, words: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, PLATE), Sense::click());
    let visuals = ui.style().interact(&response);
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
    painter.rect_stroke(rect, 6.0, visuals.bg_stroke, egui::StrokeKind::Inside);
    painter.text(
        rect.left_center() + vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(24.0),
        visuals.fg_stroke.color,
    );
    let room = rect.width() - 48.0 - 8.0;
    let font = egui::FontId::proportional(14.0);
    let colour = visuals.fg_stroke.color;
    let mut galley = painter.layout_no_wrap(words.to_owned(), font.clone(), colour);
    if galley.size().x > room {
        galley = painter.layout(words.replacen(" \u{b7} ", "\n", 1), font, colour, room);
    }
    painter.galley(
        rect.left_center() + vec2(48.0, -galley.size().y / 2.0),
        galley,
        colour,
    );
    response.named(words)
}

/// A colour of the pattern as the toolkit draws it.
pub fn colour(Rgb([red, green, blue]): Rgb) -> Color32 {
    Color32::from_rgb(red, green, blue)
}
