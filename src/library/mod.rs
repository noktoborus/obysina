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
//!
//! The files are of `unifs`, so every call is `async` and the same on every
//! platform; the window runs them away from the frame (`src/work.rs`).

pub mod archive;

use std::time::Duration;

use obusina_pattern::{Pattern, PatternId};
use unifs::{Files, SystemTime};

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
#[derive(Debug, Clone)]
pub struct Library {
    files: Files,
}

impl Library {
    /// The library in the directory of `files`; its directories are made
    /// when it is first written.
    pub fn new(files: Files) -> Library {
        Library { files }
    }

    /// The patterns of the list, the last changed first.
    pub async fn list(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        for listed in self.files.list(PATTERNS).await? {
            let Some(id) = pattern_id(&listed.name) else {
                continue;
            };
            if let Some(pattern) = self.read_logged(&pattern_path(PATTERNS, id)).await {
                entries.push(Entry {
                    pattern,
                    changed: listed.modified,
                });
            }
        }
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.changed));
        Ok(entries)
    }

    /// The patterns of the trash, the last trashed first.
    pub async fn trash(&self) -> Result<Vec<Trashed>> {
        let mut trashed = Vec::new();
        for id in self.ids_in(TRASH).await? {
            if let Some(pattern) = self.read_logged(&pattern_path(TRASH, id)).await {
                let since = self.trashed_at(id).await;
                trashed.push(Trashed { pattern, since });
            }
        }
        trashed.sort_by_key(|trashed| std::cmp::Reverse(trashed.since));
        Ok(trashed)
    }

    /// True when the list has a pattern of this identifier.
    pub async fn has(&self, id: PatternId) -> Result<bool> {
        Ok(self
            .files
            .modified(&pattern_path(PATTERNS, id))
            .await?
            .is_some())
    }

    /// The pattern of the list of this identifier, and its history.
    pub async fn open(&self, id: PatternId) -> Result<(Pattern, History)> {
        let path = pattern_path(PATTERNS, id);
        let bytes = self.files.read(&path).await?.ok_or(AppError::NoPattern)?;
        let text = String::from_utf8_lossy(&bytes);
        let pattern = Pattern::from_json(&text)?;
        let history = self
            .files
            .read(&history_path(PATTERNS, id))
            .await
            .ok()
            .flatten()
            .and_then(|file| crate::history_file::read(&String::from_utf8_lossy(&file), &text))
            .unwrap_or_default();
        Ok((pattern, history))
    }

    /// Writes a pattern of the list and its history; the pattern first, so
    /// a history left behind by a failure is of another text and is not read.
    pub async fn save(&self, pattern: &Pattern, history: &History) -> Result<()> {
        let text = pattern.to_json()?;
        let id = pattern.id();
        self.files
            .write(&pattern_path(PATTERNS, id), text.as_bytes())
            .await?;
        match crate::history_file::write(history, &text) {
            Ok(file) => Ok(self
                .files
                .write(&history_path(PATTERNS, id), file.as_bytes())
                .await?),
            Err(error) => {
                log::error!("a history does not turn into text: {error}");
                Ok(())
            }
        }
    }

    /// Adds a pattern to the list with no history.
    pub async fn add(&self, pattern: &Pattern) -> Result<()> {
        self.save(pattern, &History::default()).await
    }

    /// A copy of a pattern of the list under a new identifier, named `name`,
    /// with no history; it is added to the list.
    pub async fn duplicate(&self, id: PatternId, name: &str) -> Result<Pattern> {
        let (mut pattern, _) = self.open(id).await?;
        pattern.set_id(PatternId::new());
        pattern.set_name(name);
        self.add(&pattern).await?;
        Ok(pattern)
    }

    /// Gives a pattern of the list a new name; its history stays.
    pub async fn rename(&self, id: PatternId, name: &str) -> Result<()> {
        let (mut pattern, history) = self.open(id).await?;
        pattern.set_name(name);
        self.save(&pattern, &history).await
    }

    /// Locks a pattern against changes, or lets it be changed again.
    pub async fn set_locked(&self, id: PatternId, locked: bool) -> Result<()> {
        let (mut pattern, history) = self.open(id).await?;
        pattern.set_locked(locked);
        self.save(&pattern, &history).await
    }

    /// Moves patterns of the list to the trash at `now`, with their
    /// histories.
    pub async fn to_trash(&self, ids: &[PatternId], now: SystemTime) -> Result<()> {
        let seconds = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for id in ids {
            self.files
                .write(&trashed_path(*id), seconds.to_string().as_bytes())
                .await?;
            self.move_files(*id, PATTERNS, TRASH).await?;
        }
        Ok(())
    }

    /// Puts a pattern of the trash back in the list. When the list has a
    /// pattern of its identifier, it comes back under a new one.
    pub async fn restore(&self, id: PatternId) -> Result<PatternId> {
        if self.has(id).await? {
            let bytes = self
                .files
                .read(&pattern_path(TRASH, id))
                .await?
                .ok_or(AppError::NoPattern)?;
            let mut pattern = Pattern::from_json(&String::from_utf8_lossy(&bytes))?;
            pattern.set_id(PatternId::new());
            self.add(&pattern).await?;
            self.purge(id).await?;
            return Ok(pattern.id());
        }
        self.move_files(id, TRASH, PATTERNS).await?;
        self.files.remove(&trashed_path(id)).await?;
        Ok(id)
    }

    /// Deletes a pattern of the trash for good.
    pub async fn purge(&self, id: PatternId) -> Result<()> {
        self.files.remove(&pattern_path(TRASH, id)).await?;
        self.files.remove(&history_path(TRASH, id)).await?;
        Ok(self.files.remove(&trashed_path(id)).await?)
    }

    /// Deletes for good the patterns of the trash that went there
    /// [`TRASH_DAYS`] before `now`, and gives their identifiers.
    pub async fn purge_old(&self, now: SystemTime) -> Vec<PatternId> {
        let most = Duration::from_secs(TRASH_DAYS * DAY);
        let mut gone = Vec::new();
        for id in self.ids_in(TRASH).await.unwrap_or_default() {
            let old = now
                .duration_since(self.trashed_at(id).await)
                .is_ok_and(|age| age >= most);
            if !old {
                continue;
            }
            match self.purge(id).await {
                Ok(()) => gone.push(id),
                Err(error) => log::error!("an old pattern of the trash stays: {error}"),
            }
        }
        gone
    }

    /// The patterns of a file that came in: each is added with its
    /// identifier, passed over when the list has the same work under it,
    /// or added under a new identifier when the list or the trash has
    /// another pattern under it.
    pub async fn import(&self, patterns: Vec<Pattern>) -> Result<Report> {
        let mut report = Report::default();
        for mut pattern in patterns {
            let id = pattern.id();
            let listed = self.has(id).await?;
            if listed {
                let (present, _) = self.open(id).await?;
                if present.same_work(&pattern) {
                    report.present.push(present.name().to_owned());
                    continue;
                }
            }
            let trashed = self.files.modified(&pattern_path(TRASH, id)).await?;
            if listed || trashed.is_some() {
                pattern.set_id(PatternId::new());
                self.add(&pattern).await?;
                report.copies.push(pattern.id());
            } else {
                self.add(&pattern).await?;
                report.new.push(id);
            }
        }
        Ok(report)
    }

    /// Takes in the one pattern file the version before the list kept on
    /// android, `pattern.json` of the data directory: it is the work of the
    /// person. It is renamed once it is in the list.
    pub async fn take_in_old(&self) -> Result<()> {
        let Some(bytes) = self.files.read(OLD_PATTERN).await? else {
            return Ok(());
        };
        let pattern = Pattern::from_json(&String::from_utf8_lossy(&bytes))?;
        self.import(vec![pattern]).await?;
        Ok(self.files.rename(OLD_PATTERN, OLD_PATTERN_TAKEN).await?)
    }

    /// The identifiers of the pattern files of a directory.
    async fn ids_in(&self, directory: &str) -> Result<Vec<PatternId>> {
        let listed = self.files.list(directory).await?;
        Ok(listed
            .iter()
            .filter_map(|listed| pattern_id(&listed.name))
            .collect())
    }

    /// When a pattern went to the trash; the time of its pattern file when
    /// that is not kept.
    async fn trashed_at(&self, id: PatternId) -> SystemTime {
        let kept = self.files.read(&trashed_path(id)).await.ok().flatten();
        let seconds = kept.and_then(|bytes| String::from_utf8_lossy(&bytes).trim().parse().ok());
        match seconds {
            Some(seconds) => SystemTime::UNIX_EPOCH + Duration::from_secs(seconds),
            None => self
                .files
                .modified(&pattern_path(TRASH, id))
                .await
                .ok()
                .flatten()
                .unwrap_or(SystemTime::UNIX_EPOCH),
        }
    }

    /// Moves the pattern file and the history of `id` from one directory to
    /// another.
    async fn move_files(&self, id: PatternId, from: &str, to: &str) -> Result<()> {
        for path in [pattern_path, history_path] {
            self.files.rename(&path(from, id), &path(to, id)).await?;
        }
        Ok(())
    }

    /// Reads a pattern file; one that cannot be read is logged and passed
    /// over.
    async fn read_logged(&self, path: &str) -> Option<Pattern> {
        let read = match self.files.read(path).await {
            Ok(Some(bytes)) => {
                Pattern::from_json(&String::from_utf8_lossy(&bytes)).map_err(AppError::from)
            }
            Ok(None) => return None,
            Err(error) => Err(error.into()),
        };
        read.inspect_err(|error| log::warn!("{path} is passed over: {error}"))
            .ok()
    }
}

/// The identifier of a pattern file by its name; none for a history or any
/// other file.
fn pattern_id(name: &str) -> Option<PatternId> {
    let stem = name.strip_suffix(PATTERN)?;
    if stem.ends_with(".history") {
        return None;
    }
    stem.parse().ok()
}

fn pattern_path(directory: &str, id: PatternId) -> String {
    format!("{directory}/{id}{PATTERN}")
}

fn history_path(directory: &str, id: PatternId) -> String {
    format!("{directory}/{id}{HISTORY}")
}

fn trashed_path(id: PatternId) -> String {
    format!("{TRASH}/{id}{TRASHED}")
}

/// The one pattern the version before the list kept on android.
const OLD_PATTERN: &str = "pattern.json";
/// What it is renamed to once it is in the list.
const OLD_PATTERN_TAKEN: &str = "pattern.json.taken";

#[cfg(test)]
mod tests;
