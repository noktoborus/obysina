//! What can go wrong with the receiver.

use std::path::PathBuf;

/// One failure of the receiver.
#[derive(Debug, thiserror::Error)]
pub enum NotesError {
    /// The port is not a number.
    #[error("the port {text:?} is not a number")]
    Port {
        /// What was given.
        text: String,
    },

    /// The port cannot be listened on.
    #[error("cannot listen on {address}: {message}")]
    Listen {
        /// The address.
        address: String,
        /// What the server said.
        message: String,
    },

    /// A file of the directory cannot be read or written.
    #[error("the file {} cannot be read or written", path.display())]
    File {
        /// The file.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },

    /// The number of a note to delete is not a number.
    #[error("the note {text:?} is not a number")]
    Id {
        /// What was given.
        text: String,
    },

    /// A note is not the JSON of a note.
    #[error("the note is not valid JSON")]
    Json {
        /// What the parser said.
        #[source]
        source: serde_json::Error,
    },

    /// The pixels of a picture are not base64.
    #[error("the picture is not base64")]
    Base64 {
        /// What the decoder said.
        #[source]
        source: base64::DecodeError,
    },

    /// The pixels of a picture are not as many as its size says.
    #[error("the picture is {width} by {height} but has {bytes} bytes")]
    PictureSize {
        /// Pixels across.
        width: usize,
        /// Pixels down.
        height: usize,
        /// The bytes given.
        bytes: usize,
    },

    /// A picture cannot be written as PNG.
    #[error("the picture cannot be written as PNG")]
    Png {
        /// What the encoder said.
        #[source]
        source: png::EncodingError,
    },
}

/// Result of the receiver.
pub type Result<T> = std::result::Result<T, NotesError>;
