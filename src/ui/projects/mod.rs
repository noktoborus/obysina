//! The list of patterns: the screen the program opens on, which takes the
//! place of the field while it is shown; and the trash, a list of its own.
//! The mockup it was drawn from is `experiments/mockups/src/mock/projects.rs`.
//!
//! ```text
//! the list ──"+ New pattern"──► a new pattern, named by the time, opened
//!          ──"Import"─────────► a .obusina, .obusiny or .json, each pattern
//!                               added, passed over or copied (src/library)
//!          ──a tap on a card──► the pattern opened
//!          ──a long press, the second button ──► its menu (menu.rs)
//!          ──"Trash" │ "Export all" ──► the trash │ one .obusiny;
//!            not while patterns are chosen: the bar on top acts on them
//! the trash ──"Back"──► the list; a card ──► restore │ delete for good
//!           ──"Empty the trash…" ──► asks
//! select: a mark on the cards; a bar on top: the count, export, to the trash, ✖
//! a move to the trash ──► a toast with "Undo" for UNDO_SECONDS
//! ```
//!
//! The list is in the order of the last change. The cards fill the width,
//! as many columns as cards of `CARD_LEAST` fit. The search line shows when
//! the cards do not fit the screen, or a query is typed. The row of the trash
//! and the export, and in the trash the plate that empties it, stand at the
//! bottom of the screen while the cards leave room. The list scrolls by its
//! bar, the wheel, a finger or a drag of the first button; the bar takes no
//! width.
//!
//! The keys are `LIST_KEYS` and `TRASH_KEYS`; a letter typed searches
//! (`docs/keys.md`).

mod cards;
mod menu;
mod thumbs;

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use egui::{Key, Vec2, vec2};
use egui_notify::ToastId;
use obusina_pattern::{Pattern, PatternId};
use plate_menu::PlateMenu;
use rust_i18n::t;

use crate::app::{App, say};
use crate::keys::{Binding, Layer, command, plain};
use crate::library::{Entry, Report, Trashed, archive};
use crate::pick::{Choice, Kinds, Picker, Saved, Saver};
use crate::ui::access::Named;
use crate::ui::icons;
use cards::{CARD_LEAST, CARD_MARGIN, CARD_TEXT, GAP, Lines, PLATE, THUMB_RATIO};
pub use menu::Of;
use thumbs::Thumbs;

/// The seconds the toast that undoes a move to the trash stays.
pub const UNDO_SECONDS: u64 = 5;
/// The cards of actions before the patterns: new and import.
const ACTIONS: usize = 2;

/// Which list is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// The patterns.
    #[default]
    List,
    /// The trash.
    Trash,
}

/// The list as the screen shows it.
pub struct Projects {
    /// The patterns of the list, the last changed first.
    pub(crate) entries: Vec<Entry>,
    /// The patterns of the trash.
    pub(crate) trash: Vec<Trashed>,
    /// The list is read again before it is next drawn.
    stale: bool,
    pub(crate) view: View,
    query: String,
    menu: PlateMenu,
    menu_of: Option<Of>,
    /// The name being written in the menu of a rename.
    renaming: Arc<Mutex<String>>,
    /// The patterns chosen, while patterns are chosen.
    pub(crate) chosen: Option<BTreeSet<PatternId>>,
    /// The card the keys stand on: an index among the cards shown, the
    /// cards of actions first.
    pub(crate) focus: Option<usize>,
    /// The toast that undoes the last move to the trash, and what it moved.
    undo: Option<(ToastId, Vec<PatternId>)>,
    /// What is to be told in a toast.
    told: Vec<String>,
    pub(crate) thumbs: Thumbs,
    /// The choice of a file to import.
    pub(crate) picker: Picker,
    /// The save of an export.
    pub(crate) saver: Saver,
    /// The search line is to take the keys.
    search_focus: bool,
    /// The columns of cards drawn last, which the arrows walk by.
    columns: usize,
}

