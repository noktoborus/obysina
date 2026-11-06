#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod shadow;
mod sprite;
mod texture;
mod view;

pub use shadow::{FRAME_FADE, NEAR_RADIUS};
pub use view::{DEFAULT_BEAD, MAX_BEAD, MIN_BEAD, Turn, View};

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Colour, Pattern, Rgb};

/// The part of a bead left as a gap to its neighbours.
const GAP: f32 = 0.08;
/// The rounding of a bead, as a part of its height.
const ROUNDING: f32 = 0.3;
/// The ground of the active area: a neutral grey, the same in every theme, so
/// that a colour looks the same and a black bead is seen.
pub const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The ground outside the active area: the paper in shade.
const SHADE: Color32 = Color32::from_rgb(0xb8, 0xb6, 0xb0);
/// The shadow of an empty place on a line of the ruling, at its strongest: a
/// bead of faint black, seen on the paper and in the shade alike.
const SHADOW: Color32 = Color32::from_black_alpha(36);
/// The shadow of any other empty place, at its strongest: lighter, so the
/// lines of the ruling stand out.
const SHADOW_CELL: Color32 = Color32::from_black_alpha(18);

/// The colour of the hatch of a bead of a removed or hidden colour.
const HATCH: Color32 = Color32::from_gray(0x40);

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
}

impl<'a> Canvas<'a> {
    /// A field over `pattern`, moved and zoomed by `view`.
    pub fn new(pattern: &'a Pattern, view: &'a mut View) -> Self {
        Canvas {
            pattern,
            view,
            shadows: true,
        }
    }

    /// Whether the empty places show their shadows; they do by default.
    pub fn shadows(mut self, shadows: bool) -> Self {
        self.shadows = shadows;
        self
    }

    /// Draws the field over all the space left in `ui`.
    pub fn show(mut self, ui: &mut Ui) -> CanvasResponse {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let resized = self.resize(rect);
        if !self.view.centred && rect.is_positive() {
            self.view.centre(rect.size(), self.middle());
        }
        if self.view.sheet.is_none() && rect.is_positive() {
            self.view.sheet = Some(self.cells_under(rect.min + self.view.offset, rect));
        }
        if resized && self.work_area_hidden(rect) {
            self.view.centre(rect.size(), self.middle());
        }
        self.navigate(ui, &response);

        let painter = ui.painter_at(rect);
        let origin = rect.min + self.view.offset;
        let visible = self.cells_under(origin, rect);
        painter.rect_filled(rect, 0.0, SHADE);
        self.draw_paper(&painter, origin, visible);
        let pointer = response.hover_pos();
        if self.shadows {
            self.draw_shadows(ui, &painter, (origin, rect), visible, pointer);
        }
        self.draw_beads(ui, &painter, origin, visible);

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
    fn draw_paper(&self, painter: &egui::Painter, origin: Pos2, visible: Bounds) {
        let Some(cells) = self.work_area().and_then(|area| area.intersection(visible)) else {
            return;
        };
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let mut mesh = egui::Mesh::default();
        for column in texture::columns(grid, ratio, cells) {
            texture::add_quad(&mut mesh, self.view, origin, column, PAPER, |_| {
                egui::epaint::WHITE_UV
            });
        }
        painter.add(egui::Shape::mesh(mesh));
    }

    /// The shadows of the empty places that [`shadow`] gives, each a sprite
    /// of a bead as strong as it says: tinted `SHADOW` on a line of the
    /// ruling, `SHADOW_CELL` elsewhere. The pattern stands at `origin` of the
    /// field `rect`.
    fn draw_shadows(
        &self,
        ui: &Ui,
        painter: &egui::Painter,
        (origin, rect): (Pos2, Rect),
        visible: Bounds,
        pointer: Option<Pos2>,
    ) {
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let texture = sprite::texture(ui.ctx(), ratio, self.view.bead, GAP, ROUNDING);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let ruling = self.pattern.ruling();
        for (cell, strength) in self.shadowed(origin, rect, visible, pointer) {
            let shadow = if ruling.marks(cell) {
                SHADOW
            } else {
                SHADOW_CELL
            };
            let colour = shadow.gamma_multiply(strength);
            sprite::add_bead(&mut mesh, grid, ratio, self.view, origin, cell, colour);
        }
        painter.add(egui::Shape::mesh(mesh));
    }

    /// The beads on the screen, each a sprite tinted its colour; a bead of a
    /// removed or hidden colour is a sprite of the hatch.
    fn draw_beads(&self, ui: &Ui, painter: &egui::Painter, origin: Pos2, visible: Bounds) {
        let pattern = self.pattern;
        let (grid, ratio) = (pattern.grid(), pattern.ratio());
        let texture = sprite::texture(ui.ctx(), ratio, self.view.bead, GAP, ROUNDING);
        let hatch = sprite::hatch_texture(ui.ctx(), ratio, self.view.bead, GAP, ROUNDING);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        let mut hatched = egui::Mesh::with_texture(hatch.id());
        for (cell, id) in pattern.beads().filter(|(cell, _)| visible.contains(*cell)) {
            match pattern.colour(id).and_then(Colour::seen) {
                Some(Rgb([red, green, blue])) => {
                    let colour = Color32::from_rgb(red, green, blue);
                    sprite::add_bead(&mut mesh, grid, ratio, self.view, origin, cell, colour);
                }
                None => {
                    sprite::add_bead(&mut hatched, grid, ratio, self.view, origin, cell, HATCH);
                }
            }
        }
        painter.add(egui::Shape::mesh(mesh));
        painter.add(egui::Shape::mesh(hatched));
    }

    /// Keeps the middle of the field where it is when the field changes its
    /// size. A field of no size, as when the window is folded away, is
    /// skipped. True when the size changed.
    fn resize(&mut self, rect: Rect) -> bool {
        if !rect.is_positive() {
            return false;
        }
        let before = self.view.field;
        self.view.field = rect.size();
        let resized = before.x > 0.0 && before.y > 0.0 && before != rect.size();
        if resized {
            self.view.offset += (rect.size() - before) / 2.0;
        }
        resized
    }

    /// True when no cell of the work area is on the screen.
    fn work_area_hidden(&self, screen: Rect) -> bool {
        let visible = self.cells_under(screen.min + self.view.offset, screen);
        self.work_area()
            .is_some_and(|area| area.intersection(visible).is_none())
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
