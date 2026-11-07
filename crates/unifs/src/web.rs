//! The files of the private file system of the origin (OPFS), through the
//! calls of `web-sys` that are stable: a directory handle, a file handle,
//! the file it gives, its writable stream. Each call is a promise.
//!
//! ```text
//! navigator.storage.getDirectory() ──► the top ──getDirectoryHandle──► the root
//! read     getFileHandle ──► getFile ──► arrayBuffer
//! write    getFileHandle {create} ──► createWritable ──► write ──► close
//! list     values() of the directory ──► each file handle ──► getFile ──► lastModified
//! rename   read `from`, write `to`, remove `from`: the move of a handle is
//!          no stable call of web-sys
//! ```

use js_sys::{AsyncIterator, Promise, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    DomException, File, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemHandle,
    FileSystemHandleKind, FileSystemWritableFileStream, StorageManager,
};

use crate::error::{Error, Result};
use crate::{Files, Listed, SystemTime, path};

/// The name of the exception of a file or a directory that is not there.
const NOT_FOUND: &str = "NotFoundError";

impl Files {
    /// The directory of `parts` under the root; `None` when it is not there
    /// and is not to be made.
    async fn directory(
        &self,
        parts: &[String],
        create: bool,
    ) -> Result<Option<FileSystemDirectoryHandle>> {
        let shown = [self.root.as_slice(), parts].concat().join("/");
        let mut directory = top().await?;
        for part in self.root.iter().chain(parts) {
            let options = FileSystemGetDirectoryOptions::new();
            options.set_create(create);
            let promise = directory.get_directory_handle_with_options(part, &options);
            match settle(promise).await {
                Ok(handle) => directory = handle.unchecked_into(),
                Err(error) if is_not_found(&error) => return Ok(None),
                Err(error) => return Err(failure(&shown, &error, create)),
            }
        }
        Ok(Some(directory))
    }

    /// The handle of the file `path`; `None` when it is not there and is not
    /// to be made.
    async fn handle(&self, path: &str, create: bool) -> Result<Option<FileSystemFileHandle>> {
        let (parts, name) = path::split(path)?;
        let Some(directory) = self.directory(&parts, create).await? else {
            return Ok(None);
        };
        let options = FileSystemGetFileOptions::new();
        options.set_create(create);
        match settle(directory.get_file_handle_with_options(&name, &options)).await {
            Ok(handle) => Ok(Some(handle.unchecked_into())),
            Err(error) if is_not_found(&error) => Ok(None),
            Err(error) => Err(failure(path, &error, create)),
        }
    }

    pub(crate) async fn read_impl(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let Some(handle) = self.handle(path, false).await? else {
            return Ok(None);
        };
        let file = match settle(handle.get_file()).await {
            Ok(file) => file.unchecked_into::<File>(),
            Err(error) if is_not_found(&error) => return Ok(None),
            Err(error) => return Err(failure(path, &error, false)),
        };
        let buffer = settle(file.array_buffer())
            .await
            .map_err(|error| failure(path, &error, false))?;
        Ok(Some(Uint8Array::new(&buffer).to_vec()))
    }

    pub(crate) async fn write_impl(&self, path: &str, bytes: &[u8]) -> Result<()> {
        let failed = |error: JsValue| failure(path, &error, true);
        let handle = self
            .handle(path, true)
            .await?
            .ok_or_else(|| failed(JsValue::from_str("not made")))?;
        let stream: FileSystemWritableFileStream = settle(handle.create_writable())
            .await
            .map_err(failed)?
            .unchecked_into();
        let written = match stream.write_with_u8_array(bytes) {
            Ok(promise) => settle(promise).await.map(|_| ()),
            Err(error) => Err(error),
        };
        if let Err(error) = written {
            // The stream is dropped unclosed: the file keeps its old bytes.
            let _ = settle(stream.abort()).await;
            return Err(failed(error));
        }
        settle(stream.close()).await.map_err(failed)?;
        Ok(())
    }

    pub(crate) async fn remove_impl(&self, path: &str) -> Result<()> {
        let (parts, name) = path::split(path)?;
        let Some(directory) = self.directory(&parts, false).await? else {
            return Ok(());
        };
        match settle(directory.remove_entry(&name)).await {
            Ok(_) => Ok(()),
            Err(error) if is_not_found(&error) => Ok(()),
            Err(error) => Err(failure(path, &error, true)),
        }
    }

