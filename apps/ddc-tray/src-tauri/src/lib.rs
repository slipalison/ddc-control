//! `ddc-tray` — the system-tray driving adapter of `ddc-control`.
//!
//! A small popup webview turns clicks and slider moves into calls on the
//! core's `MonitorControl` port. Every rule about monitors — which writes
//! are dangerous, which values are valid — lives in the core; this crate
//! only presents, forwards and wires. [`run`] is its composition root.

#![forbid(unsafe_code)]

pub mod autostart;
pub mod commands;
#[cfg(test)]
mod docs;
pub mod dto;
pub mod fixture;
pub mod follow;
pub mod follow_config;
pub mod i18n;
pub mod menu;
pub mod panel;
pub mod popup;
pub mod scroll;
#[cfg(target_os = "linux")]
mod stop_signals;
pub mod tray;

use std::ffi::OsStr;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use ddc_adapters::{
    CachingMonitorBackend, DdcHiMonitorBackend, InMemoryMonitorBackend, default_cache_dir,
};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::DdcError;
use tauri::{AppHandle, Emitter, Manager, RunEvent, Runtime, Window, WindowEvent};

use crate::autostart::{PluginEntry, SharedEntry};
use crate::commands::{AppState, SharedOsd};
use crate::dto::POPUP_SHOWN;
use crate::follow_config::{ConfigStore, FollowConfig, LoadOutcome};
use crate::popup::PopupGate;

/// Label of the popup window in `tauri.conf.json`.
const POPUP: &str = "popup";

/// WebKitGTK's switch that turns its DMA-BUF renderer off.
#[cfg(target_os = "linux")]
const DMABUF_SWITCH: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// Set to `1` to have the app say on stderr what the popup and the tray do
/// (`ddc-tray: popup shown`…); the tray's smoke test reads those lines.
pub const DIAGNOSTICS_SWITCH: &str = "DDC_TRAY_DEBUG";

/// Set to `1` to have the app serve the simulated monitor of [`fixture`]
/// instead of the real ones (D-2026-09-27-tray-app-5): end-to-end checks
/// of the tray — the wheel, the shortcuts — that must never write to a
/// real monitor. Off unless set; no I2C device is opened while on.
pub const SIMULATION_SWITCH: &str = "DDC_TRAY_FAKE";

/// Set, with [`SIMULATION_SWITCH`] on, to the sysfs tree of USB devices the
/// USB switch follow reads instead of the real one: the fake switch of the
/// follow's smoke test (D-2026-10-02-usb-switch-follow-9). Read only in
/// simulation.
pub const USB_ROOT_SWITCH: &str = "DDC_TRAY_USB_ROOT";

/// The sysfs tree of the USB devices on Linux.
pub const REAL_USB_ROOT: &str = "/sys/bus/usb/devices";

/// What the app always says on stderr when [`SIMULATION_SWITCH`] is on, so
/// the mode is never mistaken for the real one.
pub const SIMULATION_NOTICE: &str =
    "ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched";

/// Whether the diagnostic lines are on, decided once at start.
static DIAGNOSTICS: OnceLock<bool> = OnceLock::new();

/// Builds the core: over the simulated monitor when [`SIMULATION_SWITCH`]
/// is `1`, else over the real DDC/CI backend, with the on-disk
/// capabilities cache the CLI also uses (D-2026-09-26-tray-app-3). Without
/// a cache directory the core reads the capabilities from the monitor, as
/// the CLI does.
///
/// # Errors
///
/// The real backend's error when it cannot start.
pub fn compose_osd() -> Result<SharedOsd, DdcError> {
    if switch_on(std::env::var_os(SIMULATION_SWITCH).as_deref()) {
        eprintln!("{SIMULATION_NOTICE}");
        return Ok(simulated_osd());
    }
    let real = DdcHiMonitorBackend::new()?;
    Ok(match default_cache_dir() {
        Some(dir) => Arc::new(SoftwareOsd::new(CachingMonitorBackend::new(real, dir))),
        None => Arc::new(SoftwareOsd::new(real)),
    })
}

