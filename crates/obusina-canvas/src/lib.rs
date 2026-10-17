#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod view;

pub use view::{MAX_BEAD, MIN_BEAD, View};

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use obusina_pattern::{Cell, Pattern};

/// The part of a bead left as a gap to its neighbours.
const GAP: f32 = 0.08;
/// The rounding of a bead, as a part of its height.
const ROUNDING: f32 = 0.3;
/// The ground under the beads: a neutral grey, the same in every theme, so
/// that a colour looks the same and a black bead is seen.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The outline of an empty place.
const EMPTY: Color32 = Color32::from_rgb(0xa8, 0xa6, 0xa0);

/// Which colour a press asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The first button, or one finger.
    Primary,
    /// The second button.
    Secondary,
}

/// What the pointer did on the field in this frame.
pub struct CanvasResponse {
    /// The response of the whole field.
    pub response: Response,
    /// The bead under the pointer.
    pub hovered: Option<Cell>,
    /// The bead pressed on, and how; `None` while the field is moved.
    pub pressed: Option<(Cell, Press)>,
    /// The press that goes on became a move: a second finger came, or the
    /// middle button. What the press painted is to be taken back.
    pub moving: bool,
    /// A press ended in this frame.
    pub released: bool,
}

/// The field of beads: draws a pattern over all the space it is given.
///
/// Two fingers, the wheel and the middle button move the field; a pinch and
/// the wheel with ctrl zoom it. One finger and the other buttons press on a
/// bead.
pub struct Canvas<'a> {
    pattern: &'a Pattern,
    view: &'a mut View,
}

impl<'a> Canvas<'a> {
    /// A field over `pattern`, moved and zoomed by `view`.
    pub fn new(pattern: &'a Pattern, view: &'a mut View) -> Self {
        Canvas { pattern, view }
    }

    /// Draws the field over all the space left in `ui`.
    pub fn show(mut self, ui: &mut Ui) -> CanvasResponse {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let moving = self.navigate(ui, &response);

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, PAPER);
        let origin = rect.min + self.view.offset;
        for cell in self.pattern.cells() {
            let bead = self.bead_rect(origin, cell);
            if !rect.intersects(bead) {
                continue;
            }
            let rounding = bead.height() * ROUNDING;
            match self.pattern.get(cell) {
                Some(index) => {
                    let [red, green, blue] = self.pattern.palette()[index].rgb.0;
                    painter.rect_filled(bead, rounding, Color32::from_rgb(red, green, blue));
                }
                None => {
                    painter.rect_stroke(
                        bead,
                        rounding,
                        Stroke::new(1.0, EMPTY),
                        StrokeKind::Inside,
                    );
                }
            }
        }

        let moving = self.moving(ui, &response, moving);
        let hovered = response
            .hover_pos()
            .and_then(|point| self.cell_at(origin, point));
        let pressed = if moving {
            None
        } else {
            self.pressed(ui, &response, origin)
        };
        let released =
            response.drag_stopped() || response.clicked() || response.secondary_clicked();
        CanvasResponse {
            response,
            hovered,
            pressed,
            moving,
            released,
        }
    }

    /// Moves and zooms the view by what the input asks. True while the field
    /// is moved, so that no bead is pressed on by the same gesture.
    ///
    /// A touch does not move the pointer, so two fingers are asked where they
    /// are, not whether the field is hovered.
    fn navigate(&mut self, ui: &Ui, response: &Response) -> bool {
        let rect = response.rect;
        let (touch, zoom, scroll, middle) = ui.input(|input| {
            (
                input.multi_touch(),
                input.zoom_delta(),
                input.smooth_scroll_delta(),
                input.pointer.middle_down(),
            )
        });
        if let Some(touch) = touch.filter(|touch| rect.contains(touch.center_pos)) {
            self.view
                .zoom(touch.zoom_delta, touch.center_pos - rect.min);
            self.view.offset += touch.translation_delta;
            return true;
        }
        if let Some(pointer) = response.hover_pos() {
            if zoom != 1.0 {
                self.view.zoom(zoom, pointer - rect.min);
            }
            self.view.offset += scroll;
        }
        if middle && response.dragged() {
            self.view.offset += response.drag_delta();
        }
        middle
    }

    /// True from the frame a press becomes a move until nothing is down.
    ///
    /// The first finger of a pinch comes as a press, a frame before the
    /// second one; the press stays a move until the last finger is lifted.
    fn moving(&self, ui: &Ui, response: &Response, now: bool) -> bool {
        let id = response.id.with("moving");
        let before = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
        let down = ui.input(|input| input.pointer.any_down() || input.any_touches());
        let moving = (before || now) && (down || now);
        ui.data_mut(|data| data.insert_temp(id, moving));
        moving
    }

    /// The bead pressed on in this frame, and by which button.
    fn pressed(&self, ui: &Ui, response: &Response, origin: Pos2) -> Option<(Cell, Press)> {
        let point = response.interact_pointer_pos()?;
        let down = |button| ui.input(|input| input.pointer.button_down(button));
        let held = response.is_pointer_button_down_on();
        let press =
            if (held && down(egui::PointerButton::Secondary)) || response.secondary_clicked() {
                Press::Secondary
            } else if (held && down(egui::PointerButton::Primary)) || response.clicked() {
                Press::Primary
            } else {
                return None;
            };
        self.cell_at(origin, point).map(|cell| (cell, press))
    }

    /// The rectangle of one bead on the screen.
    fn bead_rect(&self, origin: Pos2, cell: Cell) -> Rect {
        let pattern = self.pattern;
        let (x, y) = pattern.grid().origin(cell, pattern.ratio());
        let size = vec2(pattern.ratio(), 1.0) * self.view.bead;
        let gap = Vec2::splat(self.view.bead * GAP);
        let min = origin + vec2(x, y) * self.view.bead;
        Rect::from_min_size(min + gap / 2.0, size - gap)
    }

    /// The bead under a point of the screen.
    fn cell_at(&self, origin: Pos2, point: Pos2) -> Option<Cell> {
        let pattern = self.pattern;
        let at = (point - origin) / self.view.bead;
        pattern.grid().cell_at(
            at.x,
            at.y,
            pattern.width(),
            pattern.height(),
            pattern.ratio(),
        )
    }
}

/// The point of the screen at the centre of a bead, for a test that presses
/// on it.
pub fn bead_centre(pattern: &Pattern, view: &View, field: Rect, cell: Cell) -> Pos2 {
    let (x, y) = pattern.grid().origin(cell, pattern.ratio());
    let centre = vec2(x + pattern.ratio() / 2.0, y + 0.5) * view.bead;
    pos2(field.min.x, field.min.y) + view.offset + centre
}
