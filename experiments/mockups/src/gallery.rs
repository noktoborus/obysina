//! The gallery: a strip of the names of the mockups over the one shown, and
//! the switches of the notes on it.
//!
//! A press on the strip is taken out of the input before egui sees it, so
//! the mockup never sees it either: a menu of the mockup that closes on a
//! press beside it stays open while the notes are switched on.

use egui::{Event, Pos2, Rect};

use crate::mock::{self, Mockup};
use crate::notes::Notes;

/// The height of the strip.
const STRIP: f32 = 28.0;

/// Every mockup, the one shown and the notes on them.
pub struct Gallery {
    mocks: Vec<Box<dyn Mockup>>,
    shown: usize,
    pub(crate) notes: Notes,
    /// Where the strip stood in the last frame.
    strip: Rect,
    /// A press on the strip taken out of the input, for the strip to read.
    press: Option<Pos2>,
    /// The release of that press is taken out too.
    pressing: bool,
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
            strip: Rect::NOTHING,
            press: None,
            pressing: false,
        }
    }

    /// Takes a press on the strip, and its release, out of the input.
    pub(crate) fn take_strip_presses(&mut self, events: &mut Vec<Event>) {
        let strip = self.strip;
        events.retain(|event| match event {
            Event::PointerButton {
                pos, pressed: true, ..
            } if strip.contains(*pos) => {
                self.press = Some(*pos);
                self.pressing = true;
                false
            }
            Event::PointerButton { pressed: false, .. } if self.pressing => {
                self.pressing = false;
                false
            }
            Event::Touch { pos, .. } => !strip.contains(*pos),
            _ => true,
        });
    }

    /// The strip: the names of the mockups and the switches of the notes.
    fn strip(&mut self, context: &egui::Context, room: Rect) {
        let press = self.press.take();
        let tapped = |response: &egui::Response| {
            response.clicked() || press.is_some_and(|at| response.rect.contains(at))
        };
        egui::Area::new(egui::Id::unique("mockups-strip"))
            .order(egui::Order::Tooltip)
            .fixed_pos(room.min)
            .show(context, |ui| {
                ui.set_width(room.width());
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    for (index, mock) in self.mocks.iter().enumerate() {
                        if tapped(&ui.selectable_label(index == self.shown, mock.name()))
                            && index != self.shown
                        {
                            self.shown = index;
                            self.notes.off();
                        }
                    }
                    ui.separator();
                    self.notes.switch(ui, self.mocks[self.shown].name(), tapped);
                });
            });
        self.strip = room;
    }
}

impl eframe::App for Gallery {
    fn raw_input_hook(&mut self, _context: &egui::Context, raw_input: &mut egui::RawInput) {
        self.take_strip_presses(&mut raw_input.events);
    }

    /// The strip takes its room as a panel, and is drawn over it on a layer
    /// above every window of a mockup: a modal window of egui blocks the
    /// layers under it. A frozen mockup is not drawn: its picture is.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        mock::forget_named(ui.ctx());
        let room = egui::Panel::top("mockups")
            .exact_size(STRIP)
            .show(ui, |_| {})
            .response
            .rect;
        self.strip(ui.ctx(), room);
        let central = egui::CentralPanel::default().show(ui, |ui| {
            if self.notes.frozen() {
                self.notes.paint_frozen(ui);
            } else {
                self.mocks[self.shown].show(ui);
            }
        });
        let mock = &self.mocks[self.shown];
        self.notes
            .show(ui.ctx(), central.response.rect, mock.as_ref());
    }
}
