//! The directory of pictures: `<n>.<ext>`, a copy of the picture, and
//! `<n>.yaml`, how it lies over the field of its pattern.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BackdropError, Result};
use crate::kind::Kind;

/// The extension of a description.
const DESCRIPTION: &str = "yaml";
/// The extension a file is written under before it takes its place.
const PARTIAL: &str = "partial";

/// How a picture lies over the field of its pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backdrop {
    /// The identifier of the pattern it lies under.
    pub pattern: String,
    /// The corners of the picture on the field, in bead units: top left, top
    /// right, bottom right, bottom left.
    pub corners: [[f32; 2]; 4],
    /// How much of the picture shows, from 0 to 1.
    pub opacity: f32,
    /// It is kept but not drawn.
    pub hidden: bool,
    /// The name of the file it was taken from, for a person.
    pub source: Option<String>,
}

/// A picture of the store and how it lies.
#[derive(Debug, Clone, PartialEq)]
pub struct Kept {
    /// Its number in the store.
    pub number: u64,
    /// The name of the file of the picture in the store: `<number>.<ext>`.
    pub file: String,
    /// How it lies.
    pub backdrop: Backdrop,
}

/// The directory of pictures. It is made when the first picture is put.
///
/// Every call is `async`, so a store of the browser, whose files answer in
/// futures, stands behind the same calls. Here a call does its work on the
/// files at once; the caller runs it away from the frame.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    /// The store in `dir`.
    pub fn new(dir: PathBuf) -> Store {
        Store { dir }
    }

    /// The directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The picture of a pattern: the one of the highest number when there
    /// are several. A description this version cannot read is passed over:
    /// what an older version wrote is lost. A directory that is not there
    /// holds nothing.
    pub async fn find(&self, pattern: &str) -> Result<Option<Kept>> {
        let found = self
            .described()?
            .into_iter()
            .filter(|kept| kept.backdrop.pattern == pattern)
            .max_by_key(|kept| kept.number);
        Ok(found)
    }

    /// The bytes of a picture of the store.
    pub async fn read(&self, kept: &Kept) -> Result<Vec<u8>> {
        let path = self.dir.join(&kept.file);
        std::fs::read(&path).map_err(|source| BackdropError::Read { path, source })
    }

    /// Puts a picture under a pattern, with a number no picture of the store
    /// has, and takes away the one the pattern had. `bytes` are kept as they
    /// are.
    pub async fn put(&self, bytes: &[u8], backdrop: Backdrop) -> Result<Kept> {
        let kind = Kind::of(bytes).ok_or(BackdropError::Kind)?;
        std::fs::create_dir_all(&self.dir).map_err(|source| BackdropError::Write {
            path: self.dir.clone(),
            source,
        })?;
        let old = self.find(&backdrop.pattern).await?;
        let number = self.numbers()?.into_iter().max().unwrap_or(0) + 1;
        let file = format!("{number}.{}", kind.extension());
        write(&self.dir.join(&file), bytes)?;
        let kept = Kept {
            number,
            file,
            backdrop,
        };
        self.save(&kept).await?;
        if let Some(old) = old {
            self.remove(&old).await?;
        }
        Ok(kept)
    }

    /// Writes how a picture lies.
    pub async fn save(&self, kept: &Kept) -> Result<()> {
        let text =
            serde_yaml_ng::to_string(&kept.backdrop).map_err(|source| BackdropError::Yaml {
                number: kept.number,
                source,
            })?;
        write(&self.description(kept.number), text.as_bytes())
    }

    /// Takes a picture and its description away.
    pub async fn remove(&self, kept: &Kept) -> Result<()> {
        for path in [self.description(kept.number), self.dir.join(&kept.file)] {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => return Err(BackdropError::Write { path, source }),
            }
        }
        Ok(())
    }

    /// The file of the description of a number.
    fn description(&self, number: u64) -> PathBuf {
        self.dir.join(format!("{number}.{DESCRIPTION}"))
    }

    /// The number of every file of the store, read or not.
    fn numbers(&self) -> Result<Vec<u64>> {
        Ok(self
            .files()?
            .iter()
            .filter_map(|path| path.file_stem()?.to_str()?.parse().ok())
            .collect())
    }

    /// Every file of the directory; none when it is not there.
    fn files(&self) -> Result<Vec<PathBuf>> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(BackdropError::Read {
                    path: self.dir.clone(),
                    source,
                });
            }
        };
        Ok(entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect())
    }

    /// Every picture with a description this version reads, and a file of
    /// the picture beside it.
    fn described(&self) -> Result<Vec<Kept>> {
        let files = self.files()?;
        let kept = files
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == DESCRIPTION))
            .filter_map(|path| {
                let number: u64 = path.file_stem()?.to_str()?.parse().ok()?;
                let text = std::fs::read_to_string(path).ok()?;
                let backdrop: Backdrop = serde_yaml_ng::from_str(&text).ok()?;
                let file = files
                    .iter()
                    .find(|other| {
                        other.file_stem().and_then(|stem| stem.to_str())
                            == Some(&number.to_string())
                            && other.extension().is_some_and(|ext| {
                                Kind::EXTENSIONS.iter().any(|known| ext == *known)
                            })
                    })?
                    .file_name()?
                    .to_str()?
                    .to_owned();
                Some(Kept {
                    number,
                    file,
                    backdrop,
                })
            })
            .collect();
        Ok(kept)
    }
}

