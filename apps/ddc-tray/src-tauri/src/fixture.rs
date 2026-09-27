//! The simulated monitor behind `DDC_TRAY_FAKE=1` (D-2026-09-27-tray-app-5):
//! the dev monitor "RTK QHD HDR" as the popup's contract pins it — the
//! values of the golden `tests/fixtures/contract-rtk.json`, which the demo
//! bridge mirrors — for end-to-end checks that must never touch a real
//! monitor. The unit tests build on the same data. Only data lives here;
//! the backend serving it is built by the composition root.

use ddc_adapters::FakeMonitor;
use ddc_core::domain::{MonitorId, MonitorInfo, VcpCode};

/// Id the real backend derives for the dev monitor "RTK QHD HDR".
pub const RTK_ID: &str = "RTK-RTK-QHD-HDR-01010101";

/// Capabilities of the dev monitor, from the fixture the core's tests share.
pub const RTK_CAPS: &str =
    include_str!("../../../../crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt")
        .trim_ascii_end();

/// 0x0C — colour temperature request, declared by the dev monitor.
pub const COLOR_TEMP: VcpCode = VcpCode(0x0C);

/// The id of the simulated monitor.
pub fn rtk_id() -> MonitorId {
    MonitorId::new(RTK_ID)
}

/// The identity the dev monitor's EDID gives.
pub fn rtk_info() -> MonitorInfo {
    MonitorInfo {
        id: rtk_id(),
        manufacturer: Some("RTK".to_owned()),
        model: Some("RTK QHD HDR".to_owned()),
        serial: Some("01010101".to_owned()),
    }
}

/// The contract's RTK scenario: the dev monitor's real capabilities and
/// identity, the six quick controls at fixed values — volume answering
/// although the capabilities omit it — and the seven "all settings" codes.
/// The demo bridge mirrors exactly these values.
pub fn rtk_monitor() -> FakeMonitor {
    FakeMonitor::new(rtk_info())
        .with_capabilities(RTK_CAPS)
        .with_value(VcpCode::BRIGHTNESS, 75, 100)
        .with_value(VcpCode::CONTRAST, 50, 100)
        .with_value(VcpCode::AUDIO_VOLUME, 30, 100)
        .with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x03)
        .with_value(VcpCode::COLOR_PRESET, 0x01, 0x0B)
        .with_value(VcpCode::POWER_MODE, 0x01, 0x05)
        .with_value(COLOR_TEMP, 70, 100)
        .with_value(VcpCode::RED_GAIN, 50, 100)
        .with_value(VcpCode::GREEN_GAIN, 48, 100)
        .with_value(VcpCode::BLUE_GAIN, 46, 100)
        .with_value(VcpCode::SHARPNESS, 5, 10)
        .with_value(VcpCode::OSD_LOCK, 0x02, 0x02)
        .with_value(VcpCode::OSD_LANGUAGE, 0x02, 0x0D)
}
