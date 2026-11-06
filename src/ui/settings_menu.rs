//! The menu of the settings, a `PlateMenu` in the middle of the window: the
//! kind of grid, the ruling and the paper of the pattern, and the glow of
//! the beads. Each entry shows what it is now right of its label.
//!
//! ```text
//! settings ──► grid <the kind> ›──► loom | peyote ──► the kind, a step of undo
//!              ruling… <step>  ──► the four numbers on top, then
//!                                    │──accept──► the ruling stays, a step of undo
//!                                    └──cancel──► the ruling goes back
//!              paper… <swatch> ──► the picker and the swatches on top, then
//!                                    │──accept──► the paper stays, a step of undo
//!                                    └──cancel──► the paper goes back
//!              glow <on | off> ──► on or off, a setting of the program;
//!                                  the menu opens again on it
//! closed with no entry while a ruling or a paper is changed ──► it goes back
//! ```
//!
//! A number of the ruling and a colour of the paper show on the field at
//! once.

use std::sync::{Arc, Mutex};

use egui::Color32;
use obusina_pattern::{DEFAULT_PAPER, Grid, Rgb, Ruling};
use plate_menu::{Detail, MenuItem, Notice};
use rust_i18n::t;

use crate::app::App;
use crate::ui::access::Named;

/// The most columns or rows from one line of the ruling to the next.
pub const MAX_RULING_STEP: u32 = 100;
/// The farthest a line of the ruling is set from cell (0, 0).
pub const MAX_RULING_OFFSET: i32 = 1000;
/// The papers offered under the picker: the one of a new pattern, white,
/// linen and black, each with the key of its name.
const PAPERS: [(Rgb, &str); 4] = [
    (DEFAULT_PAPER, "paper.grey"),
    (Rgb([0xfa, 0xfa, 0xf7]), "paper.white"),
    (Rgb([0xe6, 0xdc, 0xc8]), "paper.linen"),
    (Rgb([0x1e, 0x1e, 0x1e]), "paper.black"),
];
/// The size of a swatch of a paper under the picker.
const SWATCH: egui::Vec2 = egui::vec2(44.0, 30.0);
/// The width of the swatch of the paper right of its entry.
const DETAIL_SWATCH: f32 = 40.0;

/// What the menu changes while it is open, put back when it is not
/// accepted.
pub enum Editing {
    /// The ruling: the one the menu opened on, and the one the numbers of the
    /// notice change.
    Ruling {
        before: Ruling,
        editing: Arc<Mutex<Ruling>>,
    },
    /// The paper: the one the menu opened on, and the one the picker
    /// changes.
    Paper {
        before: Rgb,
        editing: Arc<Mutex<Color32>>,
    },
}

/// Opens the menu, on its first entry.
pub fn open(app: &mut App) {
    open_on(app, "grid");
}

/// Opens the menu on the entry `id`.
fn open_on(app: &mut App, id: &str) {
    let grid = app.pattern.grid();
    let kinds = Grid::ALL
        .iter()
        .map(|kind| {
            let mark = if *kind == grid { "\u{2714}" } else { "" };
            MenuItem::new(kind_id(*kind), name(*kind)).detail(mark)
        })
        .collect();
    let [columns, rows] = app.pattern.ruling().step;
    let glow = match app.settings.glow() {
        true => t!("settings.on"),
        false => t!("settings.off"),
    };
    let items = vec![
        MenuItem::new("grid", t!("grid.kind"))
            .detail(name(grid))
            .opens_at(kind_id(grid))
            .children(kinds),
        MenuItem::new("ruling", t!("ruling.open")).detail(format!("{columns}\u{d7}{rows}")),
        MenuItem::new("paper", t!("paper.open")).detail(swatch(app.pattern.paper())),
        MenuItem::new("glow", t!("view.glow")).detail(glow),
    ];
    if let Err(error) = app.menu.open_at(items, id) {
        log::error!("the menu of the settings does not open: {error}");
    }
}

/// The swatch of the paper right of its entry.
fn swatch(paper: Rgb) -> Detail {
    let fill = colour(paper);
    Detail::widget(DETAIL_SWATCH, move |ui| {
        let size = egui::vec2(DETAIL_SWATCH, ui.available_height());
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter().rect_filled(rect, 3.0, fill);
        ui.painter()
            .rect_stroke(rect, 3.0, stroke, egui::StrokeKind::Inside);
    })
}

