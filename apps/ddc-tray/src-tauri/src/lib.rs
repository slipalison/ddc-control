//! `ddc-tray` — the system-tray driving adapter of `ddc-control`.
//!
//! A small popup webview turns clicks and slider moves into calls on the
//! core's `MonitorControl` port. Every rule about monitors — which writes
//! are dangerous, which values are valid — lives in the core; this crate
//! only presents, forwards and wires.

#![forbid(unsafe_code)]

pub mod dto;
pub mod panel;

/// Starts the tray app and blocks until it exits.
///
/// # Errors
///
/// Returns the Tauri error when the runtime or the webview cannot start.
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default().run(tauri::generate_context!())
}
