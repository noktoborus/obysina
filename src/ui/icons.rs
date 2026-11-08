//! Icons of the interface, one code point each (`RULE icon.glyph`).
//!
//! Each is taken from the list `cargo run -p glyphs` writes, from a face of
//! the proportional family. The test below fails if no font carries one.

/// Undo, the arrow bending back to the left.
pub const UNDO: &str = "\u{21a9}";
/// Redo, the arrow bending on to the right.
pub const REDO: &str = "\u{21aa}";
/// The other palette, of two: the arrow both ways.
pub const SWITCH: &str = "\u{2194}";
/// The way back to the list of patterns, the arrow to the left.
pub const BACK: &str = "\u{2b05}";
/// A new pattern, the plus.
pub const NEW: &str = "+";
/// The import of a file of patterns, the inbox tray.
pub const IMPORT: &str = "\u{1f4e5}";
/// The export to a file, the outbox tray.
pub const EXPORT: &str = "\u{1f4e4}";
/// The trash, the wastebasket.
pub const TRASH: &str = "\u{1f5d1}";
/// The settings of the pattern and of the view, the gear.
pub const SETTINGS: &str = "\u{2699}";
/// The window of the palettes: a palette of a painter.
pub const PALETTE: &str = "\u{1f3a8}";

/// The shadows of the empty places, the dark shade.
pub const SHADOW: &str = "\u{2593}";

/// The picture under the pattern, the framed picture.
pub const PICTURE: &str = "\u{1f5bc}";
/// Keeps what was edited, the heavy check mark.
pub const DONE: &str = "\u{2714}";
/// Puts back what was edited, the heavy multiplication sign.
pub const CANCEL: &str = "\u{2716}";

/// A pattern locked against changes, the closed lock; drawn red left of the
/// name of its card.
pub const LOCK: &str = "\u{1f512}";
/// Lets the patterns chosen change again, the open lock.
pub const UNLOCK: &str = "\u{1f513}";
/// The menu of the patterns chosen, the trigram for heaven.
pub const MENU: &str = "\u{2630}";

/// The empty place `0`, no colour, which takes beads away. The fonts of the
/// toolkit carry no eraser; the no entry sign is the nearest.
pub const EMPTY: &str = "\u{1f6ab}";

/// Every icon, for the test that the fonts carry them all.
#[cfg(test)]
pub const ALL: &[&str] = &[
    UNDO, REDO, SWITCH, BACK, NEW, IMPORT, EXPORT, TRASH, SETTINGS, SHADOW, PICTURE, DONE, CANCEL,
    EMPTY, LOCK, UNLOCK, MENU, PALETTE,
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The icons of a list that no face of the family has a glyph for.
    fn missing<'a>(family: egui::FontFamily, icons: &'a [&'a str]) -> Vec<&'a str> {
        let context = egui::Context::default();
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            ui.label("");
        });
        output.textures_delta.clear();

        context.fonts_mut(move |fonts| {
            let carried = fonts.characters(&family).clone();
            icons
                .iter()
                .copied()
                .filter(|icon| icon.chars().any(|point| !carried.contains_key(&point)))
                .collect()
        })
    }

    #[test]
    fn every_icon_is_in_the_fonts_the_toolkit_ships_with() {
        let missing = missing(egui::FontFamily::Proportional, ALL);

        assert!(
            missing.is_empty(),
            "no font of the toolkit carries these: {missing:?}"
        );
    }

    #[test]
    fn a_code_point_no_font_carries_is_named() {
        let missing = missing(egui::FontFamily::Proportional, &["\u{e000}"]);

        assert_eq!(missing, ["\u{e000}"]);
    }
}
