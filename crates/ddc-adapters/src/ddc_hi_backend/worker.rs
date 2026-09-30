//! The single worker thread that owns every display handle (D-6), the seam
//! it drives the hardware through, and the client that talks to it.
//!
//! Handles never leave the worker thread, so the client holds only a job
//! queue and is `Send + Sync` whatever the handles are. Each job answers on
//! its own channel; a caller waits at most its budget, then gets
//! [`DdcError::Timeout`] even if the worker is still busy. A job whose
//! caller already gave up never reaches the hardware, so a write reported
//! as timed out is never applied later.
//!
//! Every step runs under the budget of its own kind
//! (D-2026-09-26-ddc-backends-1): the worker never enumerates inside a
//! transaction's budget. An unknown id is reported back at once, and the
//! client enumerates under the enumeration budget before asking again.
//!
//! A panic inside one attempt of a transaction fails only that transaction
//! (D-2026-09-26-full-osd-control-6): the worker keeps its display table and
//! serves the next job.

use std::any::Any;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use super::DdcHiBudgets;
use super::hardware::capabilities_text;
use super::identity::{DisplayIdentity, monitor_infos};
use super::retry::{Clock, Failure, RetryPolicies, RetryPolicy, SystemClock};

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

    /// Get VCP Feature, as the monitor answered it: the worker checks which
    /// code the reply is for.
    fn read_vcp(&mut self, code: VcpCode) -> Result<VcpReply, HandleError>;

    /// Set VCP Feature.
    fn write_vcp(&mut self, code: VcpCode, value: u16) -> Result<(), HandleError>;
}

/// A Get VCP Feature reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VcpReply {
    /// Current value and maximum.
    pub(crate) value: VcpValue,
    /// The VCP code the monitor echoes in the reply; `None` where the
    /// platform backend does not hand it over.
    pub(crate) echoed: Option<VcpCode>,
}

impl VcpReply {
    /// The value, when the reply answers `code`. A reply for another code is
    /// a late reply to an earlier request, left on the bus when that one
    /// gave up. Taking it would report, and bound writes by, another
    /// feature's value; like any garbled reply it is a transient failure, so
    /// the read is tried again (D-2026-09-26-full-osd-control-10). A reply
    /// with no echo is taken as it is.
    pub(crate) fn answering(self, code: VcpCode) -> Result<VcpValue, HandleError> {
        match self.echoed {
            Some(other) if other != code => Err(HandleError::new(format_args!(
                "reply answers VCP code {other}, not {code}"
            ))),
            _ => Ok(self.value),
        }
    }
}

/// A failed transaction, reduced to its message so no transport error type
/// crosses the seam, and to whether asking again could change it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HandleError {
    kind: HandleErrorKind,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandleErrorKind {
    /// A NAK, a bus error, a garbled reply: a retry may cure it.
    Transient,
    /// The monitor answered that it does not support the VCP code.
    Unsupported,
    /// The transport panicked; the same request would panic again.
    Panicked,
}

impl HandleError {
    /// A failure a retry may cure. Keeps the whole cause chain of `error`
    /// (`{:#}`).
    pub(crate) fn new(error: impl fmt::Display) -> Self {
        Self::of_kind(HandleErrorKind::Transient, error)
    }

    /// The monitor's answer that it does not support the VCP code: final,
    /// asking again gets the same answer (D-2026-09-26-cli-4). Keeps the
    /// whole cause chain of `error` (`{:#}`).
    pub(crate) fn unsupported(error: impl fmt::Display) -> Self {
        Self::of_kind(HandleErrorKind::Unsupported, error)
    }

    /// A panic caught inside the transaction: final, since the same request
    /// would panic again (D-2026-09-26-full-osd-control-6). Keeps the panic
    /// message when it is text.
    pub(crate) fn panicked(payload: &(dyn Any + Send)) -> Self {
        Self::of_kind(HandleErrorKind::Panicked, panic_text(payload))
    }

    fn of_kind(kind: HandleErrorKind, error: impl fmt::Display) -> Self {
        Self {
            kind,
            message: format!("{error:#}"),
        }
    }

    /// Whether the monitor refused the request as unsupported.
    pub(crate) fn is_unsupported(&self) -> bool {
        self.kind == HandleErrorKind::Unsupported
    }

    /// Whether the transport panicked during the request.
    pub(crate) fn is_panic(&self) -> bool {
        self.kind == HandleErrorKind::Panicked
    }
}

/// The message of a panic: `panic!` hands over a `&str` or a `String`.
fn panic_text(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("panic payload is not text")
}

