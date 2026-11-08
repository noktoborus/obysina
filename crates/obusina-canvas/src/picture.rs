//! A picture under the beads, laid over four corners on the field, and the
//! handles that move them.
//!
//! ```text
//! editing ──► a handle on each corner
//!             drag that starts on a handle: that corner moves
//!             drag that starts inside:      the whole picture moves
//!             drag that starts outside:     the field moves, as always
//!             two fingers, their middle inside: the picture moves, turns
//!                                               and grows with them
//!             two fingers, their middle outside: the field, as always
//!             over the picture, Ctrl and the wheel: it grows
//!                               Shift and the wheel: it turns
//! ```
//!
//! A turn sticks to a whole quarter, 0°, 90°, 180° or 270° of its top edge,
//! within `SNAP_TURN`: the turn the gesture made is kept apart, so the
//! picture leaves the quarter once the gesture goes past it.
//!
//! The four corners need make no rectangle. The picture is drawn through the
//! projection they give, a mesh of `STEPS` by `STEPS` cells, so a line of the
//! picture stays straight: a photo taken at an angle comes out straight.
//!
//! A corner moves only as far as the four stay convex and in their turn, so
//! the picture is never turned inside out: a corner dragged past that stops
//! at the last place that keeps them so.
//!
//! A picture with no texture yet, while it loads, is a panel over the same
//! corners with a light band running across it, `SHIMMER_PERIOD` a pass.

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
/// The ground of the panel of a picture that loads.
const SHIMMER_GROUND: Color32 = Color32::from_rgb(0xb0, 0xae, 0xa8);
/// The light of the band that runs across it.
const SHIMMER_LIGHT: Color32 = Color32::from_rgb(0xec, 0xea, 0xe4);
/// The seconds the band takes to run across the panel once.
pub const SHIMMER_PERIOD: f64 = 1.4;
/// The width of the band, as a part of the diagonal of the panel.
const SHIMMER_WIDTH: f32 = 0.18;
/// The least of the turn at a corner, in bead units squared: a corner of
/// nothing, its two sides in one line, is no corner.
const LEAST_TURN: f32 = 0.05;
/// The halvings that find how far a corner may move.
const HALVINGS: usize = 12;
/// How near a whole quarter a turn sticks to it, in radians: three degrees.
const SNAP_TURN: f32 = 3.0 * std::f32::consts::PI / 180.0;
/// The points of a line of the wheel, for the turn by the wheel.
const POINTS_PER_LINE: f32 = 40.0;
/// The points of a page of the wheel, for the turn by the wheel.
const POINTS_PER_PAGE: f32 = 400.0;
/// How far a point of the wheel with Shift turns the picture, in radians.
const TURN_PER_POINT: f32 = 0.3 * std::f32::consts::PI / 180.0;

/// A picture to draw under the beads.
pub struct Picture<'a> {
    /// The texture of the picture; `None` while it loads, and a panel that
    /// shimmers stands in its place.
    pub texture: Option<TextureId>,
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

/// A gesture on the picture under way: the turn of its top edge as the
/// gesture alone made it, before it stuck to a quarter. A turn of the wheel
/// lasts while egui still smooths its scroll, which the field does not take.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Gesture {
    /// The turn, in radians.
    free: f32,
}

