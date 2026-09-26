use super::{Risk, risk_for_code};
use crate::domain::VcpCode;

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
