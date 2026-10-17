//! Obusina, an editor of bead patterns.
//!
//! The program is a library with two entries: [`desktop`], which
//! `src/main.rs` calls, and `android`, which the `cdylib` of `android/` calls.

mod app;
mod edit;
mod error;
mod ui;

#[cfg(test)]
mod window_tests;

use std::path::PathBuf;

pub use app::App;

rust_i18n::i18n!("locales", fallback = "en");

/// The name the desktop and android know the program by (`RULE app.id`).
pub const APP_ID: &str = "ru.styxheim.obusina";

/// The name of the pattern android keeps in the data of the program.
#[cfg(target_os = "android")]
const DEFAULT_FILE: &str = "pattern.json";

/// Starts the window on the desktop with the pattern named on the command
/// line, if any.
pub fn desktop() -> std::process::ExitCode {
    env_logger::init();
    choose_locale();
    let path = std::env::args_os().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_title(rust_i18n::t!("app.name"))
            .with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    match run(options, path) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            log::error!("the window failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Starts the window on android, with the pattern kept in the data of the
/// program.
#[cfg(target_os = "android")]
pub fn android(android: winit::platform::android::activity::AndroidApp) {
    choose_locale();
    let path = android
        .internal_data_path()
        .map(|directory| directory.join(DEFAULT_FILE));
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(error) = run(options, path) {
        log::error!("the window failed: {error}");
    }
}

/// Runs the window until it is closed.
fn run(options: eframe::NativeOptions, path: Option<PathBuf>) -> eframe::Result {
    eframe::run_native(
        "Obusina",
        options,
        Box::new(move |_| Ok(Box::new(App::open(path)))),
    )
}

/// Speaks the language of the system when there is a translation for it.
fn choose_locale() {
    let system = sys_locale::get_locale().unwrap_or_default();
    let language = system.split(['-', '_']).next().unwrap_or_default();
    if rust_i18n::available_locales!()
        .iter()
        .any(|known| known == language)
    {
        rust_i18n::set_locale(language);
    }
}
