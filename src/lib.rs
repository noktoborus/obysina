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
mod new_colour;
mod pick;
mod render;
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
pub use pick::android_picked;

rust_i18n::i18n!("locales", fallback = "en");

/// The name the desktop and android know the program by (`RULE app.id`).
pub const APP_ID: &str = "ru.styxheim.obusina";

/// The name of the pattern android keeps in the data of the program.
#[cfg(target_os = "android")]
const DEFAULT_FILE: &str = "pattern.json";

/// The cache of android, beside the data of the program (`files`): the one
/// `Context.getCacheDir` names.
#[cfg(target_os = "android")]
const CACHE: &str = "cache";

/// Starts the window on the desktop with the pattern named on the command
/// line, if any.
pub fn desktop() -> std::process::ExitCode {
    env_logger::init();
    choose_locale();
    let path = std::env::args_os().nth(1).map(PathBuf::from);
    let directories = directories::ProjectDirs::from_path(PathBuf::from(APP_ID));
    let settings = directories
        .as_ref()
        .map(|directories| directories.config_dir().join(settings::FILE));
    let pictures = directories.map(|directories| directories.cache_dir().join(backdrop::DIRECTORY));
    let viewport = egui::ViewportBuilder::default()
        .with_app_id(APP_ID)
        .with_title(rust_i18n::t!("app.name"))
        .with_inner_size([1000.0, 700.0])
        .with_icon(window_icon::icon());
    let options = render::native_options(viewport);
    match run(options, path, settings, pictures) {
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
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("obusina"),
    );
    activity::keep(android.clone());
    choose_locale();
    let data = android.internal_data_path();
    let path = data.as_ref().map(|directory| directory.join(DEFAULT_FILE));
    let settings = data
        .as_ref()
        .map(|directory| directory.join(settings::FILE));
    let pictures =
        data.and_then(|directory| Some(directory.parent()?.join(CACHE).join(backdrop::DIRECTORY)));
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..render::native_options(egui::ViewportBuilder::default())
    };
    if let Err(error) = run(options, path, settings, pictures) {
        log::error!("the window failed: {error}");
    }
}

/// Runs the window until it is closed. `pictures` is the store of the
/// pictures under the patterns, in the cache.
fn run(
    options: eframe::NativeOptions,
    path: Option<PathBuf>,
    settings: Option<PathBuf>,
    pictures: Option<PathBuf>,
) -> eframe::Result {
    eframe::run_native(
        "Obusina",
        options,
        Box::new(move |creation| {
            if let Some(state) = &creation.wgpu_render_state {
                render::log_adapter(state);
            }
            Ok(Box::new(App::open(path, settings, pictures)))
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
