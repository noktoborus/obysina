//! The window: the field of beads and the floating panels over it.

use std::path::PathBuf;

use egui::{Key, KeyboardShortcut, Modifiers};
use obusina_canvas::{Canvas, Press, View};
use obusina_pattern::{Bead, Cell, DEFAULT_BEAD_RATIO, Grid, Pattern, Rgb};
use rust_i18n::t;

use crate::edit::{Edit, History};
use crate::error::AppError;
use crate::ui::access::Named;

/// The size of a new pattern: columns and rows.
const NEW_SIZE: (usize, usize) = (20, 30);

/// The colours of a new pattern.
const NEW_PALETTE: [(char, [u8; 3]); 5] = [
    ('A', [0x1b, 0x1b, 0x1b]),
    ('B', [0xf2, 0xef, 0xe6]),
    ('C', [0xc0, 0x30, 0x30]),
    ('D', [0x2f, 0x5f, 0xb3]),
    ('E', [0xe8, 0xc5, 0x47]),
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
    /// Paints the region of one colour under the pointer.
    Fill,
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
    /// The colour of the first and of the second press; `None` is an empty
    /// place.
    pub(crate) colours: [Option<usize>; 2],
    /// The steps to undo and redo.
    pub(crate) history: History,
    /// A line for the user: what was saved, what failed.
    pub(crate) message: Option<String>,
    /// The menu of the kinds of grid.
    pub(crate) menu: plate_menu::PlateMenu,
}

impl App {
    /// The window over the pattern at `path`. A file that is not there is a
    /// new pattern to be saved there. A file that cannot be read is never
    /// written over (`RULE pattern.newer`).
    pub fn open(path: Option<PathBuf>) -> App {
        let (pattern, path, message) = match path {
            Some(path) if path.exists() => match obusina_pattern::read(&path) {
                Ok(pattern) => (pattern, Some(path), None),
                Err(error) => (new_pattern(), None, Some(say(&error.into()))),
            },
            other => (new_pattern(), other, None),
        };
        App {
            pattern,
            path,
            view: View::default(),
            tool: Tool::Pencil,
            colours: [Some(0), None],
            history: History::default(),
            message,
            menu: plate_menu::PlateMenu::new(),
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

    /// Does what the tool does to a bead pressed on.
    fn press(&mut self, cell: Cell, press: Press) {
        let which = match press {
            Press::Primary => 0,
            Press::Secondary => 1,
        };
        let colour = self.colours[which];
        match self.tool {
            Tool::Pencil => {
                let switched = match self.pattern.get(cell) {
                    present if present == colour => None,
                    _ => colour,
                };
                self.history.paint(&mut self.pattern, &[cell], switched);
            }
            Tool::Fill => {
                let region = self.pattern.region(cell);
                self.history.paint(&mut self.pattern, &region, colour);
            }
            Tool::Pick => self.colours[which] = self.pattern.get(cell),
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
    fn field(&mut self, ui: &mut egui::Ui) {
        let field = Canvas::new(&self.pattern, &mut self.view).show(ui);
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

/// A new pattern with the colours of [`NEW_PALETTE`].
fn new_pattern() -> Pattern {
    let palette = NEW_PALETTE
        .into_iter()
        .map(|(key, rgb)| Bead {
            key,
            rgb: Rgb(rgb),
            catalog: None,
            code: None,
        })
        .collect();
    let (width, height) = NEW_SIZE;
    Pattern::new(Grid::Loom, width, height, DEFAULT_BEAD_RATIO, palette)
        .expect("the new pattern is valid")
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
