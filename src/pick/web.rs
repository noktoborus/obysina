//! The file input of the browser, through `rfd`: it answers in a future,
//! which the browser runs.

use std::sync::mpsc::Sender;

use super::{Answer, Choice, Kinds, Picked};

/// Opens the input of files; its answer goes to `sender`.
pub fn start(context: egui::Context, (title, kinds): (String, Kinds), sender: Sender<Answer>) {
    wasm_bindgen_futures::spawn_local(async move {
        let dialog = rfd::AsyncFileDialog::new().add_filter(title, kinds.extensions());
        let answer = match dialog.pick_file().await {
            Some(file) => Choice::Picked(Picked {
                name: Some(file.file_name()),
                bytes: file.read().await,
            }),
            None => Choice::GivenUp,
        };
        if sender.send(answer).is_err() {
            log::warn!("a picture was chosen after the window went");
        }
        context.request_repaint();
    });
}
