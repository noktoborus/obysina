//! The dialog of files of the desktop, through `rfd`: the portal of the
//! desktop on linux, the dialog of windows. It runs in a thread of its own, so
//! the window keeps drawing.

use std::sync::mpsc::Sender;

use obusina_backdrop::Kind;

use super::{Answer, Picked};

/// Opens the dialog in a thread; its answer goes to `sender`.
pub fn start(context: egui::Context, title: String, sender: Sender<Answer>) {
    std::thread::spawn(move || {
        let dialog = rfd::AsyncFileDialog::new().add_filter(title, &Kind::EXTENSIONS);
        let answer = pollster::block_on(async {
            let file = dialog.pick_file().await?;
            Some(Picked {
                name: Some(file.file_name()),
                bytes: file.read().await,
            })
        });
        if sender.send(answer).is_err() {
            log::warn!("a picture was chosen after the window went");
        }
        context.request_repaint();
    });
}
