//! Choosing the monitor (D-2026-09-25-cli-2).

use ddc_adapters::BackendCall;
use ddc_cli::Exit;
use ddc_core::domain::VcpCode;
use predicates::prelude::predicate;

use crate::support::{backend_of, fake_cli, id, monitor, run_with, touched_a_monitor};

const THREE: [&str; 3] = ["RTK-QHD-1", "GSM-LG-TV", "RTK-QHD-2"];

fn three_monitors() -> ddc_adapters::InMemoryMonitorBackend {
    backend_of(THREE.map(monitor))
}

#[test]
fn resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many() {
    fake_cli()
        .args(["get", "brightness"])
        .assert()
        .success()
        .stdout(predicate::str::contains("50"));

    let none = run_with(&backend_of([]), &["get", "brightness"]);
    assert_eq!(none.exit, Exit::Monitor);
    assert!(none.err.contains("no monitor"), "{}", none.err);
    assert!(none.out.is_empty());

    let many = run_with(&three_monitors(), &["get", "brightness"]);
    assert_eq!(many.exit, Exit::Monitor);
    for (index, key) in THREE.iter().enumerate() {
        let line = format!("{}  {key}", index + 1);
        assert!(many.err.contains(&line), "missing {line:?} in {}", many.err);
    }
    assert!(many.err.contains("--monitor <id|index>"), "{}", many.err);
    assert!(many.out.is_empty());
    assert_eq!(many.calls, [BackendCall::Enumerate]);
}

#[test]
fn monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches() {
    for wanted in ["FAKE-CLI-TEST", "1", "cli-test"] {
        fake_cli()
            .args(["--monitor", wanted, "get", "brightness"])
            .assert()
            .success()
            .stdout(predicate::str::contains("50"));
    }

    let backend = three_monitors();
    for (wanted, key) in [
        ("2", "GSM-LG-TV"),
        ("lg-tv", "GSM-LG-TV"),
        ("RTK-QHD-2", "RTK-QHD-2"),
    ] {
        let chosen = run_with(&backend, &["-m", wanted, "get", "brightness"]);
        assert_eq!(
            chosen.exit,
            Exit::Success,
            "--monitor {wanted}: {}",
            chosen.err
        );
        assert_eq!(
            chosen.calls.last(),
            Some(&BackendCall::ReadVcp(id(key), VcpCode::BRIGHTNESS)),
            "--monitor {wanted}"
        );
    }

    let prefixed = backend_of(["RTK-QHD-1", "RTK-QHD-10"].map(monitor));
    let exact = run_with(&prefixed, &["-m", "RTK-QHD-1", "get", "brightness"]);
    assert_eq!(exact.exit, Exit::Success, "{}", exact.err);
    assert_eq!(
        exact.calls.last(),
        Some(&BackendCall::ReadVcp(id("RTK-QHD-1"), VcpCode::BRIGHTNESS)),
        "an exact id wins over the longer id that contains it"
    );
    let only_a_part = run_with(&prefixed, &["-m", "rtk-qhd-1", "get", "brightness"]);
    assert_eq!(only_a_part.exit, Exit::Monitor, "{}", only_a_part.err);
    assert!(!touched_a_monitor(&only_a_part.calls));

    let ambiguous = run_with(&backend, &["-m", "rtk", "get", "brightness"]);
    assert_eq!(ambiguous.exit, Exit::Monitor);
    assert!(ambiguous.err.contains("1  RTK-QHD-1"), "{}", ambiguous.err);
    assert!(ambiguous.err.contains("3  RTK-QHD-2"), "{}", ambiguous.err);
    assert!(!ambiguous.err.contains("GSM-LG-TV"), "{}", ambiguous.err);
    assert!(!touched_a_monitor(&ambiguous.calls));

    let unknown = run_with(&backend, &["-m", "nope", "get", "brightness"]);
    assert_eq!(unknown.exit, Exit::Monitor);
    assert!(unknown.err.contains("'nope'"), "{}", unknown.err);
    assert!(!touched_a_monitor(&unknown.calls));
}

#[test]
fn list_ignores_the_monitor_flag() {
    let listed = run_with(&three_monitors(), &["-m", "nope", "list"]);

    assert_eq!(listed.exit, Exit::Success);
    assert_eq!(listed.out, "1  RTK-QHD-1\n2  GSM-LG-TV\n3  RTK-QHD-2\n");
    assert_eq!(listed.calls, [BackendCall::Enumerate]);
}