    pub(crate) async fn rename_impl(&self, from: &str, to: &str) -> Result<()> {
        path::split(to)?;
        let Some(bytes) = self.read_impl(from).await? else {
            return Ok(());
        };
        self.write_impl(to, &bytes).await?;
        self.remove_impl(from).await
    }

    pub(crate) async fn list_impl(&self, path: &str) -> Result<Vec<Listed>> {
        let parts = path::parts(path)?;
        let Some(directory) = self.directory(&parts, false).await? else {
            return Ok(Vec::new());
        };
        let failed = |error: JsValue| failure(path, &error, false);
        let entries: AsyncIterator = directory.values();
        let mut listed = Vec::new();
        loop {
            let next = entries.next().map_err(failed)?;
            let step = settle(next).await.map_err(failed)?;
            let done = js_sys::Reflect::get(&step, &JsValue::from_str("done")).map_err(failed)?;
            if done.as_bool().unwrap_or(true) {
                break;
            }
            let value = js_sys::Reflect::get(&step, &JsValue::from_str("value")).map_err(failed)?;
            let handle: FileSystemHandle = value.unchecked_into();
            if handle.kind() != FileSystemHandleKind::File {
                continue;
            }
            let handle: FileSystemFileHandle = handle.unchecked_into();
            let file = match settle(handle.get_file()).await {
                Ok(file) => file.unchecked_into::<File>(),
                Err(error) if is_not_found(&error) => continue,
                Err(error) => return Err(failed(error)),
            };
            listed.push(Listed {
                name: file.name(),
                modified: time(file.last_modified()),
            });
        }
        Ok(listed)
    }

    pub(crate) async fn modified_impl(&self, path: &str) -> Result<Option<SystemTime>> {
        let Some(handle) = self.handle(path, false).await? else {
            return Ok(None);
        };
        match settle(handle.get_file()).await {
            Ok(file) => Ok(Some(time(file.unchecked_into::<File>().last_modified()))),
            Err(error) if is_not_found(&error) => Ok(None),
            Err(error) => Err(failure(path, &error, false)),
        }
    }
}

/// Asks the browser to keep the files of the origin when the disk runs
/// short, in place of deleting them first; true when it will.
pub async fn persist() -> bool {
    let Ok(storage) = storage() else {
        return false;
    };
    let Ok(promise) = storage.persist() else {
        return false;
    };
    settle(promise)
        .await
        .ok()
        .and_then(|kept| kept.as_bool())
        .unwrap_or(false)
}

/// The storage of the window or of the worker the code runs in.
fn storage() -> Result<StorageManager> {
    let global = js_sys::global();
    if let Some(window) = global.dyn_ref::<web_sys::Window>() {
        return Ok(window.navigator().storage());
    }
    if let Some(worker) = global.dyn_ref::<web_sys::WorkerGlobalScope>() {
        return Ok(worker.navigator().storage());
    }
    Err(Error::Unavailable {
        detail: "no window and no worker".to_owned(),
    })
}

/// The top of the private file system of the origin.
async fn top() -> Result<FileSystemDirectoryHandle> {
    let top = settle(storage()?.get_directory())
        .await
        .map_err(|error| Error::Unavailable {
            detail: describe(&error),
        })?;
    Ok(top.unchecked_into())
}

/// The value of a promise, or what it was refused with.
async fn settle(promise: Promise) -> std::result::Result<JsValue, JsValue> {
    JsFuture::from(promise).await
}

/// True when the browser said the file or the directory is not there.
fn is_not_found(error: &JsValue) -> bool {
    error
        .dyn_ref::<DomException>()
        .is_some_and(|exception| exception.name() == NOT_FOUND)
}

/// A failure of `path`: of a write when `writing`, else of a read.
fn failure(path: &str, error: &JsValue, writing: bool) -> Error {
    let (path, detail) = (path.to_owned(), describe(error));
    match writing {
        true => Error::Write { path, detail },
        false => Error::Read { path, detail },
    }
}

/// What the browser said, as text.
fn describe(error: &JsValue) -> String {
    if let Some(exception) = error.dyn_ref::<DomException>() {
        return format!("{}: {}", exception.name(), exception.message());
    }
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}

/// A time of the browser, milliseconds since 1970.
fn time(milliseconds: f64) -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(milliseconds.max(0.0) as u64)
}
