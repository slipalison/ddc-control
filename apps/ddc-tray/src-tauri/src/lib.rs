//! `ddc-tray` — the system-tray driving adapter of `ddc-control`.
//!
//! A small popup webview turns clicks and slider moves into calls on the
//! core's `MonitorControl` port. Every rule about monitors — which writes
//! are dangerous, which values are valid — lives in the core; this crate
//! only presents, forwards and wires. [`run`] is its composition root.

#![forbid(unsafe_code)]

pub mod commands;
pub mod dto;
pub mod i18n;
pub mod menu;
pub mod panel;
pub mod popup;
pub mod scroll;
pub mod tray;

use std::ffi::OsStr;
use std::fmt::Display;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use ddc_adapters::{CachingMonitorBackend, DdcHiMonitorBackend, default_cache_dir};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::DdcError;
use tauri::{AppHandle, Emitter, Manager, Runtime, Window, WindowEvent};

use crate::commands::{AppState, SharedOsd};
use crate::dto::POPUP_SHOWN;
use crate::popup::PopupGate;

/// Label of the popup window in `tauri.conf.json`.
const POPUP: &str = "popup";

/// WebKitGTK's switch that turns its DMA-BUF renderer off.
#[cfg(target_os = "linux")]
const DMABUF_SWITCH: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// Set to `1` to have the app say on stderr what the popup and the tray do
/// (`ddc-tray: popup shown`…); the tray's smoke test reads those lines.
pub const DIAGNOSTICS_SWITCH: &str = "DDC_TRAY_DEBUG";

/// Whether the diagnostic lines are on, decided once at start.
static DIAGNOSTICS: OnceLock<bool> = OnceLock::new();

/// Builds the core over the real DDC/CI backend, with the on-disk
/// capabilities cache the CLI also uses (D-2026-09-26-tray-app-3). Without
/// a cache directory the core reads the capabilities from the monitor, as
/// the CLI does.
///
/// # Errors
///
/// The backend's error when it cannot start.
pub fn compose_osd() -> Result<SharedOsd, DdcError> {
    let real = DdcHiMonitorBackend::new()?;
    Ok(match default_cache_dir() {
        Some(dir) => Arc::new(SoftwareOsd::new(CachingMonitorBackend::new(real, dir))),
        None => Arc::new(SoftwareOsd::new(real)),
    })
}

/// Starts the tray app and blocks until it exits. A DDC/CI backend that
/// cannot start does not stop it: every command then answers
/// `backend_unavailable`.
///
/// # Errors
///
/// Returns the Tauri error when the runtime or the webview cannot start.
pub fn run() -> Result<(), tauri::Error> {
    #[cfg(target_os = "linux")]
    restart_without_dmabuf_renderer();
    DIAGNOSTICS.get_or_init(|| diagnostics_on(std::env::var_os(DIAGNOSTICS_SWITCH).as_deref()));
    let builder = tauri::Builder::default()
        // First plugin: a second launch only shows this popup and exits
        // before building a backend, so one process owns the I2C bus
        // (D-2026-09-26-tray-app-7).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_popup(app);
        }));
    // Windows: tracks where the tray icon is, to anchor the popup to it.
    #[cfg(not(target_os = "linux"))]
    let builder = builder.plugin(tauri_plugin_positioner::init());
    builder
        .setup(|app| {
            app.manage(AppState::new(compose_osd()));
            app.manage(PopupGate::new());
            tray::install(app.handle())?;
            Ok(())
        })
        .on_window_event(hide_popup_on_leave)
        .invoke_handler(tauri::generate_handler![
            commands::list_monitors,
            commands::select_monitor,
            commands::load_panel,
            commands::load_features,
            commands::probe_features,
            commands::set_feature,
            commands::hide_popup,
        ])
        .run(tauri::generate_context!())
}

