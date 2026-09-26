use ddc_adapters::{BackendCall, FakeMonitor};
use ddc_core::domain::{DdcError, FeatureKind, MonitorId, Risk, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::support::{
    monitor_info, osd_with, rtk_id, rtk_monitor, rtk_monitor_without_capabilities,
};

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
fn unbalanced_capabilities_do_not_block_reads() {
    let broken = FakeMonitor::new(monitor_info(rtk_id()))
        .with_capabilities("(prot(monitor)vcp(10 12")
        .with_value(VcpCode::BRIGHTNESS, 50, 100);
    let (osd, backend) = osd_with([broken]);

    let reading = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();

    assert!(!reading.declared_in_capabilities);
    assert_eq!(
        reading.value,
        VcpValue {
            current: 50,
            max: 100
        }
    );
    assert_eq!(reading.feature.kind, FeatureKind::Continuous);
    assert_eq!(reading.feature.risk, Risk::Safe);
    assert_eq!(reading.feature.allowed_values, None);
    assert!(
        backend
            .calls()
            .contains(&BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS))
    );
}

#[test]
fn unreadable_capabilities_are_fetched_once_across_reads() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);

    osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();
    let preset = osd.get_feature(&rtk_id(), VcpCode::COLOR_PRESET).unwrap();

    assert_eq!(preset.feature.kind, FeatureKind::Continuous);
    assert_eq!(preset.feature.allowed_values, None);
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
            BackendCall::ReadVcp(rtk_id(), VcpCode::COLOR_PRESET),
        ]
    );
}

#[test]
fn explicit_capabilities_request_reports_and_retries_the_failure() {
    let broken = FakeMonitor::new(monitor_info(rtk_id())).with_capabilities("(vcp(10");
    let (osd, backend) = osd_with([broken.with_value(VcpCode::BRIGHTNESS, 50, 100)]);
    osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();

    let first = osd.capabilities(&rtk_id());
    let second = osd.capabilities(&rtk_id());

    assert!(matches!(first, Err(DdcError::Transport(_))));
    assert!(matches!(second, Err(DdcError::Transport(_))));
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadCapabilities(rtk_id()),
        ]
    );
}

#[test]
fn missing_monitor_is_not_remembered_as_unreadable_capabilities() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let ghost = MonitorId::new("ghost");

    let first = osd.get_feature(&ghost, VcpCode::BRIGHTNESS);
    let second = osd.get_feature(&ghost, VcpCode::BRIGHTNESS);

    assert_eq!(first.unwrap_err(), DdcError::MonitorNotFound(ghost.clone()));
    assert_eq!(
        second.unwrap_err(),
        DdcError::MonitorNotFound(ghost.clone())
    );
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(ghost.clone()),
            BackendCall::ReadCapabilities(ghost),
        ]
    );
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

#[test]
fn explicit_capabilities_request_recovers_from_transient_failure() {
    let (osd, backend) = osd_with([rtk_monitor().with_transient_capabilities_failures(1)]);

    let before = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();
    let caps = osd.capabilities(&rtk_id()).unwrap();
    let after = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();

    assert!(!before.declared_in_capabilities);
    assert_eq!(caps.model.as_deref(), Some("RTK"));
    assert!(after.declared_in_capabilities);
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
        ]
    );
}
