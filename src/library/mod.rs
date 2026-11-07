//! The patterns of the person: the list, the trash, and the files they leave
//! the program in and come in by. Each pattern is a file of its own in the
//! data of the program, its history beside it.
//!
//! ```text
//! <data>/patterns/<id>.json          the pattern (obusina-pattern)
//! <data>/patterns/<id>.history.json  its history (src/history_file.rs)
//! <data>/trash/<id>.json             a pattern in the trash, with its history
//! <data>/trash/<id>.history.json
//! <data>/trash/<id>.trashed          the seconds since 1970 it went there
//! ```
//!
//! The time of the last change of a pattern is the time of its file. A file
//! that cannot be read is passed over and logged: it stays where it is.

pub mod archive;

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use obusina_pattern::{Pattern, PatternId};

use crate::edit::History;
use crate::error::{AppError, Result};

/// The directory of the patterns of the list.
const PATTERNS: &str = "patterns";
/// The directory of the trash.
const TRASH: &str = "trash";
/// The end of the name of a pattern file.
const PATTERN: &str = ".json";
/// The end of the name of a history file.
const HISTORY: &str = ".history.json";
/// The end of the name of the file of the time a pattern went to the trash.
const TRASHED: &str = ".trashed";
/// The days a pattern stays in the trash.
pub const TRASH_DAYS: u64 = 30;

/// A pattern of the list, and when it last changed.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The pattern.
    pub pattern: Pattern,
    /// When its file was last written.
    pub changed: SystemTime,
}

/// A pattern of the trash, and when it went there.
#[derive(Debug, Clone)]
pub struct Trashed {
    /// The pattern.
    pub pattern: Pattern,
    /// When it went to the trash.
    pub since: SystemTime,
}

impl Trashed {
    /// The days it has left in the trash at `now`, at least one.
    pub fn days_left(&self, now: SystemTime) -> u64 {
        let gone = now.duration_since(self.since).unwrap_or_default();
        TRASH_DAYS.saturating_sub(gone.as_secs() / DAY).max(1)
    }
}

/// The seconds of a day.
const DAY: u64 = 24 * 60 * 60;

/// What an import did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    /// The patterns added with the identifier they came with.
    pub new: Vec<PatternId>,
    /// The patterns added as copies, under a new identifier: the list had
    /// another pattern of theirs.
    pub copies: Vec<PatternId>,
    /// The names of the patterns not added: the list has the same work.
    pub present: Vec<String>,
}

impl Report {
    /// The patterns added, in the order they came.
    pub fn added(&self) -> impl Iterator<Item = PatternId> + '_ {
        self.new.iter().chain(&self.copies).copied()
    }
}

/// The patterns in a directory of the data of the program.
pub struct Library {
    dir: PathBuf,
}

impl Library {
    /// The library in `dir`; its directories are made when it is first
    /// written.
    pub fn new(dir: PathBuf) -> Library {
        Library { dir }
    }

