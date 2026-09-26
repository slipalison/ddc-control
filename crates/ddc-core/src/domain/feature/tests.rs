use super::{Access, Confirm, Feature, FeatureKind, Risk, authorize_write, risk_for_code};
use crate::domain::{DdcError, VcpCode};

#[test]
fn seed_safe_codes_are_safe() {
    let safe = [
        VcpCode::BRIGHTNESS,
        VcpCode::CONTRAST,
        VcpCode::COLOR_PRESET,
        VcpCode::RED_GAIN,
        VcpCode::GREEN_GAIN,
        VcpCode::BLUE_GAIN,
        VcpCode::AUDIO_VOLUME,
        VcpCode::SHARPNESS,
        VcpCode::OSD_LANGUAGE,
    ];
    for code in safe {
        assert_eq!(risk_for_code(code), Risk::Safe, "{code}");
    }
}

#[test]
fn seed_dangerous_codes_are_dangerous() {
    let dangerous = [
        VcpCode::RESTORE_FACTORY_DEFAULTS,
        VcpCode::RESTORE_FACTORY_LUMINANCE_CONTRAST,
        VcpCode::RESTORE_FACTORY_GEOMETRY,
        VcpCode::RESTORE_FACTORY_COLOR,
        VcpCode::INPUT_SOURCE,
        VcpCode::OSD_LOCK,
        VcpCode::POWER_MODE,
    ];
    for code in dangerous {
        assert_eq!(risk_for_code(code), Risk::Dangerous, "{code}");
    }
}

#[test]
fn manufacturer_specific_range_is_dangerous() {
    for raw in VcpCode::MANUFACTURER_SPECIFIC_START.0..=u8::MAX {
        assert_eq!(
            risk_for_code(VcpCode(raw)),
            Risk::Dangerous,
            "{}",
            VcpCode(raw)
        );
    }
}

#[test]
fn unclassified_codes_default_to_dangerous() {
    let rtk_codes_outside_seed = [
        0x02, 0x0B, 0x0C, 0x52, 0xAC, 0xAE, 0xB2, 0xB6, 0xC6, 0xC8, 0xDF,
    ];
    for raw in rtk_codes_outside_seed {
        assert_eq!(
            risk_for_code(VcpCode(raw)),
            Risk::Dangerous,
            "{}",
            VcpCode(raw)
        );
    }
}

fn feature(code: VcpCode, allowed_values: Option<Vec<u8>>) -> Feature {
    let kind = match allowed_values {
        Some(_) => FeatureKind::NonContinuous,
        None => FeatureKind::Continuous,
    };
    Feature {
        code,
        kind,
        access: Access::ReadWrite,
        risk: risk_for_code(code),
        allowed_values,
    }
}

#[test]
fn unconfirmed_write_is_refused_only_for_dangerous_codes() {
    assert_eq!(authorize_write(VcpCode::BRIGHTNESS, Confirm::No), Ok(()));
    assert_eq!(authorize_write(VcpCode::POWER_MODE, Confirm::Yes), Ok(()));
    assert_eq!(
        authorize_write(VcpCode::POWER_MODE, Confirm::No),
        Err(DdcError::DangerousWriteNotConfirmed(VcpCode::POWER_MODE))
    );
    assert_eq!(
        authorize_write(VcpCode(0x52), Confirm::No),
        Err(DdcError::DangerousWriteNotConfirmed(VcpCode(0x52)))
    );
}

#[test]
fn only_unlisted_features_require_a_known_max() {
    assert!(feature(VcpCode::BRIGHTNESS, None).requires_known_max());
    assert!(!feature(VcpCode::COLOR_PRESET, Some(vec![0x01])).requires_known_max());
}

#[test]
fn continuous_write_is_bounded_by_the_known_max() {
    let brightness = feature(VcpCode::BRIGHTNESS, None);

    assert_eq!(brightness.validate_write(0, Some(100)), Ok(()));
    assert_eq!(brightness.validate_write(100, Some(100)), Ok(()));
    assert_eq!(
        brightness.validate_write(101, Some(100)),
        Err(DdcError::InvalidValue {
            code: VcpCode::BRIGHTNESS,
            value: 101,
            max: 100
        })
    );
}

#[test]
fn continuous_write_without_known_max_is_unsupported() {
    let volume = feature(VcpCode::AUDIO_VOLUME, None);

    assert_eq!(
        volume.validate_write(10, None),
        Err(DdcError::UnsupportedFeature(VcpCode::AUDIO_VOLUME))
    );
}

#[test]
fn listed_write_accepts_only_listed_values() {
    let preset = feature(VcpCode::COLOR_PRESET, Some(vec![0x01, 0x05]));
    let not_allowed = |value| {
        Err(DdcError::ValueNotAllowed {
            code: VcpCode::COLOR_PRESET,
            value,
        })
    };

    assert_eq!(preset.validate_write(0x05, None), Ok(()));
    assert_eq!(preset.validate_write(0x02, Some(0xFF)), not_allowed(0x02));
    assert_eq!(preset.validate_write(0x0105, None), not_allowed(0x0105));
}

#[test]
fn read_only_feature_is_never_writable() {
    let read_only = Feature {
        access: Access::ReadOnly,
        ..feature(VcpCode(0x02), None)
    };

    assert_eq!(
        read_only.validate_write(1, Some(10)),
        Err(DdcError::UnsupportedFeature(VcpCode(0x02)))
    );
}
