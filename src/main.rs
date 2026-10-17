//! The program on the desktop.
//!
//! A release build for Windows is linked against the windows subsystem, so
//! starting the program from the desktop opens no console window behind it. A
//! debug build keeps the console, where the log goes.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() -> std::process::ExitCode {
    obusina::desktop()
}
