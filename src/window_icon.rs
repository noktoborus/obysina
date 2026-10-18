//! The icon of the window on the desktop, drawn by `build.rs` from
//! `assets/icon.svg`.
//!
//! Without it eframe gives the window the logo of egui. On Windows the taskbar
//! and the title bar show the icon of the window, not the one of the file.

/// The icon, as a PNG.
const PNG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.png"));

/// The icon of the window.
pub fn icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(PNG).expect("build.rs writes a PNG")
}
