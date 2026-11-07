//! The work of the window on its files, away from the frame: the list of
//! patterns, the pattern open, the settings. A job runs on the queue of
//! [`crate::task`]; what it gives comes back as [`Done`] and is taken at the
//! start of a later frame. The code is the same on every platform: the
//! files are of `unifs`.
//!
//! ```text
//! App::work(job) ──► the queue ──► the library, async ──► [Done] ──► the channel
//! a frame: App::take_done ──► the list read, the pattern opened, a toast
//! ```
//!
//! The jobs are done in the order they were given: a save given before the
//! list is read again is on the disk when it is read.

use std::future::Future;
use std::sync::mpsc::{Receiver, Sender, channel};

use obusina_pattern::{Pattern, PatternId};

use crate::app::{App, say};
use crate::edit::History;
use crate::error::AppError;
use crate::library::{Entry, Library, Report, Trashed};
use crate::task::{MaybeSend, Tasks};

/// What a job gives back to the window.
#[derive(Debug)]
pub(crate) enum Done {
    /// The list and the trash as they are on the disk; the patterns of the
    /// trash deleted for being old.
    Listed {
        entries: Vec<Entry>,
        trash: Vec<Trashed>,
        gone: Vec<PatternId>,
    },
    /// A pattern of the list, to be opened.
    Opened { pattern: Pattern, history: History },
    /// What an import did, and the pattern to open after it.
    Imported {
        report: Report,
        open: Option<PatternId>,
    },
    /// A copy of a pattern was made: its picture is copied too.
    Duplicated { from: PatternId, to: PatternId },
    /// The list changed: it is read again.
    Changed,
    /// A work on the list failed: told on the list.
    Failed(AppError),
    /// The pattern open was not saved: told on the field, and saved again
    /// after the next rest.
    SaveFailed(AppError),
    /// The settings were not written: told on the field.
    SettingsFailed(AppError),
}

/// The queue of the work on the files, and the way its answers come back.
pub(crate) struct Work {
    tasks: Tasks,
    answers: (Sender<Vec<Done>>, Receiver<Vec<Done>>),
    /// The window, asked to draw again when an answer comes; known from the
    /// first frame.
    context: Option<egui::Context>,
}

impl Default for Work {
    fn default() -> Work {
        Work {
            tasks: Tasks::new(),
            answers: channel(),
            context: None,
        }
    }
}

impl Work {
    /// Runs `job` on the queue; what it gives comes back to the window.
    pub(crate) fn run(&self, job: impl Future<Output = Vec<Done>> + MaybeSend + 'static) {
        let sender = self.answers.0.clone();
        let context = self.context.clone();
        self.tasks.run(async move {
            let done = job.await;
            if !done.is_empty()
                && sender.send(done).is_ok()
                && let Some(context) = context
            {
                context.request_repaint();
            }
        });
    }

    /// The queue: a test holds its jobs back; the window waits for them
    /// when it closes.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub(crate) fn tasks(&mut self) -> &mut Tasks {
        &mut self.tasks
    }
}

impl App {
    /// Runs `work` on the library away from the frame; nothing when the
    /// platform gives no place for patterns.
    pub(crate) fn work<W, F>(&self, work: W)
    where
        W: FnOnce(Library) -> F + MaybeSend + 'static,
        F: Future<Output = Vec<Done>> + MaybeSend + 'static,
    {
        if let Some(library) = self.library.clone() {
            self.work.run(work(library));
        }
    }

    /// Runs a change of the list on the library: a failure is told, and the
    /// list is read again after it.
    pub(crate) fn change<W, F>(&self, work: W)
    where
        W: FnOnce(Library) -> F + MaybeSend + 'static,
        F: Future<Output = crate::error::Result<()>> + MaybeSend + 'static,
    {
        self.work(move |library| async move {
            match work(library).await {
                Ok(()) => vec![Done::Changed],
                Err(error) => vec![Done::Failed(error), Done::Changed],
            }
        });
    }

    /// Writes the settings away from the frame.
    pub(crate) fn write_settings(&self, write: Option<crate::settings::Write>) {
        let Some(write) = write else {
            return;
        };
        self.work.run(async move {
            match write.run().await {
                Ok(()) => Vec::new(),
                Err(error) => vec![Done::SettingsFailed(error)],
            }
        });
    }

    /// Takes what the jobs gave since the last frame.
    pub(crate) fn take_done(&mut self, context: &egui::Context) {
        if self.work.context.is_none() {
            self.work.context = Some(context.clone());
        }
        let time = context.input(|input| input.time);
        while let Ok(done) = self.work.answers.1.try_recv() {
            for done in done {
                self.done(context, done, time);
            }
        }
    }

    /// Takes what the jobs gave, with no window: a test of the state alone.
    #[cfg(test)]
    pub(crate) fn settle(&mut self) {
        self.take_done(&egui::Context::default());
    }

    /// Does what one answer of a job says.
    fn done(&mut self, context: &egui::Context, done: Done, time: f64) {
        match done {
            Done::Listed {
                entries,
                trash,
                gone,
            } => {
                self.projects.listed(entries, trash);
                if !gone.is_empty() {
                    self.backdrop.forget(context, gone);
                }
            }
            Done::Opened { pattern, history } => self.open_pattern(pattern, history, time),
            Done::Imported { report, open } => {
                self.projects.tell_import(&report);
                self.projects.refresh();
                if let Some(id) = open {
                    self.open_by_id(id, time);
                }
            }
            Done::Duplicated { from, to } => {
                self.backdrop.copy(context, from, to);
                self.projects.refresh();
            }
            Done::Changed => self.projects.refresh(),
            Done::Failed(error) => self.projects.tell(say(&error)),
            Done::SaveFailed(error) => {
                self.message = Some(say(&error));
                self.saved.failed(time);
            }
            Done::SettingsFailed(error) => self.message = Some(say(&error)),
        }
    }
}

/// Imports the patterns of `bytes`; gives what came in, and the pattern to
/// open when `open`: the first one added, or the one the list had of it.
pub(crate) async fn import(library: Library, bytes: Vec<u8>, open: bool) -> Vec<Done> {
    let patterns = match crate::library::archive::read(&bytes) {
        Ok(patterns) => patterns,
        Err(error) => return vec![Done::Failed(error)],
    };
    let first = patterns.first().map(Pattern::id);
    let report = match library.import(patterns).await {
        Ok(report) => report,
        Err(error) => return vec![Done::Failed(error), Done::Changed],
    };
    let mut opened = report.added().next();
    if opened.is_none()
        && let Some(first) = first
        && library.has(first).await.unwrap_or(false)
    {
        opened = Some(first);
    }
    vec![Done::Imported {
        report,
        open: opened.filter(|_| open),
    }]
}
