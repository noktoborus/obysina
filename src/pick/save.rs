//! The place a file is saved to: a dialog of the desktop, a new document of
//! android, a download of the browser. The bytes go with the question; the
//! answer, saved or not, comes in a later frame.
//!
//! ```text
//! desktop  Saver::start ──► the dialog of rfd, in a thread ──► the file written
//! android  Saver::start ──JNI──► MainActivity.saveDocument(name, bytes)
//!                                ACTION_CREATE_DOCUMENT ──► written there
//!                                ──► fileSaved(done) ──► android_saved
//! browser  Saver::start ──► a download into the directory of downloads:
//!                            a Blob, a link `<a download>` clicked
//! ```
//!
//! The browser does not say whether the download was kept, so its answer is
//! `Downloaded`, which tells the person nothing: the browser shows the
//! download itself.

use std::sync::mpsc::{Receiver, Sender, channel};

/// The answer of a save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saved {
    /// The file was written.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Done,
    /// The browser was given the file to download; it shows that itself.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Downloaded,
    /// The person gave the save up.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    GivenUp,
    /// The file could not be written, or the dialog not opened.
    Failed,
}

/// A save under way.
#[derive(Default)]
pub struct Saver {
    answer: Option<Receiver<Saved>>,
    /// A test asks for no dialog: what would be saved is kept here.
    #[cfg(test)]
    held: Option<Vec<(String, Vec<u8>)>>,
}

impl Saver {
    /// Asks the person where to save `bytes`, offering `name`, unless a save
    /// is under way.
    pub fn start(&mut self, context: &egui::Context, name: &str, bytes: Vec<u8>) {
        if self.answer.is_some() {
            return;
        }
        let (sender, receiver) = channel();
        #[cfg(test)]
        if let Some(held) = &mut self.held {
            held.push((name.to_owned(), bytes));
            let _ = sender.send(Saved::Done);
            self.answer = Some(receiver);
            return;
        }
        self.answer = Some(receiver);
        start(context.clone(), name.to_owned(), bytes, sender);
    }

    /// Opens no dialog from now on: a test keeps what would be saved.
    #[cfg(test)]
    pub fn hold(&mut self) {
        self.held = Some(Vec::new());
    }

    /// The files a test kept, by the name offered.
    #[cfg(test)]
    pub fn kept(&self) -> &[(String, Vec<u8>)] {
        self.held.as_deref().unwrap_or_default()
    }

    /// The answer, once it has come.
    pub fn take(&mut self) -> Option<Saved> {
        let answer = self.answer.as_ref()?.try_recv();
        match answer {
            Ok(answer) => {
                self.answer = None;
                Some(answer)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.answer = None;
                Some(Saved::Failed)
            }
        }
    }
}

/// Starts the save of the platform.
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn start(context: egui::Context, name: String, bytes: Vec<u8>, sender: Sender<Saved>) {
    std::thread::spawn(move || {
        let dialog = rfd::AsyncFileDialog::new().set_file_name(&name);
        let answer = pollster::block_on(async {
            let Some(file) = dialog.save_file().await else {
                return Saved::GivenUp;
            };
            match file.write(&bytes).await {
                Ok(()) => Saved::Done,
                Err(error) => {
                    log::error!("{name} is not written: {error}");
                    Saved::Failed
                }
            }
        });
        let _ = sender.send(answer);
        context.request_repaint();
    });
}

/// Gives the browser the bytes to download into its directory of
/// downloads: a Blob, a link to it named `name`, clicked, and the link
/// taken back a little later, once the browser has begun. Not `rfd`: in the
/// browser it draws a card of its own to click once more.
#[cfg(target_arch = "wasm32")]
fn start(context: egui::Context, name: String, bytes: Vec<u8>, sender: Sender<Saved>) {
    let answer = match download(&name, &bytes) {
        Ok(()) => Saved::Downloaded,
        Err(error) => {
            log::error!("{name} is not downloaded: {error:?}");
            Saved::Failed
        }
    };
    let _ = sender.send(answer);
    context.request_repaint();
}

/// The milliseconds the link of a download is kept.
#[cfg(target_arch = "wasm32")]
const KEEP_LINK: i32 = 10_000;

#[cfg(target_arch = "wasm32")]
fn download(name: &str, bytes: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let link: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    link.set_href(&url);
    link.set_download(name);
    link.click();
    let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(revoke.unchecked_ref(), KEEP_LINK)?;
    Ok(())
}

/// The save under way on android: where its answer goes, and the window to
/// draw again.
#[cfg(target_os = "android")]
static WAITING: std::sync::Mutex<Option<(Sender<Saved>, egui::Context)>> =
    std::sync::Mutex::new(None);

/// Asks the activity to save the bytes as a new document; the answer comes
/// through [`android_saved`].
#[cfg(target_os = "android")]
fn start(context: egui::Context, name: String, bytes: Vec<u8>, sender: Sender<Saved>) {
    use jni::objects::JValue;
    use jni::{jni_sig, jni_str};
    *WAITING.lock().expect("the save is never poisoned") = Some((sender, context));
    let asked = crate::activity::call(|env, activity| {
        let name = env.new_string(&name)?;
        let bytes = env.byte_array_from_slice(&bytes)?;
        env.call_method(
            activity,
            jni_str!("saveDocument"),
            jni_sig!("(Ljava/lang/String;[B)V"),
            &[JValue::Object(&name), JValue::Object(&bytes)],
        )?;
        Ok(())
    });
    if let Err(error) = asked {
        log::error!("the save of a file does not open: {error}");
        android_saved(Some(false));
    }
}

/// The answer of the activity: `Some(true)` written, `Some(false)` not
/// written, `None` given up. Called by the native method of `android/`.
#[cfg(target_os = "android")]
pub fn android_saved(done: Option<bool>) {
    let answer = match done {
        Some(true) => Saved::Done,
        Some(false) => Saved::Failed,
        None => Saved::GivenUp,
    };
    let waiting = WAITING.lock().expect("the save is never poisoned").take();
    if let Some((sender, context)) = waiting {
        let _ = sender.send(answer);
        context.request_repaint();
    }
}
