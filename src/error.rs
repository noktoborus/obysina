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
                | PatternError::NoFormat
                | PatternError::UnknownFormat { .. }
                | PatternError::RowCount { .. }
                | PatternError::RowWidth { .. }
                | PatternError::UnknownKey { .. }
                | PatternError::DuplicateKey { .. }
                | PatternError::Colour { .. }
                | PatternError::Size { .. } => "error.pattern.invalid",
            },
        }
    }
}
