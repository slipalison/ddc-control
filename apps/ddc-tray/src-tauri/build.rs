//! Build script: embeds the Tauri config and generates an `allow-*`
//! permission per app command, so the capability grants each one by name.

use std::env;
use std::path::Path;
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

/// The Windows application manifest (Common Controls v6), a copy of the
/// one tauri-build 2.7.0 embeds by default.
const WINDOWS_MANIFEST: &str = "windows-app-manifest.xml";

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={CONFIG_DIR}");
    if targets_windows_msvc() {
        embed_windows_manifest();
    }
    let manifest = tauri_build::AppManifest::new().commands(COMMANDS);
    let attributes = tauri_build::Attributes::new()
        .app_manifest(manifest)
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    match tauri_build::try_build(attributes) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

/// The target, not the host: a build script is compiled for the host, so
/// `#[cfg(windows)]` would describe the machine running the build.
fn targets_windows_msvc() -> bool {
    let is = |key: &str, value: &str| env::var(key).is_ok_and(|v| v == value);
    is("CARGO_CFG_TARGET_OS", "windows") && is("CARGO_CFG_TARGET_ENV", "msvc")
}

/// tauri-build puts the manifest in the resource file, which only the
/// `[[bin]]` links, so every test binary of this crate dies on Windows with
/// STATUS_ENTRYPOINT_NOT_FOUND (tauri-apps/tauri#13419). Handing it to the
/// linker instead embeds it in the bin, the tests and the examples alike
/// (D-2026-09-27-ci-crossbuild-5).
fn embed_windows_manifest() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join(WINDOWS_MANIFEST);
    println!("cargo::rerun-if-changed={}", manifest.display());
    println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo::rustc-link-arg=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
