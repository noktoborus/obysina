//! The activity of android, kept for what needs it: the choice of a file,
//! and the part of the window the system bars and the cut out of the camera
//! leave free.
//!
//! `android_main` may run more than once in the life of the program, each time
//! with a new `AndroidApp`; the last one is kept.

use std::sync::Mutex;

use winit::platform::android::activity::AndroidApp;

/// The `AndroidApp` of the run of `android_main` under way.
static APP: Mutex<Option<AndroidApp>> = Mutex::new(None);

/// Keeps the `AndroidApp` of this run of `android_main`.
pub fn keep(app: AndroidApp) {
    *APP.lock().expect("the activity is never poisoned") = Some(app);
}

/// Calls `work` with the `AndroidApp` kept; `None` before one is.
pub fn with<T>(work: impl FnOnce(&AndroidApp) -> T) -> Option<T> {
    APP.lock()
        .expect("the activity is never poisoned")
        .as_ref()
        .map(work)
}

/// The insets of the part of the window the system leaves free, in points:
/// what the content rectangle of the activity leaves out of the window of
/// `raw_input`. egui lays the panels out inside it (`Context::content_rect`).
pub fn safe_area(raw_input: &egui::RawInput) -> Option<egui::SafeAreaInsets> {
    let points = raw_input.screen_rect?;
    let scale = raw_input.viewport().native_pixels_per_point?;
    let free = with(AndroidApp::content_rect)?;
    let window = points.size() * scale;
    let inset = |pixels: f32| (pixels / scale).max(0.0);
    Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
        left: inset(free.left as f32),
        top: inset(free.top as f32),
        right: inset(window.x - free.right as f32),
        bottom: inset(window.y - free.bottom as f32),
    }))
}
