//! The glow of the beads: what a bead looks like with it, a setting of the
//! program, on by default.
//!
//! ```text
//! variants ──► none     a flat bead, as now
//!              shine    a white spot at the top left, a glass bead
//!              volume   lighter in the middle, darker at the sides and the
//!                       bottom, a round bead
//!              both     volume and shine
//!              halo     a soft light of its own colour around the bead
//! ```
//!
//! Chosen: volume. "Glow" in the menu of the settings turns it on and off.
//!
//! Each is a white sprite tinted the colour of the bead, as the canvas draws
//! a bead now; the shine and the halo are a second sprite over it or under
//! it. No variant costs more than one more quad a bead.

use egui::{
    Color32, ColorImage, Mesh, Pos2, Rect, Sense, TextureHandle, TextureOptions, Vec2, pos2, vec2,
};

use super::{Lang, Mockup, named};

/// The width of a bead to its height.
const RATIO: f32 = 0.8;
/// The pixels a sprite is high.
const PIXELS: usize = 64;
/// The part of a cell left as a gap, and the rounding, parts of its height.
const GAP: f32 = 0.1;
const ROUNDING: f32 = 0.3;
/// How far the halo reaches past the bead, a part of its height.
const HALO: f32 = 0.45;
/// The paper of a new pattern, and black.
const PAPERS: [(&str, &str, Color32); 2] = [
    (
        "Grey paper",
        "Серый фон",
        Color32::from_rgb(0xd6, 0xd4, 0xce),
    ),
    (
        "Black paper",
        "Чёрный фон",
        Color32::from_rgb(0x1e, 0x1e, 0x1e),
    ),
];
/// The colours of the made up pattern, `.` an empty place.
const COLOURS: [(char, Color32); 6] = [
    ('r', Color32::from_rgb(0xc0, 0x30, 0x30)),
    ('b', Color32::from_rgb(0x2f, 0x5f, 0xb3)),
    ('y', Color32::from_rgb(0xe8, 0xc5, 0x47)),
    ('k', Color32::from_rgb(0x1b, 0x1b, 0x1b)),
    ('w', Color32::from_rgb(0xf2, 0xef, 0xe6)),
    ('g', Color32::from_rgb(0x3a, 0x9a, 0x5a)),
];
/// The made up pattern.
const PATTERN: [&str; 7] = [
    "..wwyy..", ".wbbbby.", "wbrrrrbw", "ybrkkrby", "wbrrrrbw", ".ybbbbw.", "..ggkk..",
];

/// The ways a bead can glow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    None,
    Shine,
    Volume,
    Both,
    Halo,
}

impl Variant {
    const ALL: [Variant; 5] = [
        Variant::None,
        Variant::Shine,
        Variant::Volume,
        Variant::Both,
        Variant::Halo,
    ];

    fn name(self, lang: Lang) -> &'static str {
        match self {
            Variant::None => lang.tr("A. None, as now", "A. Нет, как сейчас"),
            Variant::Shine => lang.tr("B. Shine", "Б. Блик"),
            Variant::Volume => lang.tr("C. Volume", "В. Объём"),
            Variant::Both => lang.tr("D. Volume and shine", "Г. Объём и блик"),
            Variant::Halo => lang.tr("E. Halo", "Д. Ореол"),
        }
    }
}

/// The sprites of a bead.
struct Sprites {
    flat: TextureHandle,
    volume: TextureHandle,
    shine: TextureHandle,
    halo: TextureHandle,
}

/// The variants of the glow, side by side, on two papers.
pub struct Glow {
    sprites: Option<Sprites>,
    /// The height of a bead.
    bead: f32,
}

impl Glow {
    pub fn new() -> Glow {
        Glow {
            sprites: None,
            bead: 22.0,
        }
    }

    fn sprites(&mut self, context: &egui::Context) -> &Sprites {
        self.sprites.get_or_insert_with(|| {
            let load = |name: &str, image: ColorImage| {
                context.load_texture(name, image, TextureOptions::LINEAR)
            };
            Sprites {
                flat: load("glow-flat", bead(|_, _| 1.0)),
                volume: load("glow-volume", bead(volume)),
                shine: load("glow-shine", shine()),
                halo: load("glow-halo", halo()),
            }
        })
    }

    /// The made up pattern on `paper`, its beads glowing as `variant` says.
    fn patch(&mut self, ui: &mut egui::Ui, variant: Variant, paper: Color32, name: &str) {
        let bead = self.bead;
        let columns = PATTERN[0].len() + 2;
        let rows = PATTERN.len() + 2;
        let size = vec2(columns as f32 * bead * RATIO, rows as f32 * bead);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, paper);
        let sprites = self.sprites(ui.ctx());
        let cell = |column: usize, row: usize| {
            Rect::from_min_size(
                rect.min + vec2((column + 1) as f32 * RATIO, (row + 1) as f32) * bead,
                vec2(RATIO, 1.0) * bead,
            )
        };
        let beads: Vec<(Rect, Color32)> = PATTERN
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                line.chars().enumerate().filter_map(move |(column, mark)| {
                    COLOURS
                        .iter()
                        .find(|(at, _)| *at == mark)
                        .map(|(_, colour)| (cell(column, row), *colour))
                })
            })
            .collect();
        if variant == Variant::Halo {
            let mut mesh = Mesh::with_texture(sprites.halo.id());
            for (place, colour) in &beads {
                let glow = colour.gamma_multiply(0.55);
                quad(&mut mesh, place.expand(HALO * bead), glow);
            }
            painter.add(mesh);
        }
        let body = match variant {
            Variant::Volume | Variant::Both => &sprites.volume,
            _ => &sprites.flat,
        };
        let mut mesh = Mesh::with_texture(body.id());
        for (place, colour) in &beads {
            quad(&mut mesh, *place, *colour);
        }
        painter.add(mesh);
        if matches!(variant, Variant::Shine | Variant::Both) {
            let mut mesh = Mesh::with_texture(sprites.shine.id());
            for (place, _) in &beads {
                quad(&mut mesh, *place, Color32::WHITE);
            }
            painter.add(mesh);
        }
        named(ui.ctx(), response, name);
    }
}

