//! The tray icon (D-2026-09-26-tray-app-5, -6, D-2026-09-27-tray-app-2):
//! the icon, its tooltip and the menu of [`crate::menu`] in the system's
//! language.
//!
//! - On Windows it is Tauri's tray icon (`notification_area`): a left click
//!   toggles the popup anchored above the icon, a right click opens the
//!   menu.
//! - On Linux it is a StatusNotifierItem of the app's own (`status_item`):
//!   a left click toggles the popup, the wheel steps the brightness
//!   ([`crate::scroll`]), a right click opens the menu. On KDE Plasma under
//!   Wayland a KWin script places the popup next to the icon
//!   (`kwin_placement`).
//!
//! What both share lives here: the popup toggle and the menu's actions. A
//! brightness shortcut sets the monitor the popup last selected, off the
//! main thread. "Start with system" flips the OS's own startup entry, read
//! afresh each time the menu is mounted (D-2026-09-30-input-switch-autostart-9).
//! The USB switch follow's items (Linux) change its settings off the menu's
//! thread and ask its loop to learn the switch, with the shortcut target as
//! the monitor to switch (D-2026-10-02-usb-switch-follow-5).

#[cfg(target_os = "linux")]
mod kwin_placement;
#[cfg(not(target_os = "linux"))]
mod notification_area;
#[cfg(target_os = "linux")]
mod status_item;

#[cfg(not(target_os = "linux"))]
use notification_area as platform;
#[cfg(target_os = "linux")]
use status_item as platform;

use std::time::Instant;

use ddc_core::domain::MonitorId;
use ddc_core::ports::MonitorControl;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::autostart::{self, Autostart, SharedEntry};
use crate::commands::{AppState, on_blocking_thread};
use crate::dto::{ErrorKind, PANEL_CHANGED, UiError};
use crate::follow::{FollowError, FollowState, SharedFollow};
use crate::follow_config::FollowConfig;
use crate::i18n::Locale;
use crate::menu::MenuAction;
use crate::panel::{self, BrightnessChange};
use crate::popup::{ClickAction, PopupGate, Visibility};
use crate::{POPUP, diagnose, report, show_popup};

/// Id of the app's only tray icon, stable across runs.
pub const TRAY_ID: &str = "ddc-control";

/// The app's name, as the tray shows it.
pub const APP_NAME: &str = "DDC Control";

/// Puts the icon in the tray, with the menu of this platform in the
/// system's language.
///
/// # Errors
///
/// The Tauri error when the menu or the icon cannot be created. On Linux
/// the item registers in the background, and a failure is only reported.
pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let locale = Locale::from_tag(&sys_locale::get_locale().unwrap_or_default());
    platform::install(app, locale)
}

/// Undoes what [`install`] set up outside the app, as it quits.
pub fn uninstall<R: Runtime>(app: &AppHandle<R>) {
    platform::uninstall(app);
}

/// Tells the icon the popup selected another monitor, which its tooltip
/// names on Linux.
pub(crate) fn selection_changed<R: Runtime>(app: &AppHandle<R>) {
    platform::selection_changed(app);
}

/// Sets the brightness of the shortcut target — the monitor the popup last
/// selected, else the first one listed — to `percent` of its maximum, and
/// returns the change. Brightness is a safe feature: nothing here can
/// confirm a dangerous write.
///
/// # Errors
///
/// The [`UiError`] of the enumeration, of the read of the maximum or of the
/// write.
pub fn brightness_shortcut<M: MonitorControl + ?Sized>(
    osd: &M,
    selected: Option<MonitorId>,
    percent: u8,
) -> Result<BrightnessChange, UiError> {
    let id = panel::shortcut_target(osd, selected)?;
    panel::set_brightness_percent(osd, &id, percent)
}

/// What a menu item does, on either platform.
fn run_menu_action<R: Runtime>(app: &AppHandle<R>, action: MenuAction) {
    match action {
        MenuAction::OpenPanel => on_main_thread(app, open_panel),
        MenuAction::Brightness(percent) => apply_brightness(app, percent),
        MenuAction::Autostart => toggle_autostart(app),
        MenuAction::Follow => on_blocking(app, flip_follow),
        MenuAction::Learn => learn_switch(app),
        MenuAction::FollowInput(code) => {
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || pick_follow_input(&app, code));
        }
        MenuAction::Quit => app.exit(0),
    }
}

/// The follow's settings, for a menu being mounted: off and nothing learned
/// without the follow's state, which only Linux sets up.
fn follow_config<R: Runtime>(app: &AppHandle<R>) -> FollowConfig {
    app.try_state::<SharedFollow>()
        .map(|follow| follow.config())
        .unwrap_or_default()
}

