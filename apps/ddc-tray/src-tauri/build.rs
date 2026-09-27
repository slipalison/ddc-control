//! Build script: embeds the Tauri config and generates an `allow-*`
//! permission per app command, so the capability grants each one by name.

use std::process::ExitCode;

/// Every `#[tauri::command]` the popup may invoke (D-2026-09-26-tray-app-4).
const COMMANDS: &[&str] = &[
    "list_monitors",
    "select_monitor",
    "load_panel",
    "load_features",
    "probe_features",
    "set_feature",
    "hide_popup",
];

/// Where Tauri reads the app's config: `tauri.conf.json` and the target's
/// `tauri.<platform>.conf.json`, merged into what the app embeds.
/// tauri-build has Cargo watch only the files there were at its last run,
/// so a platform file added since would go unseen by an incremental build
/// — and by the test of the effective CSP (D-2026-09-27-tray-app-11).
const CONFIG_DIR: &str = ".";

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={CONFIG_DIR}");
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    match tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