/// Adds a quad of the whole texture over `rect`, tinted `colour`.
fn quad(mesh: &mut Mesh, rect: Rect, colour: Color32) {
    mesh.add_rect_with_uv(rect, Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)), colour);
}

/// The size of a sprite of a cell, in pixels.
fn sprite_size() -> (usize, usize) {
    ((PIXELS as f32 * RATIO).round() as usize, PIXELS)
}

/// How much of the pixel at `x`, `y` the rounded bead covers, from 0 to 1,
/// and where it is in the bead, from 0 to 1 each way.
fn cover(x: usize, y: usize) -> (f32, f32, f32) {
    let (width, height) = sprite_size();
    let gap = GAP * height as f32 / 2.0;
    let half = vec2(width as f32 / 2.0 - gap, height as f32 / 2.0 - gap);
    let radius = ROUNDING * height as f32;
    let at = vec2(
        x as f32 + 0.5 - width as f32 / 2.0,
        y as f32 + 0.5 - height as f32 / 2.0,
    );
    let inner = (at.abs() - half + Vec2::splat(radius)).max(Vec2::ZERO);
    let outside =
        inner.length() + (at.abs() - half + Vec2::splat(radius)).max_elem().min(0.0) - radius;
    let covered = (0.5 - outside).clamp(0.0, 1.0);
    (
        covered,
        at.x / half.x / 2.0 + 0.5,
        at.y / half.y / 2.0 + 0.5,
    )
}

/// A white bead, its brightness at each place given by `light`.
fn bead(light: impl Fn(f32, f32) -> f32) -> ColorImage {
    let (width, height) = sprite_size();
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let (covered, u, v) = cover(x, y);
            let grey = (light(u, v).clamp(0.0, 1.0) * 255.0) as u8;
            pixels.push(Color32::from_gray(grey).gamma_multiply(covered));
        }
    }
    ColorImage::new([width, height], pixels)
}

/// A round bead: lighter in the middle, darker at the sides and the bottom.
fn volume(u: f32, v: f32) -> f32 {
    let across = (u.clamp(0.0, 1.0) * std::f32::consts::PI).sin().powf(0.6);
    let down = 1.0 - 0.25 * v.clamp(0.0, 1.0).powi(2);
    (0.62 + 0.38 * across) * down
}

/// A white spot at the top left, see through elsewhere.
fn shine() -> ColorImage {
    let (width, height) = sprite_size();
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let (covered, u, v) = cover(x, y);
            let distance = vec2((u - 0.32) / 0.16, (v - 0.24) / 0.11).length();
            let alpha = (1.0 - distance).clamp(0.0, 1.0).powf(0.8) * 0.75 * covered;
            pixels.push(Color32::from_white_alpha((alpha * 255.0) as u8));
        }
    }
    ColorImage::new([width, height], pixels)
}

/// A soft light around a bead, strongest at its edge, gone `HALO` past it.
fn halo() -> ColorImage {
    let (width, height) = sprite_size();
    let reach = HALO * height as f32;
    let (full_width, full_height) = (width + 2 * reach as usize, height + 2 * reach as usize);
    let half = vec2(width as f32 / 2.0, height as f32 / 2.0) * (1.0 - GAP);
    let radius = ROUNDING * height as f32;
    let mut pixels = Vec::with_capacity(full_width * full_height);
    for y in 0..full_height {
        for x in 0..full_width {
            let at = vec2(
                x as f32 + 0.5 - full_width as f32 / 2.0,
                y as f32 + 0.5 - full_height as f32 / 2.0,
            );
            let inner = (at.abs() - half + Vec2::splat(radius)).max(Vec2::ZERO);
            let outside = (inner.length() - radius).max(0.0);
            let alpha = (1.0 - outside / reach).clamp(0.0, 1.0).powi(2);
            pixels.push(Color32::from_white_alpha((alpha * 255.0) as u8));
        }
    }
    ColorImage::new([full_width, full_height], pixels)
}

impl Mockup for Glow {
    fn name(&self) -> &'static str {
        "glow"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({ "bead": self.bead })
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            ui.label(lang.tr("The height of a bead", "Высота бусины"));
            ui.add(egui::Slider::new(&mut self.bead, 10.0..=48.0).text(lang.tr("points", "точек")));
        });
        ui.label(lang.tr(
            "A setting of the program, on by default: \"Glow\" in the menu of the settings.",
            "Настройка программы, по умолчанию включена: «Свечение» в меню настроек.",
        ));
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("glow-variants")
                .spacing(vec2(16.0, 8.0))
                .show(ui, |ui| {
                    for variant in Variant::ALL {
                        ui.strong(variant.name(lang));
                    }
                    ui.end_row();
                    for (en, ru, paper) in PAPERS {
                        for variant in Variant::ALL {
                            let name = format!("{} · {}", variant.name(lang), lang.tr(en, ru));
                            self.patch(ui, variant, paper, &name);
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
