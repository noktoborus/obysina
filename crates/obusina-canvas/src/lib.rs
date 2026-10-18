#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod sprite;
mod texture;
mod tile;
mod view;

pub use view::{MAX_BEAD, MIN_BEAD, Turn, View};

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Pattern};

/// The part of a bead left as a gap to its neighbours.
const GAP: f32 = 0.08;
/// The rounding of a bead, as a part of its height.
const ROUNDING: f32 = 0.3;
/// The width of the outline of an empty place, in points.
const OUTLINE: f32 = 1.0;
/// The ground of the active area: a neutral grey, the same in every theme, so
/// that a colour looks the same and a black bead is seen.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The ground outside the active area: the paper in shade.
const SHADE: Color32 = Color32::from_rgb(0xb8, 0xb6, 0xb0);
/// The outline of an empty place: a faint dark line, seen on the paper and
/// in the shade alike.
const OUTLINE_COLOUR: Color32 = Color32::from_black_alpha(40);

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
        if self.pattern.sheet().is_none() && self.view.sheet.is_none() && rect.is_positive() {
            self.view.sheet = Some(self.cells_under(rect.min + self.view.offset, rect));
        }
        self.navigate(ui, &response);

        let painter = ui.painter_at(rect);
        let origin = rect.min + self.view.offset;
        let visible = self.cells_under(origin, rect);
        painter.rect_filled(rect, 0.0, SHADE);
        self.draw_paper(&painter, origin, visible);
        self.draw_outlines(ui, &painter, origin, visible);
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

    /// The outlines of the empty places of the field on the screen, as
    /// quads of the repeating tile, one look on the paper and in the shade.
    /// They are drawn at every size of a bead. A bead is drawn over its
    /// outline, which lies inside it.
    fn draw_outlines(&self, ui: &Ui, painter: &egui::Painter, origin: Pos2, visible: Bounds) {
        let Some(cells) = self.field().and_then(|field| field.intersection(visible)) else {
            return;
        };
        let (grid, ratio) = (self.pattern.grid(), self.pattern.ratio());
        let texture = tile::texture(
            ui.ctx(),
            grid,
            ratio,
            self.view.bead,
            GAP,
            ROUNDING,
            OUTLINE,
        );
        let mut mesh = egui::Mesh::with_texture(texture.id());
        tile::add_columns(
            &mut mesh,
            grid,
            ratio,
            self.view,
            origin,
            cells,
            OUTLINE_COLOUR,
        );
        painter.add(egui::Shape::mesh(mesh));
    }

    /// The beads on the screen, each a sprite tinted its colour.
    fn draw_beads(&self, ui: &Ui, painter: &egui::Painter, origin: Pos2, visible: Bounds) {
        let pattern = self.pattern;
        let (grid, ratio) = (pattern.grid(), pattern.ratio());
        let texture = sprite::texture(ui.ctx(), ratio, self.view.bead, GAP, ROUNDING);
        let mut mesh = egui::Mesh::with_texture(texture.id());
        for (cell, index) in pattern.beads().filter(|(cell, _)| visible.contains(*cell)) {
            let [red, green, blue] = pattern.palette()[index].rgb.0;
            let colour = Color32::from_rgb(red, green, blue);
            sprite::add_bead(&mut mesh, grid, ratio, self.view, origin, cell, colour);
        }
        painter.add(egui::Shape::mesh(mesh));
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
