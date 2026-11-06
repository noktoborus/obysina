//! The menu of the settings of a pattern, opened by the button that opened
//! the menu of the grid: the grid, the ruling and the colour of the paper.
//!
//! ```text
//! settings ──► PlateMenu, each entry with its value right of it:
//!                 grid <the kind in use> ›───► loom │ peyote, the one in use marked
//!                 ruling… <columns×rows> ────► its menu
//!                 colour of the paper <a swatch> ──► a menu: the picker and the
//!                                 swatches on top, then accept and cancel;
//!                                 the field changes at once
//! ```
//!
//! The shadows, the ruling and the shade around the paper turn with its
//! lightness: darker than a light paper, lighter than a dark one.

use std::sync::{Arc, Mutex};

use egui::{Color32, Rect, Sense, Vec2, vec2};
use obusina_button::SquareButton;
use plate_menu::{Detail, MenuItem, Notice, PlateMenu};

use super::{BUTTON, Lang, Mockup, named};

/// The paper of a new pattern, `PAPER` of the canvas.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The swatches offered under the picker: the paper of a new pattern,
/// white, linen and black.
const SWATCHES: [(&str, &str, Color32); 4] = [
    ("grey", "серый", PAPER),
    ("white", "белый", Color32::from_rgb(0xfa, 0xfa, 0xf7)),
    ("linen", "лён", Color32::from_rgb(0xe6, 0xdc, 0xc8)),
    ("black", "чёрный", Color32::from_rgb(0x1e, 0x1e, 0x1e)),
];
/// The made up field: its columns and rows.
const FIELD_SIZE: (i32, i32) = (30, 18);
/// The height of a bead of the made up field.
const BEAD: f32 = 18.0;
/// The width of a bead to its height.
const RATIO: f32 = 0.8;
/// The shade around the paper: the columns and rows it reaches past it.
const SHADE: i32 = 3;
/// A line of the ruling every so many columns and rows.
const RULING: i32 = 5;
/// The size of a swatch under the picker.
const SWATCH: Vec2 = vec2(44.0, 30.0);
/// The icon of the button of the settings.
const ICON: &str = "\u{2699}";

/// The kinds of grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grid {
    Loom,
    Peyote,
}

/// The settings of a made up pattern, and its field.
pub struct Settings {
    grid: Grid,
    paper: Color32,
    /// The paper while its menu is open, which the picker changes.
    editing: Arc<Mutex<Color32>>,
    /// The paper before its menu opened, put back by "cancel".
    before: Option<Color32>,
    menu: PlateMenu,
    /// What the last choice did.
    done: String,
}

impl Settings {
    /// A pattern of loom on the paper of a new one.
    pub fn new() -> Settings {
        Settings {
            grid: Grid::Loom,
            paper: PAPER,
            editing: Arc::new(Mutex::new(PAPER)),
            before: None,
            menu: PlateMenu::new().with_id("settings-menu"),
            done: String::new(),
        }
    }

