//! A bead, drawn once into a texture and laid as a sprite.
//!
//! The beads differ only by their colour. One white bead is drawn into a
//! texture for the size of a bead, and every bead is a quad of it tinted
//! its colour: no tessellation, four vertices a bead.

use egui::{Color32, ColorImage, Context, Mesh, Pos2, TextureHandle, TextureOptions, vec2};
use obusina_pattern::Cell;

use crate::View;
use crate::texture::{add_quad, cached, rounded_box};

/// What a sprite is drawn from.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    ratio: f32,
    /// Pixels a bead is high.
    pixels: f32,
    /// The part of a bead left as a gap, and the rounding, both parts of the
    /// height of a bead.
    gap: f32,
    rounding: f32,
}

/// The texture of one bead `bead` points high, a whole cell with its gap:
/// drawn again only when one of these changes.
pub fn texture(context: &Context, ratio: f32, bead: f32, gap: f32, rounding: f32) -> TextureHandle {
    let key = Key {
        ratio,
        pixels: bead * context.pixels_per_point(),
        gap,
        rounding,
    };
    cached(
        context,
        "obusina-canvas-bead",
        key,
        TextureOptions::LINEAR,
        || draw(key),
    )
}

/// Adds the bead of `cell` to `mesh`, tinted `colour`. The pattern stands
/// at `origin`, shown by `view`.
pub fn add_bead(
    mesh: &mut Mesh,
    grid: obusina_pattern::Grid,
    ratio: f32,
    view: &View,
    origin: Pos2,
    cell: Cell,
    colour: Color32,
) {
    let (left, top) = grid.origin(cell, ratio);
    let min = vec2(left, top);
    let max = min + vec2(ratio, 1.0);
    add_quad(mesh, view, origin, (min, max), colour, |corner| {
        Pos2::new((corner.x - min.x) / ratio, corner.y - min.y)
    });
}

/// Draws a bead: a white rounded box on nothing, its gap left around it.
fn draw(key: Key) -> ColorImage {
    let width = (key.ratio * key.pixels).round().max(1.0) as usize;
    let height = key.pixels.round().max(1.0) as usize;
    let per_pixel = vec2(key.ratio / width as f32, 1.0 / height as f32);
    let shrink = key.gap / 2.0;
    let half = vec2(key.ratio / 2.0 - shrink, 0.5 - shrink);
    let centre = vec2(key.ratio, 1.0) / 2.0;
    let radius = (1.0 - key.gap) * key.rounding;
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let point = vec2(x as f32 + 0.5, y as f32 + 0.5) * per_pixel;
            let distance = rounded_box(point - centre, half, radius) * key.pixels;
            let cover = (0.5 - distance).clamp(0.0, 1.0);
            Color32::from_white_alpha((cover * 255.0).round() as u8)
        })
        .collect();
    ColorImage::new([width, height], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bead_is_whole_in_the_middle_and_leaves_its_gap() {
        let image = draw(Key {
            ratio: 0.8,
            pixels: 20.0,
            gap: 0.1,
            rounding: 0.3,
        });
        let alpha = |x: usize, y: usize| image.pixels[y * image.size[0] + x].a();

        assert_eq!(image.size, [16, 20]);
        assert_eq!(alpha(8, 10), 255);
        assert_eq!(alpha(0, 10), 0, "the gap is empty");
        assert_eq!(alpha(1, 1), 0, "the corner is rounded off");
    }
}