impl Canvas<'_> {
    /// Moves, turns and grows the picture by two fingers whose middle is on
    /// it, or by the wheel with Ctrl or Shift over it, while it is edited:
    /// true when it did, and the field is not to move.
    pub(crate) fn gesture_picture(
        &self,
        ui: &Ui,
        rect: Rect,
        origin: Pos2,
        picture: &mut Picture<'_>,
    ) -> bool {
        let id = Id::unique("obusina-canvas-picture-gesture");
        if !picture.editing {
            ui.data_mut(|data| data.remove::<Gesture>(id));
            return false;
        }
        let shown = self.shown_corners(picture.corners, origin);
        let (touch, zoom, wheel, settling, pointer) = ui.input(|input| {
            (
                input.multi_touch(),
                input.zoom_delta(),
                shift_wheel(&input.events),
                input.smooth_scroll_delta() != Vec2::ZERO,
                input.pointer.hover_pos(),
            )
        });
        let held = ui.data(|data| data.get_temp::<Gesture>(id));
        let step = match touch {
            Some(touch) if held.is_some() || inside(shown, touch.center_pos) => Some((
                touch.center_pos,
                touch.translation_delta,
                touch.zoom_delta,
                touch.rotation_delta,
            )),
            Some(_) => None,
            None => pointer
                .filter(|at| rect.contains(*at) && inside(shown, *at))
                .and_then(|at| {
                    let turn = wheel * TURN_PER_POINT;
                    (zoom != 1.0 || turn != 0.0).then_some((at, Vec2::ZERO, zoom, turn))
                }),
        };
        let Some((centre, moved, grown, turned)) = step else {
            if held.is_some() && touch.is_none() && settling {
                return true;
            }
            ui.data_mut(|data| data.remove::<Gesture>(id));
            return false;
        };
        let now = top_turn(picture.corners);
        let free = held.map_or(now, |gesture| gesture.free) + turned;
        let turn = stuck(free) - now;
        let (sin, cos) = turn.sin_cos();
        let shown = shown.map(|corner| {
            let from = (corner - centre) * grown;
            let rotated = egui::vec2(from.x * cos - from.y * sin, from.x * sin + from.y * cos);
            centre + rotated + moved
        });
        for (corner, at) in picture.corners.iter_mut().zip(shown) {
            *corner = self.view.to_pattern(at - origin) / self.view.bead;
        }
        ui.data_mut(|data| data.insert_temp(id, Gesture { free }));
        ui.ctx().request_repaint();
        true
    }

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
        let pressed_at = ui.input(|input| input.pointer.press_origin());
        if response.drag_started()
            && let Some(at) = pressed_at.or(response.interact_pointer_pos())
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
                Grab::Corner(index) => move_corner(picture.corners, index, delta),
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

    /// Draws the picture, or the panel that shimmers in its place while it
    /// loads.
    pub(crate) fn draw_picture(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        origin: Pos2,
        picture: &Picture<'_>,
    ) {
        let shown = self.shown_corners(picture.corners, origin);
        match picture.texture {
            Some(texture) => {
                let tint = Color32::WHITE.gamma_multiply(picture.opacity);
                painter.add(mesh(texture, shown, |_, _| tint));
            }
            None => {
                let time = ui.input(|input| input.time);
                let phase = shimmer_phase(time);
                painter.add(mesh(TextureId::default(), shown, |u, v| {
                    shimmer(u, v, phase)
                }));
                ui.ctx().request_repaint();
            }
        }
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

/// Moves corner `index` by `delta`, or as far along it as the corners stay
/// convex and in their turn ([`convex`]). Corners that are not so already
/// move freely, so a picture laid wrong is mended by hand.
fn move_corner(corners: &mut [Vec2; 4], index: usize, delta: Vec2) {
    let moved = |part: f32| {
        let mut moved = *corners;
        moved[index] += delta * part;
        moved
    };
    if convex(&moved(1.0)) || !convex(corners) {
        corners[index] += delta;
        return;
    }
    let (mut kept, mut refused) = (0.0, 1.0);
    for _ in 0..HALVINGS {
        let middle = (kept + refused) / 2.0;
        if convex(&moved(middle)) {
            kept = middle;
        } else {
            refused = middle;
        }
    }
    *corners = moved(kept);
}

/// Turns the corners of a picture about their middle by `radians`,
/// clockwise on the field: the picture keeps its shape and its place.
pub fn turn_corners(corners: &mut [Vec2; 4], radians: f32) {
    let middle = corners.iter().fold(Vec2::ZERO, |sum, corner| sum + *corner) / 4.0;
    let (sin, cos) = radians.sin_cos();
    for corner in corners.iter_mut() {
        let from = *corner - middle;
        *corner = middle + egui::vec2(from.x * cos - from.y * sin, from.x * sin + from.y * cos);
    }
}

/// Makes the corners of a picture squarer: the corners go part of the way
/// to the rectangle of the picture, so far that the angle farthest from a
/// right one stands at the next whole `step` nearer to it. A few presses make
/// the rectangle.
///
/// The rectangle has the middle of the corners, the turn of the top edge,
/// the width of the mean of the top and the bottom edges and the height of
/// the mean of the sides.
pub fn square_corners(corners: &mut [Vec2; 4], step: f32) {
    let rectangle = rectangle_of(corners);
    let worst = farthest_from_right(corners);
    let wanted = ((worst / step).ceil() - 1.0).max(0.0) * step;
    let towards = |part: f32| {
        let mut moved = *corners;
        for (corner, square) in moved.iter_mut().zip(rectangle) {
            *corner += (square - *corner) * part;
        }
        moved
    };
    let (mut short, mut far) = (0.0, 1.0);
    for _ in 0..HALVINGS * 2 {
        let middle = (short + far) / 2.0;
        if farthest_from_right(&towards(middle)) <= wanted {
            far = middle;
        } else {
            short = middle;
        }
    }
    let moved = towards(far);
    if convex(&moved) {
        *corners = moved;
    }
}

/// The rectangle of a picture: see [`square_corners`].
fn rectangle_of(corners: &[Vec2; 4]) -> [Vec2; 4] {
    let middle = corners.iter().fold(Vec2::ZERO, |sum, corner| sum + *corner) / 4.0;
    let width = ((corners[1] - corners[0]).length() + (corners[2] - corners[3]).length()) / 2.0;
    let height = ((corners[3] - corners[0]).length() + (corners[2] - corners[1]).length()) / 2.0;
    let mut rectangle = [
        middle + egui::vec2(-width, -height) / 2.0,
        middle + egui::vec2(width, -height) / 2.0,
        middle + egui::vec2(width, height) / 2.0,
        middle + egui::vec2(-width, height) / 2.0,
    ];
    turn_corners(&mut rectangle, top_turn(corners));
    rectangle
}

/// How far the angle at a corner farthest from a right one is from it, in
/// radians.
fn farthest_from_right(corners: &[Vec2; 4]) -> f32 {
    (0..corners.len())
        .map(|index| {
            let at = corners[index];
            let before = corners[(index + corners.len() - 1) % corners.len()] - at;
            let after = corners[(index + 1) % corners.len()] - at;
            let angle = before.angle() - after.angle();
            let angle = angle.rem_euclid(std::f32::consts::TAU);
            let angle = angle.min(std::f32::consts::TAU - angle);
            (angle - std::f32::consts::FRAC_PI_2).abs()
        })
        .fold(0.0, f32::max)
}

/// The points the wheel turned with Shift held in this frame, as given, not
/// smoothed: a notch turns the picture past a quarter it stuck to.
fn shift_wheel(events: &[egui::Event]) -> f32 {
    events
        .iter()
        .filter_map(|event| match event {
            egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            } if modifiers.shift => {
                let points = match unit {
                    egui::MouseWheelUnit::Point => 1.0,
                    egui::MouseWheelUnit::Line => POINTS_PER_LINE,
                    egui::MouseWheelUnit::Page => POINTS_PER_PAGE,
                };
                Some((delta.x + delta.y) * points)
            }
            _ => None,
        })
        .sum()
}

