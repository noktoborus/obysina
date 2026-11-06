//! The identifier of a pattern: what files kept apart from the pattern, such
//! as the picture under it, name it by.

use std::fmt;
use std::str::FromStr;

use crate::error::PatternError;

/// A random identifier, unique to one pattern and kept in its file.
///
/// A copy of the file has the same one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PatternId(uuid::Uuid);

impl PatternId {
    /// A new identifier no other pattern has.
    pub fn new() -> PatternId {
        PatternId(uuid::Uuid::new_v4())
    }
}

impl Default for PatternId {
    fn default() -> Self {
        PatternId::new()
    }
}

impl fmt::Display for PatternId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(formatter)
    }
}

impl FromStr for PatternId {
    type Err = PatternError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(text)
            .map(PatternId)
            .map_err(|source| PatternError::Id {
                text: text.to_owned(),
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identifier_reads_back_from_its_text() {
        let id = PatternId::new();

        assert_eq!(id.to_string().parse::<PatternId>().expect("it reads"), id);
        assert_ne!(PatternId::new(), id, "two are never the same");
    }

    #[test]
    fn a_text_that_is_no_identifier_is_refused() {
        assert!(matches!(
            "12".parse::<PatternId>(),
            Err(PatternError::Id { .. })
        ));
    }
}
