use std::fmt;

/// Stable identity of a monitor, derived from its EDID (manufacturer, model,
/// serial) by the backend — never a bare enumeration index when EDID exists.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorId(String);

impl MonitorId {
    /// Wraps an already-computed stable key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A monitor reachable over DDC/CI, as enumerated by a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorInfo {
    /// Stable identity used by every other operation.
    pub id: MonitorId,
    /// EDID manufacturer code, when known.
    pub manufacturer: Option<String>,
    /// EDID model name, when known.
    pub model: Option<String>,
    /// EDID serial number, when known.
    pub serial: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::MonitorId;

    #[test]
    fn monitor_id_exposes_and_displays_its_key() {
        let id = MonitorId::new("RTK-QHD-HDR-0001");
        assert_eq!(id.as_str(), "RTK-QHD-HDR-0001");
        assert_eq!(id.to_string(), "RTK-QHD-HDR-0001");
    }
}