    /// Opens the menu of the settings.
    fn open(&mut self, lang: Lang) {
        let mark = |grid: Grid| if self.grid == grid { "\u{2714}" } else { "" };
        let grid = match self.grid {
            Grid::Loom => lang.tr("Loom", "Станок"),
            Grid::Peyote => lang.tr("Peyote", "Мозаика"),
        };
        let paper = self.paper;
        let swatch = Detail::widget(SWATCH.x, move |ui: &mut egui::Ui| {
            let size = vec2(SWATCH.x, ui.available_height());
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect_filled(rect, 3.0, paper);
            ui.painter().rect_stroke(
                rect,
                3.0,
                ui.visuals().widgets.noninteractive.bg_stroke,
                egui::StrokeKind::Inside,
            );
        });
        let items = vec![
            MenuItem::new("grid", lang.tr("Grid", "Сетка"))
                .detail(grid)
                .children(vec![
                    MenuItem::new("loom", lang.tr("Loom", "Станок")).detail(mark(Grid::Loom)),
                    MenuItem::new("peyote", lang.tr("Peyote", "Мозаика"))
                        .detail(mark(Grid::Peyote)),
                ]),
            MenuItem::new("ruling", lang.tr("Ruling…", "Разметка…"))
                .detail(format!("{RULING}\u{d7}{RULING}")),
            MenuItem::new("paper", lang.tr("Colour of the paper", "Цвет фона")).detail(swatch),
        ];
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of the settings does not open: {error}");
        }
    }

    /// Opens the menu of the colour of the paper: the picker and the
    /// swatches on top, then accept and cancel.
    fn open_paper(&mut self, lang: Lang) {
        let items = vec![
            MenuItem::new("paper-accept", lang.tr("Accept", "Принять")),
            MenuItem::new("paper-cancel", lang.tr("Cancel", "Отмена")),
        ];
        if self.menu.open(items).is_err() {
            return;
        }
        *self.editing.lock().expect("the paper") = self.paper;
        self.before = Some(self.paper);
        let editing = Arc::clone(&self.editing);
        let title = lang.tr("Colour of the paper", "Цвет фона").to_owned();
        let names: Vec<String> = SWATCHES
            .iter()
            .map(|(en, ru, _)| lang.tr(en, ru).to_owned())
            .collect();
        self.menu.notice(Notice::widget(move |ui| {
            let mut paper = editing.lock().expect("the paper");
            ui.label(egui::RichText::new(title.as_str()).strong());
            ui.spacing_mut().slider_width = ui.available_width();
            egui::color_picker::color_picker_color32(
                ui,
                &mut paper,
                egui::color_picker::Alpha::Opaque,
            );
            ui.horizontal(|ui| {
                for ((_, _, colour), name) in SWATCHES.iter().zip(&names) {
                    let button = SquareButton::new("", SWATCH)
                        .fill(*colour)
                        .selected(*paper == *colour)
                        .name(name);
                    if ui.add(button).clicked() {
                        *paper = *colour;
                    }
                }
            });
        }));
    }

    /// Does what the menu chose; while the menu of the paper is open, the
    /// field shows the colour of its picker.
    fn menu(&mut self, context: &egui::Context, lang: Lang) {
        let chosen = self.menu.show(context);
        if self.before.is_some() {
            self.paper = *self.editing.lock().expect("the paper");
        }
        let Some(chosen) = chosen else {
            if !self.menu.is_open()
                && let Some(before) = self.before.take()
            {
                self.paper = before;
            }
            return;
        };
        match chosen.id.as_str() {
            "loom" => self.grid = Grid::Loom,
            "peyote" => self.grid = Grid::Peyote,
            "ruling" => self.done = lang.tr("ruling: its menu", "разметка: её меню").to_owned(),
            "paper" => self.open_paper(lang),
            "paper-accept" => {
                self.before = None;
                self.done = lang.tr("the paper kept", "фон принят").to_owned();
            }
            "paper-cancel" => {
                if let Some(before) = self.before.take() {
                    self.paper = before;
                }
            }
            _ => {}
        }
    }

    /// The field: the shade, the paper, the shadows with the ruling, and a
    /// few beads; the shadows and the shade turn with the paper.
    fn field(&self, ui: &mut egui::Ui) {
        let (columns, rows) = FIELD_SIZE;
        let size = vec2(
            columns as f32 * BEAD * RATIO,
            rows as f32 * BEAD + BEAD / 2.0,
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(rect);
        let (shade, shadow, line) = around(self.paper);
        painter.rect_filled(rect, 0.0, shade);
        let paper = Rect::from_min_max(
            rect.min + vec2(SHADE as f32 * BEAD * RATIO, SHADE as f32 * BEAD),
            rect.max - vec2(SHADE as f32 * BEAD * RATIO, SHADE as f32 * BEAD),
        );
        painter.rect_filled(paper, 0.0, self.paper);
        let beads = [
            ((12, 7), Color32::from_rgb(0xc0, 0x30, 0x30)),
            ((13, 8), Color32::from_rgb(0xc0, 0x30, 0x30)),
            ((14, 7), Color32::from_rgb(0x2f, 0x5f, 0xb3)),
            ((13, 9), Color32::from_rgb(0xe8, 0xc5, 0x47)),
            ((15, 8), Color32::from_rgb(0x1b, 0x1b, 0x1b)),
            ((11, 8), Color32::from_rgb(0xf2, 0xef, 0xe6)),
        ];
        for column in 0..columns {
            for row in 0..rows {
                let lower = if self.grid == Grid::Peyote && column % 2 == 1 {
                    0.5
                } else {
                    0.0
                };
                let min = rect.min + vec2(column as f32 * RATIO, row as f32 + lower) * BEAD;
                let cell = Rect::from_min_size(min, vec2(RATIO, 1.0) * BEAD).shrink(BEAD * 0.05);
                let colour = beads
                    .iter()
                    .find(|(at, _)| *at == (column, row))
                    .map(|(_, colour)| *colour)
                    .unwrap_or(if column % RULING == 0 || row % RULING == 0 {
                        line
                    } else {
                        shadow
                    });
                painter.rect_filled(cell, BEAD * 0.3, colour);
            }
        }
        named(ui.ctx(), response, "field");
    }
}

/// The shade around a paper, and the shadows of the empty places and of the
/// lines of the ruling on it: darker than a light paper, lighter than a dark
/// one.
fn around(paper: Color32) -> (Color32, Color32, Color32) {
    let [red, green, blue, _] = paper.to_array();
    let light = (0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32) / 255.0;
    if light > 0.5 {
        (
            paper.lerp_to_gamma(Color32::BLACK, 0.14),
            Color32::from_black_alpha(18),
            Color32::from_black_alpha(36),
        )
    } else {
        (
            paper.lerp_to_gamma(Color32::WHITE, 0.10),
            Color32::from_white_alpha(14),
            Color32::from_white_alpha(30),
        )
    }
}

impl Mockup for Settings {
    fn name(&self) -> &'static str {
        "settings"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "grid": format!("{:?}", self.grid),
            "paper": format!("#{:02x}{:02x}{:02x}", self.paper.r(), self.paper.g(), self.paper.b()),
            "editing": self.before.is_some(),
        })
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            let button = SquareButton::new(ICON, BUTTON).name(lang.tr("Settings", "Настройки"));
            if ui.add(button).clicked() && !self.menu.is_open() {
                self.open(lang);
            }
            ui.label(lang.tr(
                "The button of the grid becomes the button of the settings.",
                "Кнопка сетки становится кнопкой настроек.",
            ));
        });
        self.field(ui);
        ui.label(&self.done);
        self.menu(ui.ctx(), lang);
    }
}
