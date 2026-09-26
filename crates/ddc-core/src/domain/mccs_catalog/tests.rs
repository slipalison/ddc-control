use std::collections::BTreeSet;

use super::{
    Interpretation, catalog, catalog_codes, catalog_entry, code_for_alias, hertz, interpret,
    same_value_name, value_for_name, value_name, version_from_u16,
};
use crate::domain::Access::{ReadOnly as RO, ReadWrite as RW, WriteOnly as WO};
use crate::domain::FeatureKind::{Continuous as C, NonContinuous as NC};
use crate::domain::Risk::{Dangerous, Safe};
use crate::domain::{Access, Capabilities, FeatureKind, Risk, VcpCode, risk_for_code};

/// Capabilities string of the dev monitor "RTK QHD HDR", verbatim.
const RTK_QHD_HDR_CAPS: &str =
    include_str!("../../../tests/fixtures/rtk_qhd_hdr_caps.txt").trim_ascii_end();

/// Codes the dev monitor answers without declaring them (ddcutil, 2026-09-25).
const ANSWERED_UNDECLARED: [u8; 9] = [0x1E, 0x20, 0x30, 0x62, 0x6C, 0x6E, 0x70, 0x7E, 0xC9];
/// Manufacturer-specific codes the full 0x00..=0xFF scan found answering
/// (D-2026-09-26-full-osd-control-7).
const SCANNED_MANUFACTURER: [u8; 2] = [0xE6, 0xF1];

/// The locked kind, access and risk of every catalogued code
/// (D-2026-09-26-full-osd-control-1, -7).
const LOCKED: [(u8, FeatureKind, Access, Risk); 39] = [
    (0x02, NC, RW, Safe),
    (0x04, NC, WO, Dangerous),
    (0x05, NC, WO, Dangerous),
    (0x06, NC, WO, Dangerous),
    (0x08, NC, WO, Dangerous),
    (0x0B, C, RO, Safe),
    (0x0C, C, RW, Safe),
    (0x10, C, RW, Safe),
    (0x12, C, RW, Safe),
    (0x14, NC, RW, Safe),
    (0x16, C, RW, Safe),
    (0x18, C, RW, Safe),
    (0x1A, C, RW, Safe),
    (0x1E, NC, RW, Dangerous),
    (0x20, C, RW, Dangerous),
    (0x30, C, RW, Dangerous),
    (0x52, C, RO, Safe),
    (0x60, NC, RW, Dangerous),
    (0x62, C, RW, Safe),
    (0x6C, C, RW, Safe),
    (0x6E, C, RW, Safe),
    (0x70, C, RW, Safe),
    (0x7E, C, RW, Dangerous),
    (0x87, C, RW, Safe),
    (0xAC, C, RO, Safe),
    (0xAE, C, RO, Safe),
    (0xB2, NC, RO, Safe),
    (0xB6, NC, RO, Safe),
    (0xC6, C, RO, Safe),
    (0xC8, NC, RO, Safe),
    (0xC9, C, RO, Safe),
    (0xCA, NC, RW, Dangerous),
    (0xCC, NC, RW, Safe),
    (0xD6, NC, RW, Dangerous),
    (0xDF, C, RO, Safe),
    (0xE6, C, RW, Dangerous),
    (0xF1, C, RW, Dangerous),
    (0xFD, C, RO, Safe),
    (0xFF, C, RO, Safe),
];

fn raw_codes(codes: &[VcpCode]) -> Vec<u8> {
    codes.iter().map(|code| code.0).collect()
}

#[test]
fn mccs_catalog_covers_every_observed_code_with_the_locked_kind_access_and_risk_including_the_read_only_never_dangerous_invariant()
 {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();
    let observed: BTreeSet<u8> = caps
        .vcp
        .keys()
        .copied()
        .chain(ANSWERED_UNDECLARED)
        .chain(SCANNED_MANUFACTURER)
        .collect();

    assert_eq!(catalog_codes().len(), 39);
    assert_eq!(
        raw_codes(catalog_codes()),
        observed.into_iter().collect::<Vec<_>>()
    );
    for (raw, kind, access, risk) in LOCKED {
        let entry = catalog_entry(VcpCode(raw)).unwrap();
        assert_eq!(
            (entry.kind, entry.access, entry.risk),
            (kind, access, risk),
            "{}",
            VcpCode(raw)
        );
    }
    for entry in catalog() {
        assert!(
            !(entry.access == Access::ReadOnly && entry.risk == Risk::Dangerous),
            "{} is read-only and dangerous",
            entry.code
        );
    }
    for raw in [0x1E, 0x20, 0x30, 0x7E] {
        assert_eq!(risk_for_code(VcpCode(raw)), Risk::Dangerous, "{raw:#04X}");
    }
}

/// The one value of every factory reset (D-2026-09-26-full-osd-control-8).
const RESET: &[(u8, &str)] = &[(0x01, "Reset")];