/// Runs one attempt of a transaction, turning a panic inside it into a
/// final [`HandleError`], so a bug in the transport never unwinds the
/// worker (D-2026-09-26-full-osd-control-6).
// WHY AssertUnwindSafe: after a panic the handle is only ever asked for
// whole new transactions, each of which resends its request from scratch.
fn isolated<T>(attempt: impl FnOnce() -> Result<T, HandleError>) -> Result<T, HandleError> {
    panic::catch_unwind(AssertUnwindSafe(attempt))
        .unwrap_or_else(|payload| Err(HandleError::panicked(payload.as_ref())))
}

/// Why the worker ended a transaction without a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TransactError {
    /// The id is not in the display table. Never reaches a caller: the
    /// client enumerates and asks once more.
    UnknownMonitor,
    /// The caller's answer.
    Failed(DdcError),
}

impl TransactError {
    /// The caller's error once the id has been looked up afresh.
    fn for_caller(self, id: &MonitorId) -> DdcError {
        match self {
            Self::UnknownMonitor => DdcError::MonitorNotFound(id.clone()),
            Self::Failed(error) => error,
        }
    }
}

impl From<DdcError> for TransactError {
    fn from(error: DdcError) -> Self {
        Self::Failed(error)
    }
}

/// How long a caller waits for a write of `code`. A write of the input source
/// waits out the settle window of `policies` on top of the VCP budget.
pub(crate) fn write_budget(
    budgets: &DdcHiBudgets,
    policies: &RetryPolicies,
    code: VcpCode,
) -> Duration {
    if code == VcpCode::INPUT_SOURCE {
        budgets.vcp + policies.input_settle.window
    } else {
        budgets.vcp
    }
}

type Job<S> = Box<dyn FnOnce(&mut Worker<S, SystemClock>) + Send>;

/// Owner of the displays; runs one job at a time on its own thread.
pub(crate) struct Worker<S: DisplaySource, C> {
    source: S,
    displays: Vec<(MonitorId, S::Handle)>,
    /// How long the last enumeration took; `None` before the first one.
    enumeration_cost: Option<Duration>,
    policies: RetryPolicies,
    clock: C,
}

impl<S: DisplaySource> Worker<S, SystemClock> {
    /// Serves jobs until every client is gone.
    fn run(mut self, jobs: Receiver<Job<S>>) {
        for job in jobs {
            job(&mut self);
        }
    }
}

impl<S: DisplaySource, C: Clock> Worker<S, C> {
    fn new(source: S, policies: RetryPolicies, clock: C) -> Self {
        Self {
            source,
            displays: Vec::new(),
            enumeration_cost: None,
            policies,
            clock,
        }
    }

    /// Runs `op` unless its caller's `deadline` has passed: a job nobody
    /// waits for anymore touches no hardware.
    fn serve<T, E: From<DdcError>>(
        &mut self,
        deadline: Instant,
        op: impl FnOnce(&mut Self, Instant) -> Result<T, E>,
    ) -> Result<T, E> {
        if self.clock.now() >= deadline {
            return Err(DdcError::Timeout.into());
        }
        op(self, deadline)
    }

    /// Replaces the whole display table with a fresh enumeration, and
    /// remembers how long it took.
    fn enumerate(&mut self) -> Vec<MonitorInfo> {
        let started = self.clock.now();
        let (identities, handles): (Vec<_>, Vec<_>) = self.source.enumerate().into_iter().unzip();
        self.enumeration_cost = Some(self.clock.now().saturating_duration_since(started));
        let infos = monitor_infos(&identities);
        self.displays = infos
            .iter()
            .map(|info| info.id.clone())
            .zip(handles)
            .collect();
        infos
    }

    /// A DDC/CI capabilities reply carries no result code, so a refusal
    /// cannot happen with `ddc-hi`; were one reported, it would be a
    /// transport failure.
    fn read_capabilities(
        &mut self,
        id: &MonitorId,
        deadline: Instant,
    ) -> Result<String, TransactError> {
        let policy = self.policies.capabilities;
        self.transact(
            id,
            deadline,
            policy,
            DdcError::Transport,
            DdcHandle::read_capabilities,
        )
        .map(|raw| capabilities_text(&raw))
    }

    fn read_vcp(
        &mut self,
        id: &MonitorId,
        code: VcpCode,
        deadline: Instant,
    ) -> Result<VcpValue, TransactError> {
        let policy = self.policies.vcp;
        let refused = move |_| DdcError::UnsupportedFeature(code);
        self.transact(id, deadline, policy, refused, |handle| {
            handle.read_vcp(code)?.answering(code)
        })
    }

