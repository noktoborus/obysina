#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod picture;
mod shadow;
mod sprite;
mod still;
mod texture;
mod tone;
mod view;

pub use picture::{HANDLE, Picture, SHIMMER_PERIOD, handles, square_corners, turn_corners};
pub use shadow::{FRAME_FADE, NEAR_RADIUS};
pub use still::still;
pub use tone::{LIGHT, Tones, lightness};
pub use view::{DEFAULT_BEAD, MAX_BEAD, MIN_BEAD, Turn, View};

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Colour, Pattern, Rgb};

/// The part of a bead left as a gap to its neighbours.
const GAP: f32 = 0.08;
/// The rounding of a bead, as a part of its height.
const ROUNDING: f32 = 0.3;

/// The beads of the field that stay on the screen along each axis, however
/// far the field is moved; all of it when it is smaller.
pub const KEEP_BEADS: f32 = 10.0;

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
    shadows: bool,
    glow: bool,
    picture: Option<Picture<'a>>,
}

impl<'a> Canvas<'a> {
    /// A field over `pattern`, moved and zoomed by `view`.
    pub fn new(pattern: &'a Pattern, view: &'a mut View) -> Self {
        Canvas {
            pattern,
            view,
            shadows: true,
            glow: true,
            picture: None,
        }
    }

    /// A picture under the beads, over the paper and under the shadows.
    pub fn picture(mut self, picture: Picture<'a>) -> Self {
        self.picture = Some(picture);
        self
    }

    /// Whether the empty places show their shadows; they do by default.
    pub fn shadows(mut self, shadows: bool) -> Self {
        self.shadows = shadows;
        self
    }

    /// Whether the beads are drawn round, lighter in the middle; they are by
    /// default. Off, they are flat.
    pub fn glow(mut self, glow: bool) -> Self {
        self.glow = glow;
        self
    }