/// Opens the menu of the ruling: its numbers on top, then accept and cancel.
fn open_ruling(app: &mut App) {
    if !open_accept(app) {
        return;
    }
    let before = app.pattern.ruling();
    let editing = Arc::new(Mutex::new(before));
    app.menu.notice(numbers(Arc::clone(&editing)));
    app.settings_edit = Some(Editing::Ruling { before, editing });
}

/// Opens the menu of the paper: the picker and the swatches on top, then
/// accept and cancel.
fn open_paper(app: &mut App) {
    if !open_accept(app) {
        return;
    }
    let before = app.pattern.paper();
    let editing = Arc::new(Mutex::new(colour(before)));
    app.menu.notice(picker(Arc::clone(&editing)));
    app.settings_edit = Some(Editing::Paper { before, editing });
}

/// Opens the menu on accept and cancel; false when it does not open.
fn open_accept(app: &mut App) -> bool {
    let items = vec![
        MenuItem::new("accept", t!("ruling.accept")),
        MenuItem::new("cancel", t!("ruling.cancel")),
    ];
    match app.menu.open(items) {
        Ok(()) => true,
        Err(error) => {
            log::error!("the menu of accept and cancel does not open: {error}");
            false
        }
    }
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

/// The picker of the paper, as wide as the menu, and the swatches of
/// [`PAPERS`] under it.
fn picker(editing: Arc<Mutex<Color32>>) -> Notice {
    Notice::widget(move |ui| {
        let mut paper = editing.lock().expect("the paper is never poisoned");
        ui.label(egui::RichText::new(t!("paper.open")).strong());
        ui.spacing_mut().slider_width = ui.available_width();
        egui::color_picker::color_picker_color32(ui, &mut paper, egui::color_picker::Alpha::Opaque);
        ui.horizontal(|ui| {
            for (rgb, key) in PAPERS {
                let fill = colour(rgb);
                let name = t!(key);
                let button = obusina_button::SquareButton::new("", SWATCH)
                    .fill(fill)
                    .selected(*paper == fill)
                    .name(&name);
                if ui.add(button).clicked() {
                    *paper = fill;
                }
            }
        });
    })
}

/// Draws the menu while it is open, and does what is chosen. While nothing
/// else of the window answers (`src/ui/blocker.rs`), the ruling and the
/// paper show on the field as they change.
pub fn show(app: &mut App, context: &egui::Context) {
    app.menu.nothing_found(t!("menu.nothing_found"));
    if app.menu.is_open() {
        crate::ui::blocker::under(context, "settings-menu", &app.menu);
    }
    let chosen = app.menu.show(context);
    match &app.settings_edit {
        Some(Editing::Ruling { editing, .. }) => {
            let shown = *editing.lock().expect("the ruling is never poisoned");
            app.pattern.set_ruling(shown);
        }
        Some(Editing::Paper { editing, .. }) => {
            let shown = *editing.lock().expect("the paper is never poisoned");
            app.pattern.set_paper(rgb(shown));
        }
        None => {}
    }
    let Some(chosen) = chosen else {
        if !app.menu.is_open() {
            put_back(app);
        }
        return;
    };
    match chosen.id.as_str() {
        "accept" => match app.settings_edit.take() {
            Some(Editing::Ruling { before, .. }) => app.history.keep_ruling(&app.pattern, before),
            Some(Editing::Paper { before, .. }) => app.history.keep_paper(&app.pattern, before),
            None => {}
        },
        "cancel" => put_back(app),
        "ruling" => open_ruling(app),
        "paper" => open_paper(app),
        "glow" => {
            app.keep_glow(!app.settings.glow());
            open_on(app, "glow");
        }
        kind => {
            if let Some(grid) = Grid::ALL.into_iter().find(|grid| kind_id(*grid) == kind) {
                app.set_grid(grid);
            }
        }
    }
}

/// Puts back the ruling or the paper the menu opened on.
fn put_back(app: &mut App) {
    match app.settings_edit.take() {
        Some(Editing::Ruling { before, .. }) => app.pattern.set_ruling(before),
        Some(Editing::Paper { before, .. }) => app.pattern.set_paper(before),
        None => {}
    }
}

/// The identifier of the entry of a kind.
fn kind_id(grid: Grid) -> &'static str {
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

/// A colour of the pattern as the toolkit draws it.
fn colour(Rgb([red, green, blue]): Rgb) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

/// A colour of the toolkit as the pattern keeps it.
fn rgb(colour: Color32) -> Rgb {
    Rgb([colour.r(), colour.g(), colour.b()])
}
