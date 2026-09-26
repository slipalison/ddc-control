//! The fixed monitor served by the hidden `--fake` flag
//! (D-2026-09-25-cli-4): deterministic data for end-to-end tests, never
//! real hardware. Only data lives here; the backend serving it is built by
//! the composition root.

use ddc_adapters::FakeMonitor;
use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};

/// Id of the fake monitor.
pub const FIXTURE_ID: &str = "FAKE-CLI-TEST";

/// Capabilities string of the fake monitor, declaring the six shortcuts.
pub const FIXTURE_CAPS: &str = "(prot(monitor)type(LCD)model(FAKE)vcp(10 12 14(01 02 03) \
                                60(01 0F 11) 62 D6(01 04 05))mccs_ver(2.2))";

/// A code whose every read and write times out, to exercise the
/// transport exit code end to end.
pub const SIMULATED_TIMEOUT: VcpCode = VcpCode(0xDF);

/// The fake monitor, with a value for each of the six shortcuts.
#[doc(hidden)]
pub fn fixture_monitor() -> FakeMonitor {
    FakeMonitor::new(MonitorInfo {
        id: MonitorId::new(FIXTURE_ID),
        manufacturer: Some("FAK".to_owned()),
        model: Some("CLI TEST".to_owned()),
        serial: None,
    })
    .with_capabilities(FIXTURE_CAPS)
    .with_value(VcpCode::BRIGHTNESS, 50, 100)
    .with_value(VcpCode::CONTRAST, 70, 100)
    .with_value(VcpCode::AUDIO_VOLUME, 30, 100)
    .with_value(VcpCode::COLOR_PRESET, 0x01, 0x03)
    .with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x11)
    .with_value(VcpCode::POWER_MODE, 0x01, 0x05)
    .with_vcp_failure(SIMULATED_TIMEOUT, DdcError::Timeout)
}
