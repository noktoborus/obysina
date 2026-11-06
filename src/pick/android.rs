//! The documents of android. `MainActivity.pickPicture` and `pickPatterns`
//! of `android/app` open the choice of the system; its answer comes back
//! through [`picked`], which the native method of `android/` calls.
//!
//! ```text
//! start ──JNI──► MainActivity.pickPicture | pickPatterns ──► ACTION_OPEN_DOCUMENT
//!                onActivityResult ──► picturePicked(bytes, name) ──► picked
//! ```

use std::sync::Mutex;
use std::sync::mpsc::Sender;

use jni::{jni_sig, jni_str};

use super::{Answer, Choice, Kinds, Picked};

/// The choice under way: where its answer goes, and the window to draw again.
static WAITING: Mutex<Option<(Sender<Answer>, egui::Context)>> = Mutex::new(None);

/// Asks the activity for a file of `kinds`; the answer comes through
/// [`picked`]. The title is the system's own.
pub fn start(context: egui::Context, (_, kinds): (String, Kinds), sender: Sender<Answer>) {
    *WAITING.lock().expect("the choice is never poisoned") = Some((sender, context));
    let asked = crate::activity::call(|env, activity| {
        let method = match kinds {
            Kinds::Pictures => jni_str!("pickPicture"),
            Kinds::Patterns => jni_str!("pickPatterns"),
        };
        env.call_method(activity, method, jni_sig!("()V"), &[])?;
        Ok(())
    });
    if let Err(error) = asked {
        log::error!("the choice of a file does not open: {error}");
        answer(Choice::Failed);
    }
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
            log::warn!("a file was chosen after the window went");
        }
        context.request_repaint();
    }
}
