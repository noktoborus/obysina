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

/// The answer of a choice of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// The file the person chose.
    Picked(Picked),
    /// The person gave the choice up.
    GivenUp,
    /// The choice could not be asked of the platform.
    Failed,
}

/// What a choice answers by.
type Answer = Choice;

/// A choice of a file under way.
#[derive(Default)]
pub struct Picker {
    answer: Option<Receiver<Answer>>,
    /// A test asks for no dialog: a choice is only noted, and its answer
    /// never comes.
    #[cfg(test)]
    held: Option<Option<Sender<Answer>>>,
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
        #[cfg(test)]
        if let Some(held) = &mut self.held {
            *held = Some(sender);
            return;
        }
        start(context.clone(), title.to_owned(), sender);
    }

    /// Opens no dialog from now on: a test notes the choice only.
    #[cfg(test)]
    pub fn hold(&mut self) {
        self.held = Some(None);
    }

    /// True while a choice is under way.
    #[cfg(test)]
    pub fn asked(&self) -> bool {
        self.answer.is_some()
    }

    /// The answer, once it has come; `None` while the choice is under way, or
    /// when none was asked.
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
                Some(Choice::Failed)
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
