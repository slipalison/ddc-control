use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::{MonitorId, MonitorInfo, VcpCode};

/// Capabilities string of the dev monitor "RTK QHD HDR", verbatim, from the
/// fixture every test site shares.
pub const RTK_QHD_HDR_CAPS: &str =
    include_str!("../fixtures/rtk_qhd_hdr_caps.txt").trim_ascii_end();

pub type Osd = SoftwareOsd<InMemoryMonitorBackend>;

pub fn rtk_id() -> MonitorId {
    MonitorId::new("RTK-QHD-HDR-0001")
}

pub fn monitor_info(id: MonitorId) -> MonitorInfo {
    MonitorInfo {
        id,
        manufacturer: Some("RTK".to_owned()),
        model: Some("QHD HDR".to_owned()),
        serial: None,
    }
}

/// 0x6C — video black level, red: answered by the dev monitor, undeclared.
pub const RED_BLACK_LEVEL: VcpCode = VcpCode(0x6C);

/// The dev monitor: real capabilities plus values for a few codes, including
/// 0x62 volume and 0x6C red black level, which answer although the
/// capabilities omit them.
pub fn rtk_monitor() -> FakeMonitor {
    with_rtk_values(FakeMonitor::new(monitor_info(rtk_id())).with_capabilities(RTK_QHD_HDR_CAPS))
}

/// The dev monitor's values behind a scaler that never answers the
/// capabilities request.
pub fn rtk_monitor_without_capabilities() -> FakeMonitor {
    with_rtk_values(FakeMonitor::new(monitor_info(rtk_id())))
}

fn with_rtk_values(monitor: FakeMonitor) -> FakeMonitor {
    monitor
        .with_value(VcpCode::BRIGHTNESS, 50, 100)
        .with_value(VcpCode::COLOR_PRESET, 0x05, 0x0B)
        .with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x12)
        .with_value(VcpCode::AUDIO_VOLUME, 30, 100)
        .with_value(RED_BLACK_LEVEL, 50, 100)
        .with_value(VcpCode::POWER_MODE, 0x01, 0x05)
}

/// A core wired to `monitors`, plus a handle on the same fake to inspect it.
pub fn osd_with(monitors: impl IntoIterator<Item = FakeMonitor>) -> (Osd, InMemoryMonitorBackend) {
    let backend = monitors
        .into_iter()
        .fold(InMemoryMonitorBackend::builder(), |builder, monitor| {
            builder.monitor(monitor)
        })
        .build();
    (SoftwareOsd::new(backend.clone()), backend)
}

/// Whether any `write_vcp` reached the backend.
pub fn wrote_anything(backend: &InMemoryMonitorBackend) -> bool {
    backend
        .calls()
        .iter()
        .any(|call| matches!(call, BackendCall::WriteVcp(..)))
}