    fn write_vcp(
        &mut self,
        id: &MonitorId,
        code: VcpCode,
        value: u16,
        deadline: Instant,
    ) -> Result<(), TransactError> {
        let policy = self.policies.vcp;
        let refused = move |_| DdcError::UnsupportedFeature(code);
        self.transact(id, deadline, policy, refused, |handle| {
            handle.write_vcp(code, value)
        })?;
        if code == VcpCode::INPUT_SOURCE {
            self.settle_input(id, value, deadline);
        }
        Ok(())
    }

    /// Waits for the monitor to show the input it was just told to switch
    /// to: reads the input every settle step of the policies until it reads
    /// `value`, or until the settle window after the write, never past
    /// `deadline`. A monitor that keeps the old input is not an error: the
    /// write was accepted, and the caller's own read after it tells what the
    /// monitor kept (D-2026-09-30-input-switch-autostart-3).
    fn settle_input(&mut self, id: &MonitorId, value: u16, deadline: Instant) {
        let settle = self.policies.input_settle;
        let end = (self.clock.now() + settle.window).min(deadline);
        loop {
            let left = end.saturating_duration_since(self.clock.now());
            if left.is_zero() {
                return;
            }
            self.clock.sleep(left.min(settle.step));
            if !self.input_unsettled(id, value) {
                return;
            }
        }
    }

    /// Whether the input still has to be waited for after one more read of
    /// it. The loop around this read is the retry, so the read is a single
    /// [`isolated`] attempt: a failed or garbled reply counts as "not yet".
    /// A panic or a refusal is final, as for any read.
    fn input_unsettled(&mut self, id: &MonitorId, value: u16) -> bool {
        let Some(handle) = handle_of(&mut self.displays, id) else {
            return false;
        };
        let code = VcpCode::INPUT_SOURCE;
        match isolated(|| handle.read_vcp(code)?.answering(code)) {
            Ok(read) => !same_input(read.current, value),
            Err(error) => !(error.is_unsupported() || error.is_panic()),
        }
    }

    /// Runs `op` on the handle of `id` under `policy`, each attempt
    /// [`isolated`] from panics. An unknown id is reported at once, without
    /// enumerating inside this `deadline`. A request the monitor refuses as
    /// unsupported answers `refused` of the refusal's message.
    fn transact<T>(
        &mut self,
        id: &MonitorId,
        deadline: Instant,
        policy: RetryPolicy,
        refused: impl FnOnce(String) -> DdcError,
        mut op: impl FnMut(&mut S::Handle) -> Result<T, HandleError>,
    ) -> Result<T, TransactError> {
        let handle = handle_of(&mut self.displays, id).ok_or(TransactError::UnknownMonitor)?;
        let outcome = policy.run(&self.clock, deadline, || isolated(|| op(handle)));
        outcome
            .map_err(|failure| TransactError::Failed(self.explain(id, failure, deadline, refused)))
    }

    /// The caller's error for a give-up. A refusal is the monitor's final
    /// answer, reported at once as `refused` says (D-2026-09-26-cli-4). A
    /// panic is a transport failure, reported at once with its message
    /// (D-2026-09-26-full-osd-control-6). Otherwise, when one more enumeration fits before `deadline`, the
    /// monitor is looked up again: an unplugged one is reported missing,
    /// not as a transport failure the core would remember against its id.
    /// Else the transport failure is reported at once, so a mute display
    /// frees the queue fast.
    fn explain(
        &mut self,
        id: &MonitorId,
        failure: Failure,
        deadline: Instant,
        refused: impl FnOnce(String) -> DdcError,
    ) -> DdcError {
        let (attempts, max_attempts, last) = match failure {
            Failure::Expired => return DdcError::Timeout,
            Failure::Unsupported(refusal) => return refused(refusal.message),
            Failure::Panicked(panic) => {
                return DdcError::Transport(format!("ddc-hi panicked: {}", panic.message));
            }
            Failure::Exhausted {
                attempts,
                max_attempts,
                last,
            } => (attempts, max_attempts, last),
        };
        if self.enumeration_fits(deadline) {
            self.enumerate();
            if !self.knows(id) {
                return DdcError::MonitorNotFound(id.clone());
            }
        }
        DdcError::Transport(format!(
            "{} (gave up after attempt {attempts} of {max_attempts})",
            last.message
        ))
    }

    /// Whether an enumeration as long as the last one would end before
    /// `deadline`. Never true before the first enumeration.
    fn enumeration_fits(&self, deadline: Instant) -> bool {
        self.enumeration_cost
            .and_then(|cost| self.clock.now().checked_add(cost))
            .is_some_and(|end| end < deadline)
    }

