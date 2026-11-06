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

use super::{Answer, Choice, Picked};

/// The choice under way: where its answer goes, and the window to draw again.
static WAITING: Mutex<Option<(Sender<Answer>, egui::Context)>> = Mutex::new(None);

/// Asks the activity for a picture; the answer comes through [`picked`]. The
/// title is the system's own.
pub fn start(context: egui::Context, _title: String, sender: Sender<Answer>) {
    *WAITING.lock().expect("the choice is never poisoned") = Some((sender, context));
    if let Err(error) = ask_activity() {
        log::error!("the choice of a picture does not open: {error}");
        answer(Choice::Failed);
    }
}

/// Calls `pickPicture` of the activity. The activity is the one of the
/// `AndroidApp` of this run of `android_main`; the context of ndk-context is
/// the application, which has no such method.
fn ask_activity() -> jni::errors::Result<()> {
    let Some((vm, activity)) =
        crate::activity::with(|app| (app.vm_as_ptr(), app.activity_as_ptr()))
    else {
        return Err(jni::errors::Error::NullPtr("no activity"));
    };
    // SAFETY: android-activity keeps the virtual machine for the life of the
    // program.
    let vm = unsafe { jni::JavaVM::from_raw(vm.cast()) };
    vm.attach_current_thread(|env| {
        // SAFETY: the activity of android-activity is a global reference it
        // owns, alive while its `AndroidApp` is; it is not deleted here.
        let activity = unsafe { JObject::from_raw(env, activity.cast()) };
        env.call_method(&activity, jni_str!("pickPicture"), jni_sig!("()V"), &[])?;
        Ok(())
    })
}

/// The answer of the activity: a file, or `None` when the person gave the
/// choice up or the file could not be read. Called by the native method of
/// `android/`.
pub fn picked(picked: Option<Picked>) {
    answer(picked.map_or(Choice::GivenUp, Choice::Picked));
}

/// Sends the answer of the choice under way, and asks the window to draw.
fn answer(answer: Answer) {
    let waiting = WAITING.lock().expect("the choice is never poisoned").take();
    if let Some((sender, context)) = waiting {
        if sender.send(answer).is_err() {
            log::warn!("a picture was chosen after the window went");
        }
        context.request_repaint();
    }
}