/// Runs `task` on a blocking thread: the follow's settings are a file, and
/// the menu must not wait for it.
fn on_blocking<R: Runtime>(app: &AppHandle<R>, task: fn(&AppHandle<R>)) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || task(&app));
}

/// Turns the follow on or off; turning it on with settings that lack
/// something leaves it off and reports what is missing.
fn flip_follow<R: Runtime>(app: &AppHandle<R>) {
    change_follow(app, "change the USB follow", FollowState::toggle_enabled);
}

/// Makes `code` the input the follow switches to.
fn pick_follow_input<R: Runtime>(app: &AppHandle<R>, code: u8) {
    change_follow(app, "choose the USB follow input", |follow| {
        follow.choose_input(code)
    });
}

/// Applies `change` to the follow's settings, reports a refusal, and has
/// the menu show what is true.
fn change_follow<R: Runtime>(
    app: &AppHandle<R>,
    action: &str,
    change: impl FnOnce(&FollowState) -> Result<FollowConfig, FollowError>,
) {
    match app.try_state::<SharedFollow>() {
        Some(follow) => {
            if let Err(error) = change(&follow) {
                report(action, &error);
            }
        }
        None => report(action, &"the USB follow is not set up"),
    }
    platform::follow_changed(app);
}

/// Asks the follow's loop to learn the switch, off the menu's thread.
fn learn_switch<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { ask_to_learn(&app).await });
}

/// Asks the follow's loop to learn the switch, with the shortcut target —
/// the monitor the popup last selected, else the first one listed — as the
/// monitor it records (D-2026-10-02-usb-switch-follow-5).
async fn ask_to_learn<R: Runtime>(app: &AppHandle<R>) {
    match app.try_state::<SharedFollow>() {
        Some(follow) => match learning_monitor(app).await {
            Ok(monitor) => follow.request_learning(monitor),
            Err(error) => report_ui("learn the USB switch", &error),
        },
        None => report("learn the USB switch", &"the USB follow is not set up"),
    }
    platform::follow_changed(app);
}

/// The monitor a learning records: the shortcut target, found on a blocking
/// thread since it may enumerate the monitors.
async fn learning_monitor<R: Runtime>(app: &AppHandle<R>) -> Result<MonitorId, UiError> {
    let Some(state) = app.try_state::<AppState>() else {
        return Err(UiError {
            kind: ErrorKind::BackendUnavailable,
            message: "the monitors are not set up".to_owned(),
        });
    };
    let selected = state.selected();
    on_blocking_thread(&state, move |osd| panel::shortcut_target(osd, selected)).await
}

/// The OS state of "Start with system", for a menu being mounted. It only
/// reads; a state the OS will not tell reads as off, and is reported.
fn autostart_state<R: Runtime>(app: &AppHandle<R>) -> Autostart {
    match app.try_state::<SharedEntry>() {
        Some(entry) => autostart::state_or_report(&**entry, report),
        None => {
            report("read the start-with-system entry", &"it is not set up");
            Autostart::Disabled
        }
    }
}

/// Flips "Start with system" off the menu's thread: the entry is a file or
/// a registry value, and the menu must not wait for it.
fn toggle_autostart<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || flip_autostart(&app));
}

/// Flips the OS entry, then has the menu show the state the OS is in: a
/// refused flip is reported and the item goes back to what is true.
fn flip_autostart<R: Runtime>(app: &AppHandle<R>) {
    match app.try_state::<SharedEntry>() {
        Some(entry) => {
            autostart::toggle_or_report(&**entry, report);
        }
        None => report("change the start-with-system entry", &"it is not set up"),
    }
    platform::autostart_changed(app);
}

/// Runs `task` on the main thread, where the popup's window lives and its
/// blur is handled.
fn on_main_thread<R: Runtime>(app: &AppHandle<R>, task: fn(&AppHandle<R>)) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || task(&handle)) {
        report("reach the main thread", &error);
    }
}

/// A click on the icon hides an open popup and shows a hidden one — unless
/// that same click already hid it through the blur (see [`PopupGate`]).
fn toggle_popup<R: Runtime>(app: &AppHandle<R>) {
    let (Some(popup), Some(gate)) = (app.get_webview_window(POPUP), app.try_state::<PopupGate>())
    else {
        return;
    };
    let visibility = if popup.is_visible().unwrap_or(false) {
        Visibility::Shown
    } else {
        Visibility::Hidden
    };
    match gate.tray_clicked(visibility, Instant::now()) {
        ClickAction::Show => open_panel(app),
        ClickAction::Hide => match popup.hide() {
            Ok(()) => diagnose("popup hidden"),
            Err(error) => report("hide the popup", &error),
        },
        ClickAction::Nothing => {}
    }
}

