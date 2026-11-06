//! What can go wrong with the store of pictures.

use std::path::PathBuf;

/// One failure of the store.
#[derive(Debug, thiserror::Error)]
pub enum BackdropError {
    /// A file or the directory cannot be read.
    #[error("{} cannot be read", path.display())]
    Read {
        /// The file or the directory.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },

    /// A file or the directory cannot be written.
    #[error("{} cannot be written", path.display())]
    Write {
        /// The file or the directory.
        path: PathBuf,
        /// What the file system said.
        #[source]
        source: std::io::Error,
    },

    /// A description cannot be written as YAML.
    #[error("the description of picture {number} cannot be written")]
    Yaml {
        /// The number of the picture.
        number: u64,
        /// What the writer said.
        #[source]
        source: serde_yaml_ng::Error,
    },

    /// The bytes are no PNG, JPEG or WebP.
    #[error("the picture is no PNG, JPEG or WebP")]
    Kind,
}

/// The result of the store.
pub type Result<T> = std::result::Result<T, BackdropError>;
