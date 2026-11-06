//! The list of patterns: the screen the program opens on, which takes the
//! place of the field while it is shown; and the trash, a list of its own.
//!
//! ```text
//! start ──► the list, the last changed first
//!             │──the card "+ New pattern"──► a new pattern, named by its date, opened
//!             │──the card "Import"─────────► a copy of a file in the list, opened
//!             │──a tap on a card───────────► the pattern opened
//!             │──a long press or the second button on a card ──► PlateMenu:
//!             │       rename │ duplicate │ export to a file… │ select │ move to the trash
//!             │       rename: the name in a field on top, then accept, cancel
//!             │       duplicate: "<name> (copy)", first in the list
//!             │       move to the trash: a bar at the bottom for `UNDO_SECONDS`,
//!             │                          "Undo" puts it back
//!             │       select: a mark on the cards; a tap marks or unmarks;
//!             │               a bar on top: the count, export, to the trash, ✖
//!             └──the plate of the trash, after the cards ──► the trash
//! the trash ──► the plate "Back";
//!               its cards, each with the days it has left;
//!               the plate "Empty the trash…" after them
//!             │──"Back"─────────────────────► the list
//!             │──"Empty the trash…"─────────► asks; every pattern of it gone
//!             └──a tap or a long press on a card ──► PlateMenu:
//!                     restore │ delete for good… (asks)
//! a pattern in the trash for `TRASH_DAYS` ──► gone, when the list is opened
//! ```
//!
//! The list is in the order of the last change, the latest first. A picture
//! of a card keeps the sides of its sheet, with margins where it is narrower
//! than the card.
//!
//! The plate of the trash, and in the trash the plate that empties it, stand
//! at the bottom edge of the screen while the cards leave room above them;
//! with more cards they come after them.
//!
//! The search line shows only when the cards do not fit the screen; it
//! scrolls with them, above the first. The cards fill the width: as many
//! columns as cards of `CARD_LEAST` fit, each as wide as its share. The list
//! scrolls by its bar at the side of the screen, the wheel, a finger, or a
//! drag of the first button. The bar takes no width: it shows over the cards
//! while it is used, and no card changes its size for it.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use egui::{Color32, Rect, Sense, Vec2, pos2, vec2};
use plate_menu::{MenuItem, Notice, PlateMenu};

use super::{Lang, Mockup, named};

/// The least width of a card.
const CARD_LEAST: f32 = 150.0;
/// The height of the picture of a card to its width.
const THUMB_RATIO: f32 = 110.0 / 138.0;
/// The margin of a card around its picture.
const CARD_MARGIN: f32 = 6.0;
/// The height of the lines of a card under its picture.
const CARD_TEXT: f32 = 62.0;
/// The space between two cards.
const GAP: f32 = 10.0;
/// The height of a plate across the list.
const PLATE: f32 = 56.0;
/// The days a pattern stays in the trash.
const TRASH_DAYS: u32 = 30;
/// The ground of a picture, `PAPER` of the canvas.
const PAPER: Color32 = Color32::from_rgb(0xd6, 0xd4, 0xce);
/// The icon of the trash.
const TRASH: &str = "\u{1f5d1}";
/// The icon of the import.
const IMPORT: &str = "\u{1f4e5}";
/// The icon of the way back, the arrow to the left.
const BACK: &str = "\u{2b05}";
/// The icon of the export.
const EXPORT: &str = "\u{1f4e4}";
/// The mark of a card chosen, and the end of the choice.
const MARK: &str = "\u{2714}";
/// The end of the choice of cards.
const CLOSE: &str = "\u{2716}";
/// The seconds the bar that undoes a move to the trash stays.
const UNDO_SECONDS: f64 = 5.0;
/// The width of a bead to its height, in the made up patterns.
const RATIO: f32 = 0.8;
/// The screens the list is shown at: a phone held upright, a window of the
/// desktop.
const SCREENS: [(&str, &str, Vec2); 2] = [
    ("Phone", "Телефон", vec2(360.0, 640.0)),
    ("Desktop", "Десктоп", vec2(980.0, 600.0)),
];