impl Default for Projects {
    fn default() -> Projects {
        Projects {
            entries: Vec::new(),
            trash: Vec::new(),
            stale: true,
            view: View::List,
            query: String::new(),
            menu: PlateMenu::new().with_id("projects-menu"),
            menu_of: None,
            renaming: Arc::new(Mutex::new(String::new())),
            chosen: None,
            focus: None,
            undo: None,
            told: Vec::new(),
            thumbs: Thumbs::default(),
            picker: Picker::default(),
            saver: Saver::default(),
            search_focus: false,
            columns: 1,
        }
    }
}

impl Projects {
    /// Whether the menu of a card or of the trash is open.
    pub(crate) fn menu_open(&self) -> bool {
        self.menu.is_open()
    }

    /// Reads the list again before it is next drawn.
    pub fn refresh(&mut self) {
        self.stale = true;
    }

    /// Tells `text` in a toast.
    pub fn tell(&mut self, text: String) {
        self.told.push(text);
    }

    /// Tells what an import did: how many came new, how many as copies, and
    /// the names of those the list had already.
    pub fn tell_import(&mut self, report: &Report) {
        let mut text = t!(
            "projects.imported",
            new = report.new.len(),
            copies = report.copies.len()
        )
        .to_string();
        if !report.present.is_empty() {
            let names: Vec<String> = report
                .present
                .iter()
                .map(|name| format!("\u{ab}{name}\u{bb}"))
                .collect();
            text = format!(
                "{text}, {}",
                t!("projects.present", names = names.join(", "))
            );
        }
        self.tell(text);
    }

    /// What is to be told in a toast, for a test.
    #[cfg(test)]
    pub fn told(&self) -> &[String] {
        &self.told
    }

    /// The name of a pattern of the list or of the trash.
    fn name_of(&self, id: PatternId) -> String {
        self.entries
            .iter()
            .map(|entry| &entry.pattern)
            .chain(self.trash.iter().map(|trashed| &trashed.pattern))
            .find(|pattern| pattern.id() == id)
            .map(|pattern| pattern.name().to_owned())
            .unwrap_or_default()
    }

    /// The patterns of the list the query finds.
    fn shown(&self) -> Vec<usize> {
        let query = self.query.to_lowercase();
        (0..self.entries.len())
            .filter(|index| {
                query.is_empty()
                    || self.entries[*index]
                        .pattern
                        .name()
                        .to_lowercase()
                        .contains(&query)
            })
            .collect()
    }
}

/// Does the action of a toast pressed: the undo of a move to the trash.
pub fn action(app: &mut App, _context: &egui::Context, pressed: ToastId) {
    let Some((undo, ids)) = app.projects.undo.take() else {
        return;
    };
    if undo != pressed {
        app.projects.undo = Some((undo, ids));
        return;
    }
    menu::done(app, |library| {
        ids.iter()
            .try_for_each(|id| library.restore(*id).map(|_| ()))
    });
}

/// Moves patterns to the trash, and shows the toast that undoes it.
fn to_trash(app: &mut App, ids: &[PatternId]) {
    let what = match ids {
        [one] => format!("\u{ab}{}\u{bb}", app.projects.name_of(*one)),
        many => many.len().to_string(),
    };
    let moved = ids.to_vec();
    menu::done(app, |library| library.to_trash(&moved, SystemTime::now()));
    let toast = app
        .toasts
        .basic(t!("projects.in_trash", what = what))
        .action(t!("projects.undo"))
        .duration(Some(Duration::from_secs(UNDO_SECONDS)))
        .id();
    app.projects.undo = Some((toast, ids.to_vec()));
    app.projects.chosen = None;
}

/// Asks where to save the patterns of `ids`: one as `<name>.obusina`, more
/// as `obusina-<date>.obusiny`.
fn export(app: &mut App, context: &egui::Context, ids: &[PatternId]) {
    let patterns: Vec<&Pattern> = app
        .projects
        .entries
        .iter()
        .map(|entry| &entry.pattern)
        .filter(|pattern| ids.contains(&pattern.id()))
        .collect();
    let packed = match patterns.as_slice() {
        [] => return,
        [one] => archive::one(one).map(|bytes| (file_name(one.name(), archive::ONE), bytes)),
        many => archive::many(many.iter().copied()).map(|bytes| {
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            (format!("obusina-{date}.{}", archive::MANY), bytes)
        }),
    };
    match packed {
        Ok((name, bytes)) => app.projects.saver.start(context, &name, bytes),
        Err(error) => app.projects.tell(say(&error)),
    }
}

