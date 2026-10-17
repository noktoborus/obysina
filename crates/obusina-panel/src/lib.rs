#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod zone;

pub use zone::{EDGE, Orientation, Zone};

use egui::{Align2, Context, Id, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};

/// The thin side of the grip: wide enough for a finger.
const GRIP_THIN: f32 = 18.0;
/// The long side of the grip.
const GRIP_LONG: f32 = 36.0;
/// The distance between two dots of the grip.
const DOT_STEP: f32 = 6.0;
/// The radius of a dot of the grip.
const DOT_RADIUS: f32 = 1.5;

/// What a panel remembers from frame to frame.
#[derive(Debug, Clone, Copy)]
struct State {
    /// The top left corner.
    position: Pos2,
    /// Only the grip is shown.
    folded: bool,
    /// How the content is laid out.
    orientation: Orientation,
}

/// What the panel did in this frame.
pub struct PanelResponse<R> {
    /// What the content gave back; `None` while the panel is folded.
    pub inner: Option<R>,
    /// The grip.
    pub grip: Response,
    /// The whole panel.
    pub rect: Rect,
    /// How the content is laid out now.
    pub orientation: Orientation,
}

/// A floating panel with no title: a grip of dots moves it, a tap on the grip
/// folds it.
///
/// While it is dragged, the zone of the window under the pointer lays it out
/// ([`Zone::orientation`]). It is kept inside the window.
pub struct FloatingPanel {
    id: Id,
    name: String,
    corner: Align2,
    offset: Vec2,
    orientation: Orientation,
}

impl FloatingPanel {
    /// A panel; `name` is what a screen reader reads for its grip.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, name: impl Into<String>) -> Self {
        FloatingPanel {
            id: Id::unique(("floating-panel", id_salt)),
            name: name.into(),
            corner: Align2::LEFT_TOP,
            offset: Vec2::ZERO,
            orientation: Orientation::Horizontal,
        }
    }

    /// Where the panel stands the first time: its `corner` at the same
    /// corner of the window, moved by `offset`.
    pub fn place(mut self, corner: Align2, offset: Vec2) -> Self {
        self.corner = corner;
        self.offset = offset;
        self
    }

    /// How the panel is laid out the first time.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Shows the panel; `add_contents` lays out the content in the
    /// orientation it is given.
    pub fn show<R>(
        self,
        context: &Context,
        add_contents: impl FnOnce(&mut Ui, Orientation) -> R,
    ) -> PanelResponse<R> {
        let window = context.content_rect();
        let stored = context.data(|data| data.get_temp::<State>(self.id));
        let mut state = stored.unwrap_or(State {
            position: self.corner.pos_in_rect(&window) + self.offset,
            folded: false,
            orientation: self.orientation,
        });
        let pivot = if stored.is_some() {
            Align2::LEFT_TOP
        } else {
            self.corner
        };

        let shown = egui::Area::new(self.id)
            .order(egui::Order::Foreground)
            .movable(false)
            .constrain(false)
            .pivot(pivot)
            .current_pos(state.position)
            .show(context, |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| match state.orientation {
                    Orientation::Horizontal => {
                        ui.horizontal(|ui| self.body(ui, &state, add_contents))
                            .inner
                    }
                    Orientation::Vertical => {
                        ui.vertical(|ui| self.body(ui, &state, add_contents)).inner
                    }
                })
            });
        let (grip, inner) = shown.inner.inner;
        let rect = shown.response.rect;

        if stored.is_none() {
            let anchor = state.position;
            let pivot = vec2(self.corner.x().to_factor(), self.corner.y().to_factor());
            state.position = anchor - pivot * rect.size();
        } else if grip.dragged() {
            state.position = rect.min + grip.drag_delta();
            let zone = grip
                .interact_pointer_pos()
                .and_then(|point| Zone::of(window, point).orientation());
            if let Some(orientation) = zone {
                state.orientation = orientation;
            }
        } else {
            state.position = rect.min;
        }
        if grip.clicked() {
            state.folded = !state.folded;
        }
        let measured = window.is_positive();
        if measured {
            state.position = inside(window, Rect::from_min_size(state.position, rect.size()));
            context.data_mut(|data| data.insert_temp(self.id, state));
        }
        if !measured || stored.is_none() || grip.dragged() || grip.clicked() {
            context.request_repaint();
        }

        PanelResponse {
            inner,
            grip,
            rect,
            orientation: state.orientation,
        }
    }

    /// The grip, then the content unless the panel is folded.
    fn body<R>(
        &self,
        ui: &mut Ui,
        state: &State,
        add_contents: impl FnOnce(&mut Ui, Orientation) -> R,
    ) -> (Response, Option<R>) {
        let grip = self.grip(ui, state);
        let inner = (!state.folded).then(|| add_contents(ui, state.orientation));
        (grip, inner)
    }

    /// The grip: a field of dots across the start of the panel.
    fn grip(&self, ui: &mut Ui, state: &State) -> Response {
        let size = match state.orientation {
            Orientation::Horizontal => vec2(GRIP_THIN, GRIP_LONG),
            Orientation::Vertical => vec2(GRIP_LONG, GRIP_THIN),
        };
        let response = ui.allocate_response(size, Sense::click_and_drag());
        let name = self.name.clone();
        let folded = state.folded;
        response.widget_info(move || {
            let mut info = egui::WidgetInfo::labeled(egui::accesskit::Role::Button, true, &name);
            info.selected = Some(folded);
            info
        });
        let visuals = ui.style().interact(&response);
        let colour = visuals.fg_stroke.color;
        let painter = ui.painter();
        let columns = (size.x / DOT_STEP).floor() as usize;
        let rows = (size.y / DOT_STEP).floor() as usize;
        let start =
            response.rect.center() - vec2(columns as f32 - 1.0, rows as f32 - 1.0) * DOT_STEP / 2.0;
        for row in 0..rows {
            for column in 0..columns {
                let centre = start + vec2(column as f32, row as f32) * DOT_STEP;
                painter.circle_filled(centre, DOT_RADIUS, colour);
            }
        }
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        response
    }
}

/// The top left corner that keeps `rect` inside `window`, as far as it fits.
fn inside(window: Rect, rect: Rect) -> Pos2 {
    let x = rect
        .min
        .x
        .min(window.max.x - rect.width())
        .max(window.min.x);
    let y = rect
        .min
        .y
        .min(window.max.y - rect.height())
        .max(window.min.y);
    egui::pos2(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn a_panel_past_the_edge_comes_back_inside() {
        let window = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 300.0));
        let rect = Rect::from_min_size(pos2(380.0, -10.0), vec2(100.0, 50.0));

        assert_eq!(inside(window, rect), pos2(300.0, 0.0));
    }

    #[test]
    fn a_panel_wider_than_the_window_keeps_its_start_in_sight() {
        let window = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 300.0));
        let rect = Rect::from_min_size(pos2(50.0, 10.0), vec2(200.0, 50.0));

        assert_eq!(inside(window, rect), pos2(0.0, 10.0));
    }
}
