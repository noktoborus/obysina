#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod mesh;
mod view;

pub use view::{MAX_BEAD, MIN_BEAD, Turn, View};

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Pattern};

/// The part of a bead left as a gap to its neighbours.
const GAP: f32 = 0.08;
/// The rounding of a bead, as a part of its height.
const ROUNDING: f32 = 0.3;
/// The ground under the beads: a neutral grey, the same in every theme, so
/// that a colour looks the same and a black bead is seen.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The outline of an empty place in the active area.
const EMPTY: Color32 = Color32::from_rgb(0xa8, 0xa6, 0xa0);
/// The outline of an empty place outside the active area: halfway from
/// [`EMPTY`] to [`PAPER`].
const PALE: Color32 = Color32::from_rgb(0xbf, 0xbd, 0xb7);
/// The smallest height of a bead, in points, at which the empty places
/// outside the active area are drawn. Below it the ground stays bare there:
/// a screen of small beads holds tens of thousands of outlines.
pub const PALE_MIN_BEAD: f32 = 10.0;

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
    /// The bead under the pointer, if it is on the field.
    pub hovered: Option<Cell>,
    /// The bead pressed on and let go without a drag, and by which button.
    pub pressed: Option<(Cell, Press)>,
    /// The beads were tessellated anew in this frame; otherwise the mesh of
    /// an earlier frame was drawn, moved where the field is.
    pub tessellated: bool,
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
        self.view.field = rect.size();
        if !self.view.centred && rect.is_positive() {
            self.view.centre(rect.size(), self.middle());
        }
        self.navigate(ui, &response);

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, PAPER);
        let origin = rect.min + self.view.offset;
        let tessellated = self.draw_beads(ui, &painter, response.id, origin, rect);

        let hovered = response
            .hover_pos()
            .map(|point| self.cell_at(origin, point));
        let pressed = self.pressed(&response, origin);
        CanvasResponse {
            response,
            hovered,
            pressed,
            tessellated,
        }
    }

    /// Draws the beads from the mesh kept for the field, tessellated anew
    /// only when what they look like changed or the screen left the cells
    /// it holds. True when it was tessellated.
    fn draw_beads(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        field: egui::Id,
        origin: Pos2,
        screen: Rect,
    ) -> bool {
        let id = field.with("beads");
        let key = mesh::Key {
            revision: self.pattern.revision(),
            bead: self.view.bead,
            degrees: self.view.degrees(),
            pixels_per_point: ui.ctx().pixels_per_point(),
        };
        let visible = self.cells_under(origin, screen);
        let kept = ui
            .data_mut(|data| data.remove_temp::<mesh::Kept>(id))
            .filter(|kept| kept.key == key && kept.window.holds(visible));
        let tessellated = kept.is_none();
        let mut kept = kept.unwrap_or_else(|| {
            let window = self.cells_under(origin, screen.expand2(screen.size() / 2.0));
            mesh::tessellate(ui, key, window, origin, self.bead_shapes(origin, window))
        });
        kept.move_to(origin);
        painter.add(egui::Shape::Mesh(kept.mesh.clone()));
        ui.data_mut(|data| data.insert_temp(id, kept));
        tessellated
    }

    /// The cells whose beads lie, even in part, under `area` of the screen,
    /// the pattern standing at `origin`.
    fn cells_under(&self, origin: Pos2, area: Rect) -> Bounds {
        let corners = [
            area.left_top(),
            area.right_top(),
            area.right_bottom(),
            area.left_bottom(),
        ]
        .map(|corner| self.view.to_pattern(corner - origin) / self.view.bead);
        let min = corners
            .iter()
            .fold(corners[0], |min, corner| min.min(*corner));
        let max = corners
            .iter()
            .fold(corners[0], |max, corner| max.max(*corner));
        self.pattern
            .grid()
            .cells_in((min.x, min.y), (max.x, max.y), self.pattern.ratio())
    }

    /// A shape for every bead of `window`, the pattern standing at `origin`:
    /// a bead filled, an empty place in the active area outlined, one outside
    /// it outlined pale, or left bare when beads are small.
    ///
    /// The rounding is a part of the height of a bead before the field is
    /// turned, so a turned bead is rounded the same.
    fn bead_shapes(&self, origin: Pos2, window: Bounds) -> Vec<egui::Shape> {
        let active = self.pattern.bounds();
        let pale = self.view.bead >= PALE_MIN_BEAD;
        let rounding = self.view.bead * (1.0 - GAP) * ROUNDING;
        let outline = |bead, colour| {
            egui::Shape::rect_stroke(bead, rounding, Stroke::new(1.0, colour), StrokeKind::Inside)
        };
        window
            .cells()
            .filter_map(|cell| {
                let inside = active.is_some_and(|active| active.contains(cell));
                let shape = match self.pattern.get(cell) {
                    Some(index) => {
                        let [red, green, blue] = self.pattern.palette()[index].rgb.0;
                        egui::Shape::rect_filled(
                            self.bead_rect(origin, cell),
                            rounding,
                            Color32::from_rgb(red, green, blue),
                        )
                    }
                    None if inside => outline(self.bead_rect(origin, cell), EMPTY),
                    None if pale => outline(self.bead_rect(origin, cell), PALE),
                    None => return None,
                };
                Some(shape)
            })
            .collect()
    }

    /// The point of the pattern shown in the middle of the field the first
    /// time: the middle of the active area, or of cell (0, 0) when there is
    /// none.
    fn middle(&self) -> Vec2 {
        let pattern = self.pattern;
        let area = pattern
            .bounds()
            .unwrap_or(Bounds::of(Cell { column: 0, row: 0 }));
        let ((left, top), (right, bottom)) = pattern.grid().span(area, pattern.ratio());
        vec2(left + right, top + bottom) / 2.0 * self.view.bead
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
        Some((self.cell_at(origin, point), press))
    }

    /// The rectangle of one bead on the screen, turned with the field.
    fn bead_rect(&self, origin: Pos2, cell: Cell) -> Rect {
        bead_rect(self.pattern, self.view, origin, cell)
    }

    /// The bead under a point of the screen.
    fn cell_at(&self, origin: Pos2, point: Pos2) -> Cell {
        let pattern = self.pattern;
        let at = self.view.to_pattern(point - origin) / self.view.bead;
        pattern.grid().cell_at(at.x, at.y, pattern.ratio())
    }
}

/// The rectangle of one bead on the screen, turned with the field, cell
/// (0, 0) standing at `origin`.
fn bead_rect(pattern: &Pattern, view: &View, origin: Pos2, cell: Cell) -> Rect {
    let (x, y) = pattern.grid().origin(cell, pattern.ratio());
    let min = vec2(x, y) * view.bead;
    let max = min + vec2(pattern.ratio(), 1.0) * view.bead;
    let shown = Rect::from_two_pos(origin + view.to_shown(min), origin + view.to_shown(max));
    shown.shrink(view.bead * GAP / 2.0)
}

/// The point of the screen at the centre of a bead, for a test that presses
/// on it.
pub fn bead_centre(pattern: &Pattern, view: &View, field: Rect, cell: Cell) -> Pos2 {
    bead_rect(pattern, view, field.min + view.offset, cell).center()
}
