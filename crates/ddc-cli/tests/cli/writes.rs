//! `set`: confirmation of dangerous writes (D-2026-09-25-cli-3) and the
//! value read back.

use ddc_adapters::BackendCall;
use ddc_cli::Exit;
use ddc_cli::fixture::FIXTURE_ID;
use ddc_core::domain::VcpCode;
use predicates::prelude::predicate;
use serde_json::Value;

use crate::support::{backend_of, fake_cli, fixture_backend, id, monitor, run_with};

#[test]
fn dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes() {
    let backend = fixture_backend();

    for args in [&["set", "input", "0x11"][..], &["set", "0xE5", "1"]] {
        let refused = run_with(&backend, args);
        assert_eq!(refused.exit, Exit::Unconfirmed, "{args:?}");
        assert_eq!(refused.calls, [BackendCall::Enumerate], "{args:?}");
        assert!(refused.err.contains("--yes"), "{args:?}: {}", refused.err);
        assert!(refused.out.is_empty(), "{args:?}");
    }

    let confirmed = run_with(&backend, &["set", "input", "0x11", "--yes"]);
    assert_eq!(confirmed.exit, Exit::Success, "{}", confirmed.err);
    assert!(
        confirmed.calls.ends_with(&[
            BackendCall::WriteVcp(id(FIXTURE_ID), VcpCode::INPUT_SOURCE, 0x11),
            BackendCall::ReadVcp(id(FIXTURE_ID), VcpCode::INPUT_SOURCE),
        ]),
        "{:?}",
        confirmed.calls
    );
    assert!(confirmed.out.contains("0x11"), "{}", confirmed.out);

    fake_cli()
        .args(["set", "power", "0x04"])
        .assert()
        .code(5)
        .stdout("")
        .stderr(predicate::str::contains("--yes"));
}

#[test]
fn a_safe_write_is_applied_and_read_back() {
    let backend = fixture_backend();

    let written = run_with(&backend, &["set", "brightness", "60"]);

    assert_eq!(written.exit, Exit::Success, "{}", written.err);
    assert_eq!(written.out, "0x10 brightness: 60 (0x3C), max 100 (0x64)\n");
    assert!(written.err.is_empty(), "{}", written.err);
    assert_eq!(
        written.calls.last(),
        Some(&BackendCall::ReadVcp(id(FIXTURE_ID), VcpCode::BRIGHTNESS))
    );
}

/// Each run is a new process with no earlier read, and the fixture answers
/// 0x62 volume without declaring it, like the dev monitor: the core reads
/// the maximum once before writing (D-2026-09-26-cli-1).
#[test]
fn a_safe_write_to_an_answered_but_undeclared_code_succeeds_through_the_binary() {
    fake_cli()
        .args(["set", "volume", "40"])
        .assert()
        .code(0)
        .stdout("0x62 volume: 40 (0x28), max 100 (0x64)\n")
        .stderr("");

    fake_cli()
        .args(["set", "volume", "101"])
        .assert()
        .code(4)
        .stdout("")
        .stderr(predicate::str::contains("exceeds its maximum 100"));
}

#[test]
fn a_safe_write_through_the_binary_reports_json() {
    let output = fake_cli()
        .args(["set", "brightness", "0x3C", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["requested"], 60);
    assert_eq!(json["current"], 60);
    assert_eq!(json["applied"], true);
}

#[test]
fn a_write_the_monitor_ignores_is_reported_but_still_succeeds() {
    let backend = backend_of([monitor("M-1").ignoring_writes_to(VcpCode::BRIGHTNESS)]);

    let ignored = run_with(&backend, &["set", "brightness", "60", "--json"]);

    assert_eq!(ignored.exit, Exit::Success);
    let json: Value = serde_json::from_str(&ignored.out).unwrap();
    assert_eq!(json["applied"], false);
    assert_eq!(json["current"], 50);
    assert!(
        ignored.err.contains("may have ignored the write"),
        "{}",
        ignored.err
    );
}
