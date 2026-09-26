//! The single worker thread that owns every display handle (D-6), the seam
//! it drives the hardware through, and the client that talks to it.
//!
//! Handles never leave the worker thread, so the client holds only a job
//! queue and is `Send + Sync` whatever the handles are. Each job answers on
//! its own channel; a caller waits at most its budget, then gets
//! [`DdcError::Timeout`] even if the worker is still busy.

use std::fmt;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use super::DdcHiBudgets;
use super::hardware::capabilities_text;
use super::identity::{DisplayIdentity, monitor_infos};

/// Where the worker finds displays: `ddc-hi` in production, fakes in tests.
pub(crate) trait DisplaySource: Send + 'static {
    /// Handle to one display, created and used only on the worker thread.
    type Handle: DdcHandle;

    /// Every display currently attached, with what identifies it.
    fn enumerate(&mut self) -> Vec<(DisplayIdentity, Self::Handle)>;
}

/// One DDC/CI transaction per call, already in core types.
pub(crate) trait DdcHandle {
    /// Raw capabilities reply.
    fn read_capabilities(&mut self) -> Result<Vec<u8>, HandleError>;

    /// Get VCP Feature.
    fn read_vcp(&mut self, code: VcpCode) -> Result<VcpValue, HandleError>;

    /// Set VCP Feature.
    fn write_vcp(&mut self, code: VcpCode, value: u16) -> Result<(), HandleError>;
}

/// A failed transaction, reduced to its message so no transport error type
/// crosses the seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HandleError(String);

impl HandleError {
    /// Keeps the whole cause chain of `error` (`{:#}`).
    pub(crate) fn new(error: impl fmt::Display) -> Self {
        Self(format!("{error:#}"))
    }
}

type Job<S> = Box<dyn FnOnce(&mut Worker<S>) + Send>;

/// Owner of the displays; runs one job at a time on its own thread.
pub(crate) struct Worker<S: DisplaySource> {
    source: S,
    displays: Vec<(MonitorId, S::Handle)>,
}

impl<S: DisplaySource> Worker<S> {
    fn new(source: S) -> Self {
        Self {
            source,
            displays: Vec::new(),
        }
    }

    /// Serves jobs until every client is gone.
    fn run(mut self, jobs: Receiver<Job<S>>) {
        for job in jobs {
            job(&mut self);
        }
    }

    /// Replaces the whole display table with a fresh enumeration.
    fn enumerate(&mut self) -> Vec<MonitorInfo> {
        let (identities, handles): (Vec<_>, Vec<_>) = self.source.enumerate().into_iter().unzip();
        let infos = monitor_infos(&identities);
        self.displays = infos
            .iter()
            .map(|info| info.id.clone())
            .zip(handles)
            .collect();
        infos
    }

    fn read_capabilities(&mut self, id: &MonitorId) -> Result<String, DdcError> {
        self.transact(id, DdcHandle::read_capabilities)
            .map(|raw| capabilities_text(&raw))
    }

    fn read_vcp(&mut self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.transact(id, |handle| handle.read_vcp(code))
    }

    fn write_vcp(&mut self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        self.transact(id, |handle| handle.write_vcp(code, value))
    }

    fn transact<T>(
        &mut self,
        id: &MonitorId,
        op: impl FnOnce(&mut S::Handle) -> Result<T, HandleError>,
    ) -> Result<T, DdcError> {
        let handle = self.handle(id)?;
        op(handle).map_err(|HandleError(message)| DdcError::Transport(message))
    }

    /// The handle for `id`, enumerating again once if the id is unknown.
    fn handle(&mut self, id: &MonitorId) -> Result<&mut S::Handle, DdcError> {
        if !self.knows(id) {
            self.enumerate();
        }
        self.displays
            .iter_mut()
            .find(|(known, _)| known == id)
            .map(|(_, handle)| handle)
            .ok_or_else(|| DdcError::MonitorNotFound(id.clone()))
    }

    fn knows(&self, id: &MonitorId) -> bool {
        self.displays.iter().any(|(known, _)| known == id)
    }
}

/// [`MonitorBackend`] that forwards every call to a [`Worker`] thread.
pub(crate) struct WorkerClient<S: DisplaySource> {
    jobs: Sender<Job<S>>,
    budgets: DdcHiBudgets,
    _worker: JoinHandle<()>,
}

impl<S: DisplaySource> WorkerClient<S> {
    /// Starts the worker. Nothing is enumerated until the first request.
    pub(crate) fn spawn(source: S, budgets: DdcHiBudgets) -> Result<Self, DdcError> {
        let (jobs, queue) = mpsc::channel::<Job<S>>();
        let worker = thread::Builder::new()
            .name("ddc-hi-worker".to_owned())
            .spawn(move || Worker::new(source).run(queue))
            .map_err(|error| {
                DdcError::Transport(format!("cannot start the DDC worker: {error}"))
            })?;
        Ok(Self {
            jobs,
            budgets,
            _worker: worker,
        })
    }

    pub(crate) fn with_budgets(self, budgets: DdcHiBudgets) -> Self {
        Self { budgets, ..self }
    }

    /// Queues `op` and waits at most `budget` for its answer.
    fn call<T: Send + 'static>(
        &self,
        budget: Duration,
        op: impl FnOnce(&mut Worker<S>) -> Result<T, DdcError> + Send + 'static,
    ) -> Result<T, DdcError> {
        let (reply, answer) = mpsc::channel();
        let job: Job<S> = Box::new(move |worker| {
            // A caller past its budget has dropped the receiver; the answer
            // is then discarded on purpose.
            let _ = reply.send(op(worker));
        });
        self.jobs.send(job).map_err(|_| worker_gone())?;
        answer.recv_timeout(budget).map_err(|error| match error {
            RecvTimeoutError::Timeout => DdcError::Timeout,
            RecvTimeoutError::Disconnected => worker_gone(),
        })?
    }
}

impl<S: DisplaySource> fmt::Debug for WorkerClient<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerClient")
            .field("budgets", &self.budgets)
            .finish_non_exhaustive()
    }
}

impl<S: DisplaySource> MonitorBackend for WorkerClient<S> {
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        self.call(self.budgets.enumerate, |worker| Ok(worker.enumerate()))
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        let id = id.clone();
        self.call(self.budgets.capabilities, move |worker| {
            worker.read_capabilities(&id)
        })
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        let id = id.clone();
        self.call(self.budgets.vcp, move |worker| worker.read_vcp(&id, code))
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        let id = id.clone();
        self.call(self.budgets.vcp, move |worker| {
            worker.write_vcp(&id, code, value)
        })
    }
}

/// The worker thread ended (a bug in the transport panicked it); every later
/// call fails the same way.
fn worker_gone() -> DdcError {
    DdcError::Transport("the DDC worker thread is not running".to_owned())
}

#[cfg(test)]
mod tests;
