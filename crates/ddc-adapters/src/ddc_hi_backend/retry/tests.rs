//! Test support for the retry policies: kept out of `retry.rs` so the
//! production file holds no test code (D-2026-10-01-input-switch-autostart-2).

use std::time::Duration;

use super::{InputSettle, RetryPolicies, RetryPolicy};

impl RetryPolicies {
    /// Test support: the default attempts with no pause between them, and the
    /// input settling shrunk to 5 ms steps inside a 100 ms window, so tests on
    /// the system clock never sleep long.
    pub(crate) fn without_backoff() -> Self {
        let instant = |policy: RetryPolicy| RetryPolicy {
            backoff: Duration::ZERO,
            ..policy
        };
        let defaults = Self::default();
        Self {
            vcp: instant(defaults.vcp),
            capabilities: instant(defaults.capabilities),
            input_settle: InputSettle {
                step: Duration::from_millis(5),
                window: Duration::from_millis(100),
            },
        }
    }
}
