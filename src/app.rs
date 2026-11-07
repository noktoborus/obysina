//! The window: the list of patterns, or the field of beads of the pattern
//! open and the floating panels over it (`src/session.rs`).

use std::path::PathBuf;

use egui::{Key, KeyboardShortcut};
use obusina_canvas::{Canvas, Press, View};
use obusina_pattern::{Cell, Colour, ColourId, DEFAULT_BEAD_RATIO, Grid, Pattern, Rgb};
use rust_i18n::t;

use crate::edit::{Edit, History};
use crate::error::AppError;
use crate::keys::{Binding, Layer, command, command_shift, plain};
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

/// What a key of the field does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldAction {
    Redo,
    Undo,
    Save,
    Back,
    Palettes,
    NextPalette,
    PreviousPalette,
    /// Takes the colour of the place of the digit, `Taken::index`.
    Digit,
}

/// The keys of the field (`Layer::Editor`), a second way to the buttons
/// (`RULE ui.touch`).
pub(crate) const FIELD_KEYS: &[Binding<FieldAction>] = &[
    Binding {
        keys: &[command_shift(Key::Z)],
        action: FieldAction::Redo,
        name: "edit.redo",
    },
    Binding {
        keys: &[command(Key::Z)],
        action: FieldAction::Undo,
        name: "edit.undo",
    },
    Binding {
        keys: &[command(Key::S)],
        action: FieldAction::Save,
        name: "keys.save",
    },
    Binding {
        keys: &[plain(Key::Escape)],
        action: FieldAction::Back,
        name: "projects.back",
    },
    Binding {
        keys: &[plain(Key::P)],
        action: FieldAction::Palettes,
        name: "keys.palettes",
    },
    Binding {
        keys: &[plain(Key::Plus), plain(Key::Equals)],
        action: FieldAction::NextPalette,
        name: "palette.next",
    },
    Binding {
        keys: &[plain(Key::Minus)],
        action: FieldAction::PreviousPalette,
        name: "palette.previous",
    },
    Binding {
        keys: &DIGITS,
        action: FieldAction::Digit,
        name: "keys.digit",
    },
];

/// The digits, `0` first: the top row and the numpad give the same keys.
const DIGITS: [KeyboardShortcut; 10] = [
    plain(Key::Num0),
    plain(Key::Num1),
    plain(Key::Num2),
    plain(Key::Num3),
    plain(Key::Num4),
    plain(Key::Num5),
    plain(Key::Num6),
    plain(Key::Num7),
    plain(Key::Num8),
    plain(Key::Num9),
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
    /// Which screen is shown: the list or the editor.
    pub(crate) screen: crate::session::Screen,
    /// The patterns of the person; `None` when the platform gives no place
    /// to keep them, and nothing is saved.
    pub(crate) library: Option<crate::library::Library>,
    /// The list of patterns as the screen shows it.
    pub(crate) projects: crate::ui::projects::Projects,
    /// The store of the pictures under the patterns.
    pub(crate) pictures: Option<PathBuf>,
    /// What was saved last of the pattern open.
    pub(crate) saved: crate::session::Saved,
    /// The pattern being edited; a new one while the list is shown first.
    pub(crate) pattern: Pattern,
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
    /// The toasts: what was done, and the undo of a move to the trash.
    pub(crate) toasts: egui_notify::Toasts,
}

/// Where the window keeps what it keeps; `None` where the platform gives no
/// place.
#[derive(Debug, Default, Clone)]
pub struct Places {
    /// The directory of the patterns (`src/library/mod.rs`).
    pub data: Option<PathBuf>,
    /// The file of the settings.
    pub settings: Option<PathBuf>,
    /// The store of the pictures under the patterns.
    pub pictures: Option<PathBuf>,
    /// A file named on the command line: taken in as a copy and opened.
    pub file: Option<PathBuf>,
}