/// Every value list of the catalog, whole, by code. The catalog's list is
/// the one a write is checked against when the capabilities list none, so a
/// value added here is a value `set` accepts (D-2026-09-26-full-osd-control-1).
const VALUE_LISTS: [(u8, &[(u8, &str)]); 8] = [
    (0x04, RESET),
    (0x05, RESET),
    (0x06, RESET),
    (0x08, RESET),
    (
        0x14,
        &[
            (0x01, "sRGB"),
            (0x02, "Display Native"),
            (0x04, "5000 K"),
            (0x05, "6500 K"),
            (0x06, "7500 K"),
            (0x08, "9300 K"),
            (0x0B, "User 1"),
        ],
    ),
    (
        0x60,
        &[
            (0x01, "VGA-1"),
            (0x03, "DVI-1"),
            (0x04, "DVI-2"),
            (0x0F, "DisplayPort-1"),
            (0x10, "DisplayPort-2"),
            (0x11, "HDMI-1"),
            (0x12, "HDMI-2"),
        ],
    ),
    (
        0xCC,
        &[
            (0x01, "Chinese (traditional)"),
            (0x02, "English"),
            (0x03, "French"),
            (0x04, "German"),
            (0x06, "Japanese"),
            (0x0A, "Spanish"),
            (0x0D, "Chinese (simplified)"),
        ],
    ),
    (
        0xD6,
        &[
            (0x01, "On"),
            (0x04, "Off (DPM)"),
            (0x05, "Off (write-only)"),
        ],
    ),
];

/// Pins every list by equality — codes and names, nothing added, dropped or
/// renamed — and which codes have one at all.
#[test]
fn mccs_catalog_value_names_match_the_declared_lists_for_preset_input_language_power_mode_and_the_factory_reset_commands()
 {
    for (raw, values) in VALUE_LISTS {
        let code = VcpCode(raw);
        assert_eq!(catalog_entry(code).unwrap().values, values, "{code}");
        for (byte, name) in values {
            assert_eq!(value_name(code, *byte), Some(*name), "{code} = {byte:#04X}");
        }
    }
    let named: Vec<u8> = catalog()
        .iter()
        .filter(|entry| !entry.values.is_empty())
        .map(|entry| entry.code.0)
        .collect();
    assert_eq!(named, VALUE_LISTS.map(|(raw, _)| raw));
    for reset in [0x04, 0x05, 0x06, 0x08] {
        assert_eq!(value_name(VcpCode(reset), 0x00), None);
    }
    assert_eq!(value_name(VcpCode::COLOR_PRESET, 0x03), None);
    assert_eq!(value_name(VcpCode::BRIGHTNESS, 0x01), None);
    assert_eq!(value_name(VcpCode(0x8D), 0x01), None);
}

#[test]
fn hertz_and_version_from_u16_convert_the_raw_frequency_and_version_values_reported_by_the_rtk_monitor()
 {
    assert!((hertz(14400) - 144.0).abs() < f64::EPSILON);
    assert_eq!(format!("{:.2} Hz", hertz(14400)), "144.00 Hz");
    assert!((hertz(5994) - 59.94).abs() < 1e-9);
    assert_eq!(version_from_u16(0x0202), (2, 2));
    assert_eq!(version_from_u16(0x0001), (0, 1));
}

#[test]
fn interpret_names_frequencies_and_versions_and_leaves_other_codes_raw() {
    let shown = |code: VcpCode, raw: u16| interpret(code, raw).map(|it| it.to_string());

    assert_eq!(
        interpret(VcpCode::VERTICAL_FREQUENCY, 14400),
        Some(Interpretation::Hertz(144.0))
    );
    assert_eq!(
        shown(VcpCode::VERTICAL_FREQUENCY, 14400).as_deref(),
        Some("144.00 Hz")
    );
    assert_eq!(
        interpret(VcpCode::VCP_VERSION, 0x0202),
        Some(Interpretation::Version(2, 2))
    );
    assert_eq!(
        shown(VcpCode::FIRMWARE_LEVEL, 0x0001).as_deref(),
        Some("0.1")
    );
    assert_eq!(
        interpret(VcpCode::COLOR_PRESET, 0x01),
        Some(Interpretation::Named("sRGB"))
    );
    assert_eq!(
        shown(VcpCode::INPUT_SOURCE, 0x0F).as_deref(),
        Some("DisplayPort-1")
    );
    assert_eq!(
        interpret(VcpCode::INPUT_SOURCE, 0x010F),
        Some(Interpretation::Named("DisplayPort-1")),
        "the value sits in the low byte"
    );
    assert_eq!(interpret(VcpCode(0xAC), 3), None, "0xAC stays raw");
    assert_eq!(interpret(VcpCode::BRIGHTNESS, 1), None);
    assert_eq!(interpret(VcpCode::COLOR_PRESET, 0x03), None);
    assert_eq!(interpret(VcpCode(0x8D), 1), None);
}

