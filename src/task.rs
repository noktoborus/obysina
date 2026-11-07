//! The work on the files, done one job after another away from the frame:
//! on a thread of its own on the desktop and android, as a future of the
//! browser on the web. A test does each job at once, so it waits for
//! nothing.
//!
//! ```text
//! Tasks::run(job) ──► the queue ──► the worker, in order ──► the job sends its
//!                                                           answer to the window
//! ```
//!
//! The code of a job is the same on every platform: what it holds must be
//! [`MaybeSend`], which is `Send` where the worker is a thread.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedSender, unbounded};

/// What a job may hold: sent to the thread of the worker on the desktop and
/// android.
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

/// What a job may hold: anything, the browser runs it on its one thread.
#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

/// A job of the queue.
#[cfg(not(target_arch = "wasm32"))]
type Job = Pin<Box<dyn Future<Output = ()> + Send>>;
/// A job of the queue: the browser runs it on its one thread.
#[cfg(target_arch = "wasm32")]
type Job = Pin<Box<dyn Future<Output = ()>>>;

/// The jobs given and not done yet, and a signal when that changes.
type Pending = Arc<(Mutex<usize>, Condvar)>;

/// The queue of jobs, and its worker.
pub struct Tasks {
    /// Where jobs go; `None` when each is done at once.
    queue: Option<UnboundedSender<Job>>,
    pending: Pending,
    /// The jobs a test holds back, to see the window while they wait.
    #[cfg(test)]
    held: Option<std::sync::Mutex<Vec<Job>>>,
}

impl Tasks {
    /// A queue with a worker of its own; a test does each job at once.
    pub fn new() -> Tasks {
        #[cfg(test)]
        return Tasks {
            queue: None,
            pending: Pending::default(),
            held: None,
        };
        #[cfg(not(test))]
        Tasks::worker()
    }

    /// A queue with a worker of its own.
    #[cfg_attr(test, allow(dead_code))]
    fn worker() -> Tasks {
        let (queue, mut jobs) = unbounded::<Job>();
        let pending = Pending::default();
        let done = Arc::clone(&pending);
        let worker = async move {
            while let Some(job) = jobs.next().await {
                job.await;
                let (count, signal) = &*done;
                *count.lock().expect("the count is never poisoned") -= 1;
                signal.notify_all();
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || futures::executor::block_on(worker));
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(worker);
        Tasks {
            queue: Some(queue),
            pending,
            #[cfg(test)]
            held: None,
        }
    }

    /// Holds the jobs back until [`Tasks::release`].
    #[cfg(test)]
    pub fn hold(&mut self) {
        self.held = Some(std::sync::Mutex::new(Vec::new()));
    }

    /// Does the jobs held back, in order, and every job from now on at once.
    #[cfg(test)]
    pub fn release(&mut self) {
        let held = self
            .held
            .take()
            .map(|held| held.into_inner().expect("the jobs"));
        for job in held.unwrap_or_default() {
            futures::executor::block_on(job);
        }
    }

    /// Puts a job at the end of the queue.
    pub fn run(&self, job: impl Future<Output = ()> + MaybeSend + 'static) {
        self.push(Box::pin(job));
    }

    /// Waits until every job given is done, `most` at the longest: what is
    /// written when the window closes or android pauses the program is on
    /// the disk before the process may end. A browser cannot wait.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn wait(&self, most: std::time::Duration) {
        let (count, signal) = &*self.pending;
        let count = count.lock().expect("the count is never poisoned");
        let waited = signal.wait_timeout_while(count, most, |count| *count > 0);
        if waited.is_ok_and(|(_, timeout)| timeout.timed_out()) {
            log::error!("the files are not all written: the wait ran out");
        }
    }

    fn push(&self, job: Job) {
        #[cfg(test)]
        if let Some(held) = &self.held {
            held.lock().expect("the jobs").push(job);
            return;
        }
        match &self.queue {
            Some(queue) => {
                *self.pending.0.lock().expect("the count is never poisoned") += 1;
                if queue.unbounded_send(job).is_err() {
                    *self.pending.0.lock().expect("the count is never poisoned") -= 1;
                    log::error!("the worker of the files is gone");
                }
            }
            None => futures::executor::block_on(job),
        }
    }
}

impl Default for Tasks {
    fn default() -> Self {
        Tasks::new()
    }
}
