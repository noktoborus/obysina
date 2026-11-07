//! `docs/keys.md`, written from the tables by a test.

use super::{GLOBAL, Layer, Row, keymap, rows, written_keys};

impl Layer {
    /// Every layer, in the order of `docs/keys.md`.
    pub const ALL: [Layer; 10] = [
        Layer::List,
        Layer::Trash,
        Layer::Editor,
        Layer::Corners,
        Layer::Palettes,
        Layer::Dragging,
        Layer::PaletteMenu,
        Layer::Menu,
        Layer::Text,
        Layer::Help,
    ];

    /// When the layer is on top, in `docs/keys.md`.
    pub fn about(self) -> &'static str {
        match self {
            Layer::List => {
                "The list is shown, no menu is open, no field holds the keys. \
                 A letter typed searches, as Ctrl+F does; that is text, not a key."
            }
            Layer::Trash => "The trash is shown, no menu is open.",
            Layer::Editor => {
                "A pattern is open, nothing stands over the field, no field holds the keys."
            }
            Layer::Corners => {
                "The corners of the picture under the pattern are edited; \
                 the buttons end it."
            }
            Layer::Palettes => "The window of the palettes is open, its menu is not.",
            Layer::Dragging => "A colour is dragged in the window of the palettes.",
            Layer::PaletteMenu => {
                "The menu of a place of the window of the palettes. Escape closes it \
                 at once, with a query typed or the picker holding the keys; the \
                 other keys are those of a menu."
            }
            Layer::Menu => {
                "Any other menu: the settings, a colour, the picture, a card of the \
                 list. The keys are those of `plate-menu`. A menu opens with no entry \
                 selected: Down selects the first, Up the last, Enter does nothing."
            }
            Layer::Text => "A text field holds the keys. It takes all but the global ones.",
            Layer::Help => {
                "The plate of the keys of the layer under it, shown by F1. It takes \
                 every key. A tap on it or beside it closes it."
            }
        }
    }
}

/// The text of `docs/keys.md`, from the tables.
pub fn document() -> String {
    let mut text = String::from(
        "# Keys\n\n\
         <!-- Written from the tables by `UPDATE_KEYS=1 cargo test -p obusina keys`. \
         Not edited by hand. -->\n\n\
         The table of each layer stands beside its code. `src/keys.rs` says which \
         layer is on top; only that layer acts on a key. Every key is a second way \
         to an action (`RULE ui.touch`).\n\n\
         ## Global keys\n\n\
         They act in every layer.\n\n",
    );
    push_table(&mut text, &rows(GLOBAL));
    for layer in Layer::ALL {
        let title = rust_i18n::t!(layer.title(), locale = "en");
        text.push_str(&format!("## {title}\n\n{}\n\n", layer.about()));
        push_table(&mut text, &keymap(layer));
    }
    text
}

/// A table of rows, or a line that there is none.
fn push_table(text: &mut String, rows: &[Row]) {
    if rows.is_empty() {
        text.push_str("No key.\n\n");
        return;
    }
    text.push_str("| Key | Action |\n| --- | --- |\n");
    for row in rows {
        let name = rust_i18n::t!(row.name, locale = "en");
        text.push_str(&format!("| {} | {name} |\n", written_keys(&row.keys)));
    }
    text.push('\n');
}
