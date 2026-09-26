use std::collections::BTreeMap;

use super::Capabilities;
use crate::domain::{Access, DdcError, FeatureKind, Risk, VcpCode};

/// Capabilities string of the dev monitor "RTK QHD HDR" (Linux, /dev/i2c-5,
/// ddcutil, probed 2026-09-25), verbatim.
const RTK_QHD_HDR_CAPS: &str = "(prot(monitor)type(LCD)model(RTK)cmds(01 02 03 07 0C E3 F3)vcp(02 04 05 06 08 0B 0C 10 12 14(01 02 04 05 06 08 0B) 16 18 1A 52 60(01 03 04 0F 10 11 12) 87 AC AE B2 B6 C6 C8 CA CC(01 02 03 04 06 0A 0D) D6(01 04 05) DF FD FF)mswhql(1)asset_eep(40)mccs_ver(2.2))";

fn tags(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

fn is_transport_error(result: Result<Capabilities, DdcError>) -> bool {
    matches!(result, Err(DdcError::Transport(_)))
}

#[test]
fn parses_real_rtk_caps_string() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();

    assert_eq!(caps.protocol.as_deref(), Some("monitor"));
    assert_eq!(caps.monitor_type.as_deref(), Some("LCD"));
    assert_eq!(caps.model.as_deref(), Some("RTK"));
    assert_eq!(caps.commands, [0x01, 0x02, 0x03, 0x07, 0x0C, 0xE3, 0xF3]);
    let codes: Vec<u8> = caps.vcp.keys().copied().collect();
    assert_eq!(
        codes,
        [
            0x02, 0x04, 0x05, 0x06, 0x08, 0x0B, 0x0C, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x52,
            0x60, 0x87, 0xAC, 0xAE, 0xB2, 0xB6, 0xC6, 0xC8, 0xCA, 0xCC, 0xD6, 0xDF, 0xFD, 0xFF,
        ]
    );
    assert_eq!(
        caps.vcp[&0x14],
        Some(vec![0x01, 0x02, 0x04, 0x05, 0x06, 0x08, 0x0B])
    );
    assert_eq!(
        caps.vcp[&0x60],
        Some(vec![0x01, 0x03, 0x04, 0x0F, 0x10, 0x11, 0x12])
    );
    assert_eq!(
        caps.vcp[&0xCC],
        Some(vec![0x01, 0x02, 0x03, 0x04, 0x06, 0x0A, 0x0D])
    );
    assert_eq!(caps.vcp[&0xD6], Some(vec![0x01, 0x04, 0x05]));
    assert_eq!(caps.vcp[&0x10], None);
    assert!(!caps.vcp.contains_key(&0x62));
    assert_eq!(caps.mccs_version, Some((2, 2)));
    assert_eq!(
        caps.unknown_tags,
        tags(&[("mswhql", "1"), ("asset_eep", "40")])
    );
}

#[test]
fn tolerates_missing_spaces_and_unknown_tags() {
    let raw = "(prot(monitor)TYPE(lcd)model(X1)cmds(01020c)vcp(0210 12 14(0102)60(0f11)DF)\
               vcpname(10(Brightness))mswhql(1)mccs_ver(2.1))";

    let caps = Capabilities::parse(raw).unwrap();

    assert_eq!(caps.monitor_type.as_deref(), Some("lcd"));
    assert_eq!(caps.model.as_deref(), Some("X1"));
    assert_eq!(caps.commands, [0x01, 0x02, 0x0C]);
    let codes: Vec<u8> = caps.vcp.keys().copied().collect();
    assert_eq!(codes, [0x02, 0x10, 0x12, 0x14, 0x60, 0xDF]);
    assert_eq!(caps.vcp[&0x14], Some(vec![0x01, 0x02]));
    assert_eq!(caps.vcp[&0x60], Some(vec![0x0F, 0x11]));
    assert_eq!(caps.vcp[&0xDF], None);
    assert_eq!(caps.mccs_version, Some((2, 1)));
    assert_eq!(
        caps.unknown_tags,
        tags(&[("vcpname", "10(Brightness)"), ("mswhql", "1")])
    );
}

