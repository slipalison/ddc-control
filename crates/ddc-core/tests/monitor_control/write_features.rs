use ddc_adapters::{BackendCall, FakeMonitor};
use ddc_core::domain::{
    Access, Capabilities, Confirm, DdcError, Feature, FeatureKind, Risk, VcpCode, VcpValue,
};
use ddc_core::ports::MonitorControl;

use crate::support::{
    RED_BLACK_LEVEL, RTK_QHD_HDR_CAPS, osd_with, rtk_id, rtk_monitor,
    rtk_monitor_without_capabilities, wrote_anything,
};

/// 0x02 — new control value: non-continuous, no value list anywhere.
const NEW_CONTROL_VALUE: VcpCode = VcpCode(0x02);
/// 0x1E — auto setup: non-continuous, dangerous, no value list anywhere.
const AUTO_SETUP: VcpCode = VcpCode(0x1E);
/// 0xAC — horizontal frequency: read-only, declared by the dev monitor.
const HORIZONTAL_FREQUENCY: VcpCode = VcpCode(0xAC);

/// Writes `value` to `code` on a fresh core over `monitor`, and returns the
/// outcome with every backend call it made.
fn write_on(
    monitor: FakeMonitor,
    code: VcpCode,
    value: u16,
    confirm: Confirm,
) -> (Result<VcpValue, DdcError>, Vec<BackendCall>) {
    let (osd, backend) = osd_with([monitor]);
    let result = osd.set_feature(&rtk_id(), code, value, confirm);
    (result, backend.calls())
}

fn caps_only() -> Vec<BackendCall> {
    vec![BackendCall::ReadCapabilities(rtk_id())]
}

fn written_and_read_back(code: VcpCode, value: u16) -> Vec<BackendCall> {
    vec![
        BackendCall::ReadCapabilities(rtk_id()),
        BackendCall::WriteVcp(rtk_id(), code, value),
        BackendCall::ReadVcp(rtk_id(), code),
    ]
}

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