/// WebKitGTK's DMA-BUF renderer kills the app with a Wayland protocol error
/// (71) as soon as the popup is shown on NVIDIA's driver — the dev
/// machine's case. WebKit reads the switch once, at start, and setting it
/// from Rust takes `unsafe` (`std::env::set_var`), so the process replaces
/// itself — same PID, same arguments — with the switch on, unless the user
/// already chose a value. If that fails, the app starts as it is.
#[cfg(target_os = "linux")]
fn restart_without_dmabuf_renderer() {
    use std::os::unix::process::CommandExt;

    let current = std::env::var_os(DMABUF_SWITCH);
    let Some(value) = dmabuf_switch_to_set(current.as_deref()) else {
        return;
    };
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let error = std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(DMABUF_SWITCH, value)
        .exec();
    report("restart with the DMA-BUF renderer off", &error);
}

/// The value to restart with, given the switch's `current` one: `1` when
/// it is unset, else none — whatever the user set, even an empty or
/// non-UTF-8 value, is kept (D-2026-09-26-tray-app-10).
#[cfg(target_os = "linux")]
fn dmabuf_switch_to_set(current: Option<&OsStr>) -> Option<&'static str> {
    current.is_none().then_some("1")
}

/// Whether [`DIAGNOSTICS_SWITCH`], set to `value`, turns the diagnostic
/// lines on: only `1` does.
fn diagnostics_on(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

/// Shows and focuses the popup, and tells the UI to revalidate what it
/// shows.
pub(crate) fn show_popup<R: Runtime>(app: &AppHandle<R>) {
    let Some(popup) = app.get_webview_window(POPUP) else {
        return;
    };
    let shown = popup
        .show()
        .and_then(|()| popup.set_focus())
        .and_then(|()| popup.emit(POPUP_SHOWN, ()));
    match shown {
        Ok(()) => diagnose("popup shown"),
        Err(error) => report("show the popup", &error),
    }
}

/// Hides the popup when it loses the focus — remembering when, for the
/// tray click that caused it — and instead of closing it.
fn hide_popup_on_leave<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != POPUP {
        return;
    }
    match event {
        WindowEvent::Focused(true) => diagnose("popup focused"),
        WindowEvent::Focused(false) => {
            if let Some(gate) = window.try_state::<PopupGate>() {
                gate.hidden_on_blur(Instant::now());
            }
            hide(window);
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide(window);
        }
        _ => {}
    }
}

fn hide<R: Runtime>(window: &Window<R>) {
    match window.hide() {
        Ok(()) => diagnose("popup hidden"),
        Err(error) => report("hide the popup", &error),
    }
}

/// Reports a window operation the app shrugs off: the popup keeps working,
/// and the user has nothing to act on.
pub(crate) fn report(action: &str, error: &dyn Display) {
    eprintln!("ddc-tray: could not {action}: {error}");
}

/// Says on stderr what just happened, when [`DIAGNOSTICS_SWITCH`] is `1`.
pub(crate) fn diagnose(event: &str) {
    if DIAGNOSTICS.get().copied().unwrap_or(false) {
        eprintln!("ddc-tray: {event}");
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use std::ffi::OsStr;

    use super::{DIAGNOSTICS_SWITCH, diagnostics_on};

    #[test]
    fn the_diagnostics_switch_is_ddc_tray_debug() {
        assert_eq!(DIAGNOSTICS_SWITCH, "DDC_TRAY_DEBUG");
    }

    #[test]
    fn only_a_switch_set_to_1_turns_the_diagnostics_on() {
        assert!(diagnostics_on(Some(OsStr::new("1"))));
        for value in [
            None,
            Some(""),
            Some("0"),
            Some("true"),
            Some("1 "),
            Some("yes"),
        ] {
            assert!(!diagnostics_on(value.map(OsStr::new)), "{value:?}");
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    use super::dmabuf_switch_to_set;

    #[test]
    fn an_unset_switch_restarts_with_the_dmabuf_renderer_off() {
        assert_eq!(dmabuf_switch_to_set(None), Some("1"));
    }

    #[test]
    fn a_switch_the_user_set_is_kept_whatever_its_value() {
        let values = [
            OsStr::new("1"),
            OsStr::new("0"),
            OsStr::new(""),
            OsStr::from_bytes(b"\xFF"),
        ];
        for value in values {
            assert_eq!(dmabuf_switch_to_set(Some(value)), None, "{value:?}");
        }
    }
}