#[test]
fn unbalanced_parentheses_are_a_transport_error() {
    assert!(is_transport_error(Capabilities::parse(
        "(prot(monitor)vcp(10 12"
    )));
    assert!(is_transport_error(Capabilities::parse("(prot(monitor)")));
    assert!(is_transport_error(Capabilities::parse(
        "prot(monitor)vcp(10"
    )));
}

#[test]
fn garbage_after_the_closing_parenthesis_is_ignored() {
    let caps = Capabilities::parse(" (vcp(10)model(A))\0\0junk(vcp(12)").unwrap();

    assert_eq!(caps.vcp.keys().copied().collect::<Vec<_>>(), [0x10]);
    assert_eq!(caps.model.as_deref(), Some("A"));
}

#[test]
fn invalid_hex_tokens_and_junk_between_tags_are_skipped() {
    let caps = Capabilities::parse("(vcp(10 zz 12 --) ?? type (LCD)mccs_ver(two))").unwrap();

    assert_eq!(caps.vcp.keys().copied().collect::<Vec<_>>(), [0x10, 0x12]);
    assert_eq!(caps.monitor_type.as_deref(), Some("LCD"));
    assert_eq!(caps.mccs_version, None);
}

#[test]
fn accepts_strings_without_outer_parentheses() {
    let caps = Capabilities::parse("prot(monitor)vcp(10 60(01 0F))").unwrap();

    assert_eq!(caps.protocol.as_deref(), Some("monitor"));
    assert_eq!(caps.vcp[&0x60], Some(vec![0x01, 0x0F]));
}

#[test]
fn empty_or_unnamed_input_yields_empty_capabilities() {
    assert_eq!(Capabilities::parse("").unwrap(), Capabilities::default());
    assert_eq!(
        Capabilities::parse("((prot(monitor)))").unwrap(),
        Capabilities::default()
    );
}

#[test]
fn declares_only_codes_listed_in_vcp() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();

    assert!(caps.declares(VcpCode::BRIGHTNESS));
    assert!(caps.declares(VcpCode::POWER_MODE));
    assert!(!caps.declares(VcpCode::AUDIO_VOLUME));
}

#[test]
fn feature_of_bare_code_is_continuous() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();

    let feature = caps.feature(VcpCode::BRIGHTNESS);

    assert_eq!(feature.code, VcpCode::BRIGHTNESS);
    assert_eq!(feature.kind, FeatureKind::Continuous);
    assert_eq!(feature.access, Access::ReadWrite);
    assert_eq!(feature.risk, Risk::Safe);
    assert_eq!(feature.allowed_values, None);
}

#[test]
fn feature_with_value_list_is_non_continuous() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();

    let preset = caps.feature(VcpCode::COLOR_PRESET);
    let input = caps.feature(VcpCode::INPUT_SOURCE);

    assert_eq!(preset.kind, FeatureKind::NonContinuous);
    assert_eq!(preset.risk, Risk::Safe);
    assert_eq!(
        preset.allowed_values,
        Some(vec![0x01, 0x02, 0x04, 0x05, 0x06, 0x08, 0x0B])
    );
    assert_eq!(input.kind, FeatureKind::NonContinuous);
    assert_eq!(input.risk, Risk::Dangerous);
}

#[test]
fn feature_of_undeclared_code_is_continuous_with_seed_risk() {
    let caps = Capabilities::parse(RTK_QHD_HDR_CAPS).unwrap();

    let volume = caps.feature(VcpCode::AUDIO_VOLUME);

    assert_eq!(volume.kind, FeatureKind::Continuous);
    assert_eq!(volume.risk, Risk::Safe);
    assert_eq!(volume.allowed_values, None);
}
