use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use super::{BackendCall, FakeMonitor, InMemoryMonitorBackend};

fn info(key: &str) -> MonitorInfo {
    MonitorInfo {
        id: MonitorId::new(key),
        manufacturer: Some("RTK".to_owned()),
        model: Some("QHD HDR".to_owned()),
        serial: None,
    }
}

fn single_monitor(monitor: FakeMonitor) -> InMemoryMonitorBackend {
    InMemoryMonitorBackend::builder().monitor(monitor).build()
}

fn id(key: &str) -> MonitorId {
    MonitorId::new(key)
}

#[test]
fn enumerates_monitors_in_insertion_order() {
    let backend = InMemoryMonitorBackend::builder()
        .monitor(FakeMonitor::new(info("b")))
        .monitor(FakeMonitor::new(info("a")))
        .build();

    let monitors = backend.enumerate().unwrap();

    assert_eq!(monitors, [info("b"), info("a")]);
    assert_eq!(backend.calls(), [BackendCall::Enumerate]);
}

#[test]
fn returns_the_scripted_capabilities_string() {
    let backend = single_monitor(FakeMonitor::new(info("m")).with_capabilities("(vcp(10))"));

    assert_eq!(backend.read_capabilities(&id("m")).unwrap(), "(vcp(10))");
}

#[test]
fn missing_capabilities_string_is_a_transport_error() {
    let backend = single_monitor(FakeMonitor::new(info("m")));

    let result = backend.read_capabilities(&id("m"));

    assert!(matches!(result, Err(DdcError::Transport(_))));
}

#[test]
fn reads_scripted_values_regardless_of_capabilities() {
    let backend = single_monitor(
        FakeMonitor::new(info("m"))
            .with_capabilities("(vcp(10))")
            .with_value(VcpCode::AUDIO_VOLUME, 30, 100),
    );

    let value = backend.read_vcp(&id("m"), VcpCode::AUDIO_VOLUME).unwrap();

    assert_eq!(
        value,
        VcpValue {
            current: 30,
            max: 100
        }
    );
}

#[test]
fn unscripted_code_is_unsupported() {
    let backend = single_monitor(FakeMonitor::new(info("m")));

    let read = backend.read_vcp(&id("m"), VcpCode::CONTRAST);
    let write = backend.write_vcp(&id("m"), VcpCode::CONTRAST, 1);

    assert_eq!(read, Err(DdcError::UnsupportedFeature(VcpCode::CONTRAST)));
    assert_eq!(write, Err(DdcError::UnsupportedFeature(VcpCode::CONTRAST)));
}

#[test]
fn write_updates_the_value_seen_by_the_next_read() {
    let backend =
        single_monitor(FakeMonitor::new(info("m")).with_value(VcpCode::BRIGHTNESS, 50, 100));

    backend
        .write_vcp(&id("m"), VcpCode::BRIGHTNESS, 70)
        .unwrap();

    let value = backend.read_vcp(&id("m"), VcpCode::BRIGHTNESS).unwrap();
    assert_eq!(
        value,
        VcpValue {
            current: 70,
            max: 100
        }
    );
}

#[test]
fn ignored_write_is_acknowledged_but_not_applied() {
    let backend = single_monitor(
        FakeMonitor::new(info("m"))
            .with_value(VcpCode::BRIGHTNESS, 50, 100)
            .ignoring_writes_to(VcpCode::BRIGHTNESS),
    );

    backend
        .write_vcp(&id("m"), VcpCode::BRIGHTNESS, 70)
        .unwrap();

    let value = backend.read_vcp(&id("m"), VcpCode::BRIGHTNESS).unwrap();
    assert_eq!(value.current, 50);
}

#[test]
fn unknown_monitor_is_not_found() {
    let backend = single_monitor(FakeMonitor::new(info("m")).with_value(VcpCode::BRIGHTNESS, 1, 2));
    let ghost = id("ghost");
    let not_found = DdcError::MonitorNotFound(ghost.clone());

    assert_eq!(backend.read_capabilities(&ghost).unwrap_err(), not_found);
    assert_eq!(
        backend.read_vcp(&ghost, VcpCode::BRIGHTNESS).unwrap_err(),
        not_found
    );
    assert_eq!(
        backend
            .write_vcp(&ghost, VcpCode::BRIGHTNESS, 1)
            .unwrap_err(),
        not_found
    );
}

#[test]
fn logs_every_call_in_order_including_failures() {
    let backend = single_monitor(FakeMonitor::new(info("m")).with_value(VcpCode::BRIGHTNESS, 1, 2));

    let _ = backend.read_capabilities(&id("m"));
    let _ = backend.write_vcp(&id("m"), VcpCode::BRIGHTNESS, 2);
    let _ = backend.read_vcp(&id("ghost"), VcpCode::CONTRAST);

    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(id("m")),
            BackendCall::WriteVcp(id("m"), VcpCode::BRIGHTNESS, 2),
            BackendCall::ReadVcp(id("ghost"), VcpCode::CONTRAST),
        ]
    );
}

#[test]
fn clones_share_monitors_and_call_log() {
    let backend =
        single_monitor(FakeMonitor::new(info("m")).with_value(VcpCode::BRIGHTNESS, 50, 100));
    let handle = backend.clone();

    handle.write_vcp(&id("m"), VcpCode::BRIGHTNESS, 80).unwrap();

    assert_eq!(
        backend
            .read_vcp(&id("m"), VcpCode::BRIGHTNESS)
            .unwrap()
            .current,
        80
    );
    assert_eq!(handle.calls(), backend.calls());
    assert_eq!(backend.calls().len(), 2);
}