impl App {
    /// The window on the list of patterns kept in `places`. A file named on
    /// the command line is taken in as a copy and opened. With no place for
    /// patterns, a new pattern opens and nothing is saved.
    pub fn start(places: Places) -> App {
        let (settings, settings_error) = Settings::load(places.settings);
        let library = places.data.as_deref().map(crate::session::library);
        let pattern = new_pattern();
        let mut app = App {
            screen: crate::session::Screen::List,
            library,
            projects: crate::ui::projects::Projects::default(),
            pictures: places.pictures.clone(),
            saved: crate::session::Saved::default(),
            view: View::default(),
            colour: first_colour(&pattern),
            pattern,
            tool: Tool::Pencil,
            history: History::default(),
            message: None,
            menu: crate::ui::plate_menu("settings-menu"),
            settings,
            palette: 0,
            palette_window: Default::default(),
            bead_menu: crate::ui::plate_menu("bead-menu"),
            bead_menu_of: None,
            settings_edit: None,
            backdrop: crate::backdrop::Backdrop::new(places.pictures),
            backdrop_menu: crate::ui::plate_menu("backdrop-menu"),
            picker: crate::pick::Picker::default(),
            toasts: egui_notify::Toasts::new().with_anchor(egui_notify::Anchor::BottomRight),
        };
        if let Some(error) = settings_error {
            app.projects.tell(say(&error));
        }
        match (&app.library, places.file) {
            (Some(_), Some(file)) => app.take_in_file(&file, 0.0),
            (Some(_), None) => {}
            (None, _) => app.open_pattern(new_pattern(), History::default(), 0.0),
        }
        app
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

    /// The keys of the field, while it is the layer on top (`FIELD_KEYS`).
    fn shortcuts(&mut self, ui: &mut egui::Ui) {
        if crate::keys::layer(self, ui.ctx()) != Layer::Editor {
            return;
        }
        let time = ui.input(|input| input.time);
        for taken in crate::keys::take(ui.ctx(), FIELD_KEYS) {
            match taken.action {
                FieldAction::Redo => self.redo(),
                FieldAction::Undo => self.undo(),
                FieldAction::Save => self.save_now(time),
                FieldAction::Back => {
                    self.back_to_list(ui.ctx(), time);
                    return;
                }
                FieldAction::Palettes => self.palette_window.open = true,
                FieldAction::NextPalette => crate::ui::palette_window::step(self, 1),
                FieldAction::PreviousPalette => crate::ui::palette_window::step(self, -1),
                FieldAction::Digit => self.take_by_digit(taken.index),
            }
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

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        crate::ui::access::name_window(ui.ctx(), &t!("app.name"));
        crate::keys::global(self, ui.ctx());
        if self.screen == crate::session::Screen::List {
            self.projects.thumbs.render = frame.wgpu_render_state().cloned();
            crate::ui::projects::show(self, ui);
            self.show_toasts(ui.ctx());
            return;
        }
        self.shortcuts(ui);
        if self.screen != crate::session::Screen::Editor {
            return;
        }
        egui::CentralPanel::no_frame().show(ui, |ui| self.field(ui));
        crate::ui::tools::show(self, ui.ctx());
        crate::ui::palette::show(self, ui.ctx());
        crate::ui::palette_window::show(self, ui.ctx());
        crate::ui::palette::menu(self, ui.ctx());
        crate::ui::settings_menu::show(self, ui.ctx());
        crate::ui::backdrop_menu::show(self, ui.ctx());
        crate::ui::palette_window::keep_in_hand(self);
        self.autosave(ui.ctx());
        self.show_toasts(ui.ctx());
    }

    /// Saves the pattern open when android pauses the program: eframe calls
    /// it on suspend, as on a close and every `auto_save_interval`.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_now(f64::MAX);
    }

    /// Saves the pattern open when the window is closed.
    fn on_exit(&mut self) {
        self.save_now(f64::MAX);
    }
}

/// The colour in hand at the start: the first of the first palette shown.
pub(crate) fn first_colour(pattern: &Pattern) -> Option<ColourId> {
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

/// The text of an error for the user.
pub(crate) fn say(error: &AppError) -> String {
    log::warn!("{error}: {:?}", std::error::Error::source(error));
    t!(error.message_key()).to_string()
}
