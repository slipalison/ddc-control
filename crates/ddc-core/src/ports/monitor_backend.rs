use crate::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};

/// Driven port: raw DDC/CI access to the monitors attached to this machine.
///
/// Implementations own the transport: one in-flight transaction per monitor,
/// bounded retries of transient failures within their own timeout, and
/// mapping of transport failures to [`DdcError::Transport`] or
/// [`DdcError::Timeout`]. They apply no domain rule — they never validate,
/// classify or refuse a value.
pub trait MonitorBackend {
    /// Lists the monitors currently reachable.
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError>;

    /// Reads the monitor's raw MCCS capabilities string.
    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError>;

    /// Reads the current and maximum value of a VCP feature.
    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError>;

    /// Writes `value` to a VCP feature as is.
    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError>;
}