/// A made up pattern of the list.
#[derive(Clone)]
struct Project {
    name: String,
    /// The columns and rows of its sheet.
    size: (usize, usize),
    beads: usize,
    /// When it was last changed, as the list says it.
    changed: (&'static str, &'static str),
    /// The seed of its made up picture.
    seed: u32,
}

/// Which list is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    List,
    Trash,
}

/// What the menu is of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Of {
    /// A pattern of the list, by its index.
    Live(usize),
    /// A pattern of the trash, by its index.
    Trashed(usize),
    /// The whole trash.
    AllTrash,
}

/// The list of patterns, at the width of a phone or of the desktop.
pub struct Projects {
    projects: Vec<Project>,
    /// The patterns in the trash, each with the days it has left.
    trash: Vec<(Project, u32)>,
    /// How many patterns of the list are shown.
    count: usize,
    screen: usize,
    view: View,
    query: String,
    menu: PlateMenu,
    menu_of: Option<Of>,
    /// The name being written in the menu of a rename.
    renaming: Arc<Mutex<String>>,
    /// The bar that undoes the last move to the trash, and the time it goes.
    undo: Option<(String, f64)>,
    /// The cards chosen, while cards are being chosen.
    chosen: Option<BTreeSet<usize>>,
    /// What the last tap did, said under the screen.
    done: String,
}

impl Projects {
    /// The list with two dozen made up patterns, eight of them shown, and
    /// three in the trash.
    pub fn new() -> Projects {
        let changed = [
            ("today 14:20", "сегодня 14:20"),
            ("today 09:05", "сегодня 09:05"),
            ("yesterday", "вчера"),
            ("3 Nov", "3 ноя"),
            ("28 Oct", "28 окт"),
            ("12 Oct", "12 окт"),
        ];
        let sizes = [(12, 44), (60, 12)];
        let made = |index: usize| Project {
            name: format!(
                "2026-11-{:02} {:02}:{:02}",
                6 - (index % 6),
                9 + index % 9,
                (index * 7) % 60
            ),
            size: sizes
                .get(index.wrapping_sub(1))
                .copied()
                .unwrap_or((20 + (index * 7) % 40, 14 + (index * 5) % 30)),
            beads: 120 + (index * 97) % 900,
            changed: changed[index % changed.len()],
            seed: (index as u32).wrapping_mul(2_654_435_761),
        };
        Projects {
            projects: (0..24).map(made).collect(),
            trash: [(30, 29), (31, 12), (32, 1)]
                .into_iter()
                .map(|(index, days)| (made(index), days))
                .collect(),
            count: 8,
            screen: 0,
            view: View::List,
            query: String::new(),
            menu: PlateMenu::new().with_id("projects-menu"),
            menu_of: None,
            renaming: Arc::new(Mutex::new(String::new())),
            undo: None,
            chosen: None,
            done: String::new(),
        }
    }

    /// Opens a menu of `items` about `of`, `notice` on top.
    fn open(&mut self, of: Of, notice: String, items: Vec<MenuItem>) {
        if let Err(error) = self.menu.open(items) {
            log::error!("the menu of the list does not open: {error}");
            return;
        }
        self.menu.notice(notice);
        self.menu_of = Some(of);
    }

    /// Opens the menu of a card of the list or of the trash, or the question
    /// before the trash is emptied.
    fn open_menu(&mut self, of: Of, lang: Lang) {
        match of {
            Of::Live(index) => {
                let items = vec![
                    MenuItem::new("rename", lang.tr("Rename", "Переименовать")),
                    MenuItem::new("duplicate", lang.tr("Duplicate", "Дублировать")),
                    MenuItem::new("export", lang.tr("Export to a file…", "Экспорт в файл…")),
                    MenuItem::new("select", lang.tr("Select", "Выбрать")),
                    MenuItem::new(
                        "trash",
                        lang.tr("Move to the trash", "Переместить в корзину"),
                    ),
                ];
                self.open(of, self.projects[index].name.clone(), items);
            }
            Of::Trashed(index) => {
                let items = vec![
                    MenuItem::new("restore", lang.tr("Restore", "Восстановить")),
                    MenuItem::new("purge", lang.tr("Delete for good…", "Удалить навсегда…")),
                ];
                self.open(of, self.trash[index].0.name.clone(), items);
            }
            Of::AllTrash => {
                let items = vec![
                    MenuItem::new("empty-yes", lang.tr("Empty", "Очистить")),
                    MenuItem::new("keep", lang.tr("Keep", "Оставить")),
                ];
                let notice = format!(
                    "{} {}?",
                    lang.tr(
                        "Delete for good the patterns of the trash:",
                        "Удалить навсегда схемы из корзины:"
                    ),
                    self.trash.len()
                );
                self.open(of, notice, items);
            }
        }
    }

