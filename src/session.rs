//! The pattern open in the editor: how it opens from the list, how it is
//! saved without being asked, and how the window goes back to the list.
//!
//! ```text
//! the list ──a card──► open ──► the editor ──back, Esc──► save ──► the list
//!          ──"New"───► a pattern named by the time, added, opened
//! the editor: a change ──► 6 s with no other (AUTOSAVE_IDLE) ──► save
//!                     └──► AUTOSAVE_STEPS steps since the last save ──► save
//!             the window closed, android pauses the program ──► save
//! ```
//!
//! A save writes the pattern and its history (`src/library/mod.rs`). Nothing
//! is saved when nothing changed since the last save.

use std::path::Path;

use obusina_canvas::View;
use obusina_pattern::{Pattern, PatternId};

use crate::app::{App, Tool, first_colour, say};
use crate::edit::History;
use crate::library::Library;

/// The seconds with no change after which a changed pattern is saved.
pub const AUTOSAVE_IDLE: f64 = 6.0;
/// The steps of the history after which a changed pattern is saved, however
/// busy the person is.
pub const AUTOSAVE_STEPS: u64 = 100;

/// Which screen the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The list of patterns, and the trash.
    List,
    /// The field of the pattern open.
    Editor,
}

/// What was saved last, and when the pattern last changed.
#[derive(Debug, Default, Clone, Copy)]
pub struct Saved {
    /// The revision of the pattern written last.
    revision: u64,
    /// The steps of the history kept when it was written.
    steps: u64,
    /// The revision seen in the last frame.
    seen: u64,
    /// The time of the frame the pattern last changed in.
    changed_at: f64,
}

impl Saved {
    /// A pattern just opened or written, at `time`.
    fn of(pattern: &Pattern, history: &History, time: f64) -> Saved {
        Saved {
            revision: pattern.revision(),
            steps: history.kept(),
            seen: pattern.revision(),
            changed_at: time,
        }
    }
}

impl App {
    /// Opens `pattern` with its `history` in the editor: the field starts
    /// as a new window would, the picture of the pattern is looked for.
    pub(crate) fn open_pattern(&mut self, pattern: Pattern, history: History, time: f64) {
        let mut view = View::default();
        view.set_degrees(pattern.turn());
        self.colour = first_colour(&pattern);
        self.saved = Saved::of(&pattern, &history, time);
        self.pattern = pattern;
        self.history = history;
        self.view = view;
        self.tool = Tool::Pencil;
        self.palette = 0;
        self.message = None;
        self.settings_edit = None;
        self.bead_menu_of = None;
        self.palette_window.open = false;
        for menu in [&mut self.menu, &mut self.bead_menu, &mut self.backdrop_menu] {
            menu.close();
        }
        self.backdrop = crate::backdrop::Backdrop::new(self.pictures.clone());
        self.screen = Screen::Editor;
    }

    /// Opens the pattern of the list of this identifier.
    pub(crate) fn open_by_id(&mut self, id: PatternId, time: f64) {
        let Some(library) = &self.library else {
            return;
        };
        match library.open(id) {
            Ok((pattern, history)) => self.open_pattern(pattern, history, time),
            Err(error) => self.projects.tell(say(&error)),
        }
    }

