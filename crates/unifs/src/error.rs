//! What can go wrong with the files.

/// One failure of a file or a directory.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A path is empty where a file is meant, starts with `/`, or has a
    /// part that is empty, `.` or `..`.
    #[error("{path:?} is no path of a file under the directory")]
    Path {
        /// The path.
        path: String,
    },

    /// A file or a directory cannot be read.
    #[error("{path} cannot be read: {detail}")]
    Read {
        /// The file or the directory, under the root.
        path: String,
        /// What the platform said.
        detail: String,
    },

    /// A file or a directory cannot be written.
    #[error("{path} cannot be written: {detail}")]
    Write {
        /// The file or the directory, under the root.
        path: String,
        /// What the platform said.
        detail: String,
    },

    /// The platform gives no file system: a browser with no private file
    /// system of the origin, or a page with no window and no worker.
    #[error("no file system: {detail}")]
    Unavailable {
        /// What the platform said.
        detail: String,
    },
}

/// The result of the files.
pub type Result<T> = std::result::Result<T, Error>;
