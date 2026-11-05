//! Icons of the interface, one code point each (`RULE icon.glyph`).
//!
//! Each is taken from the list `cargo run -p glyphs` writes, from a face of
//! the proportional family. The test below fails if no font carries one.

/// Undo, the arrow bending back to the left.
pub const UNDO: &str = "\u{21a9}";
/// Redo, the arrow bending on to the right.
pub const REDO: &str = "\u{21aa}";
/// A quarter turn of the field counter clockwise, the open circle arrow.
///
/// Undo and redo are the bent arrows so that these two, turning, are not
/// read as them.
pub const TURN_LEFT: &str = "\u{21ba}";
/// A quarter turn of the field clockwise, the open circle arrow.
pub const TURN_RIGHT: &str = "\u{21bb}";
/// The pencil, which paints one bead at a time.
pub const PENCIL: &str = "\u{270f}";
/// The picker, which takes the colour of a bead.
pub const PICK: &str = "\u{1f4a7}";
/// The other palette, of two: the arrow both ways.
pub const SWITCH: &str = "\u{2194}";
/// Save, the floppy disk.
pub const SAVE: &str = "\u{1f4be}";
/// The kind of grid, the square cut in four.
pub const GRID: &str = "\u{229e}";

/// Every icon, for the test that the fonts carry them all.
#[cfg(test)]
pub const ALL: &[&str] = &[
    UNDO, REDO, TURN_LEFT, TURN_RIGHT, PENCIL, PICK, SWITCH, SAVE, GRID,
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
