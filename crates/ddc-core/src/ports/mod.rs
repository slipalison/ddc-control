//! Ports of the hexagon: the contracts adapters implement (driven) or call
//! (driving). Traits only — no logic lives here.

mod monitor_backend;

pub use monitor_backend::MonitorBackend;