/// The name of a file of a pattern: its name, with the characters no file
/// system takes put as `_`.
fn file_name(name: &str, extension: &str) -> String {
    let stem: String = name
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    let stem = stem.trim();
    let stem = if stem.is_empty() { "obusina" } else { stem };
    format!("{stem}.{extension}")
}

/// Shows the screen of the list: reads it again when it is stale, takes the
/// answers of a choice and of a save, draws the list or the trash.
pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let context = ui.ctx().clone();
    if app.projects.stale {
        read(app, &context);
    }
    for text in std::mem::take(&mut app.projects.told) {
        app.toasts.basic(text);
    }
    take_answers(app, &context);
    keys(app, ui);
    egui::CentralPanel::default().show(ui, |ui| screen(app, ui));
    menu::show(app, &context);
}

/// Reads the list and the trash again; the patterns of the trash past their
/// days go, with their pictures.
fn read(app: &mut App, context: &egui::Context) {
    app.projects.stale = false;
    let Some(library) = &app.library else {
        return;
    };
    let gone = library.purge_old(SystemTime::now());
    app.projects.entries = library.list();
    app.projects.trash = library.trash();
    if !gone.is_empty() {
        app.backdrop.forget(context, gone);
    }
    if app.projects.trash.is_empty() {
        app.projects.view = View::List;
    }
    let ids: BTreeSet<PatternId> = app
        .projects
        .entries
        .iter()
        .map(|e| e.pattern.id())
        .collect();
    if let Some(chosen) = &mut app.projects.chosen {
        chosen.retain(|id| ids.contains(id));
    }
}

/// Takes the file chosen to import, and the answer of a save.
fn take_answers(app: &mut App, context: &egui::Context) {
    match app.projects.picker.take() {
        Some(Choice::Picked(picked)) => {
            app.import_bytes(&picked.bytes);
        }
        Some(Choice::Failed) => app.projects.tell(t!("error.pick").to_string()),
        Some(Choice::GivenUp) | None => {}
    }
    match app.projects.saver.take() {
        Some(Saved::Done) => app.projects.tell(t!("projects.exported").to_string()),
        Some(Saved::Failed) => app.projects.tell(t!("error.export").to_string()),
        Some(Saved::Downloaded | Saved::GivenUp) | None => {}
    }
    let _ = context;
}

/// What a key of the list does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListAction {
    Search,
    /// Walks the cards by the arrow `Taken::index` of `ARROWS`.
    Walk,
    Open,
    ToTrash,
    Rename,
}

/// The arrows that walk the cards: right, left, down, up.
const ARROWS: [egui::KeyboardShortcut; 4] = [
    plain(Key::ArrowRight),
    plain(Key::ArrowLeft),
    plain(Key::ArrowDown),
    plain(Key::ArrowUp),
];

/// The keys of the list (`Layer::List`).
pub(crate) const LIST_KEYS: &[Binding<ListAction>] = &[
    Binding {
        keys: &[command(Key::F)],
        action: ListAction::Search,
        name: "projects.search",
    },
    Binding {
        keys: &ARROWS,
        action: ListAction::Walk,
        name: "keys.walk",
    },
    Binding {
        keys: &[plain(Key::Enter)],
        action: ListAction::Open,
        name: "keys.open",
    },
    Binding {
        keys: &[plain(Key::Delete)],
        action: ListAction::ToTrash,
        name: "keys.to_trash",
    },
    Binding {
        keys: &[plain(Key::F2)],
        action: ListAction::Rename,
        name: "projects.rename",
    },
];

/// What a key of the trash does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrashAction {
    Back,
    /// Walks the cards by the arrow `Taken::index` of `ARROWS`.
    Walk,
    Open,
}

