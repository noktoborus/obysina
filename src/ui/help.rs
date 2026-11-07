//! The plate of the keys: F1 on any layer shows the keys of the layer on
//! top and the global keys, in the middle of the window (`docs/keys.md`).
//!
//! ```text
//! F1 ──► the plate of the keys of the layer under it
//!          │──a tap on it or beside it──► closed
//!          │──F1 or Escape──────────────► closed
//!          └──any other key──────────────► taken, nothing under it acts
//! ```
//!
//! The plate is as wide as `WIDTH`, or the window less `MARGIN` on each side.
//! A plate taller than the window scrolls; a bar shows only then.

use egui::{Id, Key, Label, Modal, RichText, ScrollArea, vec2};
use rust_i18n::t;

use crate::app::App;
use crate::keys::{Binding, GLOBAL, Layer, Row, keymap, plain, rows, written_keys};
use crate::ui::access::Named;

/// The widest the plate grows.
const WIDTH: f32 = 520.0;
/// The room kept between the plate and the edge of the window.
const MARGIN: f32 = 16.0;
/// The width of the column of the keys.
const KEYS_WIDTH: f32 = 150.0;

/// What a key of the plate does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HelpAction {
    Close,
}

/// The keys of the plate (`Layer::Help`); F1, a global key, closes it too.
pub(crate) const KEYS: &[Binding<HelpAction>] = &[Binding {
    keys: &[plain(Key::Escape)],
    action: HelpAction::Close,
    name: "keys.close_help",
}];

/// Shows the keys of `on_top`, or hides the plate when it is shown.
pub fn toggle(app: &mut App, on_top: Layer) {
    app.help = match app.help {
        Some(_) => None,
        None => Some(on_top),
    };
}

/// The keys of the plate, while it is shown, before any layer under it reads
/// the input: the keys of the plate act, and every other key and the text
/// typed are taken away.
pub fn keys(app: &mut App, context: &egui::Context) {
    if app.help.is_none() {
        return;
    }
    for taken in crate::keys::take(context, KEYS) {
        match taken.action {
            HelpAction::Close => app.help = None,
        }
    }
    context.input_mut(|input| {
        input
            .events
            .retain(|event| !matches!(event, egui::Event::Key { .. } | egui::Event::Text(_)));
    });
}

/// Draws the plate while it is shown, over everything; a tap on it or beside
/// it closes it.
pub fn show(app: &mut App, context: &egui::Context) {
    let Some(under) = app.help else {
        return;
    };
    let window = context.content_rect();
    let width = WIDTH.min(window.width() - 2.0 * MARGIN).max(0.0);
    let modal = Modal::new(Id::unique("keys-help")).show(context, |ui| {
        ui.set_width(width);
        ui.heading(t!("keys.title"));
        ui.add_space(4.0);
        let room = window.height() - 2.0 * MARGIN - ui.min_rect().height() - 2.0 * MARGIN;
        ScrollArea::vertical()
            .max_height(room.max(0.0))
            .auto_shrink([false, true])
            .show(ui, |ui| {
                section(ui, &t!(under.title()), &keymap(under));
                ui.add_space(8.0);
                section(ui, &t!("keys.global"), &rows(GLOBAL));
            });
    });
    context.move_to_top(modal.response.layer_id);
    modal.response.named(&t!("keys.title"));
    if context.input(|input| input.pointer.any_click()) {
        app.help = None;
    }
}

/// A heading, then a row of each key and what it does.
fn section(ui: &mut egui::Ui, title: &str, rows: &[Row]) {
    ui.label(RichText::new(title).strong());
    if rows.is_empty() {
        ui.label(RichText::new(t!("keys.none")).weak());
        return;
    }
    for row in rows {
        ui.horizontal_top(|ui| {
            ui.allocate_ui(vec2(KEYS_WIDTH, 0.0), |ui| {
                ui.set_width(KEYS_WIDTH);
                ui.add(Label::new(RichText::new(written_keys(&row.keys)).monospace()).wrap());
            });
            ui.add(Label::new(t!(row.name)).wrap());
        });
    }
}
