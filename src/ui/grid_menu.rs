//! The menu of the kinds of grid, a `PlateMenu` in the middle of the window,
//! and the ruling of the pattern under it.
//!
//! ```text
//! grid ──► loom | peyote ──► the kind of grid, a step of undo
//!          ruling…       ──► the four numbers on top, then
//!                              │──accept──► the ruling stays, a step of undo
//!                              └──cancel──► the ruling goes back
//!                            closed with no entry ──► the ruling goes back
//! ```
//!
//! A number of the ruling changes the shadows on the field at once.

use std::sync::{Arc, Mutex};

use obusina_pattern::{Grid, Ruling};
use plate_menu::{MenuItem, Notice};
use rust_i18n::t;

use crate::app::App;
use crate::ui::access::Named;

/// The most columns or rows from one line of the ruling to the next.
pub const MAX_RULING_STEP: u32 = 100;
/// The farthest a line of the ruling is set from cell (0, 0).
pub const MAX_RULING_OFFSET: i32 = 1000;

/// The ruling while its menu is open.
pub struct RulingEdit {
    /// The ruling the menu opened on, put back when it is not accepted.
    before: Ruling,
    /// The ruling the numbers of the notice change.
    editing: Arc<Mutex<Ruling>>,
}

/// Opens the menu on the kind the pattern has.
pub fn open(app: &mut App) {
    let mut items: Vec<MenuItem> = Grid::ALL
        .iter()
        .map(|grid| MenuItem::new(id(*grid), name(*grid)))
        .collect();
    items.push(MenuItem::separator());
    items.push(MenuItem::new("ruling", t!("ruling.open")));
    if let Err(error) = app.menu.open_at(items, id(app.pattern.grid())) {
        log::error!("the menu of grids does not open: {error}");
    }
}

/// Opens the menu of the ruling: its numbers on top, then accept and cancel.
fn open_ruling(app: &mut App) {
    let items = vec![
        MenuItem::new("accept", t!("ruling.accept")),
        MenuItem::new("cancel", t!("ruling.cancel")),
    ];
    if let Err(error) = app.menu.open(items) {
        log::error!("the menu of the ruling does not open: {error}");
        return;
    }
    let before = app.pattern.ruling();
    let editing = Arc::new(Mutex::new(before));
    app.menu.notice(numbers(Arc::clone(&editing)));
    app.ruling_edit = Some(RulingEdit { before, editing });
}

/// The numbers of the ruling: the step and the offset, across and down.
fn numbers(editing: Arc<Mutex<Ruling>>) -> Notice {
    Notice::widget(move |ui| {
        let mut ruling = editing.lock().expect("the ruling is never poisoned");
        egui::Grid::new("ruling-numbers").show(ui, |ui| {
            ui.label(t!("ruling.step"));
            for (step, name) in ruling
                .step
                .iter_mut()
                .zip([t!("ruling.step_x"), t!("ruling.step_y")])
            {
                let field = egui::DragValue::new(step).range(0..=MAX_RULING_STEP);
                ui.add(field).named(&name);
            }
            ui.end_row();
            ui.label(t!("ruling.offset"));
            for (offset, name) in ruling
                .offset
                .iter_mut()
                .zip([t!("ruling.offset_x"), t!("ruling.offset_y")])
            {
                let range = -MAX_RULING_OFFSET..=MAX_RULING_OFFSET;
                let field = egui::DragValue::new(offset).range(range);
                ui.add(field).named(&name);
            }
            ui.end_row();
        });
    })
}

/// Draws the menu while it is open, and does what is chosen. While nothing
/// else of the window answers (`src/ui/blocker.rs`), the numbers of the
/// ruling show on the field as they change.
pub fn show(app: &mut App, context: &egui::Context) {
    app.menu.nothing_found(t!("menu.nothing_found"));
    if app.menu.is_open() {
        crate::ui::blocker::under(context, "grid-menu", &app.menu);
    }
    let chosen = app.menu.show(context);
    if let Some(edit) = &app.ruling_edit {
        let shown = *edit.editing.lock().expect("the ruling is never poisoned");
        app.pattern.set_ruling(shown);
    }
    let Some(chosen) = chosen else {
        if !app.menu.is_open()
            && let Some(edit) = app.ruling_edit.take()
        {
            app.pattern.set_ruling(edit.before);
        }
        return;
    };
    match (chosen.id.as_str(), app.ruling_edit.take()) {
        ("accept", Some(edit)) => app.history.keep_ruling(&app.pattern, edit.before),
        ("cancel", Some(edit)) => app.pattern.set_ruling(edit.before),
        ("ruling", None) => open_ruling(app),
        (kind, None) => {
            if let Some(grid) = Grid::ALL.into_iter().find(|grid| id(*grid) == kind) {
                app.set_grid(grid);
            }
        }
        (_, Some(edit)) => app.pattern.set_ruling(edit.before),
    }
}

/// The identifier of the entry of a kind.
fn id(grid: Grid) -> &'static str {
    match grid {
        Grid::Loom => "loom",
        Grid::Peyote => "peyote",
    }
}

/// The name of a kind of grid.
fn name(grid: Grid) -> String {
    match grid {
        Grid::Loom => t!("grid.loom"),
        Grid::Peyote => t!("grid.peyote"),
    }
    .to_string()
}
