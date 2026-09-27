//! `features [--probe]`: every declared code with its value, and with
//! `--probe` every catalogued code the capabilities leave out
//! (D-2026-09-26-full-osd-control-3).

use ddc_adapters::{BackendCall, FakeMonitor};
use ddc_cli::Exit;
use ddc_cli::fixture::FIXTURE_ID;
use ddc_core::domain::mccs_catalog::catalog_codes;
use ddc_core::domain::{DdcError, MonitorInfo, VcpCode};
use serde_json::{Value, json};

use crate::support::{backend_of, fake_cli, fixture_backend, id, monitor, run_with};

/// What the fixture's capabilities declare, in order.
const DECLARED: [u8; 5] = [0x10, 0x12, 0x14, 0x60, 0xD6];

fn reads_of(calls: &[BackendCall]) -> Vec<u8> {
    calls
        .iter()
        .filter_map(|call| match call {
            BackendCall::ReadVcp(_, code) => Some(code.0),
            _ => None,
        })
        .collect()
}

fn wrote_anything(calls: &[BackendCall]) -> bool {
    calls
        .iter()
        .any(|call| matches!(call, BackendCall::WriteVcp(..)))
}

/// The `features --json` entry of `code`.
fn entry(listed: &Value, code: u8) -> &Value {
    listed
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["code"] == code))
        .unwrap_or(&Value::Null)
}

/// The text line of `code`.
fn line_of(text: &str, code: VcpCode) -> &str {
    let prefix = format!("{code} ");
    text.lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_default()
}

#[test]
fn features_command_shows_declared_codes_by_default_and_adds_probed_codes_distinguishing_unsupported_from_unresponsive_under_probe()
 {
    let declared = run_with(&fixture_backend(), &["features"]);
    assert_eq!(declared.exit, Exit::Success, "{}", declared.err);
    assert!(declared.err.is_empty(), "{}", declared.err);
    assert_eq!(reads_of(&declared.calls), DECLARED);
    assert_eq!(declared.out.lines().count(), 1 + DECLARED.len());
    assert!(declared.out.starts_with("CODE  NAME"), "{}", declared.out);
    let preset = line_of(&declared.out, VcpCode::COLOR_PRESET);
    for cell in [
        "preset",
        "NC",
        "RW",
        "safe",
        "caps",
        "1/3 sRGB",
        "Select Color Preset",
    ] {
        assert!(preset.contains(cell), "{cell} missing from {preset:?}");
    }
    let input = line_of(&declared.out, VcpCode::INPUT_SOURCE);
    assert!(input.contains("15/17 DisplayPort-1"), "{input:?}");
    assert!(input.contains("dangerous"), "{input:?}");

    let probed = run_with(&fixture_backend(), &["features", "--probe", "--json"]);
    assert_eq!(probed.exit, Exit::Success, "{}", probed.err);
    assert!(!wrote_anything(&probed.calls), "{:?}", probed.calls);
    let listed: Value = serde_json::from_str(&probed.out).unwrap();
    assert_eq!(listed.as_array().map(Vec::len), Some(catalog_codes().len()));
    assert_eq!(
        *entry(&listed, 0x14),
        json!({
            "code": 20,
            "name": "preset",
            "description": "Select Color Preset",
            "kind": "NC",
            "access": "RW",
            "risk": "safe",
            "declared_in_capabilities": true,
            "probe_status": "ok",
            "current": 1,
            "max": 3,
            "value_name": "sRGB",
            "interpreted": null
        })
    );
    assert_eq!(
        *entry(&listed, 0x62),
        json!({
            "code": 98,
            "name": "volume",
            "description": "Audio Speaker Volume",
            "kind": "C",
            "access": "RW",
            "risk": "safe",
            "declared_in_capabilities": false,
            "probe_status": "ok",
            "current": 30,
            "max": 100,
            "value_name": null,
            "interpreted": null
        })
    );
    let refused = entry(&listed, 0x87);
    assert_eq!(refused["probe_status"], "unsupported");
    assert_eq!(refused["declared_in_capabilities"], false);
    assert_eq!(
        (&refused["current"], &refused["max"]),
        (&Value::Null, &Value::Null)
    );
    for (code, why) in [(0xDF, "timeout"), (0x7E, "transport")] {
        let mute = entry(&listed, code);
        assert_eq!(mute["probe_status"], "unresponsive", "{code:#04X} ({why})");
        assert_eq!(mute["current"], Value::Null, "{code:#04X}");
    }
    assert_eq!(entry(&listed, 0x7E)["risk"], "dangerous");

    let text = run_with(&fixture_backend(), &["features", "--probe"]);
    assert_eq!(text.exit, Exit::Success);
    assert!(
        line_of(&text.out, VcpCode::SHARPNESS).contains("not supported by this monitor"),
        "{}",
        text.out
    );
    for code in [VcpCode(0x7E), VcpCode::VCP_VERSION] {
        let mute = line_of(&text.out, code);
        assert!(mute.contains("not responding"), "{mute:?}");
        assert!(mute.contains("probe"), "{mute:?}");
    }

    let binary = fake_cli().args(["features"]).output().unwrap();
    assert_eq!(binary.status.code(), Some(0), "{binary:?}");
    assert_eq!(String::from_utf8_lossy(&binary.stdout), declared.out);
    let binary = fake_cli()
        .args(["features", "--probe", "--json"])
        .output()
        .unwrap();
    assert_eq!(binary.status.code(), Some(0), "{binary:?}");
    let from_binary: Value = serde_json::from_slice(&binary.stdout).unwrap();
    assert_eq!(from_binary, listed);
}

