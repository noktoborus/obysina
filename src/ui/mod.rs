//! The parts of the window.

pub mod access;
pub mod backdrop_menu;
pub mod blocker;
pub mod button;
pub mod icons;
pub mod palette;
pub mod palette_window;
pub mod projects;
pub mod settings_menu;
pub mod sheet;
pub mod tools;

/// A menu of the window: it opens with no entry selected (`RULE ui.menu`).
pub fn plate_menu(id: &str) -> plate_menu::PlateMenu {
    plate_menu::PlateMenu::new()
        .with_id(id)
        .opens_unselected(true)
}
