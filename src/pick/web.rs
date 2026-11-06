//! The file input of the browser, through `rfd`: it answers in a future,
//! which the browser runs.

use std::sync::mpsc::Sender;

use obusina_backdrop::Kind;

use super::{Answer, Picked};

/// Opens the input of files; its answer goes to `sender`.
pub fn start(context: egui::Context, title: String, sender: Sender<Answer>) {
    wasm_bindgen_futures::spawn_local(async move {
        let dialog = rfd::AsyncFileDialog::new().add_filter(title, &Kind::EXTENSIONS);
        let answer = match dialog.pick_file().await {
            Some(file) => Some(Picked {
                name: Some(file.file_name()),
                bytes: file.read().await,
            }),
            None => None,
        };
        if sender.send(answer).is_err() {
            log::warn!("a picture was chosen after the window went");
        }
        context.request_repaint();
    });
}
