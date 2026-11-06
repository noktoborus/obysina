//! A picture under the beads, laid over four corners on the field, and the
//! handles that move them.
//!
//! ```text
//! editing ──► a handle on each corner
//!             drag that starts on a handle: that corner moves
//!             drag that starts inside:      the whole picture moves
//!             drag that starts outside:     the field moves, as always
//! ```
//!
//! The four corners need make no rectangle. The picture is drawn through the
//! projection they give, a mesh of `STEPS` by `STEPS` cells, so a line of the
//! picture stays straight: a photo taken at an angle comes out straight.

use egui::{Color32, Id, Pos2, Rect, Response, TextureId, Ui, Vec2, pos2};

use crate::Canvas;

/// The cells along each side of the mesh a picture is drawn by.
const STEPS: usize = 24;
/// The radius of a handle on a corner: large enough for a finger.
pub const HANDLE: f32 = 14.0;
/// How far from its middle a handle is taken, as a part of its radius.
const HANDLE_REACH: f32 = 1.5;
/// The width of the lines between the handles.
const EDGE: f32 = 2.0;
/// The colour of the lines and the rings of the handles.
const EDGE_COLOUR: Color32 = Color32::from_rgb(0x00, 0x5c, 0x80);

/// A picture to draw under the beads.
pub struct Picture<'a> {
    /// The texture of the picture.
    pub texture: TextureId,
    /// Its corners on the field, in bead units: top left, top right, bottom
    /// right, bottom left. The handles move them while `editing`.
    pub corners: &'a mut [Vec2; 4],
    /// How much of it shows, from 0 to 1.
    pub opacity: f32,
    /// Its corners show handles, and a drag on it moves it.
    pub editing: bool,
}

/// What a drag of the picture holds.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Grab {
    /// One corner, by its index.
    Corner(usize),
    /// The whole picture.
    Whole,
}

impl Canvas<'_> {
    /// The corners of the picture on the screen, the pattern standing at
    /// `origin`.
    fn shown_corners(&self, corners: &[Vec2; 4], origin: Pos2) -> [Pos2; 4] {
        corners.map(|corner| origin + self.view.to_shown(corner * self.view.bead))
    }

    /// Takes the drag of the field while it holds the picture: true when it
    /// does, and the field is not to move.
    pub(crate) fn drag_picture(
        &self,
        ui: &Ui,
        response: &Response,
        origin: Pos2,
        picture: &mut Picture<'_>,
    ) -> bool {
        let id = Id::unique("obusina-canvas-picture-grab");
        if !picture.editing {
            ui.data_mut(|data| data.remove::<Grab>(id));
            return false;
        }
        let shown = self.shown_corners(picture.corners, origin);
        if response.drag_started()
            && let Some(at) = response.interact_pointer_pos()
        {
            let grab = shown
                .iter()
                .position(|corner| corner.distance(at) <= HANDLE * HANDLE_REACH)
                .map(Grab::Corner)
                .or_else(|| inside(shown, at).then_some(Grab::Whole));
            ui.data_mut(|data| match grab {
                Some(grab) => {
                    data.insert_temp(id, grab);
                }
                None => data.remove::<Grab>(id),
            });
        }
        let Some(grab) = ui.data(|data| data.get_temp::<Grab>(id)) else {
            return false;
        };
        if response.dragged() {
            let delta = self.view.to_pattern(response.drag_delta()) / self.view.bead;
            match grab {
                Grab::Corner(index) => picture.corners[index] += delta,
                Grab::Whole => picture
                    .corners
                    .iter_mut()
                    .for_each(|corner| *corner += delta),
            }
        }
        if !response.dragged() || response.drag_stopped() {
            ui.data_mut(|data| data.remove::<Grab>(id));
        }
        true
    }

    /// Draws the picture.
    pub(crate) fn draw_picture(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        picture: &Picture<'_>,
    ) {
        let shown = self.shown_corners(picture.corners, origin);
        let tint = Color32::WHITE.gamma_multiply(picture.opacity);
        painter.add(mesh(picture.texture, shown, tint));
    }

    /// Draws the edges of the picture and the handles of its corners, over
    /// the beads.
    pub(crate) fn draw_handles(
        &self,
        painter: &egui::Painter,
        origin: Pos2,
        picture: &Picture<'_>,
    ) {
        let shown = self.shown_corners(picture.corners, origin);
        let line = egui::Stroke::new(EDGE, EDGE_COLOUR);
        for index in 0..shown.len() {
            painter.line_segment([shown[index], shown[(index + 1) % shown.len()]], line);
        }
        for corner in shown {
            painter.circle(corner, HANDLE, Color32::WHITE, line);
        }
    }
}

