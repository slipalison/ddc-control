use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::{
    Capabilities, Confirm, DdcError, Feature, FeatureReading, MonitorId, MonitorInfo, VcpCode,
    VcpValue, authorize_write,
};
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

    fn cached_max(&self, id: &MonitorId, code: VcpCode) -> Option<u16> {
        lock(&self.known_max).get(&(id.clone(), code)).copied()
    }

    /// The maximum a write to `feature` is validated against: the last one
    /// read, else one read now if the capabilities declare the code. `None`
    /// when the feature needs no maximum or none can be trusted.
    fn max_for_write(
        &self,
        id: &MonitorId,
        capabilities: &Capabilities,
        feature: &Feature,
    ) -> Result<Option<u16>, DdcError> {
        if !feature.requires_known_max() {
            return Ok(None);
        }
        if let Some(max) = self.cached_max(id, feature.code) {
            return Ok(Some(max));
        }
        if !capabilities.declares(feature.code) {
            return Ok(None);
        }
        let current = self.backend.read_vcp(id, feature.code)?;
        self.remember_max(id, feature.code, current.max);
        Ok(Some(current.max))
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

    fn set_feature(
        &self,
        id: &MonitorId,
        code: VcpCode,
        value: u16,
        confirm: Confirm,
    ) -> Result<VcpValue, DdcError> {
        authorize_write(code, confirm)?;
        let capabilities = self.load_capabilities(id)?;
        let feature = capabilities.feature(code);
        let known_max = self.max_for_write(id, &capabilities, &feature)?;
        feature.validate_write(value, known_max)?;
        self.backend.write_vcp(id, code, value)?;
        let read_back = self.backend.read_vcp(id, code)?;
        self.remember_max(id, code, read_back.max);
        Ok(read_back)
    }
}

/// The caches hold plain data that is valid after any panic, so a poisoned
/// lock is recovered rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
