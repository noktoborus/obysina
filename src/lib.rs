//! Obusina, an editor of bead patterns.
//!
//! The program is a library with two entries: [`desktop`], which
//! `src/main.rs` calls, and `android`, which the `cdylib` of `android/` calls.

#[cfg(target_os = "android")]
mod activity;
mod app;
mod backdrop;
mod edit;
mod error;
mod history_file;
mod keys;
mod library;
mod new_colour;
mod pick;
mod render;
mod session;
mod settings;
mod task;
mod ui;
mod window_icon;

#[cfg(test)]
mod window_tests;

use std::path::PathBuf;

pub use app::App;
pub use pick::Picked;
#[cfg(target_os = "android")]
pub use pick::{android_picked, android_saved};

rust_i18n::i18n!("locales", fallback = "en");

/// The name the desktop and android know the program by (`RULE app.id`).
pub const APP_ID: &str = "ru.styxheim.obusina";

/// The directory eframe keeps its own state in, beside the settings: with
/// it, eframe calls `App::save` when android pauses the program.
const EFRAME: &str = "eframe";

/// The cache of android, beside the data of the program (`files`): the one
/// `Context.getCacheDir` names.
#[cfg(target_os = "android")]
const CACHE: &str = "cache";

/// Starts the window on the desktop on the list of patterns; a pattern file
/// named on the command line is taken in as a copy and opened.
pub fn desktop() -> std::process::ExitCode {
    env_logger::init();
    choose_locale();
    let file = std::env::args_os().nth(1).map(PathBuf::from);
    let directories = directories::ProjectDirs::from_path(PathBuf::from(APP_ID));
    let places = app::Places {
        data: directories
            .as_ref()
            .map(|dirs| dirs.data_dir().to_path_buf()),
        settings: directories
            .as_ref()
            .map(|dirs| dirs.config_dir().join(settings::FILE)),
        pictures: directories
            .as_ref()
            .map(|dirs| dirs.cache_dir().join(backdrop::DIRECTORY)),
        file,
    };
    let state = directories.map(|dirs| dirs.config_dir().join(EFRAME));
    let viewport = egui::ViewportBuilder::default()
        .with_app_id(APP_ID)
        .with_title(rust_i18n::t!("app.name"))
        .with_inner_size([1000.0, 700.0])
        .with_icon(window_icon::icon());
    let options = eframe::NativeOptions {
        persistence_path: state,
        ..render::native_options(viewport)
    };
    match run(options, places) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            log::error!("the window failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Starts the window on android, on the list of the patterns kept in the
/// data of the program.
#[cfg(target_os = "android")]
pub fn android(android: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("obusina"),
    );
    activity::keep(android.clone());
    choose_locale();
    let data = android.internal_data_path();
    let places = app::Places {
        settings: data
            .as_ref()
            .map(|directory| directory.join(settings::FILE)),
        pictures: data
            .as_ref()
            .and_then(|directory| Some(directory.parent()?.join(CACHE).join(backdrop::DIRECTORY))),
        data: data.clone(),
        file: None,
    };
    let options = eframe::NativeOptions {
        android_app: Some(android),
        persistence_path: data.map(|directory| directory.join(EFRAME)),
        ..render::native_options(egui::ViewportBuilder::default())
    };
    if let Err(error) = run(options, places) {
        log::error!("the window failed: {error}");
    }
}

/// Runs the window until it is closed, keeping what it keeps in `places`.
fn run(options: eframe::NativeOptions, places: app::Places) -> eframe::Result {
    eframe::run_native(
        "Obusina",
        options,
        Box::new(move |creation| {
            if let Some(state) = &creation.wgpu_render_state {
                render::log_adapter(state);
            }
            Ok(Box::new(App::start(places)))
        }),
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
