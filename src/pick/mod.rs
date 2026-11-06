//! The choice of a picture file, the way each platform has: a dialog of the
//! desktop, the documents of android, a file input of the browser. The
//! answer comes in a later frame.

use std::sync::mpsc::{Receiver, Sender, channel};

#[cfg(target_os = "android")]
mod android;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_os = "android")]
pub use android::picked as android_picked;

/// A file the person chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    /// The name of the file, when the platform gives one.
    pub name: Option<String>,
    /// Its bytes.
    pub bytes: Vec<u8>,
}

/// The answer of a choice: a file, or nothing when it was given up.
type Answer = Option<Picked>;

/// A choice of a file under way.
#[derive(Default)]
pub struct Picker {
    answer: Option<Receiver<Answer>>,
}

impl Picker {
    /// Asks the person for a picture, unless a choice is under way. `title`
    /// names the kinds of picture in the dialog.
    pub fn start(&mut self, context: &egui::Context, title: &str) {
        if self.answer.is_some() {
            return;
        }
        let (sender, receiver) = channel();
        self.answer = Some(receiver);
        start(context.clone(), title.to_owned(), sender);
    }

    /// The answer, once it has come: a file, or `None` when the choice was
    /// given up. `None` while it is under way, or when none was asked.
    pub fn take(&mut self) -> Option<Answer> {
        let answer = self.answer.as_ref()?.try_recv();
        match answer {
            Ok(answer) => {
                self.answer = None;
                Some(answer)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.answer = None;
                Some(None)
            }
        }
    }
}

/// Starts the choice of the platform; the answer goes to `sender`, and the
/// window is asked to draw again.
fn start(context: egui::Context, title: String, sender: Sender<Answer>) {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    desktop::start(context, title, sender);
    #[cfg(target_arch = "wasm32")]
    web::start(context, title, sender);
    #[cfg(target_os = "android")]
    android::start(context, title, sender);
}
