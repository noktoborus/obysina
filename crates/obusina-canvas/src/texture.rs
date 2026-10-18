//! Textures drawn here, kept from frame to frame; the quads they are laid
//! on; and the shape they are drawn by.

use egui::epaint::Vertex;
use egui::{Color32, ColorImage, Context, Mesh, Pos2, TextureHandle, TextureOptions, Vec2, vec2};
use obusina_pattern::{Bounds, Cell, Grid};

use crate::View;

/// A texture and the key it was drawn from.
#[derive(Clone)]
struct Kept<K> {
    key: K,
    texture: TextureHandle,
}

/// The texture `name`, drawn by `draw` again only when `key` changed since
/// the last time it was asked for.
pub fn cached<K>(
    context: &Context,
    name: &'static str,
    key: K,
    options: TextureOptions,
    draw: impl FnOnce() -> ColorImage,
) -> TextureHandle
where
    K: Clone + PartialEq + Send + Sync + 'static,
{
    let id = egui::Id::unique(name);
    if let Some(kept) = context.data(|data| data.get_temp::<Kept<K>>(id))
        && kept.key == key
    {
        return kept.texture;
    }
    let texture = context.load_texture(name, draw(), options);
    context.data_mut(|data| {
        data.insert_temp(
            id,
            Kept {
                key,
                texture: texture.clone(),
            },
        );
    });
    texture
}

/// Adds to `mesh` a quad over the box from `min` to `max` of the pattern, in
/// bead units, the pattern standing at `origin` and shown by `view`. `uv`
/// gives the place in the texture of a corner of the box.
pub fn add_quad(
    mesh: &mut Mesh,
    view: &View,
    origin: Pos2,
    (min, max): (Vec2, Vec2),
    colour: Color32,
    uv: impl Fn(Vec2) -> Pos2,
) {
    let first = mesh.vertices.len() as u32;
    for corner in [min, vec2(max.x, min.y), max, vec2(min.x, max.y)] {
        mesh.vertices.push(Vertex {
            pos: origin + view.to_shown(corner * view.bead),
            uv: uv(corner),
            color: colour,
        });
    }
    mesh.add_triangle(first, first + 1, first + 2);
    mesh.add_triangle(first, first + 2, first + 3);
}

/// The box of each column of `cells`, in bead units: a peyote column stands
/// half a bead lower when it is odd.
pub fn columns(grid: Grid, ratio: f32, cells: Bounds) -> impl Iterator<Item = (Vec2, Vec2)> {
    (cells.min.column..=cells.max.column).map(move |column| {
        let first = Cell {
            column,
            row: cells.min.row,
        };
        let (left, top) = grid.origin(first, ratio);
        let min = vec2(left, top);
        (min, min + vec2(ratio, cells.height() as f32))
    })
}

/// The signed distance from `point` to a box of half size `half` with
/// corners of `radius`, centred on zero: negative inside.
pub fn rounded_box(point: Vec2, half: Vec2, radius: f32) -> f32 {
    let radius = radius.min(half.x).min(half.y).max(0.0);
    let corner = point.abs() - half + Vec2::splat(radius);
    let outside = corner.max(Vec2::ZERO).length();
    let inside = corner.x.max(corner.y).min(0.0);
    outside + inside - radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn the_distance_to_a_box_is_negative_inside() {
        let half = vec2(2.0, 1.0);
        assert!(rounded_box(Vec2::ZERO, half, 0.2) < 0.0);
        assert!((rounded_box(vec2(3.0, 0.0), half, 0.2) - 1.0).abs() < 1e-6);
    }
}