fn open_panel<R: Runtime>(app: &AppHandle<R>) {
    if let Some(popup) = app.get_webview_window(POPUP) {
        platform::place_popup(&popup);
    }
    show_popup(app);
}

/// Runs a brightness shortcut on a blocking thread — a DDC/CI round-trip
/// must not hold the menu — and tells the popup to reload the monitor it
/// changed. A failure only reaches the log: the menu has nowhere to show
/// it.
fn apply_brightness<R: Runtime>(app: &AppHandle<R>, percent: u8) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        let selected = state.selected();
        let changed = on_blocking_thread(&state, move |osd| {
            brightness_shortcut(osd, selected, percent)
        })
        .await;
        match changed {
            Ok(change) => brightness_changed(&app, &change),
            Err(error) => report_ui("apply the brightness shortcut", &error),
        }
    });
}

/// Says on stderr what the tray wrote (`brightness 75 -> 80`, with
/// `DDC_TRAY_DEBUG=1`) and tells the popup, so it reloads the monitor.
fn brightness_changed<R: Runtime>(app: &AppHandle<R>, change: &BrightnessChange) {
    diagnose(&change.to_string());
    if let Err(error) = app.emit(PANEL_CHANGED, change.panel_changed()) {
        report("tell the popup the brightness changed", &error);
    }
}

