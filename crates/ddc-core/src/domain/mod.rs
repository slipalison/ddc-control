//! Monitor-control domain model: VCP codes and values, monitors, features and
//! their write risk, the MCCS catalog, and the domain error type.

mod capabilities;
mod error;
mod feature;
pub mod mccs_catalog;
mod monitor;
mod vcp;

pub use capabilities::Capabilities;
pub use error::DdcError;
pub use feature::{
    Access, Confirm, Feature, FeatureKind, FeatureReading, Risk, authorize_write, risk_for_code,
};
pub use monitor::{MonitorId, MonitorInfo};
pub use vcp::{VcpCode, VcpValue};
