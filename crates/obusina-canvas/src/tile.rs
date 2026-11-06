//! The outlines of the empty places, drawn as one texture that repeats.
//!
//! A grid repeats: loom every cell, peyote every two columns. One tile of
//! outlines is drawn into a small texture by the distance to a rounded box,
//! and a column of the field is one quad of that texture, so the cost of the
//! outlines does not grow with the field. The tile is white; a quad tints
//! it.

use egui::{Color32, ColorImage, Context, Mesh, Pos2, TextureHandle, TextureOptions, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Grid};

use crate::View;
use crate::texture::{add_quad, cached, columns, rounded_box};

/// What a tile is drawn from.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    grid: Grid,
    ratio: f32,
    /// Pixels a bead is high.
    pixels: f32,
    /// The part of a bead left as a gap, the rounding, the width of an
    /// outline in pixels.
    gap: f32,
    rounding: f32,
    stroke: f32,
}

/// The texture of one tile of `grid`, its beads `bead` points high: drawn
/// again only when one of these changes. `gap` and `rounding` are parts of
/// the height of a bead; `stroke` is the width of an outline in points.
pub fn texture(
    context: &Context,
    grid: Grid,
    ratio: f32,
    bead: f32,
    gap: f32,
    rounding: f32,
    stroke: f32,
) -> TextureHandle {
    let pixels_per_point = context.pixels_per_point();
    let key = Key {
        grid,
        ratio,
        pixels: bead * pixels_per_point,
        gap,
        rounding,
        stroke: stroke * pixels_per_point,
    };
    let options = TextureOptions {
        wrap_mode: egui::TextureWrapMode::Repeat,
        ..TextureOptions::LINEAR
    };
    cached(context, "obusina-canvas-tile", key, options, || draw(key))
}

/// The size of a tile in bead units: one cell of loom, two columns of
/// peyote.
pub fn size(grid: Grid, ratio: f32) -> Vec2 {
    match grid {
        Grid::Loom => vec2(ratio, 1.0),
        Grid::Peyote => vec2(2.0 * ratio, 1.0),
    }
}

/// Adds to `mesh` one quad for each column of `cells`, tinted `colour`: the
/// outlines of those cells. The pattern stands at `origin`, shown by `view`.
pub fn add_columns(
    mesh: &mut Mesh,
    grid: Grid,
    ratio: f32,
    view: &View,
    origin: Pos2,
    cells: Bounds,
    colour: Color32,
) {
    let tile = size(grid, ratio);
    for column in columns(grid, ratio, cells) {
        add_quad(mesh, view, origin, column, colour, |corner| {
            Pos2::new(corner.x / tile.x, corner.y / tile.y)
        });
    }
}

/// Adds to `mesh` one quad over the bead of `cell`, tinted `colour`: its
/// outline alone. The pattern stands at `origin`, shown by `view`.
pub fn add_cell(
    mesh: &mut Mesh,
    grid: Grid,
    ratio: f32,
    view: &View,
    origin: Pos2,
    cell: Cell,
    colour: Color32,
) {
    let tile = size(grid, ratio);
    let (left, top) = grid.origin(cell, ratio);
    let min = vec2(left, top);
    let bead = (min, min + vec2(ratio, 1.0));
    add_quad(mesh, view, origin, bead, colour, |corner| {
        Pos2::new(corner.x / tile.x, corner.y / tile.y)
    });
}

/// Draws a tile: white outlines on nothing.
fn draw(key: Key) -> ColorImage {
    let tile = size(key.grid, key.ratio);
    let width = (tile.x * key.pixels).round().max(1.0) as usize;
    let height = key.pixels.round().max(1.0) as usize;
    let per_pixel = vec2(tile.x / width as f32, tile.y / height as f32);
    let columns = (tile.x / key.ratio).round() as i32;
    let shrink = key.gap / 2.0;
    let half = vec2(key.ratio / 2.0 - shrink, 0.5 - shrink);
    let radius = (1.0 - key.gap) * key.rounding;
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let point = vec2(x as f32 + 0.5, y as f32 + 0.5) * per_pixel;
            let mut cover: f32 = 0.0;
            for column in -1..=columns {
                for row in -1..=1 {
                    let (left, top) = key.grid.origin(Cell { column, row }, key.ratio);
                    let centre = vec2(left, top) + vec2(key.ratio, 1.0) / 2.0;
                    let distance = rounded_box(point - centre, half, radius) * key.pixels;
                    let inside = (0.5 - distance).clamp(0.0, 1.0);
                    let within = (distance + key.stroke + 0.5).clamp(0.0, 1.0);
                    cover = cover.max(inside * within);
                }
            }
            pixels.push(Color32::from_white_alpha((cover * 255.0).round() as u8));
        }
    }
    ColorImage::new([width, height], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(grid: Grid) -> Key {
        Key {
            grid,
            ratio: 1.0,
            pixels: 20.0,
            gap: 0.1,
            rounding: 0.3,
            stroke: 1.0,
        }
    }

    fn alpha(image: &ColorImage, x: usize, y: usize) -> u8 {
        image.pixels[y * image.size[0] + x].a()
    }

    #[test]
    fn a_loom_tile_is_one_cell_with_its_outline_inside() {
        let image = draw(key(Grid::Loom));

        assert_eq!(image.size, [20, 20]);
        assert_eq!(alpha(&image, 10, 10), 0, "the middle is empty");
        assert_eq!(alpha(&image, 0, 10), 0, "the gap is empty");
        assert!(alpha(&image, 1, 10) > 200, "the outline is inside the gap");
    }

    /// The odd column is half a bead lower: its outline crosses the top and
    /// the bottom of the tile, and the tile repeats it whole.
    #[test]
    fn a_peyote_tile_holds_the_lower_column() {
        let image = draw(key(Grid::Peyote));

        assert_eq!(image.size, [40, 20]);
        assert!(alpha(&image, 30, 1) == 0 && alpha(&image, 30, 11) > 200);
        assert!(alpha(&image, 10, 1) > 200 && alpha(&image, 10, 11) == 0);
    }
}
