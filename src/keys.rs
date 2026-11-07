//! The keys of the window. Each layer keeps its table beside its code;
//! [`GLOBAL`] is the table of the keys of every layer.
//! [`layer`] says which layer is on top: only it acts on a key. [`keymap`]
//! gives the rows of a layer; a test writes `docs/keys.md` from them.
//!
//! ```text
//! a frame ──► take(GLOBAL) ──► layer(app) ──► the code of that layer:
//!                                             take(its table) ──► its actions
//! ```
//!
//! Nothing else reads a key: `clippy.toml` refuses `key_pressed`,
//! `consume_key` and `consume_shortcut` everywhere but here and in
//! `plate-menu` (`RULE ui.keys`).

use egui::{Context, Key, KeyboardShortcut, ModifierNames, Modifiers};

use crate::app::App;
use crate::session::Screen;
use crate::ui::projects::View;

/// A row of a table: the keys, what they do, and the key of its text in
/// `locales/app.yml`.
#[derive(Debug, Clone, Copy)]
pub struct Binding<A: 'static> {
    /// The keys, any of which does it.
    pub keys: &'static [KeyboardShortcut],
    /// What they do.
    pub action: A,
    /// The key of the text that says what they do.
    pub name: &'static str,
}

/// A row of a table pressed this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Taken<A> {
    /// What it does.
    pub action: A,
    /// Which of the keys of the row was pressed: the digit of the row of the
    /// digits, the arrow of the row of the arrows.
    pub index: usize,
}

/// A key with no modifier.
pub const fn plain(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::NONE, key)
}

/// A key with `Ctrl`, `Cmd` on a mac.
pub const fn command(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}

/// A key with `Ctrl` and `Shift`, `Cmd` and `Shift` on a mac.
pub const fn command_shift(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), key)
}

/// Takes out of the input the keys of `table` pressed this frame, in the
/// order of the table.
///
/// A key matches with any more `Shift` and `Alt` held, as egui matches it:
/// a row with more modifiers stands before a row of the same key with fewer,
/// and a test holds the tables to it.
#[expect(
    clippy::disallowed_methods,
    reason = "this is the one place the window reads a key"
)]
pub fn take<A: Copy>(context: &Context, table: &[Binding<A>]) -> Vec<Taken<A>> {
    context.input_mut(|input| {
        let mut taken = Vec::new();
        for binding in table {
            for (index, key) in binding.keys.iter().enumerate() {
                if input.consume_shortcut(key) {
                    taken.push(Taken {
                        action: binding.action,
                        index,
                    });
                }
            }
        }
        taken
    })
}

/// What a global key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Global {
    /// Shows the keys of the layer on top, or hides them.
    Help,
}

/// The keys of every layer.
pub const GLOBAL: &[Binding<Global>] = &[Binding {
    keys: &[plain(Key::F1)],
    action: Global::Help,
    name: "keys.help",
}];

/// Acts on the global keys, before any layer reads the input of the frame.
pub fn global(app: &mut App, context: &Context) {
    let on_top = layer(app, context);
    for taken in take(context, GLOBAL) {
        match taken.action {
            Global::Help => crate::ui::help::toggle(app, on_top),
        }
    }
}

/// What takes the keys: the part of the window on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// The list of patterns.
    List,
    /// The trash.
    Trash,
    /// The field of the pattern open, with nothing over it.
    Editor,
    /// The field of a locked pattern, with nothing over it.
    Viewer,
    /// The corners of the picture under the pattern, while they are edited.
    Corners,
    /// The window of the palettes.
    Palettes,
    /// A colour dragged in the window of the palettes.
    Dragging,
    /// The menu of the window of the palettes.
    PaletteMenu,
    /// Any other menu.
    Menu,
    /// A text field that holds the keys.
    Text,
    /// The plate of the keys, over everything.
    Help,
}

impl Layer {
    /// The key of the name of the layer in `locales/app.yml`: the heading of
    /// its keys on the plate of the keys and in `docs/keys.md`.
    pub fn title(self) -> &'static str {
        match self {
            Layer::List => "keys.layer.list",
            Layer::Trash => "keys.layer.trash",
            Layer::Editor => "keys.layer.editor",
            Layer::Viewer => "keys.layer.viewer",
            Layer::Corners => "keys.layer.corners",
            Layer::Palettes => "keys.layer.palettes",
            Layer::Dragging => "keys.layer.dragging",
            Layer::PaletteMenu => "keys.layer.palette_menu",
            Layer::Menu => "keys.layer.menu",
            Layer::Text => "keys.layer.text",
            Layer::Help => "keys.layer.help",
        }
    }
}

