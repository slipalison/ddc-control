//! The closed exit-code table of D-2026-09-25-cli-5, through the binary.

use ddc_adapters::BackendCall;
use ddc_cli::Exit;
use predicates::prelude::{PredicateBooleanExt, predicate};

use crate::support::{fake_cli, fixture_backend, run_with};

/// Runs the `--fake` binary with `args` and checks it fails with exactly
/// `code`, prints nothing on stdout and says why on stderr.
fn assert_fails(args: &[&str], code: i32, reason: &str) {
    fake_cli()
        .args(args)
        .assert()
        .code(code)
        .stdout("")
        .stderr(predicate::str::starts_with("error: ").and(predicate::str::contains(reason)));
}

#[test]
fn exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures() {
    assert_fails(&["--monitor", "nope", "get", "brightness"], 3, "'nope'");
    assert_fails(&["--monitor", "2", "get", "brightness"], 3, "'2'");

    assert_fails(&["set", "brightness", "101"], 4, "exceeds its maximum 100");
    assert_fails(&["set", "preset", "0x07"], 4, "not an allowed value");
    assert_fails(&["get", "0x87"], 4, "0x87 is not supported");

    assert_fails(&["set", "input", "0x11"], 5, "--yes");

    assert_fails(&["get", "0xDF"], 6, "did not respond in time");
}

/// The real backend answers a code the monitor refuses with
/// `UnsupportedFeature` (D-2026-09-26-cli-4), as the fixture does for 0x87:
/// `get` and `set` both exit 4, and `set` fails on the maximum read, so it
/// never writes.
#[test]
fn a_code_the_monitor_refuses_exits_4_for_get_and_set_without_writing() {
    assert_fails(&["get", "0x87"], 4, "0x87 is not supported");
    assert_fails(&["set", "0x87", "5"], 4, "0x87 is not supported");

    let refused = run_with(&fixture_backend(), &["set", "0x87", "5"]);

    assert_eq!(refused.exit, Exit::Invalid);
    assert!(
        !refused
            .calls
            .iter()
            .any(|call| matches!(call, BackendCall::WriteVcp(..))),
        "{:?}",
        refused.calls
    );
}

#[test]
fn usage_errors_keep_clap_exit_code() {
    fake_cli()
        .args(["get", "foo"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("brightness"));
}

#[test]
fn a_successful_command_exits_zero() {
    fake_cli().args(["get", "brightness"]).assert().code(0);
}
