//! The gallery: a strip of the names of the mockups over the one shown.

use crate::mock::{self, Mockup};

/// Every mockup and the one shown.
pub struct Gallery {
    mocks: Vec<Box<dyn Mockup>>,
    shown: usize,
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
        Gallery { mocks, shown }
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("mockups").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, mock) in self.mocks.iter().enumerate() {
                    if ui
                        .selectable_label(index == self.shown, mock.name())
                        .clicked()
                    {
                        self.shown = index;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| self.mocks[self.shown].show(ui));
    }
}
