use clap::Parser;
use clap::error::ErrorKind;
use ddc_core::domain::VcpCode;
use ddc_core::domain::mccs_catalog::catalog;

use super::{Cli, Command, ResetTarget, ValueArg, alias_of, code_label};

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("ddc-cli").chain(args.iter().copied()))
}

fn get_code(vcp: &str) -> Result<VcpCode, clap::Error> {
    parse(&["get", vcp]).map(|cli| {
        match cli.command {
            Command::Get { vcp } => Some(vcp),
            _ => None,
        }
        .expect("parsed as get")
    })
}

fn set_value(value: &str) -> Result<ValueArg, clap::Error> {
    parse(&["set", "brightness", value]).map(|cli| {
        match cli.command {
            Command::Set { value, .. } => Some(value),
            _ => None,
        }
        .expect("parsed as set")
    })
}

#[test]
fn vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names() {
    for text in [
        "16",
        "0x10",
        "0X10",
        "0x010",
        "brightness",
        "BRIGHTNESS",
        "Brightness",
    ] {
        assert_eq!(get_code(text).unwrap(), VcpCode::BRIGHTNESS, "{text:?}");
    }
    let expected = [
        ("brightness", VcpCode::BRIGHTNESS),
        ("contrast", VcpCode::CONTRAST),
        ("input", VcpCode::INPUT_SOURCE),
        ("preset", VcpCode::COLOR_PRESET),
        ("volume", VcpCode::AUDIO_VOLUME),
        ("power", VcpCode::POWER_MODE),
    ];
    for (name, code) in expected {
        assert_eq!(get_code(name).unwrap(), code, "{name}");
        assert_eq!(get_code(&name.to_uppercase()).unwrap(), code, "{name}");
    }
    assert_eq!(get_code("0").unwrap(), VcpCode(0));
    assert_eq!(get_code("255").unwrap(), VcpCode(0xFF));
    assert_eq!(get_code("0xff").unwrap(), VcpCode(0xFF));

    for (alias, code) in [
        ("sharpness", VcpCode::SHARPNESS),
        ("osd-language", VcpCode::OSD_LANGUAGE),
        ("V-Frequency", VcpCode::VERTICAL_FREQUENCY),
        ("red-black-level", VcpCode(0x6C)),
    ] {
        assert_eq!(get_code(alias).unwrap(), code, "{alias}");
    }

    for text in [
        "foo", "0x", "0x1G", "256", "0x100", "-1", "", "+16", " 16", "1e1",
    ] {
        let error = get_code(text).unwrap_err();
        assert_eq!(error.exit_code(), 2, "{text:?}");
    }
}

#[test]
fn an_unknown_vcp_name_lists_the_valid_ones() {
    let message = get_code("foo").unwrap_err().to_string();

    assert!(message.contains("0 to 255"), "{message}");
    for alias in catalog().iter().filter_map(|entry| entry.alias) {
        assert!(message.contains(alias), "{alias} missing from {message}");
    }
}

/// A name is accepted when the catalog names a value of any feature so;
/// whether it fits the feature being set is checked when the command runs
/// (D-2026-09-26-full-osd-control-3).
#[test]
fn value_argument_takes_numbers_up_to_u16_max_or_a_catalog_value_name() {
    assert_eq!(set_value("0").unwrap(), ValueArg::Number(0));
    assert_eq!(set_value("65535").unwrap(), ValueArg::Number(u16::MAX));
    assert_eq!(set_value("0xFFFF").unwrap(), ValueArg::Number(u16::MAX));
    assert_eq!(set_value("0x11").unwrap(), ValueArg::Number(0x11));
    for name in [
        "srgb",
        "SRGB",
        "6500k",
        "display-native",
        "HDMI-1",
        "english",
    ] {
        assert_eq!(
            set_value(name).unwrap(),
            ValueArg::Name(name.to_owned()),
            "{name}"
        );
    }

    for text in [
        "65536",
        "0x10000",
        "brightness",
        "",
        "0x",
        "99999999999",
        "nonsense",
    ] {
        let error = set_value(text).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ValueValidation, "{text:?}");
        assert_eq!(error.exit_code(), 2, "{text:?}");
    }
}

#[test]
fn a_value_name_resolves_only_against_its_own_feature() {
    let srgb = ValueArg::Name("SRGB".to_owned());

    assert_eq!(srgb.resolve(VcpCode::COLOR_PRESET), Ok(0x01));
    assert_eq!(
        ValueArg::Name("6500 K".to_owned()).resolve(VcpCode::COLOR_PRESET),
        Ok(0x05)
    );
    assert_eq!(
        ValueArg::Number(0x0105).resolve(VcpCode::COLOR_PRESET),
        Ok(0x0105)
    );
    let other_feature = srgb.resolve(VcpCode::INPUT_SOURCE).unwrap_err();
    assert!(other_feature.contains("0x60 input"), "{other_feature}");
    assert!(other_feature.contains("'SRGB'"), "{other_feature}");
    assert!(other_feature.contains("DisplayPort-1"), "{other_feature}");
    let unnamed = srgb.resolve(VcpCode::BRIGHTNESS).unwrap_err();
    assert!(unnamed.contains("0x10 brightness"), "{unnamed}");
    assert!(unnamed.contains("give it a number"), "{unnamed}");
}

