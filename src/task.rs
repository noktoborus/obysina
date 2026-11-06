//! The work of the store of pictures, done one job after another away from
//! the frame: on a thread of its own on the desktop and android, as a future
//! of the browser on the web. A test does each job at once, so it waits for
//! nothing.
//!
//! ```text
//! Tasks::run(job) ──► the queue ──► the worker, in order ──► the job sends its
//!                                                           answer to the window
//! ```

use std::future::Future;
use std::pin::Pin;

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedSender, unbounded};

/// A job of the queue.
#[cfg(not(target_arch = "wasm32"))]
type Job = Pin<Box<dyn Future<Output = ()> + Send>>;
/// A job of the queue: the browser runs it on its one thread.
#[cfg(target_arch = "wasm32")]
type Job = Pin<Box<dyn Future<Output = ()>>>;

/// The queue of jobs, and its worker.
pub struct Tasks {
    /// Where jobs go; `None` when each is done at once.
    queue: Option<UnboundedSender<Job>>,
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
            held: None,
        };
        #[cfg(not(test))]
        Tasks::worker()
    }

    /// A queue with a worker of its own.
    #[cfg_attr(test, allow(dead_code))]
    fn worker() -> Tasks {
        let (queue, mut jobs) = unbounded::<Job>();
        let worker = async move {
            while let Some(job) = jobs.next().await {
                job.await;
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || futures::executor::block_on(worker));
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(worker);
        Tasks {
            queue: Some(queue),
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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&self, job: impl Future<Output = ()> + Send + 'static) {
        self.push(Box::pin(job));
    }

    /// Puts a job at the end of the queue.
    #[cfg(target_arch = "wasm32")]
    pub fn run(&self, job: impl Future<Output = ()> + 'static) {
        self.push(Box::pin(job));
    }

    fn push(&self, job: Job) {
        #[cfg(test)]
        if let Some(held) = &self.held {
            held.lock().expect("the jobs").push(job);
            return;
        }
        match &self.queue {
            Some(queue) => {
                if queue.unbounded_send(job).is_err() {
                    log::error!("the worker of the store of pictures is gone");
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
