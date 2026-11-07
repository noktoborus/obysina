//! The size of the sheet: its columns and its rows, two numbers on top of the
//! menu of the settings, as the ruling is (`src/ui/settings_menu.rs`).
//!
//! The sheet is the size of the work area set by hand, kept in the pattern
//! and its file. A new pattern takes the cells of the first screen; the
//! numbers show the size and change it, keeping its middle. A size is never
//! smaller than the box around the beads, and the sheet moves as little as
//! it takes to hold them.

use std::sync::{Arc, Mutex};

use obusina_pattern::Bounds;
use plate_menu::Notice;
use rust_i18n::t;

use crate::ui::access::Named;

/// The most columns or rows a sheet can have.
pub const MAX_SIDE: usize = 1000;

/// The size the numbers change, and the sheet they change it from.
pub struct Sizing {
    /// The sheet the menu opened on.
    pub before: Bounds,
    /// The box around the beads, which the sheet holds.
    pub beads: Option<Bounds>,
    /// The columns and the rows the numbers show.
    pub size: Arc<Mutex<[usize; 2]>>,
}

impl Sizing {
    /// The size of `before`, to be changed over `beads`.
    pub fn new(before: Bounds, beads: Option<Bounds>) -> Sizing {
        Sizing {
            before,
            beads,
            size: Arc::new(Mutex::new([before.width(), before.height()])),
        }
    }

    /// The sheet the numbers give: `before` resized around its middle, over
    /// the beads.
    pub fn sheet(&self) -> Bounds {
        let [width, height] = *self.size.lock().expect("the size is never poisoned");
        self.before.resized_over(width, height, self.beads)
    }

    /// The numbers on top of the menu: the columns and the rows, from the box
    /// around the beads up to [`MAX_SIDE`].
    pub fn numbers(&self) -> Notice {
        let size = Arc::clone(&self.size);
        let least = [
            self.beads.map_or(1, |beads| beads.width()).min(MAX_SIDE),
            self.beads.map_or(1, |beads| beads.height()).min(MAX_SIDE),
        ];
        Notice::widget(move |ui| {
            let mut size = size.lock().expect("the size is never poisoned");
            egui::Grid::new("sheet-numbers").show(ui, |ui| {
                ui.label(t!("sheet.size"));
                for ((side, least), name) in size
                    .iter_mut()
                    .zip(least)
                    .zip([t!("sheet.width"), t!("sheet.height")])
                {
                    let field = egui::DragValue::new(side).range(least..=MAX_SIDE);
                    ui.add(field).named(&name);
                }
                ui.end_row();
            });
        })
    }
}

/// The size of a sheet right of its entry in the menu.
pub fn detail(sheet: Bounds) -> String {
    format!("{}\u{d7}{}", sheet.width(), sheet.height())
}
