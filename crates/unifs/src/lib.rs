#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
#[cfg(not(target_arch = "wasm32"))]
mod native;
mod path;
#[cfg(target_arch = "wasm32")]
mod web;

pub use error::{Error, Result};
pub use web_time::SystemTime;

/// Asks the platform to keep the files when the disk runs short; true when
/// it will. A browser may delete the files of an origin first; a disk keeps
/// them anyway.
pub async fn persist() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::persist().await;
    #[cfg(not(target_arch = "wasm32"))]
    true
}

/// A file of a directory, as [`Files::list`] gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// Its name, with no directory.
    pub name: String,
    /// When it was last written.
    pub modified: SystemTime,
}

/// The files under one directory: of the disk, or of the private file
/// system of the origin in a browser. A path is relative to it, its parts
/// split by `/`. A handle is cheap to clone and to send to another thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Files {
    #[cfg(not(target_arch = "wasm32"))]
    root: std::path::PathBuf,
    #[cfg(target_arch = "wasm32")]
    root: Vec<String>,
}

impl Files {
    /// The files of the directory `root` of the disk. Nothing is made until
    /// the first write.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn at(root: impl Into<std::path::PathBuf>) -> Files {
        Files { root: root.into() }
    }

    /// The files of the directory `root` of the private file system of the
    /// origin: `""` is its top. Nothing is made until the first write.
    #[cfg(target_arch = "wasm32")]
    pub fn origin(root: &str) -> Result<Files> {
        Ok(Files {
            root: path::parts(root)?,
        })
    }

    /// The directory `path` under this one.
    pub fn sub(&self, path: &str) -> Result<Files> {
        let parts = path::parts(path)?;
        #[cfg(not(target_arch = "wasm32"))]
        let root = parts
            .iter()
            .fold(self.root.clone(), |root, part| root.join(part));
        #[cfg(target_arch = "wasm32")]
        let root = [self.root.clone(), parts].concat();
        Ok(Files { root })
    }

    /// The bytes of the file `path`; `None` when it is not there.
    pub async fn read(&self, path: &str) -> Result<Option<Vec<u8>>> {
        self.read_impl(path).await
    }

    /// Writes the file `path` whole, making its directories. A reader sees
    /// the old bytes or the new ones, never a part: on the disk it is
    /// written beside its place and renamed into it; in a browser the
    /// stream of a file is seen only once it is closed.
    pub async fn write(&self, path: &str, bytes: &[u8]) -> Result<()> {
        self.write_impl(path, bytes).await
    }

    /// Removes the file `path`; one that is not there is no failure.
    pub async fn remove(&self, path: &str) -> Result<()> {
        self.remove_impl(path).await
    }

    /// Moves the file `from` to `to`, in place of a file there. A file
    /// `from` that is not there is no failure: nothing moves. On the disk
    /// it is one rename; in a browser, a write of `to`, then the removal of
    /// `from`.
    pub async fn rename(&self, from: &str, to: &str) -> Result<()> {
        self.rename_impl(from, to).await
    }

    /// The files of the directory `path`, in no order; none when it is not
    /// there. Directories are not listed, nor a file a write leaves for a
    /// moment beside its place.
    pub async fn list(&self, path: &str) -> Result<Vec<Listed>> {
        self.list_impl(path).await
    }

    /// When the file `path` was last written; `None` when it is not there.
    pub async fn modified(&self, path: &str) -> Result<Option<SystemTime>> {
        self.modified_impl(path).await
    }
}

#[cfg(test)]
mod tests;