/// The core over the simulated RTK monitor of [`fixture`], held in memory:
/// writes change only that memory.
pub fn simulated_osd() -> SharedOsd {
    let simulated = InMemoryMonitorBackend::builder()
        .monitor(fixture::rtk_monitor())
        .build();
    Arc::new(SoftwareOsd::new(simulated))
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
    DIAGNOSTICS.get_or_init(|| switch_on(std::env::var_os(DIAGNOSTICS_SWITCH).as_deref()));
    let builder = tauri::Builder::default()
        // First plugin: a second launch only shows this popup and exits
        // before building a backend, so one process owns the I2C bus
        // (D-2026-09-26-tray-app-7).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_popup(app);
        }));
    let builder = autostart::register(builder);
    // Windows: tracks where the tray icon is, to anchor the popup to it.
    #[cfg(not(target_os = "linux"))]
    let builder = builder.plugin(tauri_plugin_positioner::init());
    builder
        .setup(|app| {
            app.manage(AppState::new(compose_osd()));
            app.manage(PopupGate::new());
            // Before the tray: its menu reads this entry as it is mounted.
            app.manage::<SharedEntry>(Arc::new(PluginEntry::new(app.handle())));
            // Before the tray too: its menu reads the follow's settings.
            #[cfg(target_os = "linux")]
            start_follow(app.handle());
            tray::install(app.handle())?;
            // So the KWin script is unloaded however the app is stopped.
            #[cfg(target_os = "linux")]
            stop_signals::quit_on_stop_signals(app.handle());
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
        .build(context())?
        .run(|app, event| {
            if matches!(event, RunEvent::Exit) {
                tray::uninstall(app);
            }
        });
    Ok(())
}

/// The sysfs tree the USB switch follow reads: the real one, unless
/// [`SIMULATION_SWITCH`] is on — then the one [`USB_ROOT_SWITCH`] names,
/// and without one none at all, so the simulated mode reads nothing real.
pub fn usb_root(simulation: Option<&OsStr>, usb_root: Option<&OsStr>) -> Option<PathBuf> {
    if !switch_on(simulation) {
        return Some(PathBuf::from(REAL_USB_ROOT));
    }
    usb_root.filter(|root| !root.is_empty()).map(PathBuf::from)
}

/// The follow's settings in `store`: the off default when there are none,
/// or when they cannot be read — then `report` hears why, and the file is
/// left as it is (D-2026-10-02-usb-switch-follow-4).
pub fn follow_settings(
    store: &ConfigStore,
    report: impl FnOnce(&str, &dyn Display),
) -> FollowConfig {
    match store.load() {
        LoadOutcome::Loaded(config) => config,
        LoadOutcome::Missing => FollowConfig::default(),
        LoadOutcome::NotConfigured(reason) => {
            let why = format!("{}: {reason}", store.path().display());
            report("read the USB follow settings", &why);
            FollowConfig::default()
        }
    }
}

/// Linux: loads the follow's settings, shares them with the tray menu and
/// starts the follow's loop on the core of [`AppState`]
/// (D-2026-10-02-usb-switch-follow-2).
#[cfg(target_os = "linux")]
fn start_follow<R: Runtime>(app: &AppHandle<R>) {
    use crate::follow::{FollowState, SharedFollow};

    let path = follow_config::config_path(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    );
    let Some(path) = path else {
        report(
            "set up the USB follow",
            &"neither XDG_CONFIG_HOME nor HOME is a path",
        );
        return;
    };
    let store = ConfigStore::new(path);
    let config = follow_settings(&store, report);
    let follow: SharedFollow = Arc::new(FollowState::new(store, config));
    app.manage(follow.clone());
    let simulation = std::env::var_os(SIMULATION_SWITCH);
    let Some(root) = usb_root(
        simulation.as_deref(),
        std::env::var_os(USB_ROOT_SWITCH).as_deref(),
    ) else {
        diagnose("follow: no USB tree in simulation, the follow does not run");
        return;
    };
    let osd = app.state::<AppState>().osd();
    let presence = ddc_adapters::SysfsUsbPresence::new(root);
    if let Err(error) = follow::spawn(presence, follow, osd) {
        report("start the USB follow", &error);
    }
}

/// What Tauri embeds of `tauri.conf.json` — merged with the target's
/// `tauri.<platform>.conf.json`, when there is one — and of the popup's
/// files. The one place the context is generated, so the tests check the
/// configuration the app runs with (D-2026-09-27-tray-app-11).
pub(crate) fn context() -> tauri::Context<tauri::Wry> {
    tauri::generate_context!()
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

/// Whether a switch of the app — [`DIAGNOSTICS_SWITCH`],
/// [`SIMULATION_SWITCH`] — set to `value` is on: only `1` is.
fn switch_on(value: Option<&OsStr>) -> bool {
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
mod switch_tests {
    use std::ffi::OsStr;

    use ddc_core::domain::{Confirm, VcpCode, VcpValue};

    use std::cell::RefCell;
    use std::fmt::Display;
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::{
        DIAGNOSTICS_SWITCH, REAL_USB_ROOT, SIMULATION_NOTICE, SIMULATION_SWITCH, USB_ROOT_SWITCH,
        follow_settings, simulated_osd, switch_on, usb_root,
    };
    use crate::fixture::{rtk_id, rtk_info};
    use crate::follow_config::{CONFIG_FILE, ConfigStore, FollowConfig};

    #[test]
    fn the_switches_are_ddc_tray_debug_and_ddc_tray_fake() {
        assert_eq!(DIAGNOSTICS_SWITCH, "DDC_TRAY_DEBUG");
        assert_eq!(SIMULATION_SWITCH, "DDC_TRAY_FAKE");
        assert!(SIMULATION_NOTICE.starts_with("ddc-tray: DDC_TRAY_FAKE=1"));
    }

    #[test]
    fn the_smoke_script_waits_for_the_notice_the_app_prints() {
        let script = include_str!("../../scripts/smoke-sni.sh");

        assert!(script.contains(&format!("readonly SIMULATED_LINE='{SIMULATION_NOTICE}'")));
    }

    #[test]
    fn the_simulated_core_serves_the_rtk_of_the_contract() {
        let osd = simulated_osd();

        assert_eq!(osd.list_monitors(), Ok(vec![rtk_info()]));
        let brightness = osd.get_feature(&rtk_id(), VcpCode::BRIGHTNESS).unwrap();
        assert_eq!(
            brightness.value,
            VcpValue {
                current: 75,
                max: 100
            }
        );
    }

    #[test]
    fn a_write_to_the_simulated_core_is_read_back() {
        let osd = simulated_osd();

        let read_back = osd.set_feature(&rtk_id(), VcpCode::BRIGHTNESS, 80, Confirm::No);

        assert_eq!(read_back.map(|value| value.current), Ok(80));
    }

    #[test]
    fn the_follow_reads_the_tree_of_ddc_tray_usb_root_only_in_simulation() {
        let fake = Some(OsStr::new("/tmp/fake-usb"));
        let real = Some(PathBuf::from("/sys/bus/usb/devices"));
        assert_eq!(USB_ROOT_SWITCH, "DDC_TRAY_USB_ROOT");
        assert_eq!(REAL_USB_ROOT, "/sys/bus/usb/devices");

        assert_eq!(usb_root(None, None), real);
        assert_eq!(usb_root(None, fake), real);
        assert_eq!(usb_root(Some(OsStr::new("0")), fake), real);
        assert_eq!(
            usb_root(Some(OsStr::new("1")), fake),
            Some(PathBuf::from("/tmp/fake-usb"))
        );
        assert_eq!(usb_root(Some(OsStr::new("1")), None), None);
        assert_eq!(usb_root(Some(OsStr::new("1")), Some(OsStr::new(""))), None);
    }

    #[test]
    fn settings_the_app_cannot_read_are_reported_left_alone_and_read_as_off() {
        let home = TempDir::new().unwrap();
        let store = ConfigStore::new(home.path().join(CONFIG_FILE));
        let reports = RefCell::new(Vec::new());
        let report = |action: &str, error: &dyn Display| {
            reports.borrow_mut().push(format!("{action}: {error}"));
        };
        assert_eq!(follow_settings(&store, report), FollowConfig::default());

        fs::write(store.path(), r#"{"version":7}"#).unwrap();
        assert_eq!(follow_settings(&store, report), FollowConfig::default());

        assert_eq!(
            *reports.borrow(),
            [format!(
                "read the USB follow settings: {}: version 7 is not one this app reads",
                store.path().display()
            )]
        );
        assert_eq!(
            fs::read_to_string(store.path()).unwrap(),
            r#"{"version":7}"#
        );
        let saved = FollowConfig::default().with_learned(Default::default(), rtk_id());
        store.save(&saved).unwrap();
        assert_eq!(
            follow_settings(&store, |_: &str, _: &dyn Display| {}),
            saved
        );
    }

    #[test]
    fn only_a_switch_set_to_1_is_on() {
        assert!(switch_on(Some(OsStr::new("1"))));
        for value in [
            None,
            Some(""),
            Some("0"),
            Some("true"),
            Some("1 "),
            Some("yes"),
        ] {
            assert!(!switch_on(value.map(OsStr::new)), "{value:?}");
        }
    }
}

#[cfg(test)]
mod build_tests {
    use std::path::Path;

    use tauri::utils::config::parse::read_from;
    use tauri::utils::platform::Target;

    /// The popup's Content-Security-Policy (D-2026-09-26-tray-app-7): no
    /// inline script or style, no `eval`, nothing from outside the app.
    const STRICT_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; \
        img-src 'self' data:; font-src 'self'; connect-src ipc: http://ipc.localhost; \
        object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

    /// Without `tauri/custom-protocol`, even `cargo build --release` makes a
    /// Tauri dev build, which serves `app.security.devCsp` in place of `csp`
    /// (D-2026-09-27-tray-app-10).
    #[test]
    fn the_tray_is_not_a_dev_build() {
        assert!(!tauri::is_dev(), "tauri/custom-protocol is off");
    }

    /// The policy the app is built with, as Tauri embeds it: the base config
    /// merged with the target's platform file (RFC 7396), where a
    /// `"csp": null` erases it (D-2026-09-27-tray-app-11).
    #[test]
    fn the_effective_csp_is_the_strict_policy() {
        let context = super::context();
        let security = &context.config().app.security;

        let csp = security.csp.as_ref().map(ToString::to_string);
        assert_eq!(csp.as_deref(), Some(STRICT_CSP));
        assert_eq!(security.dev_csp, None);
    }

    /// The policy of every target the app ships to, read as Tauri's build
    /// reads it for that target — the base config merged with that target's
    /// platform file, which the build of any other target never reads
    /// (D-2026-09-27-tray-app-11).
    #[test]
    fn the_effective_csp_is_the_strict_policy_on_every_target() {
        let config_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        for target in [Target::Linux, Target::Windows, Target::MacOS] {
            let (merged, _) = read_from(target, config_dir).unwrap();
            let config: tauri::Config = serde_json::from_value(merged).unwrap();
            let security = &config.app.security;

            let csp = security.csp.as_ref().map(ToString::to_string);
            assert_eq!(csp.as_deref(), Some(STRICT_CSP), "{target}");
            assert_eq!(security.dev_csp, None, "{target}");
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
