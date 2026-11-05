//! The window: the field of beads and the floating panels over it.

use std::path::PathBuf;

use egui::{Key, KeyboardShortcut, Modifiers};
use obusina_canvas::{Canvas, Press, View};
use obusina_pattern::{Cell, Colour, ColourId, DEFAULT_BEAD_RATIO, Grid, Pattern, Rgb};
use rust_i18n::t;

use crate::edit::{Edit, History};
use crate::error::AppError;
use crate::settings::Settings;
use crate::ui::access::Named;

/// The colours of a new pattern.
const NEW_PALETTE: [[u8; 3]; 5] = [
    [0x1b, 0x1b, 0x1b],
    [0xf2, 0xef, 0xe6],
    [0xc0, 0x30, 0x30],
    [0x2f, 0x5f, 0xb3],
    [0xe8, 0xc5, 0x47],
];

const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::Z);
const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

/// What a press on the field does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Switches the bead pressed on: to the colour, or to empty when it has
    /// the colour already.
    Pencil,
    /// Takes the colour of the bead under the pointer.
    Pick,
}

/// The state of the window.
pub struct App {
    /// The pattern being edited.
    pub(crate) pattern: Pattern,
    /// Where it is saved; `None` when there is nowhere to save it.
    pub(crate) path: Option<PathBuf>,
    /// How the field is moved and zoomed.
    pub(crate) view: View,
    /// What a press does.
    pub(crate) tool: Tool,
    /// The colour in hand; `None` is an empty place.
    pub(crate) colour: Option<ColourId>,
    /// The steps to undo and redo.
    pub(crate) history: History,
    /// A line for the user: what was saved, what failed.
    pub(crate) message: Option<String>,
    /// The menu of the kinds of grid.
    pub(crate) menu: plate_menu::PlateMenu,
    /// The settings: the place of each panel.
    pub(crate) settings: Settings,
}

impl App {
    /// The window over the pattern at `path`, with the settings of the file
    /// `settings`. A pattern file that is not there is a new pattern to be
    /// saved there. A file that cannot be read is never written over
    /// (`RULE pattern.newer`).
    pub fn open(path: Option<PathBuf>, settings: Option<PathBuf>) -> App {
        let (settings, settings_error) = Settings::load(settings);
        let (pattern, path, message) = match path {
            Some(path) if path.exists() => match obusina_pattern::read(&path) {
                Ok(pattern) => (pattern, Some(path), None),
                Err(error) => (new_pattern(), None, Some(say(&error.into()))),
            },
            other => (new_pattern(), other, None),
        };
        let message = message.or_else(|| settings_error.map(|error| say(&error)));
        let mut view = View::default();
        view.set_degrees(pattern.turn());
        let colour = first_colour(&pattern);
        App {
            pattern,
            path,
            view,
            tool: Tool::Pencil,
            colour,
            history: History::default(),
            message,
            menu: plate_menu::PlateMenu::new(),
            settings,
        }
    }

    /// Keeps the place of the panel `name` for the next start.
    pub(crate) fn keep_panel(&mut self, name: &str, placement: obusina_panel::Placement) {
        if let Err(error) = self.settings.keep_panel(name, placement) {
            self.message = Some(say(&error));
        }
    }

    /// Changes the kind of grid, as a step that can be undone.
    pub(crate) fn set_grid(&mut self, grid: Grid) {
        let before = self.pattern.grid();
        if before != grid {
            self.history.apply(
                &mut self.pattern,
                Edit::Grid {
                    before,
                    after: grid,
                },
            );
        }
    }

    /// Turns the field a quarter; the beads stay where they are.
    ///
    /// The turn is kept in the pattern for its file, past the history: it
    /// moves no bead (the `EXC` of `RULE ui.undo`).
    pub(crate) fn turn(&mut self, turn: obusina_canvas::Turn) {
        self.view.turn(turn);
        if let Err(error) = self.pattern.set_turn(self.view.degrees()) {
            log::error!("the view gave a turn that is not a quarter: {error}");
        }
    }