/// The keys of the trash (`Layer::Trash`).
pub(crate) const TRASH_KEYS: &[Binding<TrashAction>] = &[
    Binding {
        keys: &[plain(Key::Escape)],
        action: TrashAction::Back,
        name: "projects.back_from_trash",
    },
    Binding {
        keys: &ARROWS,
        action: TrashAction::Walk,
        name: "keys.walk",
    },
    Binding {
        keys: &[plain(Key::Enter)],
        action: TrashAction::Open,
        name: "keys.trash_open",
    },
];

/// The keys of the list or of the trash, whichever is on top.
fn keys(app: &mut App, ui: &egui::Ui) {
    match crate::keys::layer(app, ui.ctx()) {
        Layer::List => list_keys(app, ui),
        Layer::Trash => trash_keys(app, ui),
        _ => {}
    }
}

/// The keys of the list, and a letter typed, which searches.
fn list_keys(app: &mut App, ui: &egui::Ui) {
    let taken = crate::keys::take(ui.ctx(), LIST_KEYS);
    let typed: String = ui.input(|input| {
        input
            .events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    });
    let search = taken.iter().any(|taken| taken.action == ListAction::Search);
    if search || !typed.trim().is_empty() {
        app.projects.query.push_str(typed.trim_start());
        app.projects.search_focus = true;
        return;
    }
    let walks = taken
        .iter()
        .filter(|taken| taken.action == ListAction::Walk)
        .map(|taken| taken.index);
    let count = ACTIONS + app.projects.shown().len();
    let Some(at) = walk(app, walks, count) else {
        return;
    };
    let time = ui.input(|input| input.time);
    let shown = app.projects.shown();
    let id = at
        .checked_sub(ACTIONS)
        .map(|index| app.projects.entries[shown[index]].pattern.id());
    for taken in taken {
        match (taken.action, id) {
            (ListAction::Open, None) if at == 0 => app.create_pattern(time),
            (ListAction::Open, None) => import(app, ui.ctx()),
            (ListAction::Open, Some(id)) => app.open_by_id(id, time),
            (ListAction::ToTrash, Some(id)) => to_trash(app, &[id]),
            (ListAction::Rename, Some(id)) => menu::rename(app, id),
            _ => continue,
        }
        return;
    }
}

/// The keys of the trash.
fn trash_keys(app: &mut App, ui: &egui::Ui) {
    let taken = crate::keys::take(ui.ctx(), TRASH_KEYS);
    if taken.iter().any(|taken| taken.action == TrashAction::Back) {
        app.projects.view = View::List;
        app.projects.focus = None;
        return;
    }
    let walks = taken
        .iter()
        .filter(|taken| taken.action == TrashAction::Walk)
        .map(|taken| taken.index);
    let count = app.projects.trash.len();
    let Some(at) = walk(app, walks, count) else {
        return;
    };
    if taken.iter().any(|taken| taken.action == TrashAction::Open) {
        let id = app.projects.trash[at].pattern.id();
        menu::open(app, Of::Trashed(id));
    }
}

/// Moves the card the keys stand on by `walks`, indices of `ARROWS`, among
/// `count` cards, and gives it; `None` while it stands on none.
fn walk(app: &mut App, walks: impl Iterator<Item = usize>, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let columns = app.projects.columns.max(1) as isize;
    let step: isize = walks.map(|arrow| [1, -1, columns, -columns][arrow]).sum();
    if step != 0 {
        let at = app.projects.focus.map_or(0, |at| {
            (at as isize + step).clamp(0, count as isize - 1) as usize
        });
        app.projects.focus = Some(at);
    }
    app.projects.focus.filter(|at| *at < count)
}

/// Asks for a file of patterns to import.
fn import(app: &mut App, context: &egui::Context) {
    app.projects
        .picker
        .start(context, &t!("projects.kinds"), Kinds::Patterns);
}