/// The handles of the corners on the screen, for a test that drags them.
pub fn handles(canvas_rect: Rect, view: &crate::View, corners: &[Vec2; 4]) -> [Pos2; 4] {
    let origin = canvas_rect.min + view.offset;
    corners.map(|corner| origin + view.to_shown(corner * view.bead))
}

/// True when `point` lies inside the four `corners`, taken in order.
fn inside(corners: [Pos2; 4], point: Pos2) -> bool {
    let side = |a: Pos2, b: Pos2| (b - a).x * (point - a).y - (b - a).y * (point - a).x;
    let sides: Vec<f32> = (0..corners.len())
        .map(|index| side(corners[index], corners[(index + 1) % corners.len()]))
        .collect();
    sides.iter().all(|side| *side >= 0.0) || sides.iter().all(|side| *side <= 0.0)
}

/// The projection that takes the unit square onto the four `corners`: top
/// left, top right, bottom right, bottom left (square to quadrilateral, as
/// Heckbert gives it). Corners that make a parallelogram give an affine map.
fn projection(corners: [Pos2; 4]) -> impl Fn(f32, f32) -> Pos2 {
    let [p0, p1, p2, p3] = corners;
    let sx = p0.x - p1.x + p2.x - p3.x;
    let sy = p0.y - p1.y + p2.y - p3.y;
    let (dx1, dx2, dy1, dy2) = (p1.x - p2.x, p3.x - p2.x, p1.y - p2.y, p3.y - p2.y);
    let det = dx1 * dy2 - dx2 * dy1;
    let (g, h) = if det.abs() <= f32::EPSILON {
        (0.0, 0.0)
    } else {
        ((sx * dy2 - dx2 * sy) / det, (dx1 * sy - sx * dy1) / det)
    };
    let (a, b, c) = (p1.x - p0.x + g * p1.x, p3.x - p0.x + h * p3.x, p0.x);
    let (d, e, f) = (p1.y - p0.y + g * p1.y, p3.y - p0.y + h * p3.y, p0.y);
    move |u, v| {
        let w = g * u + h * v + 1.0;
        pos2((a * u + b * v + c) / w, (d * u + e * v + f) / w)
    }
}

/// The picture over the four `corners`, a mesh through their projection.
fn mesh(texture: TextureId, corners: [Pos2; 4], tint: Color32) -> egui::Shape {
    let mut mesh = egui::Mesh::with_texture(texture);
    let map = projection(corners);
    for row in 0..=STEPS {
        for column in 0..=STEPS {
            let (u, v) = (column as f32 / STEPS as f32, row as f32 / STEPS as f32);
            mesh.vertices.push(egui::epaint::Vertex {
                pos: map(u, v),
                uv: pos2(u, v),
                color: tint,
            });
        }
    }
    let line = STEPS as u32 + 1;
    for row in 0..STEPS as u32 {
        for column in 0..STEPS as u32 {
            let first = row * line + column;
            mesh.add_triangle(first, first + 1, first + line + 1);
            mesh.add_triangle(first, first + line + 1, first + line);
        }
    }
    egui::Shape::mesh(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [Pos2; 4] = [
        pos2(0.0, 0.0),
        pos2(10.0, 0.0),
        pos2(10.0, 10.0),
        pos2(0.0, 10.0),
    ];

    #[test]
    fn the_projection_takes_the_corners_to_the_corners() {
        let corners = [
            pos2(30.0, 10.0),
            pos2(70.0, 15.0),
            pos2(90.0, 80.0),
            pos2(5.0, 70.0),
        ];
        let map = projection(corners);
        for ((u, v), corner) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
            .into_iter()
            .zip(corners)
        {
            assert!(map(u, v).distance(corner) < 1e-3, "{u} {v}");
        }
    }

    #[test]
    fn a_square_is_mapped_without_a_bend() {
        let map = projection(SQUARE);

        assert!(map(0.5, 0.25).distance(pos2(5.0, 2.5)) < 1e-4);
    }

    #[test]
    fn a_point_inside_is_told_from_one_outside() {
        assert!(inside(SQUARE, pos2(5.0, 5.0)));
        assert!(!inside(SQUARE, pos2(15.0, 5.0)));
    }
}
