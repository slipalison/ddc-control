//! Driven adapters for `ddc-control`.
//!
//! Each adapter implements the core's `MonitorBackend` port for one technology
//! and translates that technology's results and errors into core types.
//! `CachingMonitorBackend` decorates any of them with an on-disk cache of
//! capabilities strings.

mod caching;
#[cfg(feature = "ddc-hi")]
mod ddc_hi_backend;
mod in_memory;

pub use caching::{CachingMonitorBackend, default_cache_dir};
#[cfg(feature = "ddc-hi")]
pub use ddc_hi_backend::{DdcHiBudgets, DdcHiMonitorBackend};
pub use in_memory::{
    BackendCall, FakeMonitor, InMemoryMonitorBackend, InMemoryMonitorBackendBuilder,
};
