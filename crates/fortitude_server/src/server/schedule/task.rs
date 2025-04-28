use crate::server::client::{Notifier, Requester, Responder};

type LocalFn<'s> = Box<dyn FnOnce(Notifier, &mut Requester, Responder) + 's>;

type BackgroundFn = Box<dyn FnOnce(Notifier, Responder) + Send + 'static>;

type BackgroundFnBuilder<'s> = Box<dyn FnOnce() -> BackgroundFn + 's>;

/// Describes how the task should be run.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::server) enum BackgroundSchedule {
    /// The task should be run on a regular-priority background thread.
    #[default]
    Worker,
}

/// A [`Task`] is a future that has not yet started, and it is the job of
/// the [`super::Scheduler`] to make that happen, via [`super::Scheduler::dispatch`].
/// A task can either run on the main thread (in other words, the same thread as the
/// scheduler) or it can run in a background thread. The main difference between
/// the two is that background threads only have a read-only snapshot of the session,
/// while local tasks have exclusive access and can modify it as they please. Keep in mind that
/// local tasks will **block** the main event loop, so only use local tasks if you **need**
/// mutable state access or you need the absolute lowest latency possible.
pub(in crate::server) enum Task<'s> {
    Sync(SyncTask<'s>),
}

pub(in crate::server) struct SyncTask<'s> {
    pub(super) func: LocalFn<'s>,
}

impl<'s> Task<'s> {
    /// Creates a new local task.
    pub(crate) fn local(func: impl FnOnce(Notifier, &mut Requester, Responder) + 's) -> Self {
        Self::Sync(SyncTask {
            func: Box::new(func),
        })
    }
    /// Creates a local task that does nothing.
    pub(crate) fn nothing() -> Self {
        Self::local(move |_, _, _| {})
    }
}