/// The screen: the bar of a choice, then the list or the trash, scrolled.
fn screen(app: &mut App, ui: &mut egui::Ui) {
    let size = ui.available_size();
    let width = size.x;
    let columns = (((width + GAP) / (CARD_LEAST + GAP)).floor() as usize).max(1);
    app.projects.columns = columns;
    let card_width = (width - GAP * (columns - 1) as f32) / columns as f32;
    let card = vec2(
        card_width,
        CARD_MARGIN + (card_width - 2.0 * CARD_MARGIN) * THUMB_RATIO + CARD_TEXT,
    );
    let mut visible = size.y;
    if app.projects.chosen.is_some() && app.projects.view == View::List {
        choice_bar(app, ui, width);
        ui.add_space(GAP);
        visible -= PLATE + GAP;
    }
    let rows = (app.projects.entries.len() + ACTIONS).div_ceil(columns) as f32;
    let needed = rows * (card.y + GAP) + PLATE;
    let search = app.projects.view == View::List
        && (needed > visible || !app.projects.query.is_empty() || app.projects.search_focus);
    ui.spacing_mut().scroll = egui::style::ScrollStyle::floating();
    egui::ScrollArea::vertical()
        .id_salt(("projects", app.projects.view == View::Trash))
        .auto_shrink(false)
        .scroll_source(egui::scroll_area::ScrollSource::ALL)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(GAP);
            if search {
                ui.allocate_ui(vec2(width, 0.0), |ui| search_line(app, ui));
            }
            match app.projects.view {
                View::List => list(app, ui, (columns, card, width), visible),
                View::Trash => trash(app, ui, (columns, card, width), visible),
            }
        });
}

/// The search line, by its name, with room above and below its letters.
fn search_line(app: &mut App, ui: &mut egui::Ui) {
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let field = egui::TextEdit::singleline(&mut app.projects.query)
        .hint_text(t!("projects.search"))
        .margin(vec2(line, line))
        .desired_width(f32::INFINITY);
    let response = ui.add(field).named(&t!("projects.search"));
    if std::mem::take(&mut app.projects.search_focus) {
        response.request_focus();
    }
    if response.changed() {
        app.projects.focus = None;
    }
}

/// The bar on top while patterns are chosen: their count, export, to the
/// trash, and the end of the choice.
fn choice_bar(app: &mut App, ui: &mut egui::Ui, width: f32) {
    let Some(chosen) = app.projects.chosen.clone() else {
        return;
    };
    let ids: Vec<PatternId> = chosen.into_iter().collect();
    ui.allocate_ui(vec2(width, PLATE), |ui| {
        ui.set_min_size(vec2(width, PLATE));
        ui.horizontal_centered(|ui| {
            if crate::ui::button::icon(ui, icons::CANCEL, &t!("projects.end_choice"), true)
                .clicked()
            {
                app.projects.chosen = None;
            }
            ui.label(t!("projects.chosen", count = ids.len()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let some = !ids.is_empty();
                if crate::ui::button::icon(ui, icons::TRASH, &t!("projects.to_trash"), some)
                    .clicked()
                {
                    to_trash(app, &ids);
                }
                if crate::ui::button::icon(ui, icons::EXPORT, &t!("projects.export"), some)
                    .clicked()
                {
                    export(app, ui.ctx(), &ids);
                }
            });
        });
    });
}

