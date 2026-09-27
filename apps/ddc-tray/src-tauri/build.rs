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

fn main() -> ExitCode {
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    match tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
