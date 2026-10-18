//! The receiver of the notes left on the mockups.
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

#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod receiver;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    match receiver::run(std::env::args().skip(1)) {
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