    /// Does what the menu chose.
    fn menu(&mut self, context: &egui::Context, lang: Lang) {
        let time = context.input(|input| input.time);
        let Some(chosen) = self.menu.show(context) else {
            return;
        };
        let Some(of) = self.menu_of else {
            return;
        };
        match (chosen.id.as_str(), of) {
            ("trash", Of::Live(index)) => {
                self.move_to_trash(&[index], time);
                self.menu_of = None;
            }
            ("rename", Of::Live(index)) => {
                *self.renaming.lock().expect("the name") = self.projects[index].name.clone();
                let items = vec![
                    MenuItem::new("rename-accept", lang.tr("Accept", "Принять")),
                    MenuItem::new("keep", lang.tr("Cancel", "Отмена")),
                ];
                let renaming = Arc::clone(&self.renaming);
                let label = lang.tr("New name", "Новое имя").to_owned();
                if self.menu.open(items).is_ok() {
                    self.menu.notice(Notice::widget(move |ui| {
                        let mut name = renaming.lock().expect("the name");
                        let line = ui.text_style_height(&egui::TextStyle::Body);
                        let label = ui.label(label.as_str());
                        let field = egui::TextEdit::singleline(&mut *name)
                            .margin(vec2(line, line))
                            .desired_width(f32::INFINITY);
                        let response = ui.add(field).labelled_by(label.id);
                        if !response.has_focus() && !response.lost_focus() {
                            response.request_focus();
                        }
                        named(ui.ctx(), response, "name");
                    }));
                    self.menu_of = Some(of);
                }
            }
            ("rename-accept", Of::Live(index)) => {
                let mut project = self.projects.remove(index);
                project.name = self.renaming.lock().expect("the name").clone();
                project.changed = ("just now", "только что");
                self.done = format!("{} «{}»", lang.tr("renamed", "переименована"), project.name);
                self.projects.insert(0, project);
                self.menu_of = None;
            }
            ("duplicate", Of::Live(index)) => {
                let mut copy = self.projects[index].clone();
                copy.name = format!("{} ({})", copy.name, lang.tr("copy", "копия"));
                copy.changed = ("just now", "только что");
                self.done = format!("«{}»", copy.name);
                self.projects.insert(0, copy);
                self.count += 1;
                self.menu_of = None;
            }
            ("select", Of::Live(index)) => {
                self.chosen = Some(BTreeSet::from([index]));
                self.menu_of = None;
            }
            ("restore", Of::Trashed(index)) => {
                let (project, _) = self.trash.remove(index);
                self.done = format!("«{}» {}", project.name, lang.tr("is back", "восстановлена"));
                self.projects.insert(0, project);
                self.count += 1;
                self.menu_of = None;
                if self.trash.is_empty() {
                    self.view = View::List;
                }
            }
            ("purge", Of::Trashed(index)) => {
                let items = vec![
                    MenuItem::new("purge-yes", lang.tr("Delete for good", "Удалить навсегда")),
                    MenuItem::new("keep", lang.tr("Keep", "Оставить")),
                ];
                let notice = format!(
                    "{} «{}»?",
                    lang.tr("Delete for good", "Удалить навсегда"),
                    self.trash[index].0.name
                );
                self.open(of, notice, items);
            }
            ("purge-yes", Of::Trashed(index)) => {
                let (project, _) = self.trash.remove(index);
                self.done = format!("«{}» {}", project.name, lang.tr("is gone", "удалена"));
                self.menu_of = None;
                if self.trash.is_empty() {
                    self.view = View::List;
                }
            }
            ("empty-yes", Of::AllTrash) => {
                self.trash.clear();
                self.view = View::List;
                self.menu_of = None;
                self.done = lang.tr("the trash is empty", "корзина пуста").to_owned();
            }
            ("keep", _) => self.menu_of = None,
            (other, Of::Live(index)) => {
                self.done = format!("{other}: «{}»", self.projects[index].name);
            }
            _ => {}
        }
    }

