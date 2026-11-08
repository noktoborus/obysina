//! The size of a field set by hand: its columns and its rows, two numbers on
//! top of the level "Field" of the menu of the settings
//! (`src/ui/settings_menu.rs`).
//!
//! The numbers change the field about its middle. A field made smaller than
//! the beads keeps the beads past it; it takes no new bead there.

use std::sync::{Arc, Mutex};

use obusina_canvas::View;
use obusina_pattern::{Bounds, Pattern};
use plate_menu::Notice;
use rust_i18n::t;

use crate::ui::access::Named;

/// The most columns or rows a field can have.
pub const MAX_SIDE: usize = 1000;

/// The size the numbers change, and the field they change it from.
pub struct Sizing {
    /// The field the numbers started from.
    pub before: Bounds,
    /// The columns and the rows the numbers show.
    pub size: Arc<Mutex<[usize; 2]>>,
}

impl Sizing {
    /// The size of `before`, to be changed about its middle.
    pub fn new(before: Bounds) -> Sizing {
        Sizing {
            before,
            size: Arc::new(Mutex::new([before.width(), before.height()])),
        }
    }

    /// The field the numbers give: `before` resized about its middle.
    pub fn field(&self) -> Bounds {
        let [width, height] = *self.size.lock().expect("the size is never poisoned");
        self.before.resized(width, height)
    }

    /// The numbers on top of the menu: the columns and the rows, from 1 up to
    /// [`MAX_SIDE`].
    pub fn numbers(&self) -> Notice {
        let size = Arc::clone(&self.size);
        Notice::widget(move |ui| {
            let mut size = size.lock().expect("the size is never poisoned");
            egui::Grid::new("field-numbers").show(ui, |ui| {
                ui.label(t!("field.size"));
                for (side, name) in size.iter_mut().zip([t!("field.width"), t!("field.height")]) {
                    let number = egui::DragValue::new(side).range(1..=MAX_SIDE);
                    ui.add(number).named(&name);
                }
                ui.end_row();
            });
        })
    }
}

/// The field "Set" gives a pattern sized by the beads: the screen at zoom 1
/// about its middle (`View::first_screen`), grown to hold the beads.
pub fn first_field(pattern: &Pattern, view: &View) -> Bounds {
    let first = view.first_screen(pattern.grid(), pattern.ratio());
    pattern.bounds().map_or(first, |beads| first.union(beads))
}

/// The size of the field right of its entry in the menu: its columns and
/// rows, or "by the beads".
pub fn detail(field: Option<Bounds>) -> String {
    match field {
        Some(field) => format!("{}\u{d7}{}", field.width(), field.height()),
        None => t!("field.by_beads_detail").to_string(),
    }
}