    /// Makes a new pattern, named by the time now, adds it to the list and
    /// opens it.
    pub(crate) fn create_pattern(&mut self, time: f64) {
        let mut pattern = crate::app::new_pattern();
        pattern.set_name(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
        if let Some(library) = &self.library
            && let Err(error) = library.add(&pattern)
        {
            self.projects.tell(say(&error));
            return;
        }
        self.open_pattern(pattern, History::default(), time);
    }

    /// Saves the pattern open when it changed since it was last saved.
    pub(crate) fn save_now(&mut self, time: f64) {
        if self.screen != Screen::Editor || self.pattern.revision() == self.saved.revision {
            return;
        }
        let Some(library) = &self.library else {
            return;
        };
        match library.save(&self.pattern, &self.history) {
            Ok(()) => self.saved = Saved::of(&self.pattern, &self.history, time),
            Err(error) => self.message = Some(say(&error)),
        }
    }

    /// Saves the pattern open when it rested `AUTOSAVE_IDLE` seconds since
    /// its last change, or when `AUTOSAVE_STEPS` steps were kept since the
    /// last save; asks for a frame when the rest will be over.
    pub(crate) fn autosave(&mut self, context: &egui::Context) {
        if self.screen != Screen::Editor {
            return;
        }
        let time = context.input(|input| input.time);
        let revision = self.pattern.revision();
        if revision != self.saved.seen {
            self.saved.seen = revision;
            self.saved.changed_at = time;
        }
        if revision == self.saved.revision {
            return;
        }
        let rested = time - self.saved.changed_at;
        let steps = self.history.kept() - self.saved.steps;
        if rested >= AUTOSAVE_IDLE || steps >= AUTOSAVE_STEPS {
            self.save_now(time);
        } else {
            let rest = std::time::Duration::from_secs_f64(AUTOSAVE_IDLE - rested);
            context.request_repaint_after(rest);
        }
    }

    /// Saves the pattern open and shows the list again. Corners of the
    /// picture being edited are put back.
    pub(crate) fn back_to_list(&mut self, context: &egui::Context, time: f64) {
        if self.backdrop.editing() {
            self.backdrop.end_editing(context, false);
        }
        self.save_now(time);
        self.screen = Screen::List;
        self.projects.refresh();
    }

    /// Shows the toasts of the window, and does the action pressed on one.
    pub(crate) fn show_toasts(&mut self, context: &egui::Context) {
        for pressed in self.toasts.show(context) {
            crate::ui::projects::action(self, context, pressed);
        }
    }

    /// Takes in a file named on the command line as a copy in the list, and
    /// opens it; what was taken in is told on the list.
    pub(crate) fn take_in_file(&mut self, path: &Path, time: f64) {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(source) => {
                let error = crate::error::AppError::LibraryRead {
                    path: path.to_path_buf(),
                    source,
                };
                self.projects.tell(say(&error));
                return;
            }
        };
        if let Some(id) = self.import_bytes(&bytes) {
            self.open_by_id(id, time);
        }
    }

    /// Imports the patterns of a file; tells what came in, and gives the
    /// pattern to open: the first one added, or the one the list had of it.
    pub(crate) fn import_bytes(&mut self, bytes: &[u8]) -> Option<PatternId> {
        let library = self.library.as_ref()?;
        let imported = crate::library::archive::read(bytes).and_then(|patterns| {
            let first = patterns.first().map(Pattern::id);
            library.import(patterns).map(|report| (report, first))
        });
        match imported {
            Ok((report, first)) => {
                self.projects.tell_import(&report);
                self.projects.refresh();
                report
                    .added()
                    .next()
                    .or(first.filter(|id| library.has(*id)))
            }
            Err(error) => {
                self.projects.tell(say(&error));
                None
            }
        }
    }
}

/// The library of the data directory, with the patterns of the trash past
/// their days deleted, and the pattern file the version before kept in the
/// data directory of android taken in: it is the work of the person.
pub fn library(dir: &Path) -> Library {
    let library = Library::new(dir.to_path_buf());
    let before = dir.join(OLD_PATTERN);
    if before.exists() {
        match obusina_pattern::read(&before).map(|pattern| library.import(vec![pattern])) {
            Ok(Ok(_)) => {
                let _ = std::fs::rename(&before, dir.join(OLD_PATTERN_TAKEN));
            }
            Ok(Err(error)) => log::error!("the pattern before the list stays: {error}"),
            Err(error) => log::error!("the pattern before the list stays: {error}"),
        }
    }
    library
}

/// The one pattern the version before the list kept on android.
const OLD_PATTERN: &str = "pattern.json";
/// What it is renamed to once it is in the list.
const OLD_PATTERN_TAKEN: &str = "pattern.json.taken";
