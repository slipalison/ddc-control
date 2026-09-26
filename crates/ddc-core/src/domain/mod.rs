//! Monitor-control domain model: VCP codes and values, monitors, features and
//! their write risk, and the domain error type.

mod capabilities;
mod error;
mod feature;
mod monitor;
mod vcp;

pub use capabilities::Capabilities;
pub use error::DdcError;
pub use feature::{Access, Confirm, Feature, FeatureKind, FeatureReading, Risk, risk_for_code};
pub use monitor::{MonitorId, MonitorInfo};
pub use vcp::{VcpCode, VcpValue};
