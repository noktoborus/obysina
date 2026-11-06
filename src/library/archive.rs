//! The files a pattern leaves the program in, and comes in by.
//!
//! | File | What it holds |
//! | --- | --- |
//! | `<name>.obusina` | one pattern file, packed by xz |
//! | `<name>.obusiny` | a tar packed by xz, a pattern file `<id>.json` for each |
//! | `<name>.json` | one pattern file, as it is |
//!
//! The kind of a file is told by its bytes, not by its name: the magic of
//! xz, then the magic of tar inside it.

use std::io::{Cursor, Read};

use obusina_pattern::Pattern;

use crate::error::{AppError, Result};

/// The extension of a file of one pattern.
pub const ONE: &str = "obusina";
/// The extension of a file of many patterns.
pub const MANY: &str = "obusiny";
/// The extensions an import takes.
pub const EXTENSIONS: [&str; 3] = [ONE, MANY, "json"];

/// The first bytes of an xz stream.
const XZ_MAGIC: [u8; 6] = [0xfd, b'7', b'z', b'X', b'Z', 0x00];
/// Where the magic of a tar stands in its first block, and what it is.
const TAR_MAGIC_AT: usize = 257;
const TAR_MAGIC: &[u8] = b"ustar";
/// The most bytes a file unpacks to: more is no file of patterns.
const MOST_UNPACKED: u64 = 256 << 20;

/// A file of one pattern: its text packed by xz.
pub fn one(pattern: &Pattern) -> Result<Vec<u8>> {
    let text = pattern.to_json()?;
    pack(text.as_bytes())
}

/// A file of many patterns: a tar of their texts, packed by xz.
pub fn many<'a>(patterns: impl IntoIterator<Item = &'a Pattern>) -> Result<Vec<u8>> {
    let mut builder = tar::Builder::new(Vec::new());
    for pattern in patterns {
        let text = pattern.to_json()?;
        let mut header = tar::Header::new_ustar();
        header.set_size(text.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        let name = format!("{}.json", pattern.id());
        builder
            .append_data(&mut header, name, text.as_bytes())
            .map_err(|source| AppError::Archive { source })?;
    }
    let tar = builder
        .into_inner()
        .map_err(|source| AppError::Archive { source })?;
    pack(&tar)
}

/// The patterns of a file of any kind of [`EXTENSIONS`].
pub fn read(bytes: &[u8]) -> Result<Vec<Pattern>> {
    let unpacked;
    let bytes = if bytes.starts_with(&XZ_MAGIC) {
        unpacked = unpack(bytes)?;
        unpacked.as_slice()
    } else {
        bytes
    };
    if bytes.get(TAR_MAGIC_AT..TAR_MAGIC_AT + TAR_MAGIC.len()) == Some(TAR_MAGIC) {
        return read_tar(bytes);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| AppError::NoPatterns)?;
    Ok(vec![Pattern::from_json(text)?])
}

/// The patterns of the `.json` files of a tar; the rest of it is passed
/// over.
fn read_tar(bytes: &[u8]) -> Result<Vec<Pattern>> {
    let mut archive = tar::Archive::new(Cursor::new(bytes));
    let entries = archive
        .entries()
        .map_err(|source| AppError::Archive { source })?;
    let mut patterns = Vec::new();
    for entry in entries {
        let mut entry = entry.map_err(|source| AppError::Archive { source })?;
        let json = entry.path().is_ok_and(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        });
        if !json {
            continue;
        }
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|source| AppError::Archive { source })?;
        patterns.push(Pattern::from_json(&text)?);
    }
    if patterns.is_empty() {
        return Err(AppError::NoPatterns);
    }
    Ok(patterns)
}

/// Packs bytes by xz.
fn pack(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut packed = Vec::new();
    lzma_rs::xz_compress(&mut Cursor::new(bytes), &mut packed)
        .map_err(|source| AppError::Archive { source })?;
    Ok(packed)
}

/// Unpacks an xz stream, at most [`MOST_UNPACKED`] bytes of it.
fn unpack(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut unpacked = Vec::new();
    lzma_rs::xz_decompress(&mut Cursor::new(bytes), &mut unpacked).map_err(|error| {
        AppError::Archive {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
        }
    })?;
    if unpacked.len() as u64 > MOST_UNPACKED {
        return Err(AppError::NoPatterns);
    }
    Ok(unpacked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> Pattern {
        let mut pattern = crate::app::new_pattern();
        pattern.set_name(name);
        pattern
    }

    #[test]
    fn a_file_of_one_pattern_reads_back() {
        let pattern = named("one");

        let file = one(&pattern).expect("it packs");

        assert!(file.starts_with(&XZ_MAGIC));
        assert_eq!(read(&file).expect("it reads"), vec![pattern]);
    }

    #[test]
    fn a_file_of_many_patterns_reads_back() {
        let patterns = vec![named("first"), named("second")];

        let file = many(&patterns).expect("it packs");

        assert_eq!(read(&file).expect("it reads"), patterns);
    }

    #[test]
    fn a_plain_pattern_file_reads() {
        let pattern = named("plain");
        let text = pattern.to_json().expect("text");

        assert_eq!(read(text.as_bytes()).expect("it reads"), vec![pattern]);
    }

    #[test]
    fn bytes_of_no_pattern_are_refused() {
        assert!(read(&[0xff, 0xfe, 0x00]).is_err());
        assert!(read(&many([]).expect("an empty tar packs")).is_err());
    }
}
