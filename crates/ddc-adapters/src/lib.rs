//! Driven adapters for `ddc-control`.
//!
//! Each adapter implements the core's `MonitorBackend` port for one technology
//! and translates that technology's results and errors into core types.

#[cfg(feature = "ddc-hi")]
mod ddc_hi_backend;
mod in_memory;

#[cfg(feature = "ddc-hi")]
pub use ddc_hi_backend::{DdcHiBudgets, DdcHiMonitorBackend};
pub use in_memory::{
    BackendCall, FakeMonitor, InMemoryMonitorBackend, InMemoryMonitorBackendBuilder,
};
