//! The receiver of the notes left on the mockups.
//!
//! `notes wait <directory>` waits for a comment for the chat instead
//! (`wait.rs`).
//!
//! `notes <directory> [port]` listens on 127.0.0.1, port 6081 unless told
//! otherwise; trunk passes `/notes` of the page to it. A note posted is
//! kept as `<id>.json`, its picture as `<id>.png`, until it is read and
//! deleted.
//!
//! | Request | Answer |
//! | --- | --- |
//! | `GET /notes` | every note kept, as a JSON list |
//! | `POST /notes` | the note kept, with its number |
//! | `DELETE /notes/<id>` | the notes left |
//! | `POST /notes/chat` | the comment for the chat, kept in `chat.jsonl` |

#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod receiver;
#[cfg(not(target_arch = "wasm32"))]
mod wait;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.as_slice() {
        [command, directory] if command == "wait" => wait::run(std::path::Path::new(directory)),
        _ => receiver::run(arguments.into_iter()),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// A page cannot listen for anything: the receiver is of the desktop only.
#[cfg(target_arch = "wasm32")]
fn main() {}