/// Writes a file through a partial file beside it and a rename.
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let partial = path.with_extension(PARTIAL);
    let failed = |source| BackdropError::Write {
        path: path.to_path_buf(),
        source,
    };
    std::fs::write(&partial, bytes).map_err(failed)?;
    std::fs::rename(&partial, path).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pollster::block_on;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nnot really a picture";
    const JPEG: &[u8] = &[0xff, 0xd8, 0xff, 0xe0, 1, 2, 3];

    fn backdrop(pattern: &str) -> Backdrop {
        Backdrop {
            pattern: pattern.to_owned(),
            corners: [[0.0, 0.0], [8.0, 0.0], [8.0, 6.0], [0.0, 6.0]],
            opacity: 0.6,
            hidden: false,
            source: Some("photo.png".to_owned()),
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("a scratch directory");
        let store = Store::new(dir.path().join("backdrops"));
        (dir, store)
    }

    #[test]
    fn a_picture_put_is_found_by_its_pattern() {
        let (_dir, store) = store();
        assert_eq!(block_on(store.find("a")).expect("it reads"), None);

        let kept = block_on(store.put(PNG, backdrop("a"))).expect("it is put");
        block_on(store.put(JPEG, backdrop("b"))).expect("it is put");

        assert_eq!(kept.number, 1);
        assert_eq!(kept.file, "1.png");
        assert_eq!(block_on(store.read(&kept)).expect("the copy"), PNG);
        assert_eq!(block_on(store.find("a")).expect("it reads"), Some(kept));
        assert_eq!(
            block_on(store.find("b"))
                .expect("it reads")
                .map(|kept| kept.number),
            Some(2)
        );
    }

    #[test]
    fn a_second_picture_takes_the_place_of_the_first() {
        let (_dir, store) = store();
        let first = block_on(store.put(PNG, backdrop("a"))).expect("it is put");

        let second = block_on(store.put(JPEG, backdrop("a"))).expect("it is put");

        assert_eq!(second.number, 2);
        assert!(!store.dir().join(&first.file).exists());
        assert_eq!(block_on(store.find("a")).expect("it reads"), Some(second));
    }

    #[test]
    fn a_saved_description_is_read_back() {
        let (_dir, store) = store();
        let mut kept = block_on(store.put(PNG, backdrop("a"))).expect("it is put");

        kept.backdrop.corners[2] = [10.5, 7.25];
        kept.backdrop.hidden = true;
        block_on(store.save(&kept)).expect("it is saved");

        assert_eq!(block_on(store.find("a")).expect("it reads"), Some(kept));
    }

    #[test]
    fn a_removed_picture_leaves_nothing() {
        let (_dir, store) = store();
        let kept = block_on(store.put(PNG, backdrop("a"))).expect("it is put");

        block_on(store.remove(&kept)).expect("it is removed");

        assert_eq!(block_on(store.find("a")).expect("it reads"), None);
        assert_eq!(std::fs::read_dir(store.dir()).expect("the dir").count(), 0);
    }

    /// What another version wrote is passed over, not an error.
    #[test]
    fn a_description_of_another_shape_is_passed_over() {
        let (_dir, store) = store();
        let kept = block_on(store.put(PNG, backdrop("a"))).expect("it is put");
        std::fs::write(store.dir().join("1.yaml"), "pattern: a\nzoom: 3\n").expect("written");

        assert_eq!(block_on(store.find("a")).expect("it reads"), None);
        let next = block_on(store.put(PNG, backdrop("a"))).expect("it is put");
        assert_eq!(
            next.number,
            kept.number + 1,
            "the number is not taken again"
        );
    }

    #[test]
    fn a_picture_of_another_kind_is_refused() {
        let (_dir, store) = store();

        assert!(matches!(
            block_on(store.put(b"GIF89a", backdrop("a"))),
            Err(BackdropError::Kind)
        ));
        assert_eq!(block_on(store.find("a")).expect("it reads"), None);
    }
}
