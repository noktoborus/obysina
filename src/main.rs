//! The program on the desktop, or on the page of a browser.
//!
//! A release build for Windows is linked against the windows subsystem, so
//! starting the program from the desktop opens no console window behind it. A
//! debug build keeps the console, where the log goes.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    obusina::desktop()
}

/// Starts the window on the canvas `obusina` of `index.html`.
#[cfg(target_arch = "wasm32")]
fn main() {
    obusina::web("obusina");
}
