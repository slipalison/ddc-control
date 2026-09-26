//! The fixed monitor served by the hidden `--fake` flag
//! (D-2026-09-25-cli-4): deterministic data for end-to-end tests, never
//! real hardware. Only data lives here; the backend serving it is built by
//! the composition root.

use ddc_adapters::FakeMonitor;
use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};

/// Id of the fake monitor.
pub const FIXTURE_ID: &str = "FAKE-CLI-TEST";

/// Capabilities string of the fake monitor. It declares five of the six
/// original feature names: like on the dev monitor, 0x62 volume answers but
/// is left out, and so is 0xCC OSD language.
pub const FIXTURE_CAPS: &str = "(prot(monitor)type(LCD)model(FAKE)vcp(10 12 14(01 02 03) \
                                60(01 0F 11) D6(01 04 05))mccs_ver(2.2))";

/// A code whose every read and write times out, to exercise the
/// transport exit code end to end.
pub const SIMULATED_TIMEOUT: VcpCode = VcpCode(0xDF);

/// 0x7E trapezoid fails as it does on the dev monitor, whose reply makes
/// `ddc-i2c` 0.2.2 panic inside the real backend
/// (D-2026-09-26-full-osd-control-6).
pub const SIMULATED_PANIC: VcpCode = VcpCode(0x7E);

/// The transport error the real backend reports for [`SIMULATED_PANIC`].
pub const SIMULATED_PANIC_MESSAGE: &str =
    "ddc-hi panicked: index out of bounds: the len is 11 but the index is 11";

/// The fake monitor, answering the six original feature names, the OSD
/// language and the four factory resets — the fake only takes a write to a
/// code that has a value.
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
    .with_value(VcpCode::OSD_LANGUAGE, 0x03, 0x0D)
    .with_value(VcpCode::RESTORE_FACTORY_DEFAULTS, 0x00, 0x01)
    .with_value(VcpCode::RESTORE_FACTORY_LUMINANCE_CONTRAST, 0x00, 0x01)
    .with_value(VcpCode::RESTORE_FACTORY_GEOMETRY, 0x00, 0x01)
    .with_value(VcpCode::RESTORE_FACTORY_COLOR, 0x00, 0x01)
    .with_vcp_failure(SIMULATED_TIMEOUT, DdcError::Timeout)
    .with_vcp_failure(
        SIMULATED_PANIC,
        DdcError::Transport(SIMULATED_PANIC_MESSAGE.to_owned()),
    )
}