/// Without capabilities no code counts as declared: `features` warns and
/// lists nothing, and `--probe` reads the whole catalog.
#[test]
fn features_with_unreadable_capabilities_warns_and_lists_only_probed_codes() {
    let unreadable = || {
        backend_of([FakeMonitor::new(MonitorInfo {
            id: id("M-1"),
            manufacturer: None,
            model: None,
            serial: None,
        })
        .with_value(VcpCode::BRIGHTNESS, 80, 100)])
    };

    let plain = run_with(&unreadable(), &["features"]);
    let probed = run_with(&unreadable(), &["features", "--probe", "--json"]);

    assert_eq!(plain.exit, Exit::Success);
    assert!(plain.out.is_empty(), "{}", plain.out);
    assert!(
        plain
            .err
            .starts_with("warning: capabilities could not be read"),
        "{}",
        plain.err
    );
    assert_eq!(reads_of(&plain.calls), Vec::<u8>::new());
    assert_eq!(probed.exit, Exit::Success);
    assert!(probed.err.contains("--probe"), "{}", probed.err);
    let listed: Value = serde_json::from_str(&probed.out).unwrap();
    let rows = listed.as_array().unwrap();
    assert_eq!(rows.len(), catalog_codes().len());
    assert!(
        rows.iter()
            .all(|row| row["declared_in_capabilities"] == false)
    );
    assert_eq!(entry(&listed, 0x10)["current"], 80);
}

#[test]
fn a_monitor_missing_on_any_row_ends_features_with_exit_3() {
    let vanished =
        monitor("M-1").with_vcp_failure(VcpCode::CONTRAST, DdcError::MonitorNotFound(id("M-1")));
    let probe_vanished = monitor("M-1")
        .with_vcp_failure(VcpCode::AUDIO_VOLUME, DdcError::MonitorNotFound(id("M-1")));

    let declared = run_with(&backend_of([vanished]), &["features"]);
    let probed = run_with(&backend_of([probe_vanished]), &["features", "--probe"]);

    for outcome in [declared, probed] {
        assert_eq!(outcome.exit, Exit::Monitor, "{}", outcome.err);
        assert!(outcome.out.is_empty(), "{}", outcome.out);
        assert!(outcome.err.contains("not reachable"), "{}", outcome.err);
    }
}

#[test]
fn features_only_reads() {
    let listed = run_with(&fixture_backend(), &["features", "--probe"]);

    assert!(!wrote_anything(&listed.calls));
    assert_eq!(
        listed.calls[..2],
        [
            BackendCall::Enumerate,
            BackendCall::ReadCapabilities(id(FIXTURE_ID)),
        ]
    );
}
