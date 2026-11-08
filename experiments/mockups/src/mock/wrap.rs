//! The repeats of a closed field: past the field its copies, half as bright.
//!
//! ```text
//! closed  none        the field, the shade around it
//!         across      copies left and right, in the rows of the field
//!         down        copies above and below, in the columns of the field
//!         tile        copies everywhere
//! size    auto        the field is the box of the beads
//!         by hand     the field is set; beads past it, when it was made
//!                     smaller than them, stand over the repeats in their
//!                     own colour
//! grid    loom, peyote: an odd width shows the seam of peyote as it is
//! ```
//!
//! A repeat looks up the cell of the field it copies, so its cost does not
//! grow with the number of repeats on the screen. The shadows of empty
//! places, and the ruling with them, stand in the field only.

use egui::{Color32, Rect, Sense, Stroke, StrokeKind, vec2};

use super::{Lang, Mockup, named};

/// The width of a bead to its height.
const RATIO: f32 = 0.8;
/// The height of a bead, in points.
const BEAD: f32 = 18.0;
/// The part of a cell left as a gap, and the rounding, parts of its height.
const GAP: f32 = 0.12;
const ROUNDING: f32 = 0.3;
/// The paper, the shade past the field and the shadow of an empty place.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
const SHADE: Color32 = Color32::from_rgb(0xa8, 0xa6, 0xa0);
const SHADOW: Color32 = Color32::from_black_alpha(22);
const RULED: Color32 = Color32::from_black_alpha(48);
/// The ruling: a darker shadow every so many columns and rows.
const RULING: i32 = 4;
/// The colours of the made up pattern, `.` an empty place.
const COLOURS: [(char, Color32); 6] = [
    ('r', Color32::from_rgb(0xc0, 0x30, 0x30)),
    ('b', Color32::from_rgb(0x2f, 0x5f, 0xb3)),
    ('y', Color32::from_rgb(0xe8, 0xc5, 0x47)),
    ('k', Color32::from_rgb(0x1b, 0x1b, 0x1b)),
    ('w', Color32::from_rgb(0xf2, 0xef, 0xe6)),
    ('g', Color32::from_rgb(0x3a, 0x9a, 0x5a)),
];
/// The made up pattern, eight wide; the ninth column makes it odd.
const PATTERN: [&str; 7] = [
    "k.wwyy.kg",
    ".wbbbby.g",
    "wbrrrrbwg",
    "ybrkkrbyg",
    "wbrrrrbwg",
    ".ybbbbw.g",
    "k.ggkk.kg",
];

/// How the field is closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Closed {
    None,
    Across,
    Down,
    Tile,
}

/// How the size of the field is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Size {
    Auto,
    ByHand,
    Smaller,
}

/// The repeats of a closed field, with the closing, the size, the grid and
/// the width chosen on top.
pub struct Wrap {
    closed: Closed,
    size: Size,
    peyote: bool,
    odd: bool,
}

impl Wrap {
    pub fn new() -> Wrap {
        Wrap {
            closed: Closed::Tile,
            size: Size::Auto,
            peyote: false,
            odd: false,
        }
    }

    /// The width of the pattern in columns.
    fn width(&self) -> i32 {
        if self.odd { 9 } else { 8 }
    }

    /// The bead of the made up pattern at `column`, `row`.
    fn bead(&self, column: i32, row: i32) -> Option<Color32> {
        if column < 0 || column >= self.width() || row < 0 || row >= PATTERN.len() as i32 {
            return None;
        }
        let mark = PATTERN[row as usize].chars().nth(column as usize)?;
        COLOURS
            .iter()
            .find(|(at, _)| *at == mark)
            .map(|(_, colour)| *colour)
    }

    /// The field: its first cell and its size, in cells.
    fn field(&self) -> (i32, i32, i32, i32) {
        let (width, height) = (self.width(), PATTERN.len() as i32);
        match self.size {
            Size::Auto => (0, 0, width, height),
            Size::ByHand => (-2, -1, width + 4, height + 2),
            Size::Smaller => (1, 1, width - 3, height - 3),
        }
    }

