//! Which outlines the shade shows, and how strong each one is.
//!
//! The shade has outlines only near two places, each fading to its edge:
//!
//! ```text
//! ┌──────────────── the screen ────────────────┐
//! │      ╭──── the frame: the screen ────╮     │
//! │      │     at zoom 1, in its middle  │     │
//! │      │                     ( o )─────┼── the round under the pointer
//! │      ╰───────────────────────────────╯     │
//! └────────────────────────────────────────────┘
//! ```
//!
//! - The frame is as many cells as the screen holds at `DEFAULT_BEAD`, a box
//!   with corners rounded by an eighth of its shorter side, in the middle of
//!   the screen. Zoomed out it is smaller than the screen; at zoom 1 and in it
//!   is the screen or more.
//! - The round is `NEAR_RADIUS` cells around the pointer. With no pointer, as
//!   under a finger, there is none.
//!
//! Where both cover a cell, the stronger one counts.

use egui::{Pos2, Rect, Vec2, vec2};
use obusina_pattern::{Bounds, Cell};

use crate::Canvas;
use crate::texture::rounded_box;
use crate::view::DEFAULT_BEAD;

/// The radius of the round of outlines under the pointer, in cells: a column
/// is a cell wide, a row a cell high.
pub const NEAR_RADIUS: f32 = 5.0;
/// How many cells inside its edge the frame fades over.
pub const FRAME_FADE: f32 = 3.0;
/// The part of the radius of the round it fades over.
const ROUND_FADE: f32 = 0.5;
/// The radius of a corner of the frame, as a part of its shorter side.
const FRAME_CORNER: f32 = 1.0 / 8.0;

impl Canvas<'_> {
    /// The cells of the shade on the screen with an outline, each with its
    /// strength, more than 0 and at most 1. The pattern stands at `origin` of
    /// the field `rect`; `pointer` is where the pointer is over it.
    pub(crate) fn shade_outlines(
        &self,
        origin: Pos2,
        rect: Rect,
        visible: Bounds,
        pointer: Option<Pos2>,
    ) -> Vec<(Cell, f32)> {
        let (Some(field), Some(work_area)) = (self.field(), self.work_area()) else {
            return Vec::new();
        };
        let Some(shown) = field.intersection(visible) else {
            return Vec::new();
        };
        let at = pointer.map(|point| self.view.to_pattern(point - origin) / self.view.bead);
        let frame_box = self.frame_box(origin, rect, shown);
        let round_box = at.and_then(|at| self.round_box(at, shown));
        let in_frame_box = |cell: Cell| frame_box.is_some_and(|cells| cells.contains(cell));
        let frame_cells = frame_box.into_iter().flat_map(Bounds::cells);
        let round_cells = round_box
            .into_iter()
            .flat_map(Bounds::cells)
            .filter(move |cell| !in_frame_box(*cell));
        frame_cells
            .chain(round_cells)
            .filter(|cell| !work_area.contains(*cell))
            .filter_map(|cell| {
                let middle = self.middle_of(cell);
                let frame = self.in_frame(origin, rect, middle);
                let round = at.map_or(0.0, |at| self.in_round(at, middle));
                let strength = frame.max(round);
                (strength > 0.0).then_some((cell, strength))
            })
            .collect()
    }

    /// The middle of the bead of `cell`, in bead units.
    fn middle_of(&self, cell: Cell) -> Vec2 {
        let ratio = self.pattern.ratio();
        let (left, top) = self.pattern.grid().origin(cell, ratio);
        vec2(left + ratio / 2.0, top + 0.5)
    }

    /// The frame on the screen at the size of a bead now: the screen at
    /// zoom 1, scaled about its middle.
    fn frame_rect(&self, rect: Rect) -> Rect {
        Rect::from_center_size(rect.center(), rect.size() * self.view.bead / DEFAULT_BEAD)
    }

    /// The cells under the frame, of those `shown`.
    fn frame_box(&self, origin: Pos2, rect: Rect, shown: Bounds) -> Option<Bounds> {
        let frame = self.frame_rect(rect).intersect(rect);
        if !frame.is_positive() {
            return None;
        }
        self.cells_under(origin, frame).intersection(shown)
    }

    /// The cells around the round at `at`, of those `shown`.
    fn round_box(&self, at: Vec2, shown: Bounds) -> Option<Bounds> {
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let reach = vec2(ratio, 1.0) * (NEAR_RADIUS + 1.0);
        let (min, max) = (at - reach, at + reach);
        grid.cells_in((min.x, min.y), (max.x, max.y), ratio)
            .intersection(shown)
    }

    /// How strong the frame is at `middle`, in bead units: 1 deep inside,
    /// fading over `FRAME_FADE` cells to 0 at its edge.
    fn in_frame(&self, origin: Pos2, rect: Rect, middle: Vec2) -> f32 {
        let shown = origin + self.view.to_shown(middle * self.view.bead);
        let at_zoom_one = (shown - rect.center()) * DEFAULT_BEAD / self.view.bead;
        let half = rect.size() / 2.0;
        let corner = 2.0 * half.min_elem() * FRAME_CORNER;
        let distance = rounded_box(at_zoom_one, half, corner);
        (-distance / (FRAME_FADE * DEFAULT_BEAD)).clamp(0.0, 1.0)
    }

    /// How strong the round about `at` is at `middle`, both in bead units:
    /// 1 near its middle, fading over the outer `ROUND_FADE` of its radius.
    fn in_round(&self, at: Vec2, middle: Vec2) -> f32 {
        let apart = middle - at;
        let cells = vec2(apart.x / self.pattern.ratio(), apart.y).length();
        ((NEAR_RADIUS - cells) / (NEAR_RADIUS * ROUND_FADE)).clamp(0.0, 1.0)
    }
}
