//! The window that fills a palette: a modal with no title. The colours
//! stand in a line, a "-" under each, a "+" at the end; the colour chosen is
//! changed in a picker under the line.

use egui::{Color32, Id, Modal};

use super::{Mockup, button};

/// The most colours a palette holds.
const MAX_COLOURS: usize = 9;
/// The colour a "+" adds.
const NEW_COLOUR: Color32 = Color32::from_rgb(0x80, 0x80, 0x80);

/// A palette and whether its window is open.
pub struct PaletteEditor {
    colours: Vec<Color32>,
    chosen: Option<usize>,
    open: bool,
}

impl Default for PaletteEditor {
    fn default() -> Self {
        PaletteEditor {
            colours: vec![
                Color32::from_rgb(0x1b, 0x1b, 0x1b),
                Color32::from_rgb(0xf2, 0xef, 0xe6),
                Color32::from_rgb(0xc0, 0x30, 0x30),
                Color32::from_rgb(0x2f, 0x5f, 0xb3),
                Color32::from_rgb(0xe8, 0xc5, 0x47),
            ],
            chosen: None,
            open: true,
        }
    }
}

impl Mockup for PaletteEditor {
    fn name(&self) -> &'static str {
        "palette-editor"
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        if ui.button("open the palette").clicked() {
            self.open = true;
        }
        if !self.open {
            return;
        }
        let modal = Modal::new(Id::unique("palette-editor")).show(ui.ctx(), |ui| self.window(ui));
        if modal.should_close() {
            self.open = false;
        }
    }
}

impl PaletteEditor {
    /// The content of the window.
    fn window(&mut self, ui: &mut egui::Ui) {
        let mut removed = None;
        ui.horizontal_wrapped(|ui| {
            for (index, colour) in self.colours.iter().enumerate() {
                ui.vertical(|ui| {
                    let key = (index + 1).to_string();
                    let chosen = self.chosen == Some(index);
                    if ui
                        .add(button(&key).fill(*colour).selected(chosen))
                        .clicked()
                    {
                        self.chosen = Some(index);
                    }
                    if ui.add(button("\u{2212}")).clicked() {
                        removed = Some(index);
                    }
                });
            }
            let full = self.colours.len() >= MAX_COLOURS;
            if ui.add_enabled(!full, button("+")).clicked() {
                self.colours.push(NEW_COLOUR);
                self.chosen = Some(self.colours.len() - 1);
            }
        });
        if let Some(index) = removed {
            self.colours.remove(index);
            self.chosen = None;
        }
        if let Some(index) = self.chosen {
            ui.separator();
            egui::color_picker::color_picker_color32(
                ui,
                &mut self.colours[index],
                egui::color_picker::Alpha::Opaque,
            );
        }
    }
}
