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

/// Which button pressed on a bead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The first button, or a tap of one finger.
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
    /// The bead pressed on and let go without a drag, and by which button.
    pub pressed: Option<(Cell, Press)>,
}

/// The field of beads: draws a pattern over all the space it is given.
///
/// ```text
/// press, let go in place      ──► a press on the bead under it
/// press, drag (any button)    ──► the field moves with the pointer
/// two fingers                 ──► the field moves and zooms
/// wheel / ctrl + wheel        ──► the field moves / zooms
/// ```
///
/// egui tells a click from a drag: a press that moves further than a few
/// points, or is held too long, is a drag and presses on nothing.
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
        if !self.view.centred && rect.is_positive() {
            self.view.centre(rect.size(), self.extent());
        }
        self.navigate(ui, &response);

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

        let hovered = response
            .hover_pos()
            .and_then(|point| self.cell_at(origin, point));
        let pressed = self.pressed(&response, origin);
        CanvasResponse {
            response,
            hovered,
            pressed,
        }
    }

    /// Moves and zooms the view by what the input asks.
    ///
    /// A touch does not move the pointer, so two fingers are asked where they
    /// are, not whether the field is hovered. Their first finger is also a
    /// drag of the pointer, which is then not counted twice.
    fn navigate(&mut self, ui: &Ui, response: &Response) {
        let rect = response.rect;
        let (touch, zoom, scroll) = ui.input(|input| {
            (
                input.multi_touch(),
                input.zoom_delta(),
                input.smooth_scroll_delta(),
            )
        });
        if let Some(touch) = touch.filter(|touch| rect.contains(touch.center_pos)) {
            self.view
                .zoom(touch.zoom_delta, touch.center_pos - rect.min);
            self.view.offset += touch.translation_delta;
            return;
        }
        if let Some(pointer) = response.hover_pos() {
            if zoom != 1.0 {
                self.view.zoom(zoom, pointer - rect.min);
            }
            self.view.offset += scroll;
        }
        if response.dragged() {
            self.view.offset += response.drag_delta();
        }
    }

    /// The bead pressed on and let go in place in this frame, and by which
    /// button.
    fn pressed(&self, response: &Response, origin: Pos2) -> Option<(Cell, Press)> {
        let press = if response.clicked() {
            Press::Primary
        } else if response.secondary_clicked() {
            Press::Secondary
        } else {
            return None;
        };
        let point = response.interact_pointer_pos()?;
        self.cell_at(origin, point).map(|cell| (cell, press))
    }

    /// The size of the whole pattern on the screen.
    fn extent(&self) -> Vec2 {
        let pattern = self.pattern;
        let (width, height) =
            pattern
                .grid()
                .extent(pattern.width(), pattern.height(), pattern.ratio());
        vec2(width, height) * self.view.bead
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
