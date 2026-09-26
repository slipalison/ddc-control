//! Stable [`MonitorId`]s derived from what the platform reports about each
//! display: the EDID key when EDID was read, else the backend's description,
//! else the enumeration index.

use std::collections::HashMap;

use ddc_core::domain::{MonitorId, MonitorInfo};

/// What the platform reported about one display, before any id is derived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DisplayIdentity {
    /// Backend-specific description; on WinAPI the only identity available.
    pub(crate) description: String,
    /// EDID identity, present only when the backend read the EDID.
    pub(crate) edid: Option<EdidIdentity>,
}

/// The EDID fields that identify a display.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct EdidIdentity {
    /// Three-letter manufacturer code.
    pub(crate) manufacturer: String,
    /// Product name descriptor.
    pub(crate) model_name: Option<String>,
    /// Numeric product code.
    pub(crate) model_id: Option<u16>,
    /// Serial number descriptor.
    pub(crate) serial_number: Option<String>,
    /// Numeric serial from the EDID header.
    pub(crate) serial: Option<u32>,
}

/// One [`MonitorInfo`] per display, in enumeration order. A key already
/// given to an earlier display of the same enumeration gets a `#2`, `#3`...
/// suffix, so identical units with placeholder serials stay addressable.
pub(crate) fn monitor_infos(identities: &[DisplayIdentity]) -> Vec<MonitorInfo> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    identities
        .iter()
        .enumerate()
        .map(|(index, identity)| {
            let key = base_key(identity, index);
            let count = seen.entry(key.clone()).or_default();
            *count += 1;
            let id = match *count {
                1 => key,
                nth => format!("{key}#{nth}"),
            };
            monitor_info(identity, MonitorId::new(id))
        })
        .collect()
}

fn base_key(identity: &DisplayIdentity, index: usize) -> String {
    let edid_key = identity.edid.as_ref().map(edid_key).unwrap_or_default();
    [edid_key, sanitize(&identity.description)]
        .into_iter()
        .find(|key| !key.is_empty())
        .unwrap_or_else(|| format!("index-{index}"))
}

fn edid_key(edid: &EdidIdentity) -> String {
    let segments = [
        Some(edid.manufacturer.clone()),
        model_segment(edid),
        serial_segment(edid),
    ];
    sanitize(&segments.into_iter().flatten().collect::<Vec<_>>().join("-"))
}

fn model_segment(edid: &EdidIdentity) -> Option<String> {
    non_empty(edid.model_name.as_deref()).or_else(|| edid.model_id.map(|id| format!("{id:04X}")))
}

/// The serial descriptor wins; the numeric header serial is the fallback.
fn serial_segment(edid: &EdidIdentity) -> Option<String> {
    non_empty(edid.serial_number.as_deref())
        .or_else(|| edid.serial.map(|serial| format!("{serial:08X}")))
}

fn monitor_info(identity: &DisplayIdentity, id: MonitorId) -> MonitorInfo {
    let edid = identity.edid.as_ref();
    MonitorInfo {
        id,
        manufacturer: edid.and_then(|edid| non_empty(Some(&edid.manufacturer))),
        model: edid.and_then(|edid| non_empty(edid.model_name.as_deref())),
        serial: edid.and_then(serial_segment),
    }
}

fn non_empty(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// Keeps ASCII letters and digits; every other run of characters becomes a
/// single `-`, never at either end.
fn sanitize(raw: &str) -> String {
    raw.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests;
