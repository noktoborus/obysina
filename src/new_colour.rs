//! The colour a "+" puts in an empty place: the one farthest from the
//! colours there are and from the paper of the field (decision 6 of
//! `docs/palette.md`).

use obusina_pattern::Rgb;
use palette::{FromColor, LinSrgb, Oklab, Oklch, Srgb};

/// The lightness steps a new colour is chosen among, in OKLCh.
const LIGHTNESS: [f32; 5] = [0.25, 0.4, 0.55, 0.7, 0.85];
/// The chroma steps; the first is grey.
const CHROMA: [f32; 3] = [0.0, 0.08, 0.16];
/// The step of hue, in degrees.
const HUE_STEP: f32 = 30.0;

/// Of the steps of lightness, chroma and hue a screen can show, the one whose
/// nearest neighbour among `taken` and `paper` is farthest in OKLab.
pub fn next_colour(taken: &[Rgb], paper: Rgb) -> Rgb {
    let taken: Vec<Oklab> = taken.iter().copied().chain([paper]).map(oklab).collect();
    let nearest = |candidate: Rgb| {
        let candidate = oklab(candidate);
        taken
            .iter()
            .map(|other| distance(candidate, *other))
            .fold(f32::INFINITY, f32::min)
    };
    candidates()
        .max_by(|first, second| nearest(*first).total_cmp(&nearest(*second)))
        .unwrap_or(paper)
}

/// The distance of two colours in OKLab, near to how far apart they look.
fn distance(first: Oklab, second: Oklab) -> f32 {
    let (l, a, b) = (first.l - second.l, first.a - second.a, first.b - second.b);
    (l * l + a * a + b * b).sqrt()
}

/// A colour in OKLab.
fn oklab(Rgb(channels): Rgb) -> Oklab {
    let [red, green, blue] = channels;
    let srgb = Srgb::new(red, green, blue).into_format::<f32>();
    Oklab::from_color(srgb.into_linear())
}

/// Every step of lightness, chroma and hue a screen can show; a grey once.
fn candidates() -> impl Iterator<Item = Rgb> {
    let hues: Vec<f32> = (0..(360.0 / HUE_STEP) as usize)
        .map(|step| step as f32 * HUE_STEP)
        .collect();
    let mut steps = Vec::new();
    for lightness in LIGHTNESS {
        for chroma in CHROMA {
            let around: &[f32] = if chroma == 0.0 { &[0.0] } else { &hues };
            steps.extend(around.iter().map(|hue| Oklch::new(lightness, chroma, *hue)));
        }
    }
    steps.into_iter().filter_map(|oklch| {
        let srgb = Srgb::from_linear(LinSrgb::from_color(oklch));
        let channels = [srgb.red, srgb.green, srgb.blue];
        channels
            .iter()
            .all(|channel| (0.0..=1.0).contains(channel))
            .then(|| Rgb(channels.map(|channel: f32| (channel * 255.0).round() as u8)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAPER: Rgb = obusina_pattern::DEFAULT_PAPER;

    /// Next to black and white, a new colour is neither grey nor near
    /// either, nor near the paper.
    #[test]
    fn a_new_colour_is_far_from_the_others() {
        let taken = [Rgb([0, 0, 0]), Rgb([255, 255, 255])];

        let new = next_colour(&taken, PAPER);

        let nearest = taken
            .iter()
            .chain([&PAPER])
            .map(|other| distance(oklab(new), oklab(*other)))
            .fold(f32::INFINITY, f32::min);
        assert!(nearest > 0.15, "{new} is {nearest} from a colour");
    }

    /// Two "+" in a row give two different colours.
    #[test]
    fn the_next_colour_is_not_the_last_one() {
        let first = next_colour(&[], PAPER);

        assert_ne!(next_colour(&[first], PAPER), first);
    }
}