#[test]
fn reset_targets_map_to_the_four_factory_reset_codes_and_write_their_reset_value() {
    let targets = [
        (
            "factory",
            ResetTarget::Factory,
            VcpCode::RESTORE_FACTORY_DEFAULTS,
        ),
        (
            "brightness-contrast",
            ResetTarget::BrightnessContrast,
            VcpCode::RESTORE_FACTORY_LUMINANCE_CONTRAST,
        ),
        (
            "geometry",
            ResetTarget::Geometry,
            VcpCode::RESTORE_FACTORY_GEOMETRY,
        ),
        ("color", ResetTarget::Color, VcpCode::RESTORE_FACTORY_COLOR),
    ];
    for (name, target, code) in targets {
        let cli = parse(&["reset", name]).unwrap();
        assert_eq!(cli.command, Command::Reset { target, yes: false }, "{name}");
        assert_eq!(target.code(), code, "{name}");
        assert_eq!(target.value().resolve(code), Ok(0x01), "{name}");
    }
    for unknown in ["everything", "0x04", ""] {
        let error = parse(&["reset", unknown]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidValue, "{unknown:?}");
        assert_eq!(error.exit_code(), 2, "{unknown:?}");
    }
}

#[test]
fn yes_is_only_accepted_by_set_and_reset() {
    let cli = parse(&["set", "input", "0x11", "--yes"]).unwrap();
    assert_eq!(
        cli.command,
        Command::Set {
            vcp: VcpCode::INPUT_SOURCE,
            value: ValueArg::Number(0x11),
            yes: true,
        }
    );
    let reset = parse(&["reset", "color", "-y"]).unwrap();
    assert_eq!(
        reset.command,
        Command::Reset {
            target: ResetTarget::Color,
            yes: true,
        }
    );
    let short = parse(&["set", "power", "4", "-y"]).unwrap();
    assert!(matches!(short.command, Command::Set { yes: true, .. }));
    let unconfirmed = parse(&["set", "power", "4"]).unwrap();
    assert!(matches!(
        unconfirmed.command,
        Command::Set { yes: false, .. }
    ));

    for args in [
        ["get", "brightness", "--yes"],
        ["caps", "--yes", "--refresh"],
    ] {
        let error = parse(&args).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnknownArgument, "{args:?}");
    }
}

#[test]
fn global_options_go_before_or_after_the_subcommand() {
    let before = parse(&["--monitor", "RTK", "--json", "get", "volume"]).unwrap();
    let after = parse(&["get", "volume", "-m", "RTK", "--json"]).unwrap();

    for cli in [before, after] {
        assert_eq!(cli.monitor.as_deref(), Some("RTK"));
        assert!(cli.json);
        assert!(!cli.fake);
        assert_eq!(
            cli.command,
            Command::Get {
                vcp: VcpCode::AUDIO_VOLUME
            }
        );
    }
    let hidden = parse(&["list", "--fake"]).unwrap();
    assert!(hidden.fake);
    assert_eq!(hidden.monitor, None);
}

#[test]
fn caps_refresh_is_optional() {
    let cached = parse(&["caps"]).unwrap();
    let refreshed = parse(&["caps", "--refresh"]).unwrap();

    assert_eq!(cached.command, Command::Caps { refresh: false });
    assert_eq!(refreshed.command, Command::Caps { refresh: true });
}

#[test]
fn a_subcommand_is_required() {
    assert!(parse(&[]).is_err());
    assert!(parse(&["--json"]).is_err());
}

#[test]
fn catalog_aliases_round_trip_and_codes_without_one_are_labelled_by_number() {
    for entry in catalog() {
        let Some(alias) = entry.alias else { continue };
        assert_eq!(alias_of(entry.code), Some(alias));
        assert_eq!(get_code(alias).unwrap(), entry.code);
        assert_eq!(code_label(entry.code), format!("{} {alias}", entry.code));
    }
    assert_eq!(alias_of(VcpCode::SHARPNESS), Some("sharpness"));
    assert_eq!(alias_of(VcpCode::RESTORE_FACTORY_DEFAULTS), None);
    assert_eq!(alias_of(VcpCode(0x8D)), None);
    assert_eq!(code_label(VcpCode(0x8D)), "0x8D");
}
