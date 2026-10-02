//! Ports of the hexagon: the contracts adapters implement (driven) or call
//! (driving). Traits only — no logic lives here.

mod monitor_backend;
mod monitor_control;
mod usb_presence;

pub use monitor_backend::MonitorBackend;
pub use monitor_control::MonitorControl;
pub use usb_presence::UsbPresence;
