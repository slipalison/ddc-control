use crate::domain::{Capabilities, DdcError, FeatureReading, MonitorId, MonitorInfo, VcpCode};

/// Driving port: what a user-facing adapter (CLI, tray) can ask of the core.
pub trait MonitorControl {
    /// Lists the monitors currently reachable.
    fn list_monitors(&self) -> Result<Vec<MonitorInfo>, DdcError>;

    /// The monitor's parsed capabilities.
    fn capabilities(&self, id: &MonitorId) -> Result<Capabilities, DdcError>;

    /// Reads a feature from the monitor. The read always reaches the monitor,
    /// even for a code its capabilities do not declare.
    fn get_feature(&self, id: &MonitorId, code: VcpCode) -> Result<FeatureReading, DdcError>;
}
