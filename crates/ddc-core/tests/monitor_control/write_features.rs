use ddc_adapters::BackendCall;
use ddc_core::domain::{Confirm, DdcError, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::support::{
    osd_with, rtk_id, rtk_monitor, rtk_monitor_without_capabilities, wrote_anything,
};

#[test]
fn dangerous_write_without_confirm_is_rejected() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let unclassified = VcpCode(0x8D);

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

/// The dev monitor answers 0x62 volume without declaring it; a fresh core
/// learns its maximum with one read, then writes (D-2026-09-26-cli-1).
#[test]
fn safe_write_to_an_answered_but_undeclared_code_reads_its_max_first() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 40, Confirm::No);

    assert_eq!(
        result.unwrap(),
        VcpValue {
            current: 40,
            max: 100
        }
    );
    assert_eq!(
        backend.calls(),
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME),
            BackendCall::WriteVcp(rtk_id(), VcpCode::AUDIO_VOLUME, 40),
            BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME),
        ]
    );
}

#[test]
fn undeclared_code_is_bounded_by_the_max_it_reads() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 101, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::InvalidValue {
            code: VcpCode::AUDIO_VOLUME,
            value: 101,
            max: 100
        }
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn failed_max_read_is_reported_and_nothing_is_written() {
    let failures = [
        DdcError::Transport("bus gone".to_owned()),
        DdcError::Timeout,
        DdcError::MonitorNotFound(rtk_id()),
    ];
    for failure in failures {
        let flaky = rtk_monitor().with_vcp_failure(VcpCode::AUDIO_VOLUME, failure.clone());
        let (osd, backend) = osd_with([flaky]);

        let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 40, Confirm::No);

        assert_eq!(result.unwrap_err(), failure);
        assert_eq!(
            backend.calls(),
            [
                BackendCall::ReadCapabilities(rtk_id()),
                BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME),
            ]
        );
    }
}

#[test]
fn code_the_monitor_does_not_answer_is_unsupported_and_not_written() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::SHARPNESS, 3, Confirm::No);

    assert_eq!(
        result.unwrap_err(),
        DdcError::UnsupportedFeature(VcpCode::SHARPNESS)
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn undeclared_code_read_earlier_is_written_without_reading_again() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    osd.get_feature(&rtk_id(), VcpCode::AUDIO_VOLUME).unwrap();
    let calls_before = backend.calls().len();

    let result = osd.set_feature(&rtk_id(), VcpCode::AUDIO_VOLUME, 40, Confirm::No);

    assert_eq!(
        result.unwrap(),
        VcpValue {
            current: 40,
            max: 100
        }
    );
    assert_eq!(
        backend.calls()[calls_before..],
        [
            BackendCall::WriteVcp(rtk_id(), VcpCode::AUDIO_VOLUME, 40),
            BackendCall::ReadVcp(rtk_id(), VcpCode::AUDIO_VOLUME),
        ]
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
fn write_without_capabilities_reads_the_max_first() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);

    let result = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 70, Confirm::No);

    assert_eq!(result.unwrap().current, 70);
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
