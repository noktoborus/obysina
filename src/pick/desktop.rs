//! The dialog of files of the desktop, through `rfd`: the portal of the
//! desktop on linux, the dialog of windows. It runs in a thread of its own, so
//! the window keeps drawing.

use std::sync::mpsc::Sender;

use super::{Answer, Choice, Kinds, Picked};

/// Opens the dialog in a thread; its answer goes to `sender`.
pub fn start(context: egui::Context, (title, kinds): (String, Kinds), sender: Sender<Answer>) {
    std::thread::spawn(move || {
        let dialog = rfd::AsyncFileDialog::new().add_filter(title, kinds.extensions());
        let answer = pollster::block_on(async {
            match dialog.pick_file().await {
                Some(file) => Choice::Picked(Picked {
                    name: Some(file.file_name()),
                    bytes: file.read().await,
                }),
                None => Choice::GivenUp,
            }
        });
        if sender.send(answer).is_err() {
            log::warn!("a file was chosen after the window went");
        }
        context.request_repaint();
    });
}
