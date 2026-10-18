//! The gallery: a strip of the names of the mockups over the one shown, and
//! the switch that leaves a note on it.

use crate::mock::{self, Mockup};
use crate::notes::Notes;

/// The height of the strip.
const STRIP: f32 = 28.0;

/// Every mockup, the one shown and the notes on them.
pub struct Gallery {
    mocks: Vec<Box<dyn Mockup>>,
    shown: usize,
    pub(crate) notes: Notes,
}

impl Gallery {
    /// The gallery showing the mockup named by `hash`, `#name` of the
    /// address; the first one when it names none.
    pub fn new(hash: &str) -> Gallery {
        let mocks = mock::all();
        let name = hash.trim_start_matches('#');
        let shown = mocks
            .iter()
            .position(|mock| mock.name() == name)
            .unwrap_or(0);
        Gallery {
            mocks,
            shown,
            notes: Notes::default(),
        }
    }
}

impl eframe::App for Gallery {
    /// The strip takes its room as a panel, and is drawn over it on a layer
    /// above every window of a mockup: a modal window of egui blocks the
    /// layers under it, and the switch of the notes must work over one.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        mock::forget_named(ui.ctx());
        let room = egui::Panel::top("mockups")
            .exact_size(STRIP)
            .show(ui, |_| {})
            .response
            .rect;
        egui::Area::new(egui::Id::unique("mockups-strip"))
            .order(egui::Order::Tooltip)
            .fixed_pos(room.min)
            .show(ui.ctx(), |ui| {
                ui.set_width(room.width());
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    for (index, mock) in self.mocks.iter().enumerate() {
                        if ui
                            .selectable_label(index == self.shown, mock.name())
                            .clicked()
                        {
                            self.shown = index;
                        }
                    }
                    ui.separator();
                    self.notes.switch(ui, self.mocks[self.shown].name());
                });
            });
        let central = egui::CentralPanel::default().show(ui, |ui| self.mocks[self.shown].show(ui));
        let mock = &self.mocks[self.shown];
        self.notes
            .show(ui.ctx(), central.response.rect, mock.as_ref());
    }
}