    /// The choices on top.
    fn choices(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal_wrapped(|ui| {
            ui.label(lang.tr("Closed:", "Замыкание:"));
            for (closed, en, ru) in [
                (Closed::None, "none", "нет"),
                (Closed::Across, "across", "по горизонтали"),
                (Closed::Down, "down", "по вертикали"),
                (Closed::Tile, "tile", "плитка"),
            ] {
                let response = ui.selectable_label(self.closed == closed, lang.tr(en, ru));
                if named(ui.ctx(), response, lang.tr(en, ru)).clicked() {
                    self.closed = closed;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(lang.tr("Size:", "Размер:"));
            for (size, en, ru) in [
                (Size::Auto, "auto", "авто"),
                (Size::ByHand, "by hand", "вручную"),
                (
                    Size::Smaller,
                    "by hand, smaller than the beads",
                    "вручную, меньше бусин",
                ),
            ] {
                let response = ui.selectable_label(self.size == size, lang.tr(en, ru));
                if named(ui.ctx(), response, lang.tr(en, ru)).clicked() {
                    self.size = size;
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(lang.tr("Grid:", "Сетка:"));
            for (peyote, en, ru) in [(false, "loom", "ткацкая"), (true, "peyote", "мозаика")]
            {
                let response = ui.selectable_label(self.peyote == peyote, lang.tr(en, ru));
                if named(ui.ctx(), response, lang.tr(en, ru)).clicked() {
                    self.peyote = peyote;
                }
            }
            ui.separator();
            ui.label(lang.tr("Width:", "Ширина:"));
            for (odd, en, ru) in [
                (false, "even, 8", "чётная, 8"),
                (true, "odd, 9", "нечётная, 9"),
            ] {
                let response = ui.selectable_label(self.odd == odd, lang.tr(en, ru));
                if named(ui.ctx(), response, lang.tr(en, ru)).clicked() {
                    self.odd = odd;
                }
            }
        });
    }

    /// What the field shows now, in one line over it.
    fn note(&self, lang: Lang) -> &'static str {
        match (self.size, self.closed) {
            (Size::Smaller, _) => lang.tr(
                "The beads past the field stand in their own colour, over the repeats; the panel counts only the beads in the field.",
                "Бусины за полем — своим цветом, поверх повторов; панель считает только бусины в поле.",
            ),
            (Size::Auto, Closed::None) => lang.tr(
                "Auto: the paper is the box of the beads.",
                "Авто: бумага — рамка бусин.",
            ),
            (Size::Auto, _) => lang.tr(
                "Auto: a bead put in a repeat is a bead; the field grows, and the repeats move with it.",
                "Авто: бусина, поставленная в повтор, настоящая; поле растёт, повторы едут за ним.",
            ),
            (Size::ByHand, Closed::None) => lang.tr(
                "By hand: no bead is put past the field.",
                "Вручную: за поле бусину не поставить.",
            ),
            (Size::ByHand, _) => lang.tr(
                "By hand: a press on a repeat does nothing.",
                "Вручную: нажатие на повтор ничего не делает.",
            ),
        }
    }

    /// The field and what stands past it, over all the space left.
    fn canvas(&self, ui: &mut egui::Ui, lang: Lang) {
        ui.label(self.note(lang));
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, SHADE);
        let (fx, fy, fw, fh) = self.field();
        let cell = vec2(RATIO, 1.0) * BEAD;
        let field_size = vec2(fw as f32, fh as f32) * cell;
        let origin = rect.center() - field_size / 2.0 - vec2(fx as f32, fy as f32) * cell;
        let first = ((rect.min - origin) / cell).floor();
        let last = ((rect.max - origin) / cell).ceil();
        let inside =
            |column: i32, row: i32| (fx..fx + fw).contains(&column) && (fy..fy + fh).contains(&row);
        let repeats = |column: i32, row: i32| {
            let rows = (fy..fy + fh).contains(&row);
            let columns = (fx..fx + fw).contains(&column);
            match self.closed {
                Closed::None => false,
                Closed::Across => rows,
                Closed::Down => columns,
                Closed::Tile => true,
            }
        };
        let place = |column: i32, row: i32| -> Rect {
            let drop = if self.peyote && column.rem_euclid(2) == 1 {
                0.5
            } else {
                0.0
            };
            Rect::from_min_size(
                origin + vec2(column as f32 * cell.x, (row as f32 + drop) * cell.y),
                cell,
            )
        };
        let bead = |painter: &egui::Painter, at: Rect, colour: Color32| {
            painter.rect_filled(at.shrink(GAP * BEAD / 2.0), ROUNDING * BEAD, colour);
        };
        for row in first.y as i32 - 1..=last.y as i32 {
            for column in first.x as i32 - 1..=last.x as i32 {
                let at = place(column, row);
                if inside(column, row) {
                    painter.rect_filled(at, 0.0, PAPER);
                } else if repeats(column, row) {
                    painter.rect_filled(at, 0.0, half(PAPER));
                }
            }
        }
        for row in first.y as i32 - 1..=last.y as i32 {
            for column in first.x as i32 - 1..=last.x as i32 {
                let at = place(column, row);
                let own = self
                    .bead(column, row)
                    .filter(|_| inside(column, row) || self.size == Size::Smaller);
                if let Some(colour) = own {
                    bead(&painter, at, colour);
                    continue;
                }
                if inside(column, row) {
                    let ruled = (column - fx) % RULING == 0 || (row - fy) % RULING == 0;
                    bead(&painter, at, if ruled { RULED } else { SHADOW });
                    continue;
                }
                if !repeats(column, row) {
                    continue;
                }
                let source = (
                    fx + (column - fx).rem_euclid(fw),
                    fy + (row - fy).rem_euclid(fh),
                );
                if let Some(colour) = self.bead(source.0, source.1) {
                    bead(&painter, at, half(colour));
                }
            }
        }
        let outline = Rect::from_min_size(origin + vec2(fx as f32, fy as f32) * cell, field_size);
        painter.rect_stroke(
            outline,
            0.0,
            Stroke::new(1.5, Color32::from_rgb(0x00, 0x5c, 0x80)),
            StrokeKind::Outside,
        );
        named(ui.ctx(), response, lang.tr("Field", "Поле"));
    }
}

/// `colour` half as bright: a repeat.
fn half(colour: Color32) -> Color32 {
    let [red, green, blue, alpha] = colour.to_array();
    Color32::from_rgba_unmultiplied(red / 2, green / 2, blue / 2, alpha)
}

impl Mockup for Wrap {
    fn name(&self) -> &'static str {
        "wrap"
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        self.choices(ui, lang);
        self.canvas(ui, lang);
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "closed": format!("{:?}", self.closed),
            "size": format!("{:?}", self.size),
            "peyote": self.peyote,
            "odd": self.odd,
        })
    }
}
