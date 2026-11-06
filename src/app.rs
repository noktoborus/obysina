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
/// The digits, `0` first: the top row and the numpad give the same keys.
const DIGITS: [Key; 10] = [
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

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
    /// The menu of the settings: the grid, the ruling, the paper, the glow.
    pub(crate) menu: plate_menu::PlateMenu,
    /// The settings: the place of each panel.
    pub(crate) settings: Settings,
    /// The palette the panel and the window show.
    pub(crate) palette: usize,
    /// The window of the palettes.
    pub(crate) palette_window: crate::ui::palette_window::PaletteWindow,
    /// The menu of the colour in hand on the panel of beads.
    pub(crate) bead_menu: plate_menu::PlateMenu,
    /// What that menu is of: a colour, or the empty place.
    pub(crate) bead_menu_of: Option<Option<ColourId>>,
    /// The ruling or the paper being changed in the menu of the settings,
    /// while it is.
    pub(crate) settings_edit: Option<crate::ui::settings_menu::Editing>,
    /// The picture under the pattern.
    pub(crate) backdrop: crate::backdrop::Backdrop,
    /// The menu of the picture.
    pub(crate) backdrop_menu: plate_menu::PlateMenu,
    /// A choice of a picture file under way.
    pub(crate) picker: crate::pick::Picker,
}

impl App {
    /// The window over the pattern at `path`, with the settings of the file
    /// `settings` and the pictures of the store `pictures`. A pattern file
    /// that is not there is a new pattern to be saved there. A file that
    /// cannot be read is never written over (`RULE pattern.newer`).
    pub fn open(
        path: Option<PathBuf>,
        settings: Option<PathBuf>,
        pictures: Option<PathBuf>,
    ) -> App {
        let (settings, settings_error) = Settings::load(settings);
        let (pattern, path, message) = match path {
            Some(path) if path.exists() => match obusina_pattern::read(&path) {
                Ok(pattern) => (pattern, Some(path), None),
                Err(error) => (new_pattern(), None, Some(say(&error.into()))),
            },
            other => (new_pattern(), other, None),
        };
        let backdrop = crate::backdrop::Backdrop::new(pictures);
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
            palette: 0,
            palette_window: Default::default(),
            bead_menu: plate_menu::PlateMenu::new().with_id("bead-menu"),
            bead_menu_of: None,
            settings_edit: None,
            backdrop,
            backdrop_menu: plate_menu::PlateMenu::new().with_id("backdrop-menu"),
            picker: crate::pick::Picker::default(),
        }
    }

    /// Says what went wrong with the picture under the pattern, if anything.
    pub(crate) fn say_backdrop(&mut self, done: crate::error::Result<()>) {
        if let Err(error) = done {
            log::error!("the picture under the pattern: {error}");
            self.message = Some(say(&error));
        }
    }

    /// Puts a picture chosen by the person under the pattern.
    pub(crate) fn take_picture(&mut self, context: &egui::Context, picked: crate::pick::Picked) {
        let done = self.backdrop.put(context, &self.pattern, picked);
        self.say_backdrop(done);
    }

    /// Keeps how the places of a palette stand for the next start.
    pub(crate) fn keep_palette_view(&mut self, view: crate::settings::PaletteView) {
        if let Err(error) = self.settings.keep_palette_view(view) {
            self.message = Some(say(&error));
        }
    }

    /// Shows or hides the shadows of the empty places, and keeps that for the
    /// next start.
    pub(crate) fn keep_shadows(&mut self, shadows: bool) {
        if let Err(error) = self.settings.keep_shadows(shadows) {
            self.message = Some(say(&error));
        }
    }

    /// Draws the beads round or flat, and keeps that for the next start.
    pub(crate) fn keep_glow(&mut self, glow: bool) {
        if let Err(error) = self.settings.keep_glow(glow) {
            self.message = Some(say(&error));
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
        let free = !ui.ctx().egui_wants_keyboard_input()
            && !self.palette_window.open
            && !self.bead_menu.is_open()
            && !self.menu.is_open();
        if !free {
            return;
        }
        if ui.input(|input| input.key_pressed(Key::P)) {
            self.palette_window.open = true;
        }
        crate::ui::palette_window::step_by_keys(self, ui);
        if let Some(digit) = ui.input(|input| DIGITS.iter().position(|key| input.key_pressed(*key)))
        {
            self.take_by_digit(digit);
        }
    }

    /// Takes in hand the colour of the place of a digit of the palette shown,
    /// `0` the empty place. A digit of an empty or removed place does nothing.
    fn take_by_digit(&mut self, digit: usize) {
        if digit == 0 {
            self.colour = None;
            return;
        }
        let place = &self.pattern.palettes()[self.palette].places()[digit - 1];
        if let Some(colour) = place.as_ref().filter(|colour| colour.rgb.is_some()) {
            self.colour = Some(colour.id);
        }
    }

    /// The field of beads over the whole window.
    ///
    /// A pattern with no sheet takes the cells of the first screen as its
    /// sheet: the start value, no step of undo. The size of the first screen
    /// is the least a sheet shrinks to when a bead goes.
    fn field(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = self.backdrop.update(ui.ctx(), &self.pattern) {
            self.say_backdrop(Err(error));
        }
        let mut canvas = Canvas::new(&self.pattern, &mut self.view)
            .shadows(self.settings.shadows())
            .glow(self.settings.glow());
        if let Some(picture) = self.backdrop.picture(ui.ctx()) {
            canvas = canvas.picture(picture);
        }
        let field = canvas.show(ui);
        if self.pattern.sheet().is_none() {
            self.pattern.set_sheet(self.view.sheet);
        }
        self.history.least_sheet = self.view.sheet.map(|first| (first.width(), first.height()));
        field.response.named(&t!("field"));
        if let Some((cell, press)) = field.pressed.filter(|_| !self.backdrop.editing()) {
            self.press(cell, press);
        }
    }
}

impl eframe::App for App {
    /// Cuts the texture side the toolkit is told about down to
    /// [`crate::render::ATLAS_SIDE`]: the glyph atlas is as wide as this
    /// number, and the device is asked for the largest texture it grants.
    /// On android, gives egui the part of the window the system bars and the
    /// cut out of the camera leave free, which egui-winit gives only on iOS:
    /// the panels stand inside it; and the theme of the system, which winit
    /// gives on no android, so the window follows the night mode.
    fn raw_input_hook(&mut self, _context: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(side) = raw_input.max_texture_side.as_mut() {
            *side = (*side).min(crate::render::ATLAS_SIDE);
        }
        #[cfg(target_os = "android")]
        {
            if let Some(insets) = crate::activity::safe_area(raw_input) {
                raw_input.safe_area_insets = Some(insets);
            }
            if let Some(theme) = crate::activity::theme() {
                raw_input.system_theme = Some(theme);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::access::name_window(ui.ctx(), &t!("app.name"));
        self.shortcuts(ui);
        egui::CentralPanel::no_frame().show(ui, |ui| self.field(ui));
        crate::ui::tools::show(self, ui.ctx());
        crate::ui::palette::show(self, ui.ctx());
        crate::ui::palette_window::show(self, ui.ctx());
        crate::ui::palette::menu(self, ui.ctx());
        crate::ui::settings_menu::show(self, ui.ctx());
        crate::ui::backdrop_menu::show(self, ui.ctx());
        crate::ui::palette_window::keep_in_hand(self);
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
pub(crate) fn new_pattern() -> Pattern {
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
