//! Ports of the hexagon: the contracts adapters implement (driven) or call
//! (driving). Traits only — no logic lives here.

mod monitor_backend;
mod monitor_control;

pub use monitor_backend::MonitorBackend;
pub use monitor_control::MonitorControl;