    /// Undoes the last step.
    pub(crate) fn undo(&mut self) {
        self.history.undo(&mut self.pattern);
    }

    /// Does again the last step undone.
    pub(crate) fn redo(&mut self) {
        self.history.redo(&mut self.pattern);
    }

    /// Writes the pattern where it came from.
    pub(crate) fn save(&mut self) {
        let Some(path) = &self.path else {
            return;
        };
        self.message = Some(match write(path, &self.pattern) {
            Ok(()) => t!("file.saved").to_string(),
            Err(error) => say(&error),
        });
    }

    /// Does what the tool does to a bead pressed on. The second button does
    /// nothing.
    fn press(&mut self, cell: Cell, press: Press) {
        if press != Press::Primary {
            return;
        }
        match self.tool {
            Tool::Pencil => {
                let switched = match self.pattern.get(cell) {
                    present if present == self.colour => None,
                    _ => self.colour,
                };
                self.history.paint(&mut self.pattern, &[cell], switched);
            }
            Tool::Pick => self.colour = self.pattern.get(cell),
        }
    }

    /// The keys that are a second way to the buttons (`RULE ui.touch`).
    fn shortcuts(&mut self, ui: &mut egui::Ui) {
        let (redo, undo, save) = ui.input_mut(|input| {
            (
                input.consume_shortcut(&REDO),
                input.consume_shortcut(&UNDO),
                input.consume_shortcut(&SAVE),
            )
        });
        if redo {
            self.redo();
        }
        if undo {
            self.undo();
        }
        if save {
            self.save();
        }
    }

    /// The field of beads over the whole window.
    ///
    /// A pattern with no sheet takes the cells of the first screen as its
    /// sheet: the start value, no step of undo.
    fn field(&mut self, ui: &mut egui::Ui) {
        let field = Canvas::new(&self.pattern, &mut self.view).show(ui);
        if self.pattern.sheet().is_none() {
            self.pattern.set_sheet(self.view.sheet);
        }
        field.response.named(&t!("field"));
        if let Some((cell, press)) = field.pressed {
            self.press(cell, press);
        }
    }
}

impl eframe::App for App {
    /// Cuts the texture side the toolkit is told about down to
    /// [`crate::render::ATLAS_SIDE`]: the glyph atlas is as wide as this
    /// number, and the device is asked for the largest texture it grants.
    fn raw_input_hook(&mut self, _context: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(side) = raw_input.max_texture_side.as_mut() {
            *side = (*side).min(crate::render::ATLAS_SIDE);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::access::name_window(ui.ctx(), &t!("app.name"));
        self.shortcuts(ui);
        egui::CentralPanel::no_frame().show(ui, |ui| self.field(ui));
        crate::ui::tools::show(self, ui.ctx());
        crate::ui::palette::show(self, ui.ctx());
        crate::ui::grid_menu::show(self, ui.ctx());
    }
}

/// The colour in hand at the start: the first of the first palette shown.
fn first_colour(pattern: &Pattern) -> Option<ColourId> {
    pattern.palettes()[0]
        .places()
        .iter()
        .flatten()
        .find(|colour| colour.rgb.is_some())
        .map(|colour| colour.id)
}

/// A new pattern with the colours of [`NEW_PALETTE`].
fn new_pattern() -> Pattern {
    let palette = NEW_PALETTE
        .into_iter()
        .zip(1..)
        .map(|(rgb, id)| Colour::new(ColourId(id), Rgb(rgb)))
        .collect();
    Pattern::new(Grid::Loom, DEFAULT_BEAD_RATIO, palette).expect("the new pattern is valid")
}

/// Writes a pattern file.
fn write(path: &std::path::Path, pattern: &Pattern) -> crate::error::Result<()> {
    Ok(obusina_pattern::write(path, pattern)?)
}

/// The text of an error for the user.
fn say(error: &AppError) -> String {
    log::warn!("{error}: {:?}", std::error::Error::source(error));
    t!(error.message_key()).to_string()
}
