//! The panel of beads, in its two views: a line, which turns by its room and
//! the edge it stands at, and a block, which stands as a numpad. The numbers
//! are places of the palette, and a number is a key of the numpad.

use egui::{Color32, Sense, vec2};
use obusina_button::SquareButton;

use super::{BUTTON, Lang, Mockup, TEXT, in_hand, named};

/// One way the panel can stand.
struct Case {
    /// The filled places: an empty one is left out of a line and leaves a
    /// gap in a block.
    filled: &'static [u8],
    /// More than one palette: the switch of the palette is shown.
    switch: bool,
    en: &'static str,
    ru: &'static str,
}

const CASES: [Case; 4] = [
    Case {
        filled: &[1, 2, 3],
        switch: false,
        en: "one palette",
        ru: "одна палитра",
    },
    Case {
        filled: &[1, 2, 3, 4, 5],
        switch: true,
        en: "two palettes",
        ru: "две палитры",
    },
    Case {
        filled: &[1, 2, 4, 5],
        switch: true,
        en: "place 3 empty",
        ru: "место 3 пустое",
    },
    Case {
        filled: &[1, 2, 3, 4, 5, 6, 7, 8, 9],
        switch: true,
        en: "9 colours",
        ru: "9 цветов",
    },
];
/// The place shown as the colour in hand.
const IN_HAND: u8 = 2;
/// The space between two swatches, `SPACING` of the panel of beads.
const SPACING: f32 = 4.0;
/// The colour of each place, so a number looks the same in every case.
const COLOURS: [[u8; 3]; 9] = [
    [0x1b, 0x1b, 0x1b],
    [0xf2, 0xef, 0xe6],
    [0xc0, 0x30, 0x30],
    [0x2f, 0x5f, 0xb3],
    [0xe8, 0xc5, 0x47],
    [0x3f, 0x9a, 0x5a],
    [0x8a, 0x4f, 0xb0],
    [0xe0, 0x82, 0x3a],
    [0x4f, 0xb8, 0xc4],
];

/// The rows of a block, from the top: each is its cells from the left, a
/// number or nothing. Rows with no colour take no room.
pub(super) fn block_rows(filled: &[u8]) -> Vec<Vec<Option<u8>>> {
    let width = filled
        .iter()
        .map(|number| (usize::from(*number) - 1) % 3 + 1)
        .max()
        .unwrap_or(0);
    [[7, 8, 9], [4, 5, 6], [1, 2, 3]]
        .iter()
        .map(|row: &[u8; 3]| {
            row[..width]
                .iter()
                .map(|number| filled.contains(number).then_some(*number))
                .collect::<Vec<_>>()
        })
        .filter(|cells| cells.iter().any(Option::is_some))
        .collect()
}

/// The swatch of a place.
fn swatch(ui: &mut egui::Ui, number: u8) {
    let [red, green, blue] = COLOURS[usize::from(number) - 1];
    let fill = Color32::from_rgb(red, green, blue);
    let response = ui.add(
        SquareButton::new(&number.to_string(), BUTTON)
            .text_size(TEXT)
            .fill(fill),
    );
    if number == IN_HAND {
        in_hand(ui, response.rect, Some(fill), &number.to_string());
    }
}

/// The swatch of the empty place of the hand, wide or not.
pub(super) fn zero(ui: &mut egui::Ui, wide: bool) -> egui::Response {
    let size = if wide {
        vec2(BUTTON.x * 2.0 + SPACING, BUTTON.y)
    } else {
        BUTTON
    };
    ui.add(SquareButton::new("0", size).text_size(TEXT))
}

/// A button of the switch of the palette, for touch.
fn arrow(ui: &mut egui::Ui, label: &str) {
    ui.add(SquareButton::new(label, BUTTON).text_size(TEXT));
}

/// The switch of the palette in a block: back and on.
fn switch(ui: &mut egui::Ui) {
    arrow(ui, "<");
    arrow(ui, ">");
}

/// The line: `<`, the numbers rising, `0`, `>`. It is the same along an edge
/// and turned.
fn line(ui: &mut egui::Ui, case: &Case) {
    if case.switch {
        arrow(ui, "<");
    }
    for number in case.filled {
        swatch(ui, *number);
    }
    let _ = zero(ui, false);
    if case.switch {
        arrow(ui, ">");
    }
}

/// The block: the numbers as on a numpad, a wide `0` under `1 2`, then the
/// switch of the palette.
fn block(ui: &mut egui::Ui, case: &Case) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = vec2(SPACING, SPACING);
        for row in block_rows(case.filled) {
            ui.horizontal(|ui| {
                for cell in row {
                    match cell {
                        Some(number) => swatch(ui, number),
                        None => {
                            ui.allocate_exact_size(BUTTON, Sense::hover());
                        }
                    }
                }
            });
        }
        let _ = zero(ui, true);
        if case.switch {
            ui.horizontal(switch);
        }
    });
}

/// A frame around one case, with its name on top and a name for a note.
fn part(ui: &mut egui::Ui, title: &str, name: &str, draw: impl FnOnce(&mut egui::Ui)) {
    let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(SPACING, SPACING);
            ui.label(title);
            draw(ui);
        });
    });
    named(ui.ctx(), frame.response, name);
}

/// The panel of beads in its two views.
#[derive(Default)]
pub struct Numpad;

impl Mockup for Numpad {
    fn name(&self) -> &'static str {
        "numpad"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(lang.tr("Line, along", "Линия, вдоль края"));
            for case in &CASES {
                let title = lang.tr(case.en, case.ru);
                part(ui, title, &format!("line along, {}", case.en), |ui| {
                    ui.horizontal(|ui| line(ui, case));
                });
            }
            ui.separator();
            ui.heading(lang.tr("Line, turned", "Линия, повёрнутая"));
            ui.horizontal_wrapped(|ui| {
                for case in &CASES {
                    let title = lang.tr(case.en, case.ru);
                    part(ui, title, &format!("line turned, {}", case.en), |ui| {
                        ui.vertical(|ui| line(ui, case));
                    });
                }
            });
            ui.separator();
            ui.heading(lang.tr("Block", "Блок"));
            ui.horizontal_wrapped(|ui| {
                for case in &CASES {
                    let title = lang.tr(case.en, case.ru);
                    part(ui, title, &format!("block, {}", case.en), |ui| {
                        block(ui, case);
                    });
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A block folds its empty rows and keeps a gap in a row with a colour.
    #[test]
    fn a_block_stands_as_a_numpad() {
        assert_eq!(
            block_rows(&[1, 2, 3, 4, 5]),
            vec![
                vec![Some(4), Some(5), None],
                vec![Some(1), Some(2), Some(3)]
            ]
        );
        assert_eq!(
            block_rows(&[1, 2, 4, 5]),
            vec![vec![Some(4), Some(5)], vec![Some(1), Some(2)]]
        );
    }

    /// A full block is the keys of a numpad.
    #[test]
    fn a_full_block_is_the_keys() {
        let rows = block_rows(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);

        assert_eq!(rows[0], vec![Some(7), Some(8), Some(9)]);
        assert_eq!(rows[2], vec![Some(1), Some(2), Some(3)]);
    }
}
