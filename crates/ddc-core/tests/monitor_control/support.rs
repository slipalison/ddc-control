use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::{MonitorId, MonitorInfo, VcpCode};

/// Capabilities string of the dev monitor "RTK QHD HDR", verbatim.
pub const RTK_QHD_HDR_CAPS: &str = "(prot(monitor)type(LCD)model(RTK)cmds(01 02 03 07 0C E3 F3)vcp(02 04 05 06 08 0B 0C 10 12 14(01 02 04 05 06 08 0B) 16 18 1A 52 60(01 03 04 0F 10 11 12) 87 AC AE B2 B6 C6 C8 CA CC(01 02 03 04 06 0A 0D) D6(01 04 05) DF FD FF)mswhql(1)asset_eep(40)mccs_ver(2.2))";

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

/// The dev monitor: real capabilities plus values for a few codes, including
/// 0x62 volume, which answers although the capabilities omit it.
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
