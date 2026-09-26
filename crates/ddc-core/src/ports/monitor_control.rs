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
    /// before the monitor is touched at all. A read-only or `Table` feature
    /// is then refused as unsupported, still before anything is read. The
    /// value is validated against the feature's value list — the one its
    /// capabilities declare, else the catalog's value names — or, for a
    /// continuous feature without one, against its maximum; a maximum not
    /// known yet is read from the monitor first, and if that read fails the
    /// write is refused with its error. A non-continuous feature with no
    /// list anywhere is refused as unsupported, never written blind.
    ///
    /// Compare the returned value with `value` to catch monitors that
    /// acknowledge a write without applying it. A write-only feature (the
    /// factory resets) is never read: nothing is read back, and the value
    /// returned is the one written, as both current and maximum.
    fn set_feature(
        &self,
        id: &MonitorId,
        code: VcpCode,
        value: u16,
        confirm: Confirm,
    ) -> Result<VcpValue, DdcError>;
}