/// The list: the cards of the actions, the patterns the query finds, then
/// the row of the trash and the export.
fn list(
    app: &mut App,
    ui: &mut egui::Ui,
    (columns, card, width): (usize, Vec2, f32),
    visible: f32,
) {
    let top = ui.min_rect().top();
    let time = ui.input(|input| input.time);
    let shown = app.projects.shown();
    let mut cells: Vec<Option<usize>> = vec![None; ACTIONS];
    cells.extend(shown.into_iter().map(Some));
    let today = chrono::Local::now();
    for (row, line) in cells.chunks(columns).enumerate() {
        ui.horizontal(|ui| {
            for (place, cell) in line.iter().enumerate() {
                let at = row * columns + place;
                let focused = app.projects.focus == Some(at);
                match cell {
                    None if at == 0 => {
                        let words = t!("projects.new");
                        let face = (icons::NEW, words.as_ref());
                        if cards::action_card(ui, card, face, focused).clicked() {
                            app.create_pattern(time);
                        }
                    }
                    None => {
                        let words = t!("projects.import");
                        let face = (icons::IMPORT, words.as_ref());
                        if cards::action_card(ui, card, face, focused).clicked() {
                            import(app, ui.ctx());
                        }
                    }
                    Some(index) => {
                        let entry = &app.projects.entries[*index];
                        let id = entry.pattern.id();
                        let lines = Lines {
                            name: entry.pattern.name(),
                            size: size_words(&entry.pattern),
                            last: when(entry.changed, today),
                            paper: cards::colour(entry.pattern.paper()),
                        };
                        let mark = app
                            .projects
                            .chosen
                            .as_ref()
                            .map(|chosen| chosen.contains(&id));
                        let place = (picture_size(card), app.settings.glow());
                        let picture =
                            app.projects
                                .thumbs
                                .of(ui.ctx(), &entry.pattern, entry.changed, place);
                        let response = cards::card(ui, card, picture, &lines, (mark, focused));
                        let held = response.secondary_clicked() || response.long_touched();
                        if let Some(chosen) = &mut app.projects.chosen {
                            if (response.clicked() || held) && !chosen.remove(&id) {
                                chosen.insert(id);
                            }
                        } else if held {
                            menu::open(app, Of::Live(id));
                        } else if response.clicked() {
                            app.open_by_id(id, time);
                        }
                    }
                }
            }
        });
    }
    let choosing = app.projects.chosen.is_some();
    let export_all = !choosing && !app.projects.entries.is_empty();
    let trash = !choosing && !app.projects.trash.is_empty();
    if !export_all && !trash {
        return;
    }
    to_the_bottom(ui, top, visible);
    let shares = usize::from(export_all) + usize::from(trash);
    let share = (width - (shares - 1) as f32 * GAP) / shares as f32;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        if trash {
            let count = app.projects.trash.len();
            let words = format!(
                "{} \u{b7} {count} {}",
                t!("projects.trash"),
                patterns(count)
            );
            if cards::plate(ui, share, icons::TRASH, &words).clicked() {
                app.projects.view = View::Trash;
                app.projects.query.clear();
                app.projects.focus = None;
            }
        }
        if export_all {
            let count = app.projects.entries.len();
            let words = format!(
                "{} \u{b7} {count} {}",
                t!("projects.export_all"),
                patterns(count)
            );
            if cards::plate(ui, share, icons::EXPORT, &words).clicked() {
                let all = app.projects.entries.iter().map(|e| e.pattern.id());
                let ids: Vec<PatternId> = all.collect();
                export_many(app, ui.ctx(), &ids);
            }
        }
    });
}

/// Saves the patterns of `ids` as one `.obusiny`, even when there is one.
fn export_many(app: &mut App, context: &egui::Context, ids: &[PatternId]) {
    let patterns = app
        .projects
        .entries
        .iter()
        .map(|entry| &entry.pattern)
        .filter(|pattern| ids.contains(&pattern.id()));
    match archive::many(patterns) {
        Ok(bytes) => {
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let name = format!("obusina-{date}.{}", archive::MANY);
            app.projects.saver.start(context, &name, bytes);
        }
        Err(error) => app.projects.tell(say(&error)),
    }
}