/// The turn of the top edge of a picture, in radians, on the pattern before
/// the view turns it: 0 when it runs to the right.
fn top_turn(corners: &[Vec2; 4]) -> f32 {
    let edge = corners[1] - corners[0];
    edge.y.atan2(edge.x)
}

/// `free` stuck to a whole quarter it is within [`SNAP_TURN`] of.
fn stuck(free: f32) -> f32 {
    let quarter = std::f32::consts::FRAC_PI_2;
    let nearest = (free / quarter).round() * quarter;
    if (free - nearest).abs() <= SNAP_TURN {
        nearest
    } else {
        free
    }
}

/// True when the corners, top left, top right, bottom right, bottom left,
/// turn the same way at each corner, by at least `LEAST_TURN`: the
/// quadrilateral is convex and the picture is the right way round. On the
/// field the rows grow down, so that way is clockwise on the screen.
fn convex(corners: &[Vec2; 4]) -> bool {
    (0..corners.len()).all(|index| {
        let before = corners[index];
        let at = corners[(index + 1) % corners.len()];
        let after = corners[(index + 2) % corners.len()];
        let turn = (at - before).x * (after - at).y - (at - before).y * (after - at).x;
        turn >= LEAST_TURN
    })
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

/// Where the band of the shimmer stands at `time`, in seconds: from before
/// the top left corner to past the bottom right one, along the diagonal.
fn shimmer_phase(time: f64) -> f32 {
    let pass = (time / SHIMMER_PERIOD).fract() as f32;
    -SHIMMER_WIDTH + pass * (1.0 + 2.0 * SHIMMER_WIDTH)
}

/// The colour of the shimmer at the point `(u, v)` of the panel, the band at
/// `phase`.
fn shimmer(u: f32, v: f32, phase: f32) -> Color32 {
    let along = (u + v) / 2.0;
    let near = ((along - phase) / SHIMMER_WIDTH).powi(2);
    let light = (-near).exp();
    SHIMMER_GROUND.lerp_to_gamma(SHIMMER_LIGHT, light)
}

/// The picture over the four `corners`, a mesh through their projection,
/// each point coloured by `colour` of its place in the picture. A texture
/// of `TextureId::default()` is the white of egui: the colour alone shows.
fn mesh(
    texture: TextureId,
    corners: [Pos2; 4],
    colour: impl Fn(f32, f32) -> Color32,
) -> egui::Shape {
    let mut mesh = egui::Mesh::with_texture(texture);
    let map = projection(corners);
    for row in 0..=STEPS {
        for column in 0..=STEPS {
            let (u, v) = (column as f32 / STEPS as f32, row as f32 / STEPS as f32);
            let uv = if texture == TextureId::default() {
                egui::epaint::WHITE_UV
            } else {
                pos2(u, v)
            };
            mesh.vertices.push(egui::epaint::Vertex {
                pos: map(u, v),
                uv,
                color: colour(u, v),
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

    /// A quarter turn clockwise takes the top edge down the right side,
    /// about the middle.
    #[test]
    fn a_quarter_turn_keeps_the_middle() {
        let mut corners = [
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        turn_corners(&mut corners, std::f32::consts::FRAC_PI_2);
        let expected = [
            Vec2::new(3.0, -1.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(1.0, 3.0),
            Vec2::new(1.0, -1.0),
        ];
        for (corner, expected) in corners.iter().zip(expected) {
            assert!((*corner - expected).length() < 1e-4, "{corners:?}");
        }
    }

    /// A press brings the angle farthest from a right one to the next ten
    /// degrees nearer it, keeps the middle and the shape the right way round;
    /// two more make the rectangle.
    #[test]
    fn a_press_makes_the_corners_squarer_by_ten_degrees() {
        let mut corners = [
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 1.0),
            Vec2::new(12.0, 9.0),
            Vec2::new(-2.0, 7.0),
        ];
        let middle = |corners: &[Vec2; 4]| corners.iter().fold(Vec2::ZERO, |sum, c| sum + *c) / 4.0;
        let before = farthest_from_right(&corners).to_degrees();
        let centre = middle(&corners);
        let ten = 10f32.to_radians();
        assert!(before > 20.0 && before < 30.0, "{before}");

        square_corners(&mut corners, ten);
        let after = farthest_from_right(&corners).to_degrees();
        assert!((after - 20.0).abs() < 0.05, "{after}");
        assert!((middle(&corners) - centre).length() < 1e-3);
        assert!(convex(&corners));

        square_corners(&mut corners, ten);
        square_corners(&mut corners, ten);
        assert!(farthest_from_right(&corners).to_degrees() < 0.05);
        square_corners(&mut corners, ten);
        assert!(farthest_from_right(&corners).to_degrees() < 0.05);
    }

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

    const BOX: [Vec2; 4] = [
        Vec2::new(0.0, 0.0),
        Vec2::new(4.0, 0.0),
        Vec2::new(4.0, 3.0),
        Vec2::new(0.0, 3.0),
    ];

    #[test]
    fn a_box_is_convex_and_one_turned_inside_out_is_not() {
        assert!(convex(&BOX));
        let mut crossed = BOX;
        crossed.swap(0, 1);
        assert!(!convex(&crossed), "turned inside out");
        let mut trapezoid = BOX;
        trapezoid[0] = Vec2::new(-2.0, 0.0);
        trapezoid[1] = Vec2::new(2.0, 0.0);
        assert!(convex(&trapezoid));
        let mut dent = BOX;
        dent[2] = Vec2::new(1.0, 1.0);
        assert!(!convex(&dent), "a dent is no convex corner");
    }

    #[test]
    fn a_corner_moves_freely_while_the_box_stays_convex() {
        let mut corners = BOX;

        move_corner(&mut corners, 2, Vec2::new(1.0, 2.0));

        assert_eq!(corners[2], Vec2::new(5.0, 5.0));
    }

    /// The bottom right corner dragged past the top left one stops before
    /// the picture would turn inside out.
    #[test]
    fn a_corner_stops_before_the_picture_turns_inside_out() {
        let mut corners = BOX;

        move_corner(&mut corners, 2, Vec2::new(-10.0, -10.0));

        assert!(convex(&corners), "{corners:?}");
        assert!(corners[2].x > 0.0 && corners[2].y > 0.0, "{corners:?}");
        assert_eq!(corners[0], BOX[0], "the other corners stay");
    }

    #[test]
    fn the_band_of_the_shimmer_is_light_where_it_stands() {
        let phase = shimmer_phase(SHIMMER_PERIOD / 2.0);

        assert_eq!(shimmer(phase, phase, phase), SHIMMER_LIGHT);
        assert_eq!(shimmer(1.0, 1.0, -SHIMMER_WIDTH * 4.0), SHIMMER_GROUND);
    }

    #[test]
    fn a_point_inside_is_told_from_one_outside() {
        assert!(inside(SQUARE, pos2(5.0, 5.0)));
        assert!(!inside(SQUARE, pos2(15.0, 5.0)));
    }
}
