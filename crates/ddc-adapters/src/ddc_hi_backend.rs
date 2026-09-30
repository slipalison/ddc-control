//! [`MonitorBackend`] over real DDC/CI through `ddc-hi`: dxva2 on Windows,
//! `/dev/i2c-*` on Linux — `ddc-hi` picks the platform backend itself.
//!
//! One worker thread owns every display handle (D-6) and retries a failed
//! transaction up to 3 times within the caller's budget: 200 ms apart for
//! VCP reads and writes (D-2026-09-26-full-osd-control-9), 500 ms apart for
//! capabilities reads (D-2026-09-26-cli-2). A VCP code the monitor answers as unsupported is
//! never retried (D-2026-09-26-cli-4). A panic inside `ddc-hi` fails only the
//! transaction it happened in, without a retry, and the worker keeps serving
//! (D-2026-09-26-full-osd-control-6). Callers wait on a per-operation budget
//! (D-7) and get [`DdcError::Timeout`] past it. Monitors are identified by
//! EDID when available (D-2).
//!
//! A write of the input source (0x60) returns only after the monitor reads
//! the input back, or 3 s later when it never does, and waits that long on
//! top of the VCP budget (D-2026-09-30-input-switch-autostart-3).

mod hardware;
mod identity;
mod retry;
mod worker;

use std::time::Duration;

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use hardware::DdcHiDisplays;
use retry::RetryPolicies;
use worker::WorkerClient;

/// Default wait for a Get or Set VCP Feature (D-7).
const VCP_BUDGET: Duration = Duration::from_secs(1);
/// Default wait for a capabilities read; the dev monitor needs ~2.6 s (D-7).
const CAPABILITIES_BUDGET: Duration = Duration::from_secs(8);
/// Default wait for an enumeration; the dev machine needs ~1.1 s (D-7).
const ENUMERATE_BUDGET: Duration = Duration::from_secs(5);

/// How long a caller of [`DdcHiMonitorBackend`] waits for each kind of
/// operation before getting [`DdcError::Timeout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DdcHiBudgets {
    /// `read_vcp` and `write_vcp`. A write of the input source waits 3 s
    /// more, for the monitor to show the new input.
    pub vcp: Duration,
    /// `read_capabilities`.
    pub capabilities: Duration,
    /// `enumerate`.
    pub enumerate: Duration,
}

impl Default for DdcHiBudgets {
    /// 1 s per VCP read or write, 8 s per capabilities read, 5 s per
    /// enumeration (D-7).
    fn default() -> Self {
        Self {
            vcp: VCP_BUDGET,
            capabilities: CAPABILITIES_BUDGET,
            enumerate: ENUMERATE_BUDGET,
        }
    }
}

/// The real [`MonitorBackend`]: DDC/CI through `ddc-hi`, on a single worker
/// thread that owns every monitor handle.
///
/// No call needs an [`enumerate`](MonitorBackend::enumerate) first: an id
/// the worker does not know yet is looked up with one enumeration under the
/// enumeration budget, then the call runs under its own budget
/// (D-2026-09-26-ddc-backends-1). A monitor still unknown after that answers
/// [`DdcError::MonitorNotFound`]. A VCP code the monitor answers as
/// unsupported answers [`DdcError::UnsupportedFeature`] after one attempt
/// (D-2026-09-26-cli-4); on Windows, where `dxva2` keeps that reply to
/// itself, it counts as any other failure. A panic inside `ddc-hi` answers
/// [`DdcError::Transport`] starting with `ddc-hi panicked:` after that one
/// attempt, and the next call is served as usual
/// (D-2026-09-26-full-osd-control-6). Failures that outlast the retries
/// answer [`DdcError::Transport`], or `MonitorNotFound` when a fresh
/// enumeration fits the rest of the budget and no longer lists the monitor.
/// A write of the input source polls the input until the monitor reads the
/// new one back (250 ms apart, at most 3 s) and answers `Ok` either way: what
/// the monitor kept is for the caller's read to tell. A caller whose budget
/// runs out answers [`DdcError::Timeout`]. Dropping
/// the backend stops the worker once its current transaction ends; it never
/// waits for it.
#[derive(Debug)]
pub struct DdcHiMonitorBackend {
    client: WorkerClient<DdcHiDisplays>,
}

impl DdcHiMonitorBackend {
    /// Starts the worker thread with the default [`DdcHiBudgets`]. No
    /// monitor is touched until the first call.
    pub fn new() -> Result<Self, DdcError> {
        WorkerClient::spawn(
            DdcHiDisplays,
            DdcHiBudgets::default(),
            RetryPolicies::default(),
        )
        .map(|client| Self { client })
    }

    /// Replaces the per-operation budgets.
    pub fn with_budgets(self, budgets: DdcHiBudgets) -> Self {
        Self {
            client: self.client.with_budgets(budgets),
        }
    }
}

impl MonitorBackend for DdcHiMonitorBackend {
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        self.client.enumerate()
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        self.client.read_capabilities(id)
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.client.read_vcp(id, code)
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        self.client.write_vcp(id, code, value)
    }
}

// Compile-time proof, on every target, that the backend can be shared with
// other threads (the Windows handles are `!Send`; the worker keeps them).
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DdcHiMonitorBackend>();
};

#[cfg(test)]
mod tests;