/// The layer on top.
///
/// The plate of the keys stands over everything. A menu stands over a text
/// field: the field of a menu takes the keys through the menu. The menu of
/// the window of the palettes stands over both, so Escape closes it while
/// its picker holds the keys.
pub fn layer(app: &App, context: &Context) -> Layer {
    let text = context.egui_wants_keyboard_input();
    if app.help.is_some() {
        return Layer::Help;
    }
    match app.screen {
        Screen::List if app.projects.menu_open() => Layer::Menu,
        Screen::List if text => Layer::Text,
        Screen::List if app.projects.view == View::Trash => Layer::Trash,
        Screen::List => Layer::List,
        Screen::Editor => editor_layer(app, text),
    }
}

/// The layer on top of the editor; `text` when a text field holds the keys.
fn editor_layer(app: &App, text: bool) -> Layer {
    let window = &app.palette_window;
    if window.open && window.menu_open() {
        Layer::PaletteMenu
    } else if app.bead_menu.is_open() || app.menu.is_open() || app.backdrop_menu.is_open() {
        Layer::Menu
    } else if text {
        Layer::Text
    } else if window.open && window.dragging() {
        Layer::Dragging
    } else if window.open {
        Layer::Palettes
    } else if app.backdrop.editing() {
        Layer::Corners
    } else if app.pattern.locked() {
        Layer::Viewer
    } else {
        Layer::Editor
    }
}

/// A row of a table, whatever its action: what `docs/keys.md` and the tests
/// read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The keys, any of which does it.
    pub keys: Vec<KeyboardShortcut>,
    /// The key of the text that says what they do.
    pub name: &'static str,
}

/// The rows of a table.
pub fn rows<A>(table: &[Binding<A>]) -> Vec<Row> {
    table
        .iter()
        .map(|binding| Row {
            keys: binding.keys.to_vec(),
            name: binding.name,
        })
        .collect()
}

/// The rows of `plate-menu`: the keys every menu is walked by.
fn menu_rows() -> Vec<Row> {
    plate_menu::KEYS
        .iter()
        .map(|(key, walk)| Row {
            keys: vec![plain(*key)],
            name: walk_name(*walk),
        })
        .collect()
}

/// The key of the text of what a key of a menu does.
fn walk_name(walk: plate_menu::Walk) -> &'static str {
    use plate_menu::Walk;
    match walk {
        Walk::Close => "keys.menu.close",
        Walk::Down => "keys.menu.down",
        Walk::Up => "keys.menu.up",
        Walk::StepIn => "keys.menu.step_in",
        Walk::StepOut => "keys.menu.step_out",
        Walk::PageDown => "keys.menu.page_down",
        Walk::PageUp => "keys.menu.page_up",
        Walk::First => "keys.menu.first",
        Walk::Last => "keys.menu.last",
        Walk::Erase => "keys.menu.erase",
        Walk::Choose => "keys.menu.choose",
        Walk::Stay => "keys.menu.stay",
    }
}

/// `own` over the rows of a menu: a key of `own` takes the place of the
/// same key of the menu.
fn over_menu(own: Vec<Row>) -> Vec<Row> {
    let taken: Vec<KeyboardShortcut> = own.iter().flat_map(|row| row.keys.clone()).collect();
    let mut rows = own;
    rows.extend(
        menu_rows()
            .into_iter()
            .filter(|row| !row.keys.iter().any(|key| taken.contains(key))),
    );
    rows
}

/// The rows of `layer`, without the global ones.
pub fn keymap(layer: Layer) -> Vec<Row> {
    match layer {
        Layer::List => rows(crate::ui::projects::LIST_KEYS),
        Layer::Trash => rows(crate::ui::projects::TRASH_KEYS),
        Layer::Editor => rows(crate::app::FIELD_KEYS),
        Layer::Viewer => rows(crate::app::VIEW_KEYS),
        Layer::Corners | Layer::Dragging | Layer::Text => Vec::new(),
        Layer::Palettes => rows(crate::ui::palette_window::KEYS),
        Layer::PaletteMenu => over_menu(rows(crate::ui::palette_window::MENU_KEYS)),
        Layer::Menu => menu_rows(),
        Layer::Help => rows(crate::ui::help::KEYS),
    }
}

/// The keys of a row as the plate of the keys and `docs/keys.md` write
/// them: a run of more than `RUN` keys by its first and its last.
pub fn written_keys(keys: &[KeyboardShortcut]) -> String {
    match keys {
        [first, .., last] if keys.len() > RUN => {
            format!("{} \u{2026} {}", written(first), written(last))
        }
        _ => keys.iter().map(written).collect::<Vec<_>>().join(", "),
    }
}

/// The most keys of a row written one by one.
const RUN: usize = 4;

/// A key as it is written, the way of a desktop that is not a mac.
pub fn written(key: &KeyboardShortcut) -> String {
    let held = ModifierNames::NAMES.format(&key.modifiers, false);
    match held.is_empty() {
        true => key.logical_key.name().to_string(),
        false => format!("{held}+{}", key.logical_key.name()),
    }
}

#[cfg(test)]
mod document;
#[cfg(test)]
mod tests;