    /// The patterns of the list, the last changed first.
    pub fn list(&self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self
            .ids_in(PATTERNS)
            .into_iter()
            .filter_map(|id| {
                let path = self.pattern_path(PATTERNS, id);
                let pattern = read_logged(&path)?;
                let changed = modified(&path);
                Some(Entry { pattern, changed })
            })
            .collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.changed));
        entries
    }

    /// The patterns of the trash, the last trashed first.
    pub fn trash(&self) -> Vec<Trashed> {
        let mut trashed: Vec<Trashed> = self
            .ids_in(TRASH)
            .into_iter()
            .filter_map(|id| {
                let pattern = read_logged(&self.pattern_path(TRASH, id))?;
                let since = self.trashed_at(id);
                Some(Trashed { pattern, since })
            })
            .collect();
        trashed.sort_by_key(|trashed| std::cmp::Reverse(trashed.since));
        trashed
    }

    /// True when the list has a pattern of this identifier.
    pub fn has(&self, id: PatternId) -> bool {
        self.pattern_path(PATTERNS, id).exists()
    }

    /// The pattern of the list of this identifier, and its history.
    pub fn open(&self, id: PatternId) -> Result<(Pattern, History)> {
        let path = self.pattern_path(PATTERNS, id);
        let text = std::fs::read_to_string(&path).map_err(|source| AppError::LibraryRead {
            path: path.clone(),
            source,
        })?;
        let pattern = Pattern::from_json(&text)?;
        let history = std::fs::read_to_string(self.history_path(PATTERNS, id))
            .ok()
            .and_then(|file| crate::history_file::read(&file, &text))
            .unwrap_or_default();
        Ok((pattern, history))
    }

    /// Writes a pattern of the list and its history; the pattern first, so
    /// a history left behind by a failure is of another text and is not read.
    pub fn save(&self, pattern: &Pattern, history: &History) -> Result<()> {
        let text = pattern.to_json()?;
        let id = pattern.id();
        write(&self.pattern_path(PATTERNS, id), text.as_bytes())?;
        match crate::history_file::write(history, &text) {
            Ok(file) => write(&self.history_path(PATTERNS, id), file.as_bytes()),
            Err(error) => {
                log::error!("a history does not turn into text: {error}");
                Ok(())
            }
        }
    }

    /// Adds a pattern to the list with no history.
    pub fn add(&self, pattern: &Pattern) -> Result<()> {
        self.save(pattern, &History::default())
    }

    /// A copy of a pattern of the list under a new identifier, named `name`,
    /// with no history; it is added to the list.
    pub fn duplicate(&self, id: PatternId, name: &str) -> Result<Pattern> {
        let (mut pattern, _) = self.open(id)?;
        pattern.set_id(PatternId::new());
        pattern.set_name(name);
        self.add(&pattern)?;
        Ok(pattern)
    }

    /// Gives a pattern of the list a new name; its history stays.
    pub fn rename(&self, id: PatternId, name: &str) -> Result<()> {
        let (mut pattern, history) = self.open(id)?;
        pattern.set_name(name);
        self.save(&pattern, &history)
    }

    /// Locks a pattern against changes, or lets it be changed again.
    pub fn set_locked(&self, id: PatternId, locked: bool) -> Result<()> {
        let (mut pattern, history) = self.open(id)?;
        pattern.set_locked(locked);
        self.save(&pattern, &history)
    }

    /// Moves patterns of the list to the trash at `now`, with their
    /// histories.
    pub fn to_trash(&self, ids: &[PatternId], now: SystemTime) -> Result<()> {
        let seconds = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for id in ids {
            write(&self.trashed_path(*id), seconds.to_string().as_bytes())?;
            self.move_files(*id, PATTERNS, TRASH)?;
        }
        Ok(())
    }

    /// Puts a pattern of the trash back in the list. When the list has a
    /// pattern of its identifier, it comes back under a new one.
    pub fn restore(&self, id: PatternId) -> Result<PatternId> {
        if self.has(id) {
            let path = self.pattern_path(TRASH, id);
            let mut pattern = obusina_pattern::read(&path)?;
            pattern.set_id(PatternId::new());
            self.add(&pattern)?;
            self.purge(id)?;
            return Ok(pattern.id());
        }
        self.move_files(id, TRASH, PATTERNS)?;
        remove(&self.trashed_path(id))?;
        Ok(id)
    }

    /// Deletes a pattern of the trash for good.
    pub fn purge(&self, id: PatternId) -> Result<()> {
        remove(&self.pattern_path(TRASH, id))?;
        remove(&self.history_path(TRASH, id))?;
        remove(&self.trashed_path(id))
    }

    /// Deletes for good the patterns of the trash that went there
    /// [`TRASH_DAYS`] before `now`, and gives their identifiers.
    pub fn purge_old(&self, now: SystemTime) -> Vec<PatternId> {
        let most = Duration::from_secs(TRASH_DAYS * DAY);
        self.ids_in(TRASH)
            .into_iter()
            .filter(|id| {
                now.duration_since(self.trashed_at(*id))
                    .is_ok_and(|gone| gone >= most)
            })
            .filter(|id| {
                self.purge(*id)
                    .inspect_err(|error| log::error!("an old pattern of the trash stays: {error}"))
                    .is_ok()
            })
            .collect()
    }

    /// The patterns of a file that came in: each is added with its
    /// identifier, passed over when the list has the same work under it,
    /// or added under a new identifier when the list or the trash has
    /// another pattern under it.
    pub fn import(&self, patterns: Vec<Pattern>) -> Result<Report> {
        let mut report = Report::default();
        for mut pattern in patterns {
            let id = pattern.id();
            if self.has(id) {
                let (present, _) = self.open(id)?;
                if present.same_work(&pattern) {
                    report.present.push(present.name().to_owned());
                    continue;
                }
            }
            if self.has(id) || self.pattern_path(TRASH, id).exists() {
                pattern.set_id(PatternId::new());
                self.add(&pattern)?;
                report.copies.push(pattern.id());
            } else {
                self.add(&pattern)?;
                report.new.push(id);
            }
        }
        Ok(report)
    }

    /// The identifiers of the pattern files of a directory.
    fn ids_in(&self, directory: &str) -> Vec<PatternId> {
        let Ok(entries) = std::fs::read_dir(self.dir.join(directory)) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().into_string().ok()?;
                let stem = name.strip_suffix(PATTERN)?;
                if stem.ends_with(".history") {
                    return None;
                }
                stem.parse().ok()
            })
            .collect()
    }

    /// When a pattern went to the trash; the time of its pattern file when
    /// that is not kept.
    fn trashed_at(&self, id: PatternId) -> SystemTime {
        std::fs::read_to_string(self.trashed_path(id))
            .ok()
            .and_then(|text| text.trim().parse().ok())
            .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
            .unwrap_or_else(|| modified(&self.pattern_path(TRASH, id)))
    }

    /// Moves the pattern file and the history of `id` from one directory to
    /// another.
    fn move_files(&self, id: PatternId, from: &str, to: &str) -> Result<()> {
        std::fs::create_dir_all(self.dir.join(to)).map_err(|source| AppError::LibraryWrite {
            path: self.dir.join(to),
            source,
        })?;
        for path in [Library::pattern_path, Library::history_path] {
            let (source, target) = (path(self, from, id), path(self, to, id));
            if !source.exists() {
                continue;
            }
            std::fs::rename(&source, &target).map_err(|source| AppError::LibraryWrite {
                path: target,
                source,
            })?;
        }
        Ok(())
    }

    fn pattern_path(&self, directory: &str, id: PatternId) -> PathBuf {
        self.dir.join(directory).join(format!("{id}{PATTERN}"))
    }

    fn history_path(&self, directory: &str, id: PatternId) -> PathBuf {
        self.dir.join(directory).join(format!("{id}{HISTORY}"))
    }

    fn trashed_path(&self, id: PatternId) -> PathBuf {
        self.dir.join(TRASH).join(format!("{id}{TRASHED}"))
    }
}

/// Reads a pattern file; one that cannot be read is logged and passed over.
fn read_logged(path: &Path) -> Option<Pattern> {
    obusina_pattern::read(path)
        .inspect_err(|error| log::warn!("{} is passed over: {error}", path.display()))
        .ok()
}

/// The time a file was last written; the start of 1970 when it is not known.
fn modified(path: &Path) -> SystemTime {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Writes a file through a temporary file beside it and a rename
/// (`RULE pattern.write`), making its directory first.
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;
    let failed = |source| AppError::LibraryWrite {
        path: path.to_path_buf(),
        source,
    };
    let directory = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(directory).map_err(failed)?;
    let mut file = tempfile::NamedTempFile::new_in(directory).map_err(failed)?;
    file.write_all(bytes).map_err(failed)?;
    file.as_file().sync_all().map_err(failed)?;
    file.persist(path).map_err(|error| failed(error.error))?;
    Ok(())
}

/// Removes a file; one that is not there is no failure.
fn remove(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(AppError::LibraryWrite {
            path: path.to_path_buf(),
            source: error,
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
