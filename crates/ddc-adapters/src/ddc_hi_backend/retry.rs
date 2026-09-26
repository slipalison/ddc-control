//! Bounded retries of failed DDC/CI transactions, run inside the worker
//! (D-3, D-6). The core never retries.

use std::thread;
use std::time::{Duration, Instant};

use super::worker::HandleError;

/// Attempts per transaction, the first one included (D-3).
const MAX_ATTEMPTS: u32 = 3;
/// Pause between two attempts (D-3).
const BACKOFF: Duration = Duration::from_millis(50);

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
/// counts as transient. Retrying a write is safe: Set VCP Feature carries an
/// absolute value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetryPolicy {
    /// Attempts per transaction, the first one included.
    pub(crate) max_attempts: u32,
    /// Pause between two attempts.
    pub(crate) backoff: Duration,
}

impl Default for RetryPolicy {
    /// Three attempts, 50 ms apart (D-3).
    fn default() -> Self {
        Self {
            max_attempts: MAX_ATTEMPTS,
            backoff: BACKOFF,
        }
    }
}

impl RetryPolicy {
    /// Runs `op` until it succeeds, the attempts run out, or the next
    /// backoff would end at or past `deadline`. Nothing is attempted once
    /// `deadline` has passed.
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
            if attempts >= self.max_attempts || clock.now() + self.backoff >= deadline {
                return Err(Failure::Exhausted { attempts, last });
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
    /// Attempt number `attempts` failed with `last`, and no other was made.
    Exhausted {
        /// Attempts made.
        attempts: u32,
        /// Error of the last attempt.
        last: HandleError,
    },
}
