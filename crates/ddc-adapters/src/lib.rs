//! Driven adapters for `ddc-control`.
//!
//! Each adapter implements the core's `MonitorBackend` port for one technology
//! and translates that technology's results and errors into core types.

mod in_memory;

pub use in_memory::{
    BackendCall, FakeMonitor, InMemoryMonitorBackend, InMemoryMonitorBackendBuilder,
};
