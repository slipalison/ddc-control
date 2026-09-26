//! A scripted, in-memory `MonitorBackend` — the test adapter of the hexagon.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

/// A call received by [`InMemoryMonitorBackend`], recorded in arrival order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendCall {
    /// `enumerate()`.
    Enumerate,
    /// `read_capabilities(id)`.
    ReadCapabilities(MonitorId),
    /// `read_vcp(id, code)`.
    ReadVcp(MonitorId, VcpCode),
    /// `write_vcp(id, code, value)`.
    WriteVcp(MonitorId, VcpCode, u16),
}

/// Scripted state of one fake monitor.
#[derive(Debug, Clone)]
pub struct FakeMonitor {
    info: MonitorInfo,
    capabilities: Option<String>,
    transient_capabilities_failures: u32,
    values: BTreeMap<VcpCode, VcpValue>,
    ignored_writes: BTreeSet<VcpCode>,
}

impl FakeMonitor {
    /// A monitor with no capabilities string and no VCP values.
    pub fn new(info: MonitorInfo) -> Self {
        Self {
            info,
            capabilities: None,
            transient_capabilities_failures: 0,
            values: BTreeMap::new(),
            ignored_writes: BTreeSet::new(),
        }
    }

    /// Raw capabilities string returned by `read_capabilities`.
    pub fn with_capabilities(mut self, raw: impl Into<String>) -> Self {
        self.capabilities = Some(raw.into());
        self
    }

    /// Makes the first `count` capabilities reads fail with
    /// [`DdcError::Transport`] before the scripted string is served, like a
    /// scaler that drops the first requests.
    pub fn with_transient_capabilities_failures(mut self, count: u32) -> Self {
        self.transient_capabilities_failures = count;
        self
    }

    /// Makes `code` readable and writable with the given value.
    pub fn with_value(mut self, code: VcpCode, current: u16, max: u16) -> Self {
        self.values.insert(code, VcpValue { current, max });
        self
    }

    /// Mimics scalers that acknowledge writes to `code` but keep the old value.
    pub fn ignoring_writes_to(mut self, code: VcpCode) -> Self {
        self.ignored_writes.insert(code);
        self
    }

    fn capabilities(&mut self) -> Result<String, DdcError> {
        if self.transient_capabilities_failures > 0 {
            self.transient_capabilities_failures -= 1;
            return Err(DdcError::Transport(
                "capabilities request dropped".to_owned(),
            ));
        }
        self.capabilities
            .clone()
            .ok_or_else(|| DdcError::Transport("capabilities string unavailable".to_owned()))
    }

    fn value(&self, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.values
            .get(&code)
            .copied()
            .ok_or(DdcError::UnsupportedFeature(code))
    }

    fn store(&mut self, code: VcpCode, value: u16) -> Result<(), DdcError> {
        let stored = self
            .values
            .get_mut(&code)
            .ok_or(DdcError::UnsupportedFeature(code))?;
        if !self.ignored_writes.contains(&code) {
            stored.current = value;
        }
        Ok(())
    }
}

/// In-memory [`MonitorBackend`] serving [`FakeMonitor`]s and logging every
/// call. Unknown monitors answer [`DdcError::MonitorNotFound`]; unscripted
/// codes answer [`DdcError::UnsupportedFeature`].
///
/// Clones are handles to the same monitors and call log, so a test can hand
/// one clone to the core and inspect the log through another.
#[derive(Debug, Clone)]
pub struct InMemoryMonitorBackend {
    monitors: Arc<Mutex<Vec<FakeMonitor>>>,
    calls: Arc<Mutex<Vec<BackendCall>>>,
}

impl InMemoryMonitorBackend {
    /// Starts scripting a backend.
    pub fn builder() -> InMemoryMonitorBackendBuilder {
        InMemoryMonitorBackendBuilder::default()
    }

    /// Every call received so far, oldest first.
    pub fn calls(&self) -> Vec<BackendCall> {
        lock(&self.calls).clone()
    }

    fn record(&self, call: BackendCall) {
        lock(&self.calls).push(call);
    }

    fn with_monitor<T>(
        &self,
        id: &MonitorId,
        action: impl FnOnce(&mut FakeMonitor) -> Result<T, DdcError>,
    ) -> Result<T, DdcError> {
        let mut monitors = lock(&self.monitors);
        let monitor = monitors
            .iter_mut()
            .find(|monitor| &monitor.info.id == id)
            .ok_or_else(|| DdcError::MonitorNotFound(id.clone()))?;
        action(monitor)
    }
}

impl MonitorBackend for InMemoryMonitorBackend {
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        self.record(BackendCall::Enumerate);
        Ok(lock(&self.monitors)
            .iter()
            .map(|monitor| monitor.info.clone())
            .collect())
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        self.record(BackendCall::ReadCapabilities(id.clone()));
        self.with_monitor(id, |monitor| monitor.capabilities())
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.record(BackendCall::ReadVcp(id.clone(), code));
        self.with_monitor(id, |monitor| monitor.value(code))
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        self.record(BackendCall::WriteVcp(id.clone(), code, value));
        self.with_monitor(id, |monitor| monitor.store(code, value))
    }
}

/// Builder for [`InMemoryMonitorBackend`].
#[derive(Debug, Default)]
pub struct InMemoryMonitorBackendBuilder {
    monitors: Vec<FakeMonitor>,
}

impl InMemoryMonitorBackendBuilder {
    /// Adds a monitor; enumeration keeps insertion order.
    pub fn monitor(mut self, monitor: FakeMonitor) -> Self {
        self.monitors.push(monitor);
        self
    }

    /// Finishes the backend with an empty call log.
    pub fn build(self) -> InMemoryMonitorBackend {
        InMemoryMonitorBackend {
            monitors: Arc::new(Mutex::new(self.monitors)),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// A poisoned lock only means another test thread panicked mid-call; the
/// scripted state is still usable, so recover it instead of propagating.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
