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
        source: std::io::Error,
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
        source: std::io::Error,
    },
    /// The store of pictures could not be read or written.
    #[error("store of pictures error")]
    Backdrop {
        /// What went wrong.
        #[source]
        source: obusina_backdrop::BackdropError,
    },
    /// A picture of the store could not be read.
    #[error("picture {} unreadable", path.display())]
    PictureRead {
        /// The file.
        path: std::path::PathBuf,
        /// What went wrong.
        #[source]
        source: std::io::Error,
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
                PatternError::Read { .. } => "error.pattern.read",
                PatternError::Write { .. } => "error.pattern.write",
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
                | PatternError::Sheet { .. }
                | PatternError::Turn { .. }
                | PatternError::Outside { .. } => "error.pattern.invalid",
            },
            AppError::SettingsRead { .. } | AppError::SettingsInvalid { .. } => {
                "error.settings.read"
            }
            AppError::SettingsWrite { .. } => "error.settings.write",
            AppError::Backdrop { .. } | AppError::PictureRead { .. } | AppError::NoCache => {
                "error.backdrop.store"
            }
            AppError::PictureKind => "error.backdrop.kind",
            AppError::Picture { .. } => "error.backdrop.picture",
        }
    }
}
