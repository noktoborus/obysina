//! The library android loads. The activity calls `android_main`, which starts
//! the window of `obusina`.
//!
//! On any other platform the library is empty: a `cdylib` of the whole
//! program would export every symbol, and the linker of mingw refuses that
//! many.

/// Called by the game activity of android once the program is loaded.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    obusina::android(app);
}
