//! The colours drawn around the beads, taken from the paper: the shade past
//! the work area, the shadows of the empty places and the hatch of a bead
//! with no colour to show.
//!
//! ```text
//! paper lighter than LIGHT ──► shade darker than it, shadows of black
//! paper darker             ──► shade lighter than it, shadows of white
//! ```

use egui::Color32;
use obusina_pattern::Rgb;

/// The lightness of OKLab a paper is light from.
pub const LIGHT: f32 = 0.6;
/// How far the shade goes from the paper towards black, or white.
const SHADE_LIGHT: f32 = 0.14;
const SHADE_DARK: f32 = 0.10;

/// The colours drawn around the beads on one paper.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tones {
    /// The paper, the ground of the work area.
    pub paper: Color32,
    /// The ground past the work area: the paper in shade.
    pub shade: Color32,
    /// The shadow of an empty place on a line of the ruling, at its
    /// strongest.
    pub line: Color32,
    /// The shadow of any other empty place, at its strongest: fainter, so
    /// the lines of the ruling stand out.
    pub cell: Color32,
    /// The hatch of a bead of a removed or hidden colour.
    pub hatch: Color32,
}

impl Tones {
    /// The tones on `paper`.
    pub fn of(paper: Rgb) -> Tones {
        let Rgb([red, green, blue]) = paper;
        let paper = Color32::from_rgb(red, green, blue);
        if lightness(paper) >= LIGHT {
            Tones {
                paper,
                shade: paper.lerp_to_gamma(Color32::BLACK, SHADE_LIGHT),
                line: Color32::from_black_alpha(36),
                cell: Color32::from_black_alpha(18),
                hatch: Color32::from_gray(0x40),
            }
        } else {
            Tones {
                paper,
                shade: paper.lerp_to_gamma(Color32::WHITE, SHADE_DARK),
                line: Color32::from_white_alpha(30),
                cell: Color32::from_white_alpha(14),
                hatch: Color32::from_gray(0xc0),
            }
        }
    }
}

/// The lightness of a colour in OKLab, from 0 for black to 1 for white.
pub fn lightness(colour: Color32) -> f32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let [red, green, blue] = [colour.r(), colour.g(), colour.b()].map(linear);
    let long = 0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue;
    let middle = 0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue;
    let short = 0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue;
    0.210_454_26 * long.cbrt() + 0.793_617_8 * middle.cbrt() - 0.004_072_047 * short.cbrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_is_dark_and_white_is_light() {
        assert!(lightness(Color32::BLACK) < 0.01);
        assert!((lightness(Color32::WHITE) - 1.0).abs() < 0.01);
    }

    #[test]
    fn the_shadows_are_dark_on_a_light_paper_and_light_on_a_dark_one() {
        let light = Tones::of(obusina_pattern::DEFAULT_PAPER);
        let dark = Tones::of(Rgb([0x1e, 0x1e, 0x1e]));

        assert!(lightness(light.shade) < lightness(light.paper));
        assert!(lightness(dark.shade) > lightness(dark.paper));
        assert_eq!(light.line, Color32::from_black_alpha(36));
        assert_eq!(dark.line, Color32::from_white_alpha(30));
    }
}
