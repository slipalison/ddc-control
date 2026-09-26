use clap::Parser;
use clap::error::ErrorKind;
use ddc_core::domain::VcpCode;

use super::{Cli, Command, SHORTCUTS, shortcut_name};

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

fn set_value(value: &str) -> Result<u16, clap::Error> {
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
    for (name, _) in SHORTCUTS {
        assert!(message.contains(name), "{message}");
    }
}

#[test]
fn value_argument_takes_numbers_up_to_u16_max_only() {
    assert_eq!(set_value("0").unwrap(), 0);
    assert_eq!(set_value("65535").unwrap(), u16::MAX);
    assert_eq!(set_value("0xFFFF").unwrap(), u16::MAX);
    assert_eq!(set_value("0x11").unwrap(), 0x11);

    for text in ["65536", "0x10000", "brightness", "", "0x", "99999999999"] {
        let error = set_value(text).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ValueValidation, "{text:?}");
        assert_eq!(error.exit_code(), 2, "{text:?}");
    }
}

#[test]
fn yes_is_only_accepted_by_set() {
    let cli = parse(&["set", "input", "0x11", "--yes"]).unwrap();
    assert_eq!(
        cli.command,
        Command::Set {
            vcp: VcpCode::INPUT_SOURCE,
            value: 0x11,
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
fn shortcut_names_round_trip_and_unnamed_codes_have_none() {
    for (name, code) in SHORTCUTS {
        assert_eq!(shortcut_name(code), Some(name));
        assert_eq!(get_code(name).unwrap(), code);
    }
    assert_eq!(shortcut_name(VcpCode::SHARPNESS), None);
}
