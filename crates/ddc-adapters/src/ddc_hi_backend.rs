//! [`MonitorBackend`] over real DDC/CI through `ddc-hi`: dxva2 on Windows,
//! `/dev/i2c-*` on Linux — `ddc-hi` picks the platform backend itself.
//!
//! One worker thread owns every display handle (D-6) and retries a failed
//! transaction up to 3 times, 50 ms apart, within the caller's budget
//! (D-3); callers wait on a per-operation budget (D-7) and get
//! [`DdcError::Timeout`] past it. Monitors are identified by EDID when
//! available (D-2).

mod hardware;
mod identity;
mod retry;
mod worker;

use std::time::Duration;

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use hardware::DdcHiDisplays;
use retry::RetryPolicy;
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
    /// `read_vcp` and `write_vcp`.
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
/// A monitor that stays unknown after a fresh enumeration, or that is gone
/// after a failed transaction, answers [`DdcError::MonitorNotFound`];
/// failures that outlast the retries answer [`DdcError::Transport`], and a
/// caller whose budget runs out answers [`DdcError::Timeout`]. Dropping the backend stops the worker once its
/// current transaction ends; it never waits for it.
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
            RetryPolicy::default(),
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
