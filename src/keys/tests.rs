//! The tables of the keys and `docs/keys.md`.

use super::document::document;
use super::*;

/// The file the tables are written to.
const DOCUMENT: &str = "docs/keys.md";

/// The rows a layer acts on: the global ones, then its own.
fn acting(layer: Layer) -> Vec<Row> {
    let mut rows = super::rows(GLOBAL);
    rows.extend(keymap(layer));
    rows
}

/// True when a press of `pressed` is taken by a row of `row`, as egui
/// matches: `Shift` and `Alt` held beyond the row do not stop it.
fn takes(row: &KeyboardShortcut, pressed: &KeyboardShortcut) -> bool {
    row.logical_key == pressed.logical_key && pressed.modifiers.matches_logically(row.modifiers)
}

#[test]
fn no_key_does_two_things_in_one_layer() {
    for layer in Layer::ALL {
        let keys: Vec<KeyboardShortcut> =
            acting(layer).into_iter().flat_map(|row| row.keys).collect();
        for (at, key) in keys.iter().enumerate() {
            assert!(
                !keys[at + 1..].contains(key),
                "{layer:?}: {} stands twice",
                written(key)
            );
        }
    }
}

/// `Ctrl+Z` before `Ctrl+Shift+Z` would take the redo as an undo.
#[test]
fn no_row_takes_the_key_of_a_row_after_it() {
    for layer in Layer::ALL {
        let keys: Vec<KeyboardShortcut> =
            acting(layer).into_iter().flat_map(|row| row.keys).collect();
        for (at, before) in keys.iter().enumerate() {
            for after in &keys[at + 1..] {
                assert!(
                    !takes(before, after),
                    "{layer:?}: {} takes {}; it stands after it",
                    written(before),
                    written(after)
                );
            }
        }
    }
}

#[test]
fn every_row_has_a_text_in_both_languages() {
    for layer in Layer::ALL {
        for row in acting(layer) {
            for locale in ["en", "ru"] {
                let text = rust_i18n::t!(row.name, locale = locale);
                assert_ne!(
                    text, row.name,
                    "{layer:?}: no {locale} text for {}",
                    row.name
                );
            }
        }
    }
}

/// `docs/keys.md` is what the tables say. `UPDATE_KEYS=1` writes it.
#[test]
fn the_document_is_written_from_the_tables() {
    let text = document();
    if std::env::var_os("UPDATE_KEYS").is_some() {
        std::fs::write(DOCUMENT, &text).expect("the document is written");
        return;
    }
    let kept = std::fs::read_to_string(DOCUMENT).expect("the document is read");
    assert!(
        kept == text,
        "{DOCUMENT} is not what the tables say: run UPDATE_KEYS=1 cargo test -p obusina keys"
    );
}

#[test]
fn a_key_is_written_with_its_modifiers() {
    assert_eq!(written(&command_shift(Key::Z)), "Ctrl+Shift+Z");
    assert_eq!(written(&plain(Key::Escape)), "Escape");
}

#[test]
fn a_long_run_of_keys_is_written_by_its_ends() {
    let digits = [
        plain(Key::Num0),
        plain(Key::Num1),
        plain(Key::Num2),
        plain(Key::Num3),
        plain(Key::Num9),
    ];
    assert_eq!(written_keys(&digits), "0 \u{2026} 9");
    assert_eq!(written_keys(&digits[..2]), "0, 1");
}

#[test]
fn every_layer_has_a_title_in_both_languages() {
    for layer in Layer::ALL {
        for locale in ["en", "ru"] {
            let text = rust_i18n::t!(layer.title(), locale = locale);
            assert_ne!(text, layer.title(), "no {locale} title for {layer:?}");
        }
    }
}
