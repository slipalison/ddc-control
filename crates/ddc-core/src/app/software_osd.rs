use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::mccs_catalog::catalog_codes;
use crate::domain::{
    Capabilities, Confirm, DdcError, Feature, FeatureReading, MonitorId, MonitorInfo,
    ProbedFeature, VcpCode, VcpValue, authorize_write,
};
use crate::ports::{MonitorBackend, MonitorControl};

/// Outcome of fetching a monitor's capabilities: parsed, or why they could
/// not be read or parsed.
type CapabilitiesOutcome = Result<Capabilities, DdcError>;

/// The software on-screen display: [`MonitorControl`] on top of any
/// [`MonitorBackend`].
///
/// Caches, per monitor, the outcome of fetching its capabilities and the last
/// maximum read for each feature, so writes can be validated without extra
/// round-trips. Capabilities that cannot be read or parsed never block a
/// read: the monitor is then treated as declaring no code, and the failure is
/// remembered so it is not fetched again on every read. A write to a
/// continuous feature whose maximum is not known yet reads the feature once
/// to learn it; no other write reads anything before it is sent.
#[derive(Debug)]
pub struct SoftwareOsd<B> {
    backend: B,
    capabilities: Mutex<HashMap<MonitorId, CapabilitiesOutcome>>,
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

    /// Reads and parses the capabilities, remembering the outcome. A missing
    /// monitor is not remembered: it says nothing about its capabilities.
    fn fetch_capabilities(&self, id: &MonitorId) -> CapabilitiesOutcome {
        let outcome = self
            .backend
            .read_capabilities(id)
            .and_then(|raw| Capabilities::parse(&raw));
        if !matches!(outcome, Err(DdcError::MonitorNotFound(_))) {
            lock(&self.capabilities).insert(id.clone(), outcome.clone());
        }
        outcome
    }

    /// The capabilities a read or write is interpreted with: the remembered
    /// outcome, else a fresh fetch. Capabilities that cannot be read or parsed
    /// count as declaring nothing; only a missing monitor is an error.
    fn capabilities_or_empty(&self, id: &MonitorId) -> Result<Capabilities, DdcError> {
        let remembered = lock(&self.capabilities).get(id).cloned();
        match remembered.unwrap_or_else(|| self.fetch_capabilities(id)) {
            Err(DdcError::MonitorNotFound(missing)) => Err(DdcError::MonitorNotFound(missing)),
            Err(_) => Ok(Capabilities::default()),
            parsed => parsed,
        }
    }

    /// Reads `code` from the monitor, remembering the maximum it reports so
    /// a later write needs no extra read.
    fn read_and_track(
        &self,
        id: &MonitorId,
        capabilities: &Capabilities,
        code: VcpCode,
    ) -> Result<FeatureReading, DdcError> {
        let value = self.backend.read_vcp(id, code)?;
        self.remember_max(id, code, value.max);
        Ok(FeatureReading {
            feature: capabilities.feature(code),
            value,
            declared_in_capabilities: capabilities.declares(code),
        })
    }

    fn remember_max(&self, id: &MonitorId, code: VcpCode, max: u16) {
        lock(&self.known_max).insert((id.clone(), code), max);
    }

    fn cached_max(&self, id: &MonitorId, code: VcpCode) -> Option<u16> {
        lock(&self.known_max).get(&(id.clone(), code)).copied()
    }

    /// The maximum a write to `feature` is validated against: the last one
    /// read, else one read now, whether or not the capabilities declare the
    /// code (D-2026-09-26-cli-1). Only a continuous feature without a value
    /// list needs one; any other feature, and a write-only one, gets `None`
    /// without touching the monitor (D-2026-09-26-full-osd-control-1, -8). A
    /// failed read is the write's error.
    fn max_for_write(&self, id: &MonitorId, feature: &Feature) -> Result<Option<u16>, DdcError> {
        if !feature.requires_known_max() || !feature.is_readable() {
            return Ok(None);
        }
        if let Some(max) = self.cached_max(id, feature.code) {
            return Ok(Some(max));
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

    /// Parsed capabilities are served from the cache. A remembered failure is
    /// not: an explicit request always asks the monitor again, surfaces the
    /// error if it persists, and replaces it in the cache once it succeeds —
    /// this is how a caller recovers from a transient capabilities failure.
    fn capabilities(&self, id: &MonitorId) -> Result<Capabilities, DdcError> {
        if let Some(Ok(parsed)) = lock(&self.capabilities).get(id) {
            return Ok(parsed.clone());
        }
        self.fetch_capabilities(id)
    }

    fn get_feature(&self, id: &MonitorId, code: VcpCode) -> Result<FeatureReading, DdcError> {
        let capabilities = self.capabilities_or_empty(id)?;
        self.read_and_track(id, &capabilities, code)
    }

    fn probe_undeclared_features(&self, id: &MonitorId) -> Result<Vec<ProbedFeature>, DdcError> {
        let capabilities = self.capabilities_or_empty(id)?;
        catalog_codes()
            .iter()
            .copied()
            .filter(|code| !capabilities.declares(*code))
            .map(|code| match self.read_and_track(id, &capabilities, code) {
                Err(DdcError::MonitorNotFound(missing)) => Err(DdcError::MonitorNotFound(missing)),
                outcome => Ok(ProbedFeature { code, outcome }),
            })
            .collect()
    }

    fn set_feature(
        &self,
        id: &MonitorId,
        code: VcpCode,
        value: u16,
        confirm: Confirm,
    ) -> Result<VcpValue, DdcError> {
        authorize_write(code, confirm)?;
        let capabilities = self.capabilities_or_empty(id)?;
        let feature = capabilities.feature(code);
        feature.ensure_writable()?;
        let known_max = self.max_for_write(id, &feature)?;
        feature.validate_write(value, known_max)?;
        self.backend.write_vcp(id, code, value)?;
        if !feature.is_readable() {
            return Ok(VcpValue {
                current: value,
                max: value,
            });
        }
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