#[test]
fn catalog_is_sorted_by_code_without_repeats_and_catalog_codes_follow_it() {
    let codes = raw_codes(catalog_codes());

    assert!(codes.windows(2).all(|pair| pair[0] < pair[1]), "{codes:?}");
    assert_eq!(
        catalog()
            .iter()
            .map(|entry| entry.code.0)
            .collect::<Vec<_>>(),
        codes
    );
    for code in catalog_codes() {
        assert_eq!(catalog_entry(*code).map(|entry| entry.code), Some(*code));
    }
    assert_eq!(catalog_entry(VcpCode(0x00)), None);
    assert_eq!(catalog_entry(VcpCode(0x8D)), None);
}

#[test]
fn aliases_are_unique_lowercase_keywords_and_keep_the_six_original_shortcuts() {
    let aliases: Vec<&str> = catalog().iter().filter_map(|entry| entry.alias).collect();
    let unique: BTreeSet<&str> = aliases.iter().copied().collect();

    assert_eq!(unique.len(), aliases.len(), "{aliases:?}");
    for alias in &aliases {
        let keyword = alias
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        assert!(keyword, "{alias:?}");
        assert!(alias.parse::<u32>().is_err(), "{alias:?} reads as a number");
    }
    let original = [
        ("brightness", VcpCode::BRIGHTNESS),
        ("contrast", VcpCode::CONTRAST),
        ("input", VcpCode::INPUT_SOURCE),
        ("preset", VcpCode::COLOR_PRESET),
        ("volume", VcpCode::AUDIO_VOLUME),
        ("power", VcpCode::POWER_MODE),
    ];
    for (alias, code) in original {
        assert_eq!(code_for_alias(alias), Some(code), "{alias}");
        assert_eq!(code_for_alias(&alias.to_uppercase()), Some(code), "{alias}");
    }
    for reset in [0x04, 0x05, 0x06, 0x08, 0x52, 0xC6, 0xE6, 0xF1, 0xFD, 0xFF] {
        assert_eq!(catalog_entry(VcpCode(reset)).unwrap().alias, None);
    }
    assert_eq!(
        code_for_alias("v-frequency"),
        Some(VcpCode::VERTICAL_FREQUENCY)
    );
    assert_eq!(code_for_alias("nonsense"), None);
    assert_eq!(code_for_alias(""), None);
}

#[test]
fn value_names_are_unique_per_code_once_normalized() {
    for entry in catalog() {
        for (index, (byte, name)) in entry.values.iter().enumerate() {
            for (other_byte, other) in &entry.values[index + 1..] {
                assert_ne!(byte, other_byte, "{} repeats a byte", entry.code);
                assert!(
                    !same_value_name(name, other),
                    "{}: {name:?} and {other:?} normalize alike",
                    entry.code
                );
            }
        }
    }
}

#[test]
fn value_for_name_ignores_case_and_every_non_alphanumeric_character() {
    let preset = VcpCode::COLOR_PRESET;

    assert_eq!(value_for_name(preset, "sRGB"), Some(0x01));
    assert_eq!(value_for_name(preset, "SRGB"), Some(0x01));
    assert_eq!(value_for_name(preset, "display-native"), Some(0x02));
    assert_eq!(value_for_name(preset, "6500k"), Some(0x05));
    assert_eq!(value_for_name(preset, "6500 K"), Some(0x05));
    assert_eq!(value_for_name(preset, "user_1"), Some(0x0B));
    assert_eq!(
        value_for_name(VcpCode::INPUT_SOURCE, "displayport1"),
        Some(0x0F)
    );
    assert_eq!(value_for_name(VcpCode::OSD_LANGUAGE, "English"), Some(0x02));
    assert_eq!(
        value_for_name(VcpCode::RESTORE_FACTORY_DEFAULTS, "reset"),
        Some(0x01)
    );
    assert_eq!(value_for_name(preset, "nonsense"), None);
    assert_eq!(value_for_name(preset, ""), None);
    assert_eq!(value_for_name(VcpCode::BRIGHTNESS, "srgb"), None);
    assert_eq!(value_for_name(VcpCode(0x8D), "srgb"), None);
}

/// Every one of the 256 codes: a safe code is catalogued, an uncatalogued
/// one is dangerous, and in the manufacturer-specific range only the
/// read-only 0xFD and 0xFF are not dangerous — and those can never be
/// written.
#[test]
fn every_code_is_safe_only_when_catalogued_and_the_manufacturer_range_is_dangerous_or_read_only() {
    for raw in 0..=u8::MAX {
        let code = VcpCode(raw);
        let entry = catalog_entry(code);
        let expected = entry.map_or(Risk::Dangerous, |entry| entry.risk);
        assert_eq!(risk_for_code(code), expected, "{code}");
        if code >= VcpCode::MANUFACTURER_SPECIFIC_START && risk_for_code(code) == Risk::Safe {
            assert!(matches!(raw, 0xFD | 0xFF), "{code}");
            assert_eq!(entry.map(|entry| entry.access), Some(Access::ReadOnly));
        }
    }
}
