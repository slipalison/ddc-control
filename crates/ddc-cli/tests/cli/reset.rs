//! `reset <target>`: the four factory resets behind `--yes`
//! (D-2026-09-26-full-osd-control-4, -8).

use ddc_adapters::BackendCall;
use ddc_cli::Exit;
use ddc_cli::fixture::FIXTURE_ID;
use ddc_core::domain::VcpCode;
use predicates::prelude::predicate;
use serde_json::{Value, json};

use crate::support::{fake_cli, fixture_backend, id, run_with};

const TARGETS: [(&str, VcpCode); 4] = [
    ("factory", VcpCode::RESTORE_FACTORY_DEFAULTS),
    (
        "brightness-contrast",
        VcpCode::RESTORE_FACTORY_LUMINANCE_CONTRAST,
    ),
    ("geometry", VcpCode::RESTORE_FACTORY_GEOMETRY),
    ("color", VcpCode::RESTORE_FACTORY_COLOR),
];

#[test]
fn reset_subcommand_maps_named_targets_to_factory_reset_codes_and_is_refused_without_yes_applied_with_yes()
 {
    for (target, code) in TARGETS {
        let refused = run_with(&fixture_backend(), &["reset", target]);
        assert_eq!(refused.exit, Exit::Unconfirmed, "{target}");
        assert_eq!(refused.calls, [BackendCall::Enumerate], "{target}");
        assert!(refused.err.contains("--yes"), "{target}: {}", refused.err);
        assert!(refused.out.is_empty(), "{target}");

        let applied = run_with(&fixture_backend(), &["reset", target, "--yes"]);
        assert_eq!(applied.exit, Exit::Success, "{target}: {}", applied.err);
        assert_eq!(
            applied.calls,
            [
                BackendCall::Enumerate,
                BackendCall::ReadCapabilities(id(FIXTURE_ID)),
                BackendCall::WriteVcp(id(FIXTURE_ID), code, 0x01),
            ],
            "{target}"
        );
    }

    fake_cli()
        .args(["reset", "factory"])
        .assert()
        .code(5)
        .stdout("")
        .stderr(predicate::str::contains("--yes"));

    let backend = fixture_backend();
    let unknown = run_with(&backend, &["reset", "everything", "--yes"]);
    assert_eq!(unknown.exit, Exit::Usage);
    assert!(backend.calls().is_empty(), "{:?}", backend.calls());
    fake_cli()
        .args(["reset", "everything", "--yes"])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("factory"));
}

/// A reset writes 0x01 — MCCS ignores zero — and is never read back; the
/// output says so (D-2026-09-26-full-osd-control-8).
#[test]
fn reset_factory_with_yes_through_the_binary_sends_one_and_says_it_is_write_only() {
    fake_cli()
        .args(["reset", "factory", "--yes"])
        .assert()
        .code(0)
        .stdout("0x04: sent 1 (0x01) Reset (write-only, not read back)\n")
        .stderr("");

    let output = fake_cli()
        .args(["reset", "color", "--yes", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json,
        json!({
            "monitor": FIXTURE_ID,
            "code": 8,
            "name": null,
            "requested": 1,
            "value_name": "Reset",
            "read_back": false
        })
    );
}

/// `reset factory` is `set 0x04 reset --yes`: the same write through the
/// same core path, which also refuses any other value.
#[test]
fn reset_is_the_same_write_as_setting_the_reset_value() {
    let reset = run_with(&fixture_backend(), &["reset", "factory", "--yes"]);
    let set = run_with(&fixture_backend(), &["set", "0x04", "reset", "--yes"]);
    let zero = run_with(&fixture_backend(), &["set", "0x04", "0", "--yes"]);

    assert_eq!(set.exit, Exit::Success, "{}", set.err);
    assert_eq!(set.calls, reset.calls);
    assert_eq!(set.out, reset.out);
    assert_eq!(zero.exit, Exit::Invalid);
    assert!(zero.err.contains("not an allowed value"), "{}", zero.err);
}
