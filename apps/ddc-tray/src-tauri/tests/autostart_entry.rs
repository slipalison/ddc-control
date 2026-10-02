//! The real startup entry on Linux, in a home that is not the user's
//! (D-2026-09-30-input-switch-autostart-8, -12).
//!
//! The plugin reads `$HOME` when it writes `~/.config/autostart/*.desktop`,
//! and the test may neither touch the real one nor set the environment of
//! its own process (`std::env::set_var` is `unsafe`). So the test runs
//! itself again as a child process whose `HOME` is a temporary directory:
//! the parent test starts the child, which runs the app's own
//! [`PluginEntry`] there, and reads the `.desktop` file the child left.

#![cfg(target_os = "linux")]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use ddc_tray::autostart::{Autostart, AutostartEntry, PluginEntry, register};
use tauri::test::{mock_builder, mock_context, noop_assets};

/// Marks the child run; its value is the directory `HOME` must be under.
const SANDBOX: &str = "DDC_TRAY_AUTOSTART_SANDBOX";
/// What the child does: `enable` or `disable`.
const ACTION: &str = "DDC_TRAY_AUTOSTART_ACTION";
/// The child's test, by its exact name.
const CHILD: &str = "child_enables_or_disables_the_entry_under_the_sandboxed_home";

/// Runs the test binary again on the child test only, with `HOME` in
/// `home`, and answers whether the child said the entry is enabled.
fn run_child(home: &Path, action: &str) -> io::Result<bool> {
    let output = Command::new(std::env::current_exe()?)
        .args(["--exact", CHILD, "--nocapture"])
        .env("HOME", home)
        .env(SANDBOX, home)
        .env(ACTION, action)
        .env_remove("APPIMAGE")
        .env_remove("XDG_CONFIG_HOME")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "child failed: {output:?}");
    assert!(
        stdout.contains("test result: ok. 1 passed"),
        "the child did not run its test: {stdout}"
    );
    Ok(stdout.contains("is_enabled=true"))
}

/// The `.desktop` files under `<home>/.config/autostart`.
fn desktop_entries(home: &Path) -> io::Result<Vec<PathBuf>> {
    let dir = home.join(".config").join("autostart");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "desktop") {
            entries.push(path);
        }
    }
    Ok(entries)
}

#[test]
fn enabling_then_disabling_in_a_sandboxed_home_writes_then_removes_the_desktop_entry() {
    let home = tempfile::tempdir().unwrap();
    // The plugin makes `autostart`, not the `.config` above it.
    fs::create_dir(home.path().join(".config")).unwrap();

    let enabled = run_child(home.path(), "enable").unwrap();

    let entries = desktop_entries(home.path()).unwrap();
    assert_eq!(entries.len(), 1, "{entries:?}");
    let content = fs::read_to_string(&entries[0]).unwrap();
    // Printed raw: the run shows what the OS would read at login.
    println!("{content}");
    let exe = std::env::current_exe().unwrap();
    let exec = content
        .lines()
        .find_map(|line| line.strip_prefix("Exec="))
        .unwrap();
    assert_eq!(
        exec.trim_end(),
        exe.to_str().unwrap(),
        "Exec= is the test binary"
    );
    assert!(enabled, "the child did not see the entry enabled");

    let still_enabled = run_child(home.path(), "disable").unwrap();

    assert_eq!(desktop_entries(home.path()).unwrap(), Vec::<PathBuf>::new());
    assert!(!still_enabled, "the child still saw the entry enabled");
}

/// Only the parent runs it with a marker; in the normal suite it does
/// nothing. With one, it runs the app's own entry under the `HOME` the parent
/// gave it, and aborts if that is anywhere else.
#[test]
fn child_enables_or_disables_the_entry_under_the_sandboxed_home() {
    let Some(sandbox) = std::env::var_os(SANDBOX) else {
        return;
    };
    let under_the_sandbox = std::env::var_os("HOME")
        .is_some_and(|home| Path::new(&home).starts_with(Path::new(&sandbox)));
    if !under_the_sandbox {
        std::process::abort();
    }
    let action = std::env::var(ACTION).unwrap();
    let wanted = match action.as_str() {
        "enable" => Autostart::Enabled,
        "disable" => Autostart::Disabled,
        _ => std::process::abort(),
    };
    let app = register(mock_builder())
        .build(mock_context(noop_assets()))
        .unwrap();
    let entry = PluginEntry::new(app.handle());

    entry.set(wanted).unwrap();

    let enabled = entry.state().unwrap() == Autostart::Enabled;
    println!("child {action}: is_enabled={enabled}");
}