/// The trash: the plate back to the list, its patterns with the days each
/// has left, the plate that empties it.
fn trash(
    app: &mut App,
    ui: &mut egui::Ui,
    (columns, card, width): (usize, Vec2, f32),
    visible: f32,
) {
    let top = ui.min_rect().top();
    if cards::plate(ui, width, icons::BACK, &t!("projects.back_from_trash")).clicked() {
        app.projects.view = View::List;
        app.projects.focus = None;
    }
    let now = SystemTime::now();
    let indices: Vec<usize> = (0..app.projects.trash.len()).collect();
    for (row, line) in indices.chunks(columns).enumerate() {
        ui.horizontal(|ui| {
            for (place, index) in line.iter().enumerate() {
                let trashed = &app.projects.trash[*index];
                let id = trashed.pattern.id();
                let days = trashed.days_left(now);
                let lines = Lines {
                    name: trashed.pattern.name(),
                    size: size_words(&trashed.pattern),
                    last: t!(
                        "projects.gone_in",
                        count = days,
                        days = plural(days as usize, DAYS)
                    )
                    .to_string(),
                    paper: cards::colour(trashed.pattern.paper()),
                };
                let focused = app.projects.focus == Some(row * columns + place);
                let place = (picture_size(card), app.settings.glow());
                let picture =
                    app.projects
                        .thumbs
                        .of(ui.ctx(), &trashed.pattern, trashed.since, place);
                let response = cards::card(ui, card, picture, &lines, (None, focused));
                if response.clicked() || response.secondary_clicked() || response.long_touched() {
                    menu::open(app, Of::Trashed(id));
                }
            }
        });
    }
    to_the_bottom(ui, top, visible);
    if cards::plate(ui, width, icons::TRASH, &t!("projects.empty")).clicked() {
        menu::open(app, Of::AllTrash);
    }
}

/// The size of the place of the picture of a card of `card`.
fn picture_size(card: Vec2) -> Vec2 {
    let inner = card.x - 2.0 * CARD_MARGIN;
    vec2(inner, inner * THUMB_RATIO)
}

/// Leaves room before a row of plates, so that it stands at the bottom of
/// the screen, `visible` high, when what came since `top` leaves room.
fn to_the_bottom(ui: &mut egui::Ui, top: f32, visible: f32) {
    let room = visible - (ui.cursor().top() - top) - PLATE;
    if room > 0.0 {
        ui.add_space(room);
    }
}

/// The size of the sheet of a pattern and its beads.
fn size_words(pattern: &Pattern) -> String {
    let (columns, rows) = pattern
        .work_area()
        .map_or((0, 0), |area| (area.width(), area.height()));
    let beads = pattern.beads().count();
    format!(
        "{columns}\u{d7}{rows} \u{b7} {beads} {}",
        plural(beads, BEADS)
    )
}

/// When a file changed, as the list says it: today and yesterday with the
/// time, else the date.
fn when(changed: SystemTime, today: chrono::DateTime<chrono::Local>) -> String {
    let changed: chrono::DateTime<chrono::Local> = changed.into();
    let days = today.date_naive() - changed.date_naive();
    let time = changed.format("%H:%M").to_string();
    match days.num_days() {
        0 => t!("projects.today", time = time).to_string(),
        1 => t!("projects.yesterday", time = time).to_string(),
        _ => changed.format("%Y-%m-%d").to_string(),
    }
}

/// "patterns" after a count.
fn patterns(count: usize) -> String {
    plural(count, PATTERNS)
}

/// The keys of a word after a number: one, few, many.
const PATTERNS: [&str; 3] = [
    "plural.patterns.one",
    "plural.patterns.few",
    "plural.patterns.many",
];
const BEADS: [&str; 3] = ["plural.beads.one", "plural.beads.few", "plural.beads.many"];
const DAYS: [&str; 3] = ["plural.days.one", "plural.days.few", "plural.days.many"];

/// The form of a word of `keys` after the number `count`, by the rule of
/// the language: one, few or many in Russian, one or many in English.
fn plural(count: usize, keys: [&str; 3]) -> String {
    let form = if rust_i18n::locale().starts_with("ru") {
        let (tens, ones) = (count % 100, count % 10);
        if (11..=14).contains(&tens) {
            2
        } else if ones == 1 {
            0
        } else if (2..=4).contains(&ones) {
            1
        } else {
            2
        }
    } else if count == 1 {
        0
    } else {
        2
    };
    t!(keys[form]).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_with_a_slash_makes_a_file_name() {
        assert_eq!(file_name("a/b: c", "obusina"), "a_b_ c.obusina");
        assert_eq!(file_name("  ", "obusina"), "obusina.obusina");
    }
}
