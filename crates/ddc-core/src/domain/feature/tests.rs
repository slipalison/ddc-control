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

/// Every manufacturer-specific code is dangerous, except 0xFD and 0xFF: the
/// catalog lists them read-only, and a read-only code is never dangerous.
#[test]
fn manufacturer_specific_range_is_dangerous_except_its_read_only_codes() {
    for raw in VcpCode::MANUFACTURER_SPECIFIC_START.0..=u8::MAX {
        let expected = match raw {
            0xFD | 0xFF => Risk::Safe,
            _ => Risk::Dangerous,
        };
        assert_eq!(risk_for_code(VcpCode(raw)), expected, "{}", VcpCode(raw));
    }
}

#[test]
fn codes_outside_the_catalog_default_to_dangerous() {
    let uncatalogued = [0x00, 0x01, 0x03, 0x8D, 0xDC, 0xE0, 0xE5];
    for raw in uncatalogued {
        assert_eq!(
            risk_for_code(VcpCode(raw)),
            Risk::Dangerous,
            "{}",
            VcpCode(raw)
        );
    }
}

/// These codes of the dev monitor fell back to `Dangerous` before the
/// catalog classified them (D-2026-09-26-full-osd-control-1).
#[test]
fn rtk_codes_outside_the_seed_are_classified_by_the_catalog() {
    let rtk_codes_outside_seed = [
        0x02, 0x0B, 0x0C, 0x52, 0xAC, 0xAE, 0xB2, 0xB6, 0xC6, 0xC8, 0xDF,
    ];
    for raw in rtk_codes_outside_seed {
        assert_eq!(risk_for_code(VcpCode(raw)), Risk::Safe, "{}", VcpCode(raw));
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
        authorize_write(VcpCode(0x8D), Confirm::No),
        Err(DdcError::DangerousWriteNotConfirmed(VcpCode(0x8D)))
    );
}