    fn knows(&self, id: &MonitorId) -> bool {
        self.displays.iter().any(|(known, _)| known == id)
    }
}

/// Whether a reading of the input source shows the input that was written.
// WHY low byte only: a non-continuous value is the low byte of its reading,
// as the popup and the MCCS catalog read it; a monitor may fill the high byte.
fn same_input(read: u16, written: u16) -> bool {
    read & 0xFF == written & 0xFF
}

fn handle_of<'a, H>(displays: &'a mut [(MonitorId, H)], id: &MonitorId) -> Option<&'a mut H> {
    displays
        .iter_mut()
        .find(|(known, _)| known == id)
        .map(|(_, handle)| handle)
}

/// [`MonitorBackend`] that forwards every call to a [`Worker`] thread.
pub(crate) struct WorkerClient<S: DisplaySource> {
    jobs: Sender<Job<S>>,
    budgets: DdcHiBudgets,
    policies: RetryPolicies,
    _worker: JoinHandle<()>,
}

impl<S: DisplaySource> WorkerClient<S> {
    /// Starts the worker. Nothing is enumerated until the first request.
    pub(crate) fn spawn(
        source: S,
        budgets: DdcHiBudgets,
        policies: RetryPolicies,
    ) -> Result<Self, DdcError> {
        let (jobs, queue) = mpsc::channel::<Job<S>>();
        let worker = thread::Builder::new()
            .name("ddc-hi-worker".to_owned())
            .spawn(move || Worker::new(source, policies, SystemClock).run(queue))
            .map_err(|error| {
                DdcError::Transport(format!("cannot start the DDC worker: {error}"))
            })?;
        Ok(Self {
            jobs,
            budgets,
            policies,
            _worker: worker,
        })
    }

    pub(crate) fn with_budgets(self, budgets: DdcHiBudgets) -> Self {
        Self { budgets, ..self }
    }

    /// Queues `op` and waits at most `budget` for its answer; the worker
    /// gets the matching deadline.
    fn call<T: Send + 'static, E: From<DdcError> + Send + 'static>(
        &self,
        budget: Duration,
        op: impl FnOnce(&mut Worker<S, SystemClock>, Instant) -> Result<T, E> + Send + 'static,
    ) -> Result<T, E> {
        let deadline = Instant::now() + budget;
        let (reply, answer) = mpsc::channel();
        let job: Job<S> = Box::new(move |worker| {
            // A caller past its budget has dropped the receiver; the answer
            // is then discarded on purpose.
            let _ = reply.send(worker.serve(deadline, op));
        });
        self.jobs.send(job).map_err(|_| worker_gone())?;
        answer.recv_timeout(budget).map_err(|error| match error {
            RecvTimeoutError::Timeout => DdcError::Timeout,
            RecvTimeoutError::Disconnected => worker_gone(),
        })?
    }

    /// Runs `op` on monitor `id` within `budget`. An id the worker does not
    /// know costs one enumeration under the enumeration budget, then one
    /// more try under `budget`; still unknown, the monitor is missing.
    fn transact<T: Send + 'static>(
        &self,
        id: &MonitorId,
        budget: Duration,
        op: impl FnOnce(&mut Worker<S, SystemClock>, &MonitorId, Instant) -> Result<T, TransactError>
        + Copy
        + Send
        + 'static,
    ) -> Result<T, DdcError> {
        let ask = || {
            let id = id.clone();
            self.call(budget, move |worker, deadline| op(worker, &id, deadline))
        };
        match ask() {
            Err(TransactError::UnknownMonitor) => {
                self.enumerate()?;
            }
            answer => return answer.map_err(|error| error.for_caller(id)),
        }
        ask().map_err(|error| error.for_caller(id))
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
        self.call(self.budgets.enumerate, |worker, _| Ok(worker.enumerate()))
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        self.transact(id, self.budgets.capabilities, Worker::read_capabilities)
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.transact(id, self.budgets.vcp, move |worker, id, deadline| {
            worker.read_vcp(id, code, deadline)
        })
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        let budget = write_budget(&self.budgets, &self.policies, code);
        self.transact(id, budget, move |worker, id, deadline| {
            worker.write_vcp(id, code, value, deadline)
        })
    }
}

/// The worker thread ended: a panic outside any transaction, such as during
/// an enumeration, unwound it. Every later call fails the same way.
fn worker_gone() -> DdcError {
    DdcError::Transport("the DDC worker thread is not running".to_owned())
}

#[cfg(test)]
mod tests;
