use ddc_adapters::{BackendCall, FakeMonitor};
use ddc_core::domain::{DdcError, FeatureKind, MonitorId, Risk, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::support::{monitor_info, osd_with, rtk_id, rtk_monitor};

#[test]
fn get_feature_succeeds_for_code_absent_from_capabilities() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let reading = osd.get_feature(&rtk_id(), VcpCode::AUDIO_VOLUME).unwrap();

    assert!(!reading.declared_in_capabilities);
    assert_eq!(
        reading.value,
        VcpValue {
            current: 30,
            max: 100
        }
    );
    assert_eq!(reading.feature.code, VcpCode::AUDIO_VOLUME);
    assert!(
        backend
            .calls()
            .contains(&BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME))
    );
}

#[test]
fn declared_bare_code_reads_as_continuous() {
    let (osd, _) = osd_with([rtk_monitor()]);

    let reading = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();

    assert!(reading.declared_in_capabilities);
    assert_eq!(
        reading.value,
        VcpValue {
            current: 50,
            max: 100
        }
    );
    assert_eq!(reading.feature.kind, FeatureKind::Continuous);
    assert_eq!(reading.feature.risk, Risk::Safe);
}

#[test]
fn listed_code_reads_with_its_allowed_values() {
    let (osd, _) = osd_with([rtk_monitor()]);

    let reading = osd.get_feature(&rtk_id(), VcpCode::INPUT_SOURCE).unwrap();

    assert_eq!(reading.feature.kind, FeatureKind::NonContinuous);
    assert_eq!(reading.feature.risk, Risk::Dangerous);
    assert_eq!(
        reading.feature.allowed_values,
        Some(vec![0x01, 0x03, 0x04, 0x0F, 0x10, 0x11, 0x12])
    );
    assert_eq!(reading.value.current, 0x0F);
}

#[test]
fn capabilities_are_read_once_per_monitor() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();
    osd.get_feature(&rtk_id(), VcpCode::AUDIO_VOLUME).unwrap();
    let caps = osd.capabilities(&rtk_id()).unwrap();

    assert_eq!(caps.model.as_deref(), Some("RTK"));
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
            BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME),
        ]
    );
}

#[test]
fn unknown_monitor_is_reported_as_not_found() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let ghost = MonitorId::new("ghost");

    let result = osd.get_feature(&ghost, VcpCode::BRIGHTNESS);

    assert_eq!(result.unwrap_err(), DdcError::MonitorNotFound(ghost));
}

#[test]
fn unbalanced_capabilities_are_a_transport_error() {
    let broken = FakeMonitor::new(monitor_info(rtk_id()))
        .with_capabilities("(prot(monitor)vcp(10 12")
        .with_value(VcpCode::BRIGHTNESS, 50, 100);
    let (osd, _) = osd_with([broken]);

    let read = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS);
    let caps = osd.capabilities(&rtk_id());

    assert!(matches!(read, Err(DdcError::Transport(_))));
    assert!(matches!(caps, Err(DdcError::Transport(_))));
}

#[test]
fn backend_read_failure_is_propagated() {
    let (osd, _) = osd_with([rtk_monitor()]);

    let result = osd.get_feature(&rtk_id(), VcpCode::SHARPNESS);

    assert_eq!(
        result.unwrap_err(),
        DdcError::UnsupportedFeature(VcpCode::SHARPNESS)
    );
}

#[test]
fn list_monitors_returns_the_backend_enumeration() {
    let second = FakeMonitor::new(monitor_info(MonitorId::new("index-1")));
    let (osd, backend) = osd_with([rtk_monitor(), second]);

    let monitors = osd.list_monitors().unwrap();

    assert_eq!(
        monitors,
        [
            monitor_info(rtk_id()),
            monitor_info(MonitorId::new("index-1"))
        ]
    );
    assert_eq!(backend.calls(), [BackendCall::Enumerate]);
}
