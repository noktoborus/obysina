//! The files of a directory of the disk: `std::fs`. A call does its work at
//! once; the caller runs it away from the frame, on a thread of its own.

use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::{Files, Listed, SystemTime, path};

/// The start of the name of a file a write leaves beside its place for a
/// moment, as `tempfile` names it.
const PARTIAL: &str = ".tmp";

impl Files {
    /// The place of `path` on the disk.
    fn place(&self, path: &str) -> Result<PathBuf> {
        let parts = path::parts(path)?;
        Ok(parts
            .iter()
            .fold(self.root.clone(), |root, part| root.join(part)))
    }

    /// The place of a file `path`, which must have a name.
    fn file(&self, path: &str) -> Result<PathBuf> {
        path::split(path)?;
        self.place(path)
    }

    pub(crate) async fn read_impl(&self, path: &str) -> Result<Option<Vec<u8>>> {
        match std::fs::read(self.file(path)?) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(read(path, &error)),
        }
    }

    pub(crate) async fn write_impl(&self, path: &str, bytes: &[u8]) -> Result<()> {
        let place = self.file(path)?;
        let failed = |error: std::io::Error| write(path, &error);
        let directory = place.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(directory).map_err(failed)?;
        let mut file = tempfile::NamedTempFile::new_in(directory).map_err(failed)?;
        file.write_all(bytes).map_err(failed)?;
        file.as_file().sync_all().map_err(failed)?;
        file.persist(&place).map_err(|error| failed(error.error))?;
        Ok(())
    }

    pub(crate) async fn remove_impl(&self, path: &str) -> Result<()> {
        match std::fs::remove_file(self.file(path)?) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(write(path, &error)),
            _ => Ok(()),
        }
    }

    pub(crate) async fn rename_impl(&self, from: &str, to: &str) -> Result<()> {
        let (source, target) = (self.file(from)?, self.file(to)?);
        if !source.exists() {
            return Ok(());
        }
        let failed = |error: std::io::Error| write(to, &error);
        if let Some(directory) = target.parent() {
            std::fs::create_dir_all(directory).map_err(failed)?;
        }
        std::fs::rename(&source, &target).map_err(failed)
    }

    pub(crate) async fn list_impl(&self, path: &str) -> Result<Vec<Listed>> {
        let entries = match std::fs::read_dir(self.place(path)?) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(read(path, &error)),
        };
        let mut listed = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| read(path, &error))?;
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if !metadata.is_file() || name.starts_with(PARTIAL) {
                continue;
            }
            listed.push(Listed {
                name,
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
        Ok(listed)
    }

    pub(crate) async fn modified_impl(&self, path: &str) -> Result<Option<SystemTime>> {
        match std::fs::metadata(self.file(path)?) {
            Ok(metadata) => Ok(Some(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH))),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(read(path, &error)),
        }
    }
}

/// A read of `path` that failed.
fn read(path: &str, error: &std::io::Error) -> Error {
    Error::Read {
        path: path.to_owned(),
        detail: error.to_string(),
    }
}

/// A write of `path` that failed.
fn write(path: &str, error: &std::io::Error) -> Error {
    Error::Write {
        path: path.to_owned(),
        detail: error.to_string(),
    }
}
