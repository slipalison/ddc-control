//! Driven adapters for `ddc-control`.
//!
//! Each adapter implements the core's `MonitorBackend` port for one technology
//! and translates that technology's results and errors into core types.
//! `CachingMonitorBackend` decorates any of them with an on-disk cache of
//! capabilities strings. `SysfsUsbPresence` implements the `UsbPresence`
//! port over a sysfs tree of USB devices.

mod caching;
#[cfg(feature = "ddc-hi")]
mod ddc_hi_backend;
mod in_memory;
mod usb_sysfs;

pub use caching::{CachingMonitorBackend, default_cache_dir};
#[cfg(feature = "ddc-hi")]
pub use ddc_hi_backend::{DdcHiBudgets, DdcHiMonitorBackend};
pub use in_memory::{
    BackendCall, FakeMonitor, InMemoryMonitorBackend, InMemoryMonitorBackendBuilder,
};
pub use usb_sysfs::SysfsUsbPresence;