    /// Draws the field over all the space left in `ui`.
    pub fn show(mut self, ui: &mut Ui) -> CanvasResponse {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.resize(rect);
        if !self.view.centred && rect.is_positive() {
            self.view.centre(rect.size(), self.middle());
        }
        if self.view.sheet.is_none() && rect.is_positive() {
            self.view.sheet = Some(self.cells_under(rect.min + self.view.offset, rect));
        }
        let mut picture = self.picture.take();
        let held = picture.as_mut().is_some_and(|picture| {
            let origin = rect.min + self.view.offset;
            self.gesture_picture(ui, rect, origin, picture)
                || self.drag_picture(ui, &response, origin, picture)
        });
        if !held {
            self.navigate(ui, &response);
        }
        self.keep_on_screen(rect);

        let painter = ui.painter_at(rect);
        let origin = rect.min + self.view.offset;
        let visible = self.cells_under(origin, rect);
        let tones = Tones::of(self.pattern.paper());
        painter.rect_filled(rect, 0.0, tones.shade);
        painter.add(self.paper_mesh(origin, visible, tones.paper));
        if let Some(picture) = &picture {
            self.draw_picture(ui, &painter, origin, picture);
        }
        if self.shadows {
            let pointer = response.hover_pos();
            self.draw_shadows(ui, &painter, (origin, rect), visible, (pointer, tones));
        }
        for mesh in self.bead_meshes(ui.ctx(), origin, visible, tones.hatch) {
            painter.add(mesh);
        }
        if let Some(picture) = picture.as_ref().filter(|picture| picture.editing) {
            self.draw_handles(&painter, origin, picture);
        }

        let hovered = response
            .hover_pos()
            .map(|point| self.cell_at(origin, point))
            .filter(|cell| self.in_field(*cell));
        let pressed = self.pressed(&response, origin);
        CanvasResponse {
            response,
            hovered,
            pressed,
        }
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

    /// The work area: the box around the start sheet and the beads. A bead
    /// put past it grows it. `None` before the field was first shown, with
    /// no beads.
    fn work_area(&self) -> Option<Bounds> {
        let sheet = self.pattern.sheet().or(self.view.sheet);
        match (sheet, self.pattern.bounds()) {
            (Some(sheet), Some(beads)) => Some(sheet.union(beads)),
            (sheet, beads) => sheet.or(beads),
        }
    }

    /// The field drawn: the work area doubled. The part past the work area
    /// is in shade.
    fn field(&self) -> Option<Bounds> {
        self.work_area().map(Bounds::doubled)
    }

    /// True when `cell` is in the field drawn.
    fn in_field(&self, cell: Cell) -> bool {
        self.field().is_some_and(|field| field.contains(cell))
    }

    /// The paper under the work area, over the shade of the rest: one
    /// quad for each of its columns on the screen.
    pub(crate) fn paper_mesh(&self, origin: Pos2, visible: Bounds, paper: Color32) -> egui::Mesh {
        let Some(cells) = self.work_area().and_then(|area| area.intersection(visible)) else {
            return egui::Mesh::default();
        };
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let mut mesh = egui::Mesh::default();
        for column in texture::columns(grid, ratio, cells) {
            texture::add_quad(&mut mesh, self.view, origin, column, paper, |_| {
                egui::epaint::WHITE_UV
            });
        }
        mesh
    }

    /// The shadows of the empty places that [`shadow`] gives, each a flat
    /// sprite of a bead as strong as it says, of the tones of the paper: a
    /// line of the ruling stronger than a cell. The pattern stands at
    /// `origin` of the field `rect`.
    fn draw_shadows(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        (origin, rect): (Pos2, Rect),
        visible: Bounds,
        (pointer, tones): (Option<Pos2>, Tones),
    ) {
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let shape = (GAP, ROUNDING);
        let texture = sprite::texture(ui.ctx(), ratio, self.view.bead, shape, false);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let ruling = self.pattern.ruling();
        for (cell, strength) in self.shadowed(origin, rect, visible, pointer) {
            let shadow = if ruling.marks(cell) {
                tones.line
            } else {
                tones.cell
            };
            let colour = shadow.gamma_multiply(strength);
            sprite::add_bead(&mut mesh, grid, ratio, self.view, origin, cell, colour);
        }
        painter.add(egui::Shape::mesh(mesh));
    }

    /// The beads on the screen, each a sprite tinted its colour, round when
    /// they glow; a bead of a removed or hidden colour is a sprite of the
    /// hatch, tinted `hatch`.
    pub(crate) fn bead_meshes(
        &self,
        context: &egui::Context,
        origin: Pos2,
        visible: Bounds,
        hatch_colour: Color32,
    ) -> [egui::Mesh; 2] {
        let pattern = self.pattern;
        let (grid, ratio) = (pattern.grid(), pattern.ratio());
        let shape = (GAP, ROUNDING);
        let texture = sprite::texture(context, ratio, self.view.bead, shape, self.glow);
        let hatch = sprite::hatch_texture(context, ratio, self.view.bead, GAP, ROUNDING);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let mut hatched = egui::Mesh::with_texture(hatch.id());
        for (cell, id) in pattern.beads().filter(|(cell, _)| visible.contains(*cell)) {
            match pattern.colour(id).and_then(Colour::seen) {
                Some(Rgb([red, green, blue])) => {
                    let colour = Color32::from_rgb(red, green, blue);
                    sprite::add_bead(&mut mesh, grid, ratio, self.view, origin, cell, colour);
                }
                None => {
                    let colour = hatch_colour;
                    sprite::add_bead(&mut hatched, grid, ratio, self.view, origin, cell, colour);
                }
            }
        }
        [mesh, hatched]
    }

    /// Keeps the middle of the field where it is when the field changes its
    /// size. A field of no size, as when the window is folded away, is
    /// skipped.
    fn resize(&mut self, rect: Rect) {
        if !rect.is_positive() {
            return;
        }
        let before = self.view.field;
        self.view.field = rect.size();
        let resized = before.x > 0.0 && before.y > 0.0 && before != rect.size();
        if resized {
            self.view.offset += (rect.size() - before) / 2.0;
        }
    }

    /// Moves the field as little as it takes so that `KEEP_BEADS` of it
    /// stand on the screen `rect` along each axis, or all of it when it is
    /// smaller. A field moved off a side stays at that side.
    fn keep_on_screen(&mut self, rect: Rect) {
        let Some(field) = self.field() else {
            return;
        };
        if !rect.is_positive() {
            return;
        }
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let bead = self.view.bead;
        let ((left, top), (right, bottom)) = grid.span(field, ratio);
        let origin = rect.min + self.view.offset;
        let shown = Rect::from_two_pos(
            origin + self.view.to_shown(vec2(left, top) * bead),
            origin + self.view.to_shown(vec2(right, bottom) * bead),
        );
        let cell = self.view.to_shown(vec2(ratio, 1.0) * bead).abs();
        let keep = (cell * KEEP_BEADS).min(shown.size());
        let shift = vec2(
            keep_shift(shown.x_range(), rect.x_range(), keep.x),
            keep_shift(shown.y_range(), rect.y_range(), keep.y),
        );
        self.view.offset += shift;
    }

    /// The point of the pattern shown in the middle of the field: the middle
    /// of the work area, or of cell (0, 0) when there is none.
    fn middle(&self) -> Vec2 {
        let pattern = self.pattern;
        let area = self
            .work_area()
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
        let cell = self.cell_at(origin, point);
        self.in_field(cell).then_some((cell, press))
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

/// How far a field shown over `shown` along one axis moves so that `keep`
/// of it stands on the `screen`: as little as it takes, 0 when it does.
/// When the screen is too short for that, the field is put in its middle.
fn keep_shift(shown: egui::Rangef, screen: egui::Rangef, keep: f32) -> f32 {
    let least = screen.min + keep - shown.max;
    let most = screen.max - keep - shown.min;
    if least > most {
        (least + most) / 2.0
    } else {
        0.0_f32.clamp(least, most)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: egui::Rangef = egui::Rangef {
        min: 0.0,
        max: 100.0,
    };

    #[test]
    fn a_field_on_the_screen_stays() {
        assert_eq!(
            keep_shift(egui::Rangef::new(-50.0, 150.0), SCREEN, 10.0),
            0.0
        );
    }

    #[test]
    fn a_field_moved_off_a_side_comes_back_to_that_side() {
        assert_eq!(
            keep_shift(egui::Rangef::new(-300.0, -50.0), SCREEN, 10.0),
            60.0
        );
        assert_eq!(
            keep_shift(egui::Rangef::new(200.0, 400.0), SCREEN, 10.0),
            -110.0
        );
    }

    #[test]
    fn a_field_larger_than_the_screen_may_cover_it() {
        assert_eq!(
            keep_shift(egui::Rangef::new(-500.0, 500.0), SCREEN, 10.0),
            0.0
        );
    }

    #[test]
    fn a_screen_too_short_puts_the_field_in_its_middle() {
        assert_eq!(
            keep_shift(egui::Rangef::new(0.0, 300.0), SCREEN, 300.0),
            -100.0
        );
    }
}
