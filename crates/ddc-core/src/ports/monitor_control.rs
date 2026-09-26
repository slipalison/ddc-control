use crate::domain::{
    Capabilities, Confirm, DdcError, FeatureReading, MonitorId, MonitorInfo, VcpCode, VcpValue,
};

/// Driving port: what a user-facing adapter (CLI, tray) can ask of the core.
pub trait MonitorControl {
    /// Lists the monitors currently reachable.
    fn list_monitors(&self) -> Result<Vec<MonitorInfo>, DdcError>;

    /// The monitor's parsed capabilities, or the error that kept them from
    /// being read or parsed.
    fn capabilities(&self, id: &MonitorId) -> Result<Capabilities, DdcError>;

    /// Reads a feature from the monitor. The read always reaches the monitor,
    /// even for a code its capabilities do not declare or when they cannot be
    /// read or parsed at all.
    fn get_feature(&self, id: &MonitorId, code: VcpCode) -> Result<FeatureReading, DdcError>;

    /// Writes a feature and returns the value read back right after.
    ///
    /// A dangerous feature needs explicit user [`Confirm`]ation, checked
    /// before the monitor is touched at all. The value is then validated against the feature's
    /// allowed values or its known maximum; with neither known the write is
    /// refused. Compare the returned value with `value` to catch monitors that
    /// acknowledge a write without applying it.
    fn set_feature(
        &self,
        id: &MonitorId,
        code: VcpCode,
        value: u16,
        confirm: Confirm,
    ) -> Result<VcpValue, DdcError>;
}