/// The dev monitor answers 0x62 volume and 0x6C red black level without
/// declaring them. Both are continuous, so a fresh core learns the maximum
/// with one read, then writes (D-2026-09-26-cli-1, restricted to continuous
/// codes by D-2026-09-26-full-osd-control-1).
#[test]
fn set_feature_learns_the_max_with_one_read_for_a_continuous_code_that_answers_but_is_undeclared_and_a_safe_write_succeeds()
 {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();
    for (code, value) in [(VcpCode::AUDIO_VOLUME, 40), (RED_BLACK_LEVEL, 45)] {
        assert!(!caps.declares(code), "{code}");
        assert_eq!(caps.feature(code).kind, FeatureKind::Continuous, "{code}");

        let (result, calls) = write_on(rtk_monitor(), code, value, Confirm::No);

        assert_eq!(
            result,
            Ok(VcpValue {
                current: value,
                max: 100
            }),
            "{code}"
        );
        assert_eq!(
            calls,
            [
                BackendCall::ReadCapabilities(rtk_id()),
                BackendCall::ReadVcp(rtk_id(), code),
                BackendCall::WriteVcp(rtk_id(), code, value),
                BackendCall::ReadVcp(rtk_id(), code),
            ],
            "{code}"
        );
    }
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

/// A non-continuous code without a list from the capabilities is never
/// bounded by a maximum read from the monitor: the catalog's value names
/// are the list, and without them the write is refused
/// (D-2026-09-26-full-osd-control-1, -8). No case reads anything before
/// it writes or fails.
#[test]
fn set_feature_never_reads_a_max_for_a_non_continuous_code_without_a_known_list_and_validates_against_the_catalog_or_refuses_it()
 {
    let unreadable = || {
        rtk_monitor_without_capabilities()
            .with_value(VcpCode::OSD_LANGUAGE, 0x01, 0x0D)
            .with_value(VcpCode::RESTORE_FACTORY_DEFAULTS, 0, 1)
            .with_value(NEW_CONTROL_VALUE, 1, 2)
            .with_value(AUTO_SETUP, 0, 1)
    };
    let declared = rtk_monitor().with_value(NEW_CONTROL_VALUE, 1, 2);
    let preset = VcpCode::COLOR_PRESET;
    let reset = VcpCode::RESTORE_FACTORY_DEFAULTS;
    let not_allowed = |code, value| Err(DdcError::ValueNotAllowed { code, value });
    let unsupported = |code| Err(DdcError::UnsupportedFeature(code));

    let cases = [
        (
            unreadable(),
            preset,
            0x03,
            Confirm::No,
            not_allowed(preset, 0x03),
            caps_only(),
        ),
        (
            unreadable(),
            preset,
            0x05,
            Confirm::No,
            Ok(VcpValue {
                current: 0x05,
                max: 0x0B,
            }),
            written_and_read_back(preset, 0x05),
        ),
        (
            unreadable(),
            VcpCode::OSD_LANGUAGE,
            0x02,
            Confirm::No,
            Ok(VcpValue {
                current: 0x02,
                max: 0x0D,
            }),
            written_and_read_back(VcpCode::OSD_LANGUAGE, 0x02),
        ),
        (
            unreadable(),
            reset,
            0x01,
            Confirm::Yes,
            Ok(VcpValue {
                current: 0x01,
                max: 0x01,
            }),
            vec![
                BackendCall::ReadCapabilities(rtk_id()),
                BackendCall::WriteVcp(rtk_id(), reset, 0x01),
            ],
        ),
        (
            unreadable(),
            reset,
            0x00,
            Confirm::Yes,
            not_allowed(reset, 0x00),
            caps_only(),
        ),
        (
            unreadable(),
            AUTO_SETUP,
            1,
            Confirm::Yes,
            unsupported(AUTO_SETUP),
            caps_only(),
        ),
        (
            unreadable(),
            NEW_CONTROL_VALUE,
            1,
            Confirm::No,
            unsupported(NEW_CONTROL_VALUE),
            caps_only(),
        ),
        (
            declared,
            NEW_CONTROL_VALUE,
            1,
            Confirm::No,
            unsupported(NEW_CONTROL_VALUE),
            caps_only(),
        ),
    ];
    for (monitor, code, value, confirm, expected, expected_calls) in cases {
        let (result, calls) = write_on(monitor, code, value, confirm);

        assert_eq!(result, expected, "{code} = {value}");
        assert_eq!(calls, expected_calls, "{code} = {value}");
    }
}

/// Read-only codes, declared or not, are refused before the monitor is read,
/// with or without confirmation; `Table` features are refused by the same
/// guard (D-2026-09-26-full-osd-control-1, -5).
#[test]
fn set_feature_rejects_a_read_only_or_table_code_before_reading_its_value_from_the_backend() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();
    let monitor = || {
        rtk_monitor()
            .with_value(HORIZONTAL_FREQUENCY, 3, 0xFFFF)
            .with_value(VcpCode::VCP_VERSION, 0x0202, 0xFFFF)
            .with_value(VcpCode::FIRMWARE_LEVEL, 0x0001, 0xFFFF)
    };
    let read_only = [
        (HORIZONTAL_FREQUENCY, true),
        (VcpCode::VCP_VERSION, true),
        (VcpCode::FIRMWARE_LEVEL, false),
    ];
    for (code, declared) in read_only {
        assert_eq!(caps.declares(code), declared, "{code}");
        assert_eq!(caps.feature(code).access, Access::ReadOnly, "{code}");
        for confirm in [Confirm::No, Confirm::Yes] {
            let (result, calls) = write_on(monitor(), code, 1, confirm);

            assert_eq!(result, Err(DdcError::UnsupportedFeature(code)), "{code}");
            assert_eq!(calls, caps_only(), "{code} {confirm:?}");
        }
    }
    let table = Feature {
        code: VcpCode(0x73),
        kind: FeatureKind::Table,
        access: Access::ReadWrite,
        risk: Risk::Safe,
        allowed_values: Some(vec![0x01]),
    };
    assert_eq!(
        table.ensure_writable(),
        Err(DdcError::UnsupportedFeature(VcpCode(0x73)))
    );
}

/// A factory reset writes 0x01 — MCCS ignores zero — and is never read
/// back: the code is write-only (D-2026-09-26-full-osd-control-8).
#[test]
fn reset_writes_one_and_skips_read_back() {
    let reset = VcpCode::RESTORE_FACTORY_DEFAULTS;
    let monitor = rtk_monitor().with_value(reset, 0, 1);

    let (result, calls) = write_on(monitor, reset, 0x01, Confirm::Yes);

    assert_eq!(result, Ok(VcpValue { current: 1, max: 1 }));
    assert_eq!(
        calls,
        [
            BackendCall::ReadCapabilities(rtk_id()),
            BackendCall::WriteVcp(rtk_id(), reset, 0x01),
        ]
    );
    assert!(
        !calls
            .iter()
            .any(|call| matches!(call, BackendCall::ReadVcp(..))),
        "{calls:?}"
    );
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
