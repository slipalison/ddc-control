use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::{Capabilities, DdcError, FeatureReading, MonitorId, MonitorInfo, VcpCode};
use crate::ports::{MonitorBackend, MonitorControl};

/// The software on-screen display: [`MonitorControl`] on top of any
/// [`MonitorBackend`].
///
/// Caches each monitor's parsed capabilities and the last maximum read for
/// each feature, so writes can be validated without extra round-trips.
#[derive(Debug)]
pub struct SoftwareOsd<B> {
    backend: B,
    capabilities: Mutex<HashMap<MonitorId, Capabilities>>,
    known_max: Mutex<HashMap<(MonitorId, VcpCode), u16>>,
}

impl<B: MonitorBackend> SoftwareOsd<B> {
    /// Wraps a backend. Nothing is read from it until the first call.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            capabilities: Mutex::new(HashMap::new()),
            known_max: Mutex::new(HashMap::new()),
        }
    }

    fn load_capabilities(&self, id: &MonitorId) -> Result<Capabilities, DdcError> {
        if let Some(cached) = lock(&self.capabilities).get(id) {
            return Ok(cached.clone());
        }
        let parsed = Capabilities::parse(&self.backend.read_capabilities(id)?)?;
        lock(&self.capabilities).insert(id.clone(), parsed.clone());
        Ok(parsed)
    }

    fn remember_max(&self, id: &MonitorId, code: VcpCode, max: u16) {
        lock(&self.known_max).insert((id.clone(), code), max);
    }
}

impl<B: MonitorBackend> MonitorControl for SoftwareOsd<B> {
    fn list_monitors(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        self.backend.enumerate()
    }

    fn capabilities(&self, id: &MonitorId) -> Result<Capabilities, DdcError> {
        self.load_capabilities(id)
    }

    fn get_feature(&self, id: &MonitorId, code: VcpCode) -> Result<FeatureReading, DdcError> {
        let capabilities = self.load_capabilities(id)?;
        let value = self.backend.read_vcp(id, code)?;
        self.remember_max(id, code, value.max);
        Ok(FeatureReading {
            feature: capabilities.feature(code),
            value,
            declared_in_capabilities: capabilities.declares(code),
        })
    }
}

/// The caches hold plain data that is valid after any panic, so a poisoned
/// lock is recovered rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
