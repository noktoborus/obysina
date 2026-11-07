//! The paths under a directory: relative, split by `/`.

use crate::error::{Error, Result};

/// The parts of `path`; none for `""`. A part that is empty, `.` or `..`,
/// or a path that starts with `/`, is refused: a path stays under its
/// directory.
pub fn parts(path: &str) -> Result<Vec<String>> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    let refused = || Error::Path {
        path: path.to_owned(),
    };
    path.split('/')
        .map(|part| match part {
            "" | "." | ".." => Err(refused()),
            part if part.contains('\\') => Err(refused()),
            part => Ok(part.to_owned()),
        })
        .collect()
}

/// The directory of a file and its name: `a/b/c.json` is `[a, b]` and
/// `c.json`. A path with no name is refused.
pub fn split(path: &str) -> Result<(Vec<String>, String)> {
    let mut parts = parts(path)?;
    let name = parts.pop().ok_or_else(|| Error::Path {
        path: path.to_owned(),
    })?;
    Ok((parts, name))
}
