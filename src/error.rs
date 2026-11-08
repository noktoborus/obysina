//! Errors of the application and the keys of their text.
//!
//! The library crates return errors with no text for the user. Each variant
//! here names the key of its text in `locales/app.yml`.

use obusina_pattern::PatternError;

/// Result of the application.
pub type Result<T> = std::result::Result<T, AppError>;

/// A failure the user is told about.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A pattern could not be read or written.
    #[error("pattern file error")]
    Pattern {
        /// What went wrong.
        #[source]
        source: PatternError,
    },
    /// The settings file could not be read.
    #[error("settings file unreadable")]
    SettingsRead {
        /// What went wrong.
        #[source]
        source: unifs::Error,
    },
    /// The settings file is not one this version reads.
    #[error("settings file invalid")]
    SettingsInvalid {
        /// What went wrong.
        #[source]
        source: serde_yaml_ng::Error,
    },
    /// The settings file could not be written.
    #[error("settings file not written")]
    SettingsWrite {
        /// What went wrong.
        #[source]
        source: unifs::Error,
    },
    /// The store of pictures could not be read or written.
    #[error("store of pictures error")]
    Backdrop {
        /// What went wrong.
        #[source]
        source: obusina_backdrop::BackdropError,
    },
    /// A picture is no PNG, JPEG or WebP.
    #[error("picture of another kind")]
    PictureKind,
    /// A picture could not be decoded.
    #[error("picture not decoded")]
    Picture {
        /// What went wrong.
        #[source]
        source: image::ImageError,
    },
    /// A file of the patterns of the list could not be read or written.
    #[error("files of the list")]
    Library {
        /// What the files said.
        #[source]
        source: unifs::Error,
    },
    /// The pattern asked for is not in the list.
    #[error("no such pattern in the list")]
    NoPattern,
    /// A file of patterns could not be packed or unpacked.
    #[error("file of patterns not packed or unpacked")]
    Archive {
        /// What went wrong.
        #[source]
        source: std::io::Error,
    },
    /// A file that came in holds no pattern.
    #[error("no pattern in the file")]
    NoPatterns,
    /// The platform gives no cache to keep a picture in.
    #[error("no cache for pictures")]
    NoCache,
}

impl From<obusina_backdrop::BackdropError> for AppError {
    fn from(source: obusina_backdrop::BackdropError) -> Self {
        match source {
            obusina_backdrop::BackdropError::Kind => AppError::PictureKind,
            source => AppError::Backdrop { source },
        }
    }
}

impl From<unifs::Error> for AppError {
    fn from(source: unifs::Error) -> Self {
        AppError::Library { source }
    }
}

impl From<PatternError> for AppError {
    fn from(source: PatternError) -> Self {
        AppError::Pattern { source }
    }
}

impl AppError {
    /// The key of the text shown to the user.
    pub fn message_key(&self) -> &'static str {
        match self {
            AppError::Pattern { source } => match source {
                PatternError::Newer { .. } => "error.pattern.newer",
                PatternError::Json { .. }
                | PatternError::Id { .. }
                | PatternError::NoFormat
                | PatternError::UnknownFormat { .. }
                | PatternError::RowWidth { .. }
                | PatternError::UnknownId { .. }
                | PatternError::DuplicateId { .. }
                | PatternError::Palette { .. }
                | PatternError::NoPalette
                | PatternError::Place { .. }
                | PatternError::Full { .. }
                | PatternError::Lost { .. }
                | PatternError::Colour { .. }
                | PatternError::Ratio { .. }
                | PatternError::Field { .. }
                | PatternError::Turn { .. }
                | PatternError::Outside { .. } => "error.pattern.invalid",
            },
            AppError::SettingsRead { .. } | AppError::SettingsInvalid { .. } => {
                "error.settings.read"
            }
            AppError::SettingsWrite { .. } => "error.settings.write",
            AppError::Backdrop { .. } | AppError::NoCache => "error.backdrop.store",
            AppError::PictureKind => "error.backdrop.kind",
            AppError::Library {
                source: unifs::Error::Read { .. },
            }
            | AppError::NoPattern => "error.pattern.read",
            AppError::Library { .. } => "error.pattern.write",
            AppError::Archive { .. } | AppError::NoPatterns => "error.import",
            AppError::Picture { .. } => "error.backdrop.picture",
        }
    }
}