    /// Moves the patterns of `indices` to the trash, and shows the bar that
    /// undoes it until `UNDO_SECONDS` past `time`.
    fn move_to_trash(&mut self, indices: &[usize], time: f64) {
        let mut indices = indices.to_vec();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        let mut names = Vec::new();
        for index in indices {
            let project = self.projects.remove(index);
            self.count = self.count.saturating_sub(1).min(self.projects.len());
            names.push(project.name.clone());
            self.trash.insert(0, (project, TRASH_DAYS));
        }
        let said = match names.as_slice() {
            [one] => format!("«{one}»"),
            many => many.len().to_string(),
        };
        self.undo = Some((said, time + UNDO_SECONDS));
    }

    /// The bar at the bottom of the screen that undoes the last move to the
    /// trash, while its time lasts.
    fn undo_bar(&mut self, ui: &mut egui::Ui, lang: Lang, screen: Rect) {
        let time = ui.input(|input| input.time);
        let Some((said, until)) = self.undo.clone() else {
            return;
        };
        if time > until {
            self.undo = None;
            return;
        }
        ui.ctx().request_repaint();
        let bar = Rect::from_min_max(
            pos2(screen.left() + 8.0, screen.bottom() - 8.0 - PLATE),
            pos2(screen.right() - 8.0, screen.bottom() - 8.0),
        );
        let painter = ui.painter();
        painter.rect_filled(bar, 6.0, ui.visuals().strong_text_color());
        painter.text(
            bar.left_center() + vec2(16.0, 0.0),
            egui::Align2::LEFT_CENTER,
            format!("{said} {}", lang.tr("in the trash", "в корзине")),
            egui::FontId::proportional(14.0),
            ui.visuals().panel_fill,
        );
        let button =
            Rect::from_center_size(bar.right_center() - vec2(60.0, 0.0), vec2(100.0, 36.0));
        let response = ui.interact(button, egui::Id::unique("undo-trash"), Sense::click());
        painter.text(
            button.center(),
            egui::Align2::CENTER_CENTER,
            lang.tr("Undo", "Отменить"),
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0x00, 0x7a, 0xa8),
        );
        named(ui.ctx(), response.clone(), "undo");
        if response.clicked() {
            let back = self
                .trash
                .iter()
                .take_while(|(_, days)| *days == TRASH_DAYS)
                .count();
            let first = self.trash.len().min(back.max(1));
            for (project, _) in self.trash.drain(..first).rev() {
                self.projects.insert(0, project);
                self.count += 1;
            }
            self.undo = None;
            self.done = lang.tr("put back", "возвращено").to_owned();
        }
    }

    /// The bar on top of the screen while cards are chosen: their count,
    /// export, to the trash, and the end of the choice.
    fn choice_bar(&mut self, ui: &mut egui::Ui, lang: Lang, width: f32) {
        let Some(chosen) = self.chosen.clone() else {
            return;
        };
        let time = ui.input(|input| input.time);
        ui.allocate_ui(vec2(width, PLATE), |ui| {
            ui.set_min_size(vec2(width, PLATE));
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_min_width(width - 16.0);
                    ui.horizontal_centered(|ui| {
                        let close = ui.add(egui::Button::new(CLOSE).min_size(vec2(36.0, 36.0)));
                        named(ui.ctx(), close.clone(), "end the choice");
                        if close.clicked() {
                            self.chosen = None;
                        }
                        ui.label(format!(
                            "{}: {}",
                            lang.tr("Chosen", "Выбрано"),
                            chosen.len()
                        ));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let trash = ui.add(egui::Button::new(TRASH).min_size(vec2(36.0, 36.0)));
                            named(ui.ctx(), trash.clone(), "to the trash");
                            let export =
                                ui.add(egui::Button::new(EXPORT).min_size(vec2(36.0, 36.0)));
                            named(ui.ctx(), export.clone(), "export");
                            if trash.clicked() && !chosen.is_empty() {
                                let indices: Vec<usize> = chosen.iter().copied().collect();
                                self.move_to_trash(&indices, time);
                                self.chosen = None;
                            }
                            if export.clicked() {
                                self.done = format!(
                                    "{}: {}",
                                    lang.tr("export of", "экспорт схем"),
                                    chosen.len()
                                );
                            }
                        });
                    });
                });
        });
    }

    /// The search line, by its name, with room above and below its letters.
    fn search(&mut self, ui: &mut egui::Ui, lang: Lang) {
        let line = ui.text_style_height(&egui::TextStyle::Body);
        let field = egui::TextEdit::singleline(&mut self.query)
            .hint_text(lang.tr("Search by name", "Поиск по имени"))
            .margin(vec2(line, line))
            .desired_width(f32::INFINITY);
        let response = ui.add(field);
        named(ui.ctx(), response, "search");
    }

    /// One card: its picture, its name, size and beads, and `last`: when it
    /// changed, or the days it has left in the trash.
    fn card(
        ui: &mut egui::Ui,
        lang: Lang,
        project: &Project,
        last: &str,
        size: Vec2,
        mark: Option<bool>,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let painter = ui.painter_at(rect);
        let visuals = ui.style().interact(&response);
        painter.rect_filled(rect, 6.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(rect, 6.0, visuals.bg_stroke, egui::StrokeKind::Inside);
        let inner = size.x - 2.0 * CARD_MARGIN;
        let thumb = Rect::from_min_size(
            rect.min + Vec2::splat(CARD_MARGIN),
            vec2(inner, inner * THUMB_RATIO),
        );
        draw_thumb(&painter, thumb, project);
        let text = ui.visuals().text_color();
        let weak = ui.visuals().weak_text_color();
        let mut line = thumb.bottom() + 6.0;
        for (words, colour, size) in [
            (project.name.clone(), text, 13.0),
            (
                format!(
                    "{}×{} · {} {}",
                    project.size.0,
                    project.size.1,
                    project.beads,
                    lang.tr("beads", "бусин")
                ),
                weak,
                11.0,
            ),
            (last.to_owned(), weak, 11.0),
        ] {
            painter.text(
                pos2(rect.left() + 8.0, line),
                egui::Align2::LEFT_TOP,
                words,
                egui::FontId::proportional(size),
                colour,
            );
            line += size + 5.0;
        }
        if let Some(chosen) = mark {
            let at = pos2(rect.right() - 20.0, rect.top() + 20.0);
            let accent = Color32::from_rgb(0x00, 0x5c, 0x80);
            if chosen {
                painter.circle_filled(at, 13.0, accent);
                painter.text(
                    at,
                    egui::Align2::CENTER_CENTER,
                    MARK,
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            } else {
                painter.circle(
                    at,
                    13.0,
                    Color32::from_black_alpha(90),
                    egui::Stroke::new(2.0, Color32::WHITE),
                );
            }
        }
        named(ui.ctx(), response.clone(), &project.name);
        response
    }

    /// A card that does something rather than open a pattern: its icon
    /// over its words.
    fn action_card(ui: &mut egui::Ui, size: Vec2, icon: &str, words: &str) -> bool {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let visuals = ui.style().interact(&response);
        let painter = ui.painter_at(rect);
        painter.rect_stroke(rect, 6.0, visuals.fg_stroke, egui::StrokeKind::Inside);
        painter.text(
            rect.center() - vec2(0.0, 14.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(36.0),
            visuals.fg_stroke.color,
        );
        painter.text(
            rect.center() + vec2(0.0, 26.0),
            egui::Align2::CENTER_CENTER,
            words,
            egui::FontId::proportional(14.0),
            visuals.fg_stroke.color,
        );
        named(ui.ctx(), response.clone(), words);
        response.clicked()
    }

    /// A plate across the list: an icon, then its words. True when it is
    /// tapped.
    fn plate(ui: &mut egui::Ui, width: f32, icon: &str, words: &str) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(width, PLATE), Sense::click());
        let visuals = ui.style().interact(&response);
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
        painter.rect_stroke(rect, 6.0, visuals.bg_stroke, egui::StrokeKind::Inside);
        painter.text(
            rect.left_center() + vec2(16.0, 0.0),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(24.0),
            visuals.fg_stroke.color,
        );
        painter.text(
            rect.left_center() + vec2(56.0, 0.0),
            egui::Align2::LEFT_CENTER,
            words,
            egui::FontId::proportional(14.0),
            visuals.fg_stroke.color,
        );
        named(ui.ctx(), response.clone(), words);
        response.clicked()
    }

    /// The list: the cards of the actions, then the patterns that match the
    /// query, then the plate of the trash. When the cards leave room on the
    /// screen, `visible` high, the plate stands at its bottom edge.
    fn list(
        &mut self,
        ui: &mut egui::Ui,
        lang: Lang,
        (columns, card, width): (usize, Vec2, f32),
        visible: f32,
    ) {
        let top = ui.min_rect().top();
        let query = self.query.to_lowercase();
        let shown: Vec<usize> = (0..self.count)
            .filter(|index| {
                query.is_empty() || self.projects[*index].name.to_lowercase().contains(&query)
            })
            .collect();
        let mut cells: Vec<Option<usize>> = vec![None, None];
        cells.extend(shown.into_iter().map(Some));
        for (row, line) in cells.chunks(columns).enumerate() {
            ui.horizontal(|ui| {
                for (place, cell) in line.iter().enumerate() {
                    match cell {
                        None if row == 0 && place == 0 => {
                            let words = lang.tr("New pattern", "Новая схема");
                            if Self::action_card(ui, card, "+", words) {
                                self.done = lang
                                    .tr(
                                        "a new pattern, named by its date, opened",
                                        "новая схема с датой в имени, открыта",
                                    )
                                    .to_owned();
                            }
                        }
                        None => {
                            let words = lang.tr("Import", "Импортировать");
                            if Self::action_card(ui, card, IMPORT, words) {
                                self.done = lang
                                    .tr("import: the choice of a file", "импорт: выбор файла")
                                    .to_owned();
                            }
                        }
                        Some(index) => {
                            let project = &self.projects[*index];
                            let last = lang.tr(project.changed.0, project.changed.1);
                            let mark = self.chosen.as_ref().map(|chosen| chosen.contains(index));
                            let response = Self::card(ui, lang, project, last, card, mark);
                            if let Some(chosen) = &mut self.chosen {
                                if (response.clicked() || response.long_touched())
                                    && !chosen.remove(index)
                                {
                                    chosen.insert(*index);
                                }
                            } else {
                                if response.clicked() {
                                    self.done = format!(
                                        "{} «{}»",
                                        lang.tr("opened", "открыта"),
                                        project.name
                                    );
                                }
                                if response.secondary_clicked() || response.long_touched() {
                                    self.open_menu(Of::Live(*index), lang);
                                }
                            }
                        }
                    }
                }
            });
        }
        if !self.trash.is_empty() {
            to_the_bottom(ui, top, visible);
            let count = self.trash.len();
            let words = format!(
                "{} · {count} {}",
                lang.tr("Trash", "Корзина"),
                lang.tr(
                    if count == 1 { "pattern" } else { "patterns" },
                    russian(count, ["схема", "схемы", "схем"]),
                ),
            );
            if Self::plate(ui, width, TRASH, &words) {
                self.view = View::Trash;
                self.query.clear();
            }
        }
    }

    /// The trash: the plate back to the list; its patterns, each with the
    /// days it has left; the plate that empties it, at the bottom edge of the
    /// screen, `visible` high, while the cards leave room above it.
    fn trash(
        &mut self,
        ui: &mut egui::Ui,
        lang: Lang,
        (columns, card, width): (usize, Vec2, f32),
        visible: f32,
    ) {
        let top = ui.min_rect().top();
        if Self::plate(ui, width, BACK, lang.tr("Back", "Вернуться")) {
            self.view = View::List;
        }
        let indices: Vec<usize> = (0..self.trash.len()).collect();
        for line in indices.chunks(columns) {
            ui.horizontal(|ui| {
                for index in line {
                    let (project, days) = &self.trash[*index];
                    let last = format!(
                        "{} {days} {}",
                        lang.tr("gone in", "удалится через"),
                        lang.tr(
                            if *days == 1 { "day" } else { "days" },
                            russian(*days as usize, ["день", "дня", "дней"]),
                        )
                    );
                    let response = Self::card(ui, lang, project, &last, card, None);
                    if response.clicked() || response.secondary_clicked() || response.long_touched()
                    {
                        self.open_menu(Of::Trashed(*index), lang);
                    }
                }
            });
        }
        to_the_bottom(ui, top, visible);
        let words = lang.tr("Empty the trash…", "Очистить корзину…");
        if Self::plate(ui, width, TRASH, words) {
            self.open_menu(Of::AllTrash, lang);
        }
    }

    /// The screen: the list or the trash, scrolled, the search on top when
    /// the cards of the list do not fit.
    fn screen(&mut self, ui: &mut egui::Ui, lang: Lang, size: Vec2) {
        let margin = 8.0;
        let width = size.x - 2.0 * margin;
        let columns = (((width + GAP) / (CARD_LEAST + GAP)).floor() as usize).max(1);
        let card_width = (width - GAP * (columns - 1) as f32) / columns as f32;
        let card = vec2(
            card_width,
            CARD_MARGIN + (card_width - 2.0 * CARD_MARGIN) * THUMB_RATIO + CARD_TEXT,
        );
        let needed = (self.count + 2).div_ceil(columns) as f32 * (card.y + GAP) + PLATE;
        let search = self.view == View::List && needed > size.y - 2.0 * margin;
        let (screen, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter()
            .rect_filled(screen, 0.0, ui.visuals().panel_fill);
        ui.painter().rect_stroke(
            screen,
            0.0,
            ui.visuals().window_stroke,
            egui::StrokeKind::Outside,
        );
        let mut visible = size.y - 2.0 * margin;
        let mut inside = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(screen.shrink(margin))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        inside.spacing_mut().scroll = egui::style::ScrollStyle::floating();
        if self.chosen.is_some() && self.view == View::List {
            self.choice_bar(&mut inside, lang, width);
            inside.add_space(GAP);
            visible -= PLATE + GAP;
        }
        egui::ScrollArea::vertical()
            .id_salt(self.view == View::Trash)
            .auto_shrink(false)
            .scroll_source(egui::scroll_area::ScrollSource::ALL)
            .show(&mut inside, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(GAP);
                if search {
                    ui.allocate_ui(vec2(width, 0.0), |ui| self.search(ui, lang));
                }
                match self.view {
                    View::List => self.list(ui, lang, (columns, card, width), visible),
                    View::Trash => self.trash(ui, lang, (columns, card, width), visible),
                }
            });
        self.undo_bar(ui, lang, screen);
    }
}

