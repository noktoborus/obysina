//! The volume of a filled button: a white button, round in its light, drawn
//! once into a texture for its size and tinted with the fill. It is the light
//! of a round bead of `obusina-canvas` (`sprite.rs`, `volume`), so a swatch
//! looks like the beads of its colour.

use egui::{Color32, ColorImage, Context, Id, TextureHandle, TextureOptions, Vec2, vec2};

/// How dark the sides of the button are, and its bottom: the numbers of a
/// round bead.
const SIDES: f32 = 0.38;
const BOTTOM: f32 = 0.25;

/// What a texture is drawn for: its size in pixels and the radius of its
/// corners in pixels, both rounded to whole pixels.
type Key = ([u32; 2], u32);

/// The texture of a white button of `size` points with corners of `radius`
/// points, at the scale of `context`; drawn again only for a new size.
pub fn texture(context: &Context, size: Vec2, radius: f32) -> TextureHandle {
    let scale = context.pixels_per_point();
    let pixels = [
        (size.x * scale).round().max(1.0) as u32,
        (size.y * scale).round().max(1.0) as u32,
    ];
    let key: Key = (pixels, (radius * scale).round() as u32);
    let id = Id::unique(("obusina-button-volume", key));
    if let Some(texture) = context.data(|data| data.get_temp::<TextureHandle>(id)) {
        return texture;
    }
    let texture = context.load_texture("obusina-button-volume", draw(key), TextureOptions::LINEAR);
    context.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

/// The pixels of a white button: covered inside its rounded box, its grey
/// the light at each place, premultiplied.
fn draw(([width, height], radius): Key) -> ColorImage {
    let half = vec2(width as f32, height as f32) / 2.0;
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let at = vec2(x as f32 + 0.5, y as f32 + 0.5);
            let distance = rounded_box(at - half, half, radius as f32);
            let alpha = (0.5 - distance).clamp(0.0, 1.0);
            let light = volume(at.x / width as f32, at.y / height as f32);
            let grey = (255.0 * alpha * light).round() as u8;
            Color32::from_rgba_premultiplied(grey, grey, grey, (255.0 * alpha).round() as u8)
        })
        .collect();
    ColorImage::new([width as usize, height as usize], pixels)
}

/// The light at a place of the button, `across` and `down` from 0 to 1: whole
/// in the middle, darker to its sides and its bottom.
fn volume(across: f32, down: f32) -> f32 {
    let sides = (across * std::f32::consts::PI).sin().max(0.0).powf(0.6);
    let bottom = 1.0 - BOTTOM * down * down;
    (1.0 - SIDES + SIDES * sides) * bottom
}

/// The distance from `point` to a box of half size `half` with corners of
/// `radius`, all from its middle; negative inside.
fn rounded_box(point: Vec2, half: Vec2, radius: f32) -> f32 {
    let radius = radius.min(half.x).min(half.y).max(0.0);
    let corner = point.abs() - half + Vec2::splat(radius);
    let outside = corner.max(Vec2::ZERO).length();
    let inside = corner.x.max(corner.y).min(0.0);
    outside + inside - radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_middle_is_whole_and_the_sides_darker() {
        assert!((volume(0.5, 0.0) - 1.0).abs() < 1e-6);
        assert!(volume(0.05, 0.5) < volume(0.5, 0.5));
        assert!(volume(0.5, 1.0) < volume(0.5, 0.0));
    }

    #[test]
    fn a_corner_pixel_is_left_out() {
        let image = draw(([20, 10], 4));
        assert_eq!(image.pixels[0].a(), 0, "the top left corner");
        assert_eq!(image.pixels[5 * 20 + 10].a(), 255, "the middle");
    }
}
