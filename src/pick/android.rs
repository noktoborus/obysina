//! The documents of android. `MainActivity.pickPicture` of `android/app`
//! opens the choice of the system; its answer comes back through
//! [`picked`], which the native method of `android/` calls.
//!
//! ```text
//! start ──JNI──► MainActivity.pickPicture ──► ACTION_OPEN_DOCUMENT
//!                onActivityResult ──► picturePicked(bytes, name) ──► picked
//! ```

use std::sync::Mutex;
use std::sync::mpsc::Sender;

use jni::objects::JObject;
use jni::{jni_sig, jni_str};

use super::{Answer, Picked};

/// The choice under way: where its answer goes, and the window to draw again.
static WAITING: Mutex<Option<(Sender<Answer>, egui::Context)>> = Mutex::new(None);

/// Asks the activity for a picture; the answer comes through [`picked`]. The
/// title is the system's own.
pub fn start(context: egui::Context, _title: String, sender: Sender<Answer>) {
    *WAITING.lock().expect("the choice is never poisoned") = Some((sender, context));
    if let Err(error) = ask_activity() {
        log::error!("the choice of a picture does not open: {error}");
        picked(None);
    }
}

/// Calls `pickPicture` of the activity.
fn ask_activity() -> jni::errors::Result<()> {
    let android = ndk_context::android_context();
    // SAFETY: android-activity keeps the virtual machine and the activity of
    // ndk-context for the life of the program.
    let vm = unsafe { jni::JavaVM::from_raw(android.vm().cast()) };
    vm.attach_current_thread(|env| {
        // SAFETY: the activity of ndk-context is a global reference.
        let activity = unsafe { JObject::from_raw(env, android.context().cast()) };
        env.call_method(&activity, jni_str!("pickPicture"), jni_sig!("()V"), &[])?;
        Ok(())
    })
}

/// The answer of the activity: a file, or `None` when the person gave the
/// choice up. Called by the native method of `android/`.
pub fn picked(answer: Option<Picked>) {
    let waiting = WAITING.lock().expect("the choice is never poisoned").take();
    if let Some((sender, context)) = waiting {
        if sender.send(answer).is_err() {
            log::warn!("a picture was chosen after the window went");
        }
        context.request_repaint();
    }
}