/// Leaves room before a plate, so that it stands at the bottom edge of the
/// screen, `visible` high, when what came since `top` leaves room for it.
fn to_the_bottom(ui: &mut egui::Ui, top: f32, visible: f32) {
    let room = visible - (ui.cursor().top() - top) - PLATE;
    if room > 0.0 {
        ui.add_space(room);
    }
}

/// The Russian form of a word after the number `count`: one, a few, many.
fn russian(count: usize, forms: [&'static str; 3]) -> &'static str {
    let (tens, ones) = (count % 100, count % 10);
    if (11..=14).contains(&tens) {
        forms[2]
    } else if ones == 1 {
        forms[0]
    } else if (2..=4).contains(&ones) {
        forms[1]
    } else {
        forms[2]
    }
}

/// A made up picture of a pattern: its sheet with the sides it has, as
/// large as fits `rect`, margins where it is narrower; on it beads of a few
/// colours, in a shape the seed chooses.
fn draw_thumb(painter: &egui::Painter, rect: Rect, project: &Project) {
    let (columns, rows) = project.size;
    let sheet = vec2(columns as f32 * RATIO, rows as f32);
    let scale = (rect.width() / sheet.x).min(rect.height() / sheet.y);
    let paper = Rect::from_center_size(rect.center(), sheet * scale);
    painter.rect_filled(paper, 2.0, PAPER);
    let colours = [
        Color32::from_rgb(0x1b, 0x1b, 0x1b),
        Color32::from_rgb(0xc0, 0x30, 0x30),
        Color32::from_rgb(0x2f, 0x5f, 0xb3),
        Color32::from_rgb(0xe8, 0xc5, 0x47),
        Color32::from_rgb(0xf2, 0xef, 0xe6),
    ];
    let bead = vec2(RATIO, 1.0) * scale;
    let reach_unit = sheet.x.min(sheet.y) / 2.0 / 8.0;
    let seed = project.seed;
    for row in 0..rows {
        for column in 0..columns {
            let at =
                (vec2((column as f32 + 0.5) * RATIO, row as f32 + 0.5) - sheet / 2.0) / reach_unit;
            let angle = at.y.atan2(at.x);
            let lobes = 3 + seed % 4;
            let reach = 5.0 + 2.5 * (lobes as f32 * angle + seed as f32 * 0.001).cos();
            if at.length() > reach {
                continue;
            }
            let ring = (at.length() / 2.0) as usize + (seed as usize % 3);
            let min = paper.min + vec2(column as f32 * bead.x, row as f32 * bead.y);
            let cell = Rect::from_min_size(min, bead).shrink((bead.y * 0.08).min(0.6));
            painter.rect_filled(cell, bead.y * 0.3, colours[ring % colours.len()]);
        }
    }
}

impl Mockup for Projects {
    fn name(&self) -> &'static str {
        "projects"
    }

    fn state(&self) -> serde_json::Value {
        serde_json::json!({
            "screen": SCREENS[self.screen].0,
            "view": format!("{:?}", self.view),
            "count": self.count,
            "trash": self.trash.len(),
            "query": self.query,
            "menu_of": self.menu_of.map(|of| format!("{of:?}")),
            "chosen": self.chosen.as_ref().map(|chosen| chosen.len()),
            "undo": self.undo.as_ref().map(|(said, _)| said.clone()),
        })
    }

    fn show(&mut self, ui: &mut egui::Ui, lang: Lang) {
        ui.horizontal(|ui| {
            for (index, (en, ru, _)) in SCREENS.iter().enumerate() {
                ui.selectable_value(&mut self.screen, index, lang.tr(en, ru));
            }
            ui.separator();
            let most = self.projects.len();
            let slider = egui::Slider::new(&mut self.count, 0..=most)
                .show_value(true)
                .text(lang.tr("patterns", "схем"));
            ui.add(slider);
        });
        ui.label(lang.tr(
            "A long press or the second button on a card opens its menu.",
            "Долгое нажатие или правая кнопка на карточке открывает её меню.",
        ));
        let size = SCREENS[self.screen].2;
        self.screen(ui, lang, size);
        ui.label(&self.done);
        self.menu(ui.ctx(), lang);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    fn a_russian_word_takes_the_form_of_its_number() {
        let forms = ["день", "дня", "дней"];
        let said: Vec<&str> = [1, 2, 5, 11, 21, 22, 30]
            .into_iter()
            .map(|count| russian(count, forms))
            .collect();

        assert_eq!(said, ["день", "дня", "дней", "дней", "день", "дня", "дней"]);
    }

    /// The list and the trash draw at both screens in both languages: every
    /// glyph they use is carried by a font.
    #[test]
    fn the_list_and_the_trash_draw() {
        for view in [View::List, View::Trash] {
            for screen in 0..SCREENS.len() {
                for lang in Lang::ALL {
                    let mut mock = Projects::new();
                    mock.view = view;
                    mock.screen = screen;
                    let mut harness = Harness::new_ui(|ui| mock.show(ui, lang));
                    harness.run();
                }
            }
        }
    }
}
