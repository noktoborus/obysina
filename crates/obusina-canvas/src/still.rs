//! A pattern drawn still, as a picture of it: the paper and the beads of
//! the field, by the same code, fitted into a box. No input, no shadow, no
//! shade. A caller renders the shapes once into a texture and shows that.
//!
//! ```text
//! still(pattern, size) ──► the paper of the box ──► paper_mesh ──► bead_meshes
//!                          the work area as large as fits, in the middle,
//!                          turned as the pattern is shown
//! ```

use egui::{Pos2, Rect, Shape, Vec2, vec2};
use obusina_pattern::Pattern;

use crate::{Canvas, Tones, View};

/// The shapes of `pattern` drawn still into a box of `size` points whose top
/// left corner is the origin: the box filled with its paper, the work area
/// as large as fits in its middle, turned as the pattern is shown, its beads
/// round when they `glow`. A pattern with no work area is its paper only.
pub fn still(context: &egui::Context, pattern: &Pattern, size: Vec2, glow: bool) -> Vec<Shape> {
    let tones = Tones::of(pattern.paper());
    let whole = Rect::from_min_size(Pos2::ZERO, size);
    let mut shapes = vec![Shape::rect_filled(whole, 0.0, tones.paper)];
    let Some(area) = pattern.work_area() else {
        return shapes;
    };
    let (grid, ratio) = (pattern.grid(), pattern.ratio());
    let mut view = View::default();
    view.set_degrees(pattern.turn());
    let ((left, top), (right, bottom)) = grid.span(area, ratio);
    let span = view.to_shown(vec2(right - left, bottom - top)).abs();
    if span.x <= 0.0 || span.y <= 0.0 {
        return shapes;
    }
    view.bead = (size.x / span.x).min(size.y / span.y);
    let middle = vec2(left + right, top + bottom) / 2.0 * view.bead;
    view.centre(size, middle);
    view.field = size;
    let origin = Pos2::ZERO + view.offset;
    let canvas = Canvas::new(pattern, &mut view).glow(glow);
    shapes.push(Shape::mesh(canvas.paper_mesh(origin, area, tones.paper)));
    let beads = canvas.bead_meshes(context, origin, area, tones.hatch);
    shapes.extend(beads.into_iter().map(Shape::mesh));
    shapes
}
