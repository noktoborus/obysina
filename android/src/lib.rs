//! The library android loads. The activity calls `android_main`, which starts
//! the window of `obusina`, and `picturePicked` of `MainActivity` when the
//! person chose a picture.
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

/// The answer of the choice of a picture that `MainActivity.pickPicture`
/// opened: the bytes and the name of the file, or two nulls when the choice
/// was given up or the file could not be read.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ru_styxheim_obusina_MainActivity_picturePicked<'caller>(
    mut env: jni::EnvUnowned<'caller>,
    _class: jni::objects::JClass<'caller>,
    bytes: jni::objects::JByteArray<'caller>,
    name: jni::objects::JString<'caller>,
) {
    let outcome = env.with_env(|env| -> jni::errors::Result<Option<obusina::Picked>> {
        if bytes.is_null() {
            return Ok(None);
        }
        let bytes = env.convert_byte_array(&bytes)?;
        let name = if name.is_null() {
            None
        } else {
            Some(name.try_to_string(env)?)
        };
        Ok(Some(obusina::Picked { name, bytes }))
    });
    let answer = outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>();
    obusina::android_picked(answer);
}
