use ddc_adapters::BackendCall;
use ddc_core::domain::{Confirm, DdcError, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::support::{
    osd_with, rtk_id, rtk_monitor, rtk_monitor_without_capabilities, wrote_anything,
};

#[test]
fn dangerous_write_without_confirm_is_rejected() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let unclassified = VcpCode(0x52);

    let input = osd.set_feature(&rtk_id(), VcpCode::INPUT_SOURCE, 0x11, Confirm::No);
    let unknown = osd.set_feature(&rtk_id(), unclassified, 1, Confirm::No);

    assert_eq!(
        input.unwrap_err(),
        DdcError::DangerousWriteNotConfirmed(VcpCode::INPUT_SOURCE)
    );
    assert_eq!(
        unknown.unwrap_err(),
        DdcError::DangerousWriteNotConfirmed(unclassified)
    );
    assert!(backend.calls().is_empty());
}

#[test]
fn write_value_above_max_is_rejected() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();
    let calls_before = backend.calls();

    let result = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 101, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::InvalidValue {
            code: VcpCode::BRIGHTNESS,
            value: 101,
            max: 100
        }
    );
    assert_eq!(backend.calls(), calls_before);
}

#[test]
fn write_reads_back_value_after_success() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let (quirky_osd, _) = osd_with([rtk_monitor().ignoring_writes_to(VcpCode::BRIGHTNESS)]);

    let applied = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 70, Confirm::No);
    let dropped = quirky_osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 70, Confirm::No);

    assert_eq!(
        applied.unwrap(),
        VcpValue {
            current: 70,
            max: 100
        }
    );
    assert_eq!(
        dropped.unwrap(),
        VcpValue {
            current: 50,
            max: 100
        }
    );
    assert!(backend.calls().ends_with(&[
        BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 70),
        BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
    ]));
}

#[test]
fn declared_code_learns_its_max_before_the_first_write() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let too_high = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 150, Confirm::No);

    assert!(matches!(
        too_high,
        Err(DdcError::InvalidValue { max: 100, .. })
    ));
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
        ]
    );
}

#[test]
fn undeclared_code_never_read_is_not_written() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 40, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::UnsupportedFeature(VcpCode::AUDIO_VOLUME)
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn undeclared_code_is_writable_once_read() {
    let (osd, _) = osd_with([rtk_monitor()]);
    osd.get_feature(&rtk_id(), VcpCode::AUDIO_VOLUME).unwrap();

    let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 40, Confirm::No);

    assert_eq!(
        result.unwrap(),
        VcpValue {
            current: 40,
            max: 100
        }
    );
}

#[test]
fn listed_value_is_written_without_reading_a_max() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::COLOR_PRESET, 0x06, Confirm::No);

    assert_eq!(result.unwrap().current, 0x06);
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::WriteVcp(rtk_id(), VcpCode::COLOR_PRESET, 0x06),
            BackendCall::ReadVcp(rtk_id(), VcpCode::COLOR_PRESET),
        ]
    );
}

#[test]
fn unlisted_value_of_listed_feature_is_not_allowed() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::COLOR_PRESET, 0x03, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::ValueNotAllowed {
            code: VcpCode::COLOR_PRESET,
            value: 0x03
        }
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn confirmation_does_not_bypass_value_validation() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::POWER_MODE, 0x02, Confirm::Yes);

    assert_eq!(
        result.unwrap_err(),
        DdcError::ValueNotAllowed {
            code: VcpCode::POWER_MODE,
            value: 0x02
        }
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn confirmed_dangerous_write_is_performed() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::INPUT_SOURCE, 0x11, Confirm::Yes);

    assert_eq!(
        result.unwrap(),
        VcpValue {
            current: 0x11,
            max: 0x12
        }
    );
    assert!(wrote_anything(&backend));
}

#[test]
fn write_without_read_or_capabilities_is_unsupported() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 70, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::UnsupportedFeature(VcpCode::BRIGHTNESS)
    );
    assert_eq!(backend.calls(), [BackendCall::ReadCapabilities(rtk_id())]);
}

#[test]
fn write_after_read_uses_cached_max_without_capabilities() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);
    osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();

    let too_high = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 101, Confirm::No);
    let applied = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 70, Confirm::No);

    assert!(matches!(
        too_high,
        Err(DdcError::InvalidValue { max: 100, .. })
    ));
    assert_eq!(
        applied.unwrap(),
        VcpValue {
            current: 70,
            max: 100
        }
    );
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
            BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 70),
            BackendCall::ReadVcp(rtk_id(), VcpCode::BRIGHTNESS),
        ]
    );
}

#[test]
fn without_capabilities_a_listed_feature_is_bounded_by_its_max() {
    let (osd, _) = osd_with([rtk_monitor_without_capabilities()]);
    osd.get_feature(&rtk_id(), VcpCode::COLOR_PRESET).unwrap();

    let unlisted_in_caps = osd.set_feature(&rtk_id(), VcpCode::COLOR_PRESET, 0x03, Confirm::No);
    let above_max = osd.set_feature(&rtk_id(), VcpCode::COLOR_PRESET, 0x0C, Confirm::No);

    assert_eq!(unlisted_in_caps.unwrap().current, 0x03);
    assert!(matches!(
        above_max,
        Err(DdcError::InvalidValue { max: 0x0B, .. })
    ));
}

#[test]
fn dangerous_write_is_rejected_before_capabilities_are_fetched() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::POWER_MODE, 0x04, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::DangerousWriteNotConfirmed(VcpCode::POWER_MODE)
    );
    assert!(backend.calls().is_empty());
}
