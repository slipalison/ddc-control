//! Bounded retries of failed DDC/CI transactions, run inside the worker
//! (D-3, D-6). The core never retries.

use std::thread;
use std::time::{Duration, Instant};

use super::worker::HandleError;

/// Attempts per transaction, the first one included (D-3).
const MAX_ATTEMPTS: u32 = 3;
/// Pause between two attempts of a Get or Set VCP Feature. Read back to
/// back, the dev monitor now and then fails a VCP read three times in a
/// row 50 ms apart, and 200 ms apart it recovers
/// (D-2026-09-26-full-osd-control-9, amending D-3). Three attempts still fit
/// the 1 s VCP budget.
const VCP_BACKOFF: Duration = Duration::from_millis(200);
/// Pause between two attempts of a capabilities read. The dev monitor keeps
/// refusing capabilities reads for a few hundred milliseconds after one
/// fails, so attempts 50 ms apart all failed (D-2026-09-26-cli-2).
const CAPABILITIES_BACKOFF: Duration = Duration::from_millis(500);
/// Pause between two reads of the input source while the monitor settles.
/// Private with [`INPUT_SETTLE_WINDOW`]: only `RetryPolicies::default` names
/// them, so the worker and the client can only read the policy they were
/// given (D-2026-09-30-input-switch-autostart-17).
// WHY 250 ms: a read is one DDC/CI transaction of 50-100 ms; polling slower
// than the monitor switches would keep the popup waiting for nothing.
const INPUT_SETTLE_STEP: Duration = Duration::from_millis(250);
/// How long a write of the input source waits for the monitor to show it.
// WHY 3 s: a monitor that switches to an input with no signal scans it and
// goes back on its own within a couple of seconds; the wait has to outlast
// that to report what the monitor kept (D-2026-09-30-input-switch-autostart-3).
const INPUT_SETTLE_WINDOW: Duration = Duration::from_secs(3);

/// Time as the worker sees it; virtual in tests, so retries are checked
/// without sleeping.
pub(crate) trait Clock {
    /// The current instant.
    fn now(&self) -> Instant;

    /// Blocks the worker for `duration`.
    fn sleep(&self, duration: Duration);
}

/// Wall-clock time.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

/// How many times, and how patiently, a failed transaction is retried.
///
/// `ddc-hi` does not tell a NAK from any other failure, so every failure
/// counts as transient, except the monitor's answer that it does not support
/// the VCP code (D-2026-09-26-cli-4) and a panic inside the transport
/// (D-2026-09-26-full-osd-control-6), which are never retried. Retrying a
/// write is safe: Set VCP Feature carries an absolute value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetryPolicy {
    /// Attempts per transaction, the first one included.
    pub(crate) max_attempts: u32,
    /// Pause between two attempts.
    pub(crate) backoff: Duration,
}

/// How a write of the input source waits for the monitor to show the new
/// input. The client budgets the write by the `window` and the worker polls
/// by both, so the two cannot disagree (D-2026-09-30-input-switch-autostart-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InputSettle {
    /// Pause between two reads of the input source.
    pub(crate) step: Duration,
    /// How long after the write the reads go on.
    pub(crate) window: Duration,
}

/// The retry policy of each kind of transaction, and the settling of the
/// input source that follows its write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetryPolicies {
    /// Get and Set VCP Feature.
    pub(crate) vcp: RetryPolicy,
    /// Capabilities request.
    pub(crate) capabilities: RetryPolicy,
    /// Reads of the input source after a write of it.
    pub(crate) input_settle: InputSettle,
}

impl Default for RetryPolicies {
    /// Three attempts each: VCP ones 200 ms apart
    /// (D-2026-09-26-full-osd-control-9), capabilities ones 500 ms apart
    /// (D-2026-09-26-cli-2). The input source is read every 250 ms for up to
    /// 3 s after it is written (D-2026-09-30-input-switch-autostart-3).
    fn default() -> Self {
        Self {
            vcp: RetryPolicy {
                max_attempts: MAX_ATTEMPTS,
                backoff: VCP_BACKOFF,
            },
            capabilities: RetryPolicy {
                max_attempts: MAX_ATTEMPTS,
                backoff: CAPABILITIES_BACKOFF,
            },
            input_settle: InputSettle {
                step: INPUT_SETTLE_STEP,
                window: INPUT_SETTLE_WINDOW,
            },
        }
    }
}

impl RetryPolicy {
    /// Runs `op` until it succeeds, the monitor refuses it as unsupported,
    /// the transport panics, the attempts run out, or the next backoff would end at or past
    /// `deadline`. Nothing is attempted once `deadline` has passed.
    pub(crate) fn run<T>(
        &self,
        clock: &impl Clock,
        deadline: Instant,
        mut op: impl FnMut() -> Result<T, HandleError>,
    ) -> Result<T, Failure> {
        if clock.now() >= deadline {
            return Err(Failure::Expired);
        }
        let mut attempts = 1;
        loop {
            let last = match op() {
                Ok(value) => return Ok(value),
                Err(error) => error,
            };
            if last.is_unsupported() {
                return Err(Failure::Unsupported(last));
            }
            if last.is_panic() {
                return Err(Failure::Panicked(last));
            }
            if attempts >= self.max_attempts || clock.now() + self.backoff >= deadline {
                return Err(Failure::Exhausted {
                    attempts,
                    max_attempts: self.max_attempts,
                    last,
                });
            }
            clock.sleep(self.backoff);
            attempts += 1;
        }
    }
}

/// Why [`RetryPolicy::run`] gave up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Failure {
    /// The deadline had passed before the first attempt; nothing was sent.
    Expired,
    /// The monitor refused the request as unsupported. Asking again gets
    /// the same answer, so no other attempt was made.
    Unsupported(HandleError),
    /// The transport panicked. The same request would panic again, so no
    /// other attempt was made.
    Panicked(HandleError),
    /// Attempt number `attempts` failed with `last`, and no other was made.
    Exhausted {
        /// Attempts made.
        attempts: u32,
        /// Attempts the policy allowed.
        max_attempts: u32,
        /// Error of the last attempt.
        last: HandleError,
    },
}

#[cfg(test)]
mod tests;
