//! `list`, `caps` and `get`, and the hidden `--fake` switch
//! (D-2026-09-25-cli-4).

use clap::Parser;
use ddc_adapters::BackendCall;
use ddc_cli::fixture::FIXTURE_ID;
use ddc_cli::{Cli, Exit, report_startup_failure};
use ddc_core::domain::{DdcError, VcpCode};
use predicates::prelude::{PredicateBooleanExt, predicate};
use serde_json::{Value, json};

use crate::support::{backend_of, bare_cli, fake_cli, fixture_backend, id, monitor, run_with};

fn stdout_of(args: &[&str]) -> String {
    let assert = fake_cli().args(args).assert().success();
    String::from_utf8_lossy(&assert.get_output().stdout).into_owned()
}

#[test]
fn fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help() {
    let first = stdout_of(&["list"]);
    let second = stdout_of(&["list"]);
    assert_eq!(first, "1  FAKE-CLI-TEST  (FAK CLI TEST)\n");
    assert_eq!(first, second);

    let listed: Value = serde_json::from_str(&stdout_of(&["list", "--json"])).unwrap();
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    assert_eq!(listed[0]["id"], FIXTURE_ID);

    for args in [&["--help"][..], &["get", "--help"], &["help", "set"]] {
        bare_cli()
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage"))
            .stdout(predicate::str::contains("fake").not());
    }
}

#[test]
fn caps_prints_the_fixture_capabilities_as_text_and_json() {
    let text = stdout_of(&["caps"]);
    assert!(text.starts_with("monitor: FAKE-CLI-TEST\n"), "{text}");
    assert!(text.contains("\n  0x60 input: 0x01 0x0F 0x11\n"), "{text}");
    assert!(text.contains("mccs version: 2.2"), "{text}");

    let json: Value = serde_json::from_str(&stdout_of(&["caps", "--json"])).unwrap();
    assert_eq!(json["monitor"], FIXTURE_ID);
    assert_eq!(json["model"], "FAKE");
    assert_eq!(json["vcp"][0], json!({ "code": 16, "values": null }));
    assert_eq!(json["vcp"].as_array().map(Vec::len), Some(6));
}

#[test]
fn get_prints_a_reading_as_text_and_json() {
    assert_eq!(
        stdout_of(&["get", "contrast"]),
        "0x12 contrast: 70 (0x46), max 100 (0x64)\n"
    );

    let json: Value = serde_json::from_str(&stdout_of(&["get", "0x62", "--json"])).unwrap();
    assert_eq!(
        json,
        json!({
            "monitor": FIXTURE_ID,
            "code": 98,
            "name": "volume",
            "current": 30,
            "max": 100,
            "declared_in_capabilities": true
        })
    );
}

#[test]
fn get_warns_about_a_code_the_capabilities_do_not_declare() {
    let backend = backend_of([monitor("M-1").with_value(VcpCode::SHARPNESS, 3, 10)]);

    let read = run_with(&backend, &["get", "0x87"]);

    assert_eq!(read.exit, Exit::Success);
    assert_eq!(read.out, "0x87: 3 (0x03), max 10 (0x0A)\n");
    assert!(
        read.err.contains("not declared in capabilities"),
        "{}",
        read.err
    );
}

#[test]
fn caps_refresh_calls_back_with_the_selected_monitor_before_reading() {
    let backend = fixture_backend();

    let refreshed = run_with(&backend, &["caps", "--refresh"]);
    let cached = run_with(&backend, &["caps"]);

    assert_eq!(refreshed.exit, Exit::Success);
    assert_eq!(refreshed.refreshed, [(id(FIXTURE_ID), 1)]);
    assert_eq!(
        refreshed.calls,
        [
            BackendCall::Enumerate,
            BackendCall::ReadCapabilities(id(FIXTURE_ID)),
        ]
    );
    assert!(cached.refreshed.is_empty());
    assert_eq!(cached.out, refreshed.out);
}

#[test]
fn listing_no_monitor_prints_an_empty_json_array_and_a_warning() {
    let listed = run_with(&backend_of([]), &["list", "--json"]);

    assert_eq!(listed.exit, Exit::Success);
    assert_eq!(listed.out.trim(), "[]");
    assert_eq!(listed.err, "warning: no monitor found\n");
}

#[test]
fn a_transport_failure_names_the_monitor_on_stderr() {
    let backend = backend_of([monitor("M-1").with_vcp_failure(
        VcpCode::BRIGHTNESS,
        DdcError::Transport("bus gone".to_owned()),
    )]);

    let broken = run_with(&backend, &["get", "brightness", "--json"]);

    assert_eq!(broken.exit, Exit::Transport);
    assert!(broken.out.is_empty());
    assert_eq!(broken.err, "error: M-1: transport error: bus gone\n");
}

#[test]
fn a_backend_that_fails_to_start_is_reported_with_its_exit_code() {
    let cli = Cli::try_parse_from(["ddc-cli", "list", "--json"]).unwrap();
    let mut err = Vec::new();

    let exit = report_startup_failure(
        &cli,
        DdcError::Transport("worker thread refused to start".to_owned()),
        &mut err,
    );

    assert_eq!(exit, Exit::Transport);
    assert_eq!(
        String::from_utf8(err).unwrap(),
        "error: transport error: worker thread refused to start\n"
    );
}

#[test]
fn arguments_clap_rejects_never_reach_the_backend() {
    let backend = fixture_backend();

    let rejected = run_with(&backend, &["get", "foo"]);

    assert_eq!(rejected.exit, Exit::Usage);
    assert!(rejected.err.contains("brightness"), "{}", rejected.err);
    assert!(backend.calls().is_empty());
}
