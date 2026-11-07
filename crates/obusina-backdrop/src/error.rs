//! What can go wrong with the store of pictures.

/// One failure of the store.
#[derive(Debug, thiserror::Error)]
pub enum BackdropError {
    /// A file or the directory cannot be read or written.
    #[error("the files of the store")]
    Files {
        /// What the files said.
        #[source]
        source: unifs::Error,
    },

    /// The picture of a description is not there.
    #[error("the picture {file} is not there")]
    Missing {
        /// Its file.
        file: String,
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

impl From<unifs::Error> for BackdropError {
    fn from(source: unifs::Error) -> Self {
        BackdropError::Files { source }
    }
}