fn report_ui(action: &str, error: &UiError) {
    report(action, &format!("{:?}: {}", error.kind, error.message));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ddc_adapters::{BackendCall, InMemoryMonitorBackend, SysfsUsbPresence};
    use ddc_core::domain::{DdcError, MonitorId, VcpCode};
    use tauri::test::{MockRuntime, mock_app};
    use tauri::{App, Manager};
    use tempfile::TempDir;

    use super::{
        APP_NAME, TRAY_ID, ask_to_learn, autostart_state, brightness_shortcut, flip_autostart,
        flip_follow, follow_config, pick_follow_input,
    };
    use crate::autostart::fake::FakeEntry;
    use crate::autostart::{Autostart, AutostartEntry, SharedEntry};
    use crate::commands::{AppState, SharedOsd};
    use crate::dto::{ErrorKind, UiError};
    use crate::fixture::{RTK_ID, rtk_id, rtk_monitor};
    use crate::follow::fake::{
        FakeRoot, HUB, Inline, KEYBOARD, MOUSE, Recorder, ScriptedClock, keyboard_and_mouse, reads,
    };
    use crate::follow::{FollowLoop, FollowState, SharedFollow};
    use crate::follow_config::{CONFIG_FILE, ConfigStore, FollowConfig, LoadOutcome};
    use crate::panel::BrightnessChange;
    use crate::panel::tests::{DELL_ID, dell_monitor, osd_with};

    fn writes(backend: &InMemoryMonitorBackend) -> Vec<BackendCall> {
        backend
            .calls()
            .into_iter()
            .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
            .collect()
    }

    fn changed(id: &str, before: u16, after: u16) -> Result<BrightnessChange, UiError> {
        Ok(BrightnessChange {
            monitor_id: MonitorId::new(id),
            before,
            after,
        })
    }

    #[test]
    fn the_tray_icon_id_and_name_are_stable() {
        assert_eq!(TRAY_ID, "ddc-control");
        assert_eq!(APP_NAME, "DDC Control");
    }

    #[test]
    fn a_shortcut_sets_the_selected_monitor_and_names_it() {
        let (osd, backend) = osd_with([rtk_monitor(), dell_monitor(30, 100)]);

        let outcome = brightness_shortcut(&osd, Some(MonitorId::new(DELL_ID)), 75);

        assert_eq!(outcome, changed(DELL_ID, 30, 75));
        assert_eq!(
            writes(&backend),
            [BackendCall::WriteVcp(
                MonitorId::new(DELL_ID),
                VcpCode::BRIGHTNESS,
                75
            )]
        );
    }

    #[test]
    fn without_a_selection_a_shortcut_sets_the_first_monitor_listed() {
        let (osd, backend) = osd_with([rtk_monitor(), dell_monitor(30, 100)]);

        let outcome = brightness_shortcut(&osd, None, 25);

        assert_eq!(outcome, changed(RTK_ID, 75, 25));
        assert_eq!(
            writes(&backend),
            [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 25)]
        );
    }

    #[test]
    fn the_zero_and_full_shortcuts_reach_the_ends_of_the_range() {
        let (osd, backend) = osd_with([rtk_monitor()]);

        assert_eq!(brightness_shortcut(&osd, None, 0), changed(RTK_ID, 75, 0));
        assert_eq!(
            brightness_shortcut(&osd, None, 100),
            changed(RTK_ID, 0, 100)
        );

        assert_eq!(
            writes(&backend),
            [
                BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 0),
                BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 100),
            ]
        );
    }

    #[test]
    fn without_any_monitor_a_shortcut_writes_nothing() {
        let (osd, backend) = osd_with([]);

        let outcome = brightness_shortcut(&osd, None, 50);

        assert_eq!(
            outcome,
            Err(UiError {
                kind: ErrorKind::NotFound,
                message: "no monitor is reachable".to_owned(),
            })
        );
        assert_eq!(writes(&backend), []);
    }

    #[test]
    fn a_selected_monitor_that_is_gone_fails_as_not_found() {
        let (osd, backend) = osd_with([rtk_monitor()]);

        let outcome = brightness_shortcut(&osd, Some(MonitorId::new("GONE")), 50);

        assert_eq!(
            outcome,
            Err(UiError {
                kind: ErrorKind::NotFound,
                message: "monitor GONE not found".to_owned(),
            })
        );
        assert_eq!(writes(&backend), []);
    }

    #[test]
    fn a_monitor_that_does_not_answer_fails_the_shortcut_without_a_write() {
        let monitor = rtk_monitor().with_vcp_failure(VcpCode::BRIGHTNESS, DdcError::Timeout);
        let (osd, backend) = osd_with([monitor]);

        let outcome = brightness_shortcut(&osd, None, 50);

        assert_eq!(
            outcome,
            Err(UiError {
                kind: ErrorKind::Timeout,
                message: "monitor did not respond in time".to_owned(),
            })
        );
        assert_eq!(writes(&backend), []);
    }

    /// A mock app holding `entry` as the tray's "Start with system" entry.
    fn app_with(entry: FakeEntry) -> (App<MockRuntime>, Arc<FakeEntry>) {
        let app = mock_app();
        let entry = Arc::new(entry);
        app.manage::<SharedEntry>(entry.clone());
        (app, entry)
    }

    #[test]
    fn a_click_on_start_with_system_flips_the_os_entry_each_time() {
        let (app, entry) = app_with(FakeEntry::new(Autostart::Disabled));

        flip_autostart(app.handle());
        assert_eq!(entry.held(), Autostart::Enabled);

        flip_autostart(app.handle());
        assert_eq!(entry.held(), Autostart::Disabled);
    }

    #[test]
    fn a_refused_flip_leaves_the_entry_as_it_was() {
        for before in [Autostart::Disabled, Autostart::Enabled] {
            let (app, entry) = app_with(FakeEntry::refusing(before, "read-only home"));

            flip_autostart(app.handle());

            assert_eq!(entry.held(), before);
            assert_eq!(autostart_state(app.handle()), before);
        }
    }

    #[test]
    fn the_menu_reads_the_os_state_each_time_it_is_mounted() {
        let (app, entry) = app_with(FakeEntry::new(Autostart::Disabled));
        assert_eq!(autostart_state(app.handle()), Autostart::Disabled);

        entry.set(Autostart::Enabled).unwrap();

        assert_eq!(autostart_state(app.handle()), Autostart::Enabled);
    }

    #[test]
    fn a_menu_mounted_without_an_entry_reads_off_and_a_click_does_nothing() {
        let app = mock_app();

        assert_eq!(autostart_state(app.handle()), Autostart::Disabled);
        flip_autostart(app.handle());
    }

    /// A mock app holding the follow's state over `config`, whose file lives
    /// in `home` — not written until a change is saved.
    fn app_following(
        config: FollowConfig,
        home: &TempDir,
    ) -> (App<MockRuntime>, SharedFollow, ConfigStore) {
        let app = mock_app();
        let store = ConfigStore::new(home.path().join("ddc-control").join(CONFIG_FILE));
        let follow: SharedFollow = Arc::new(FollowState::new(store.clone(), config));
        app.manage(follow.clone());
        (app, follow, store)
    }

    /// Settings with the keyboard and the mouse learned, the RTK to switch
    /// to DisplayPort 2, the follow off.
    fn complete_off() -> FollowConfig {
        FollowConfig::default()
            .with_learned(keyboard_and_mouse(), rtk_id())
            .with_target_input(0x10)
            .unwrap()
    }

    #[test]
    fn turning_on_an_incomplete_follow_leaves_it_off_and_saves_nothing() {
        let home = TempDir::new().unwrap();
        let learned_only = FollowConfig::default().with_learned(keyboard_and_mouse(), rtk_id());
        for config in [FollowConfig::default(), learned_only] {
            let (app, follow, store) = app_following(config.clone(), &home);

            flip_follow(app.handle());

            assert_eq!(follow.config(), config);
            assert_eq!(follow_config(app.handle()), config);
            assert_eq!(store.load(), LoadOutcome::Missing);
        }
    }

    #[test]
    fn turning_on_a_complete_follow_saves_it_on_and_the_next_click_off() {
        let home = TempDir::new().unwrap();
        let (app, follow, store) = app_following(complete_off(), &home);

        flip_follow(app.handle());

        let on = complete_off().enabled_toggled().unwrap();
        assert!(on.enabled);
        assert_eq!(follow.config(), on);
        assert_eq!(store.load(), LoadOutcome::Loaded(on));

        flip_follow(app.handle());
        assert_eq!(store.load(), LoadOutcome::Loaded(complete_off()));
    }

    #[test]
    fn picking_an_input_saves_it() {
        let home = TempDir::new().unwrap();
        let (app, follow, store) = app_following(complete_off(), &home);

        pick_follow_input(app.handle(), 0x0F);

        let chosen = complete_off().with_target_input(0x0F).unwrap();
        assert_eq!(follow.config(), chosen);
        assert_eq!(store.load(), LoadOutcome::Loaded(chosen));
    }

    #[test]
    fn learning_through_the_loop_records_the_monitor_the_popup_selected_else_the_first_listed() {
        for (selected, recorded) in [(Some(DELL_ID), DELL_ID), (None, RTK_ID)] {
            let home = TempDir::new().unwrap();
            let (app, follow, store) = app_following(FollowConfig::default(), &home);
            let (osd, backend) = osd_with([rtk_monitor(), dell_monitor(30, 100)]);
            let osd: SharedOsd = Arc::new(osd);
            let state = AppState::new(Ok(osd.clone()));
            if let Some(id) = selected {
                state.select(MonitorId::new(id));
            }
            app.manage(state);
            let root = FakeRoot::new();
            let output = Recorder::new();
            let presence = SysfsUsbPresence::new(root.path());
            let mut follow_loop =
                FollowLoop::new(presence, follow.clone(), Ok(osd), output, Box::new(Inline));

            tauri::async_runtime::block_on(ask_to_learn(app.handle()));
            let reads = reads(&[(&[&HUB, &KEYBOARD, &MOUSE], 1), (&[&HUB], 3)]);
            let clock = ScriptedClock::new(&root, &reads);
            follow_loop.run(&clock, clock.keep_going());

            let learned = FollowConfig::default()
                .with_learned(keyboard_and_mouse(), MonitorId::new(recorded));
            assert_eq!(follow.config(), learned, "{selected:?}");
            assert_eq!(store.load(), LoadOutcome::Loaded(learned));
            assert_eq!(writes(&backend), []);
        }
    }

    #[test]
    fn learning_without_a_monitor_asks_nothing_of_the_loop() {
        let home = TempDir::new().unwrap();
        let (app, follow, store) = app_following(FollowConfig::default(), &home);
        let (osd, _backend) = osd_with([]);
        let osd: SharedOsd = Arc::new(osd);
        app.manage(AppState::new(Ok(osd.clone())));
        let root = FakeRoot::new();
        let output = Recorder::new();
        let presence = SysfsUsbPresence::new(root.path());
        let mut follow_loop =
            FollowLoop::new(presence, follow, Ok(osd), output.clone(), Box::new(Inline));

        tauri::async_runtime::block_on(ask_to_learn(app.handle()));
        let reads = reads(&[(&[&HUB, &KEYBOARD, &MOUSE], 1), (&[&HUB], 3)]);
        let clock = ScriptedClock::new(&root, &reads);
        follow_loop.run(&clock, clock.keep_going());

        assert_eq!(output.lines(), Vec::<String>::new());
        assert_eq!(store.load(), LoadOutcome::Missing);
    }

    #[test]
    fn without_the_follow_set_up_the_menu_reads_off_and_a_click_changes_nothing() {
        let app = mock_app();

        assert_eq!(follow_config(app.handle()), FollowConfig::default());
        flip_follow(app.handle());
        pick_follow_input(app.handle(), 0x10);
        tauri::async_runtime::block_on(ask_to_learn(app.handle()));
        assert!(app.try_state::<SharedFollow>().is_none());

        let home = TempDir::new().unwrap();
        let (app, follow, _store) = app_following(FollowConfig::default(), &home);
        tauri::async_runtime::block_on(ask_to_learn(app.handle()));
        assert_eq!(follow.config(), FollowConfig::default());
    }
}