/// A non-continuous value is one of a closed set: its maximum never bounds
/// a write, even when no list is known (D-2026-09-26-full-osd-control-1).
#[test]
fn only_unlisted_continuous_features_require_a_known_max() {
    let unlisted_preset = Feature {
        kind: FeatureKind::NonContinuous,
        ..feature(VcpCode::COLOR_PRESET, None)
    };

    assert!(feature(VcpCode::BRIGHTNESS, None).requires_known_max());
    assert!(!feature(VcpCode::COLOR_PRESET, Some(vec![0x01])).requires_known_max());
    assert!(!unlisted_preset.requires_known_max());
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

/// A `Table` write is refused first, whatever its list, maximum or access
/// say (D-2026-09-26-full-osd-control-5).
#[test]
fn validate_write_rejects_table_kind_features_before_checking_access_or_allowed_values() {
    let table = Feature {
        kind: FeatureKind::Table,
        ..feature(VcpCode(0x73), Some(vec![0x01, 0x02]))
    };
    let read_only_table = Feature {
        access: Access::ReadOnly,
        ..table.clone()
    };

    for feature in [&table, &read_only_table] {
        assert_eq!(
            feature.validate_write(0x01, Some(0xFF)),
            Err(DdcError::UnsupportedFeature(VcpCode(0x73)))
        );
        assert_eq!(
            feature.validate_write(0x03, None),
            Err(DdcError::UnsupportedFeature(VcpCode(0x73)))
        );
    }
}

#[test]
fn ensure_writable_refuses_table_and_read_only_features_only() {
    let unsupported = |code| Err(DdcError::UnsupportedFeature(code));
    let table = Feature {
        kind: FeatureKind::Table,
        ..feature(VcpCode(0x73), None)
    };
    let read_only = Feature {
        access: Access::ReadOnly,
        ..feature(VcpCode(0xDF), None)
    };
    let write_only = Feature {
        access: Access::WriteOnly,
        ..feature(VcpCode::RESTORE_FACTORY_DEFAULTS, None)
    };

    assert_eq!(table.ensure_writable(), unsupported(VcpCode(0x73)));
    assert_eq!(read_only.ensure_writable(), unsupported(VcpCode(0xDF)));
    assert_eq!(write_only.ensure_writable(), Ok(()));
    assert_eq!(feature(VcpCode::BRIGHTNESS, None).ensure_writable(), Ok(()));
}

#[test]
fn only_write_only_features_are_unreadable() {
    let write_only = Feature {
        access: Access::WriteOnly,
        ..feature(VcpCode::RESTORE_FACTORY_DEFAULTS, None)
    };
    let read_only = Feature {
        access: Access::ReadOnly,
        ..feature(VcpCode(0xDF), None)
    };

    assert!(!write_only.is_readable());
    assert!(read_only.is_readable());
    assert!(feature(VcpCode::BRIGHTNESS, None).is_readable());
}

/// Without a list from the capabilities, a catalogued non-continuous code
/// is bounded by the catalog's value names, never by a maximum.
#[test]
fn unlisted_non_continuous_write_is_checked_against_the_catalog_values() {
    let preset = Feature {
        kind: FeatureKind::NonContinuous,
        ..feature(VcpCode::COLOR_PRESET, None)
    };
    let reset = Feature {
        kind: FeatureKind::NonContinuous,
        access: Access::WriteOnly,
        ..feature(VcpCode::RESTORE_FACTORY_DEFAULTS, None)
    };
    let not_allowed = |code, value| Err(DdcError::ValueNotAllowed { code, value });

    assert_eq!(preset.validate_write(0x05, None), Ok(()));
    assert_eq!(
        preset.validate_write(0x03, Some(0x0B)),
        not_allowed(VcpCode::COLOR_PRESET, 0x03)
    );
    assert_eq!(
        preset.validate_write(0x0105, None),
        not_allowed(VcpCode::COLOR_PRESET, 0x0105)
    );
    assert_eq!(reset.validate_write(0x01, None), Ok(()));
    assert_eq!(
        reset.validate_write(0x00, Some(0xFF)),
        not_allowed(VcpCode::RESTORE_FACTORY_DEFAULTS, 0x00)
    );
}

/// 0x1E and 0xCA take only their catalog values
/// (D-2026-09-26-full-osd-control-10), and 0xCA only with the button byte
/// (SH) at zero, since the value is checked as a single byte.
#[test]
fn auto_setup_and_osd_control_accept_only_their_catalog_values() {
    let non_continuous = |code| Feature {
        kind: FeatureKind::NonContinuous,
        ..feature(code, None)
    };
    let auto_setup = non_continuous(VcpCode(0x1E));
    let osd = non_continuous(VcpCode::OSD_LOCK);
    let not_allowed = |code, value| Err(DdcError::ValueNotAllowed { code, value });

    for value in [0x00, 0x01, 0x02] {
        assert_eq!(auto_setup.validate_write(value, None), Ok(()), "{value}");
    }
    assert_eq!(
        auto_setup.validate_write(0x03, Some(0xFF)),
        not_allowed(VcpCode(0x1E), 0x03)
    );
    for value in [0x01, 0x02] {
        assert_eq!(osd.validate_write(value, None), Ok(()), "{value}");
    }
    for value in [0x00, 0x03, 0x0102] {
        assert_eq!(
            osd.validate_write(value, Some(0xFFFF)),
            not_allowed(VcpCode::OSD_LOCK, value),
            "{value:#06X}"
        );
    }
}

/// A code the catalog gives no value names keeps needing a maximum, and
/// without one it is refused, never written blind.
#[test]
fn unlisted_non_continuous_write_without_catalog_values_needs_a_max() {
    let new_control_value = Feature {
        kind: FeatureKind::NonContinuous,
        ..feature(VcpCode(0x02), None)
    };

    assert_eq!(
        new_control_value.validate_write(1, None),
        Err(DdcError::UnsupportedFeature(VcpCode(0x02)))
    );
}
