//! The tray icon and its menu (D-2026-09-26-tray-app-5, -6): the icon, its
//! tooltip and the menu of [`crate::menu`] in the system's language; a left
//! click toggles the popup anchored to the icon where the desktop reports
//! clicks (Windows); a brightness shortcut sets the brightness of the
//! monitor the popup last selected, off the main thread.

use std::time::Instant;

use ddc_core::domain::MonitorId;
use ddc_core::ports::MonitorControl;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::commands::{AppState, on_blocking_thread, shortcut_target};
use crate::dto::{PANEL_CHANGED, PanelChangedDto, UiError};
use crate::i18n::Locale;
use crate::menu::{MenuAction, MenuEntry, Platform, menu_entries};
use crate::panel;
use crate::popup::{ClickAction, PopupGate, Visibility};
use crate::{POPUP, report, show_popup};

/// Id of the app's only tray icon.
pub const TRAY_ID: &str = "ddc-control";

/// Puts the icon in the tray, with the menu of this platform in the
/// system's language.
///
/// # Errors
///
/// The Tauri error when the menu or the icon cannot be created.
pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let locale = Locale::from_tag(&sys_locale::get_locale().unwrap_or_default());
    let menu = native_menu(app, &menu_entries(locale, Platform::current()))?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::include_image!("icons/tray.png"))
        .tooltip(locale.labels().tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;
    Ok(())
}

/// Sets the brightness of the shortcut target — the monitor the popup last
/// selected, else the first one listed — to `percent` of its maximum, and
/// names the monitor that changed. Brightness is a safe feature: nothing
/// here can confirm a dangerous write.
///
/// # Errors
///
/// The [`UiError`] of the enumeration, of the read of the maximum or of the
/// write.
pub fn brightness_shortcut<M: MonitorControl + ?Sized>(
    osd: &M,
    selected: Option<MonitorId>,
    percent: u8,
) -> Result<PanelChangedDto, UiError> {
    let id = shortcut_target(osd, selected)?;
    panel::set_brightness_percent(osd, &id, percent)?;
    Ok(PanelChangedDto {
        monitor_id: id.as_str().to_owned(),
    })
}

/// Whether `event` is the end of a left click on the icon — the gesture
/// that toggles the popup.
fn is_left_click(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}

fn native_menu<R: Runtime>(app: &AppHandle<R>, entries: &[MenuEntry]) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    for entry in entries {
        match entry {
            MenuEntry::Item { action, label } => {
                menu.append(&MenuItem::with_id(
                    app,
                    action.id(),
                    label,
                    true,
                    None::<&str>,
                )?)?;
            }
            MenuEntry::Separator => menu.append(&PredefinedMenuItem::separator(app)?)?,
        }
    }
    Ok(menu)
}

fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    match MenuAction::from_id(event.id().as_ref()) {
        Some(MenuAction::OpenPanel) => open_panel(app),
        Some(MenuAction::Brightness(percent)) => apply_brightness(app, percent),
        Some(MenuAction::Quit) => app.exit(0),
        None => {}
    }
}

fn on_tray_icon_event<R: Runtime>(tray: &TrayIcon<R>, event: TrayIconEvent) {
    let app = tray.app_handle();
    tauri_plugin_positioner::on_tray_event(app, &event);
    if is_left_click(&event) {
        toggle_popup(app);
    }
}

/// A left click hides an open popup and shows a hidden one — unless that
/// same click already hid it through the blur (see [`PopupGate`]).
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
        ClickAction::Hide => {
            if let Err(error) = popup.hide() {
                report("hide the popup", &error);
            }
        }
        ClickAction::Nothing => {}
    }
}

fn open_panel<R: Runtime>(app: &AppHandle<R>) {
    if let Some(popup) = app.get_webview_window(POPUP) {
        anchor_to_tray(&popup);
    }
    show_popup(app);
}

/// Places the popup above the icon, kept on the icon's screen. A
/// StatusNotifierItem host never reports the icon, and Wayland lets no app
/// place its window: there the popup opens where the compositor puts it,
/// and the failed anchoring is expected, not worth a line in the log.
fn anchor_to_tray<R: Runtime>(popup: &WebviewWindow<R>) {
    let anchored = popup.move_window_constrained(Position::TrayCenter);
    if let Err(error) = anchored
        && Platform::current() == Platform::Windows
    {
        report("anchor the popup to the tray icon", &error);
    }
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
        let emitted = match changed {
            Ok(changed) => app.emit(PANEL_CHANGED, changed),
            Err(error) => {
                report_ui("apply the brightness shortcut", &error);
                return;
            }
        };
        if let Err(error) = emitted {
            report("tell the popup the brightness changed", &error);
        }
    });
}

fn report_ui(action: &str, error: &UiError) {
    report(action, &format!("{:?}: {}", error.kind, error.message));
}

#[cfg(test)]
mod tests {
    use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend};
    use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent, TrayIconId};
    use tauri::{PhysicalPosition, Rect};

    use super::{TRAY_ID, brightness_shortcut, is_left_click};
    use crate::dto::{ErrorKind, PanelChangedDto, UiError};
    use crate::panel::tests::{RTK_ID, osd_with, rtk_id, rtk_monitor};

    const DELL_ID: &str = "DEL-U2720Q-7";

    fn dell_monitor() -> FakeMonitor {
        FakeMonitor::new(MonitorInfo {
            id: MonitorId::new(DELL_ID),
            manufacturer: Some("DEL".to_owned()),
            model: Some("U2720Q".to_owned()),
            serial: Some("7".to_owned()),
        })
        .with_value(VcpCode::BRIGHTNESS, 30, 100)
    }

    fn writes(backend: &InMemoryMonitorBackend) -> Vec<BackendCall> {
        backend
            .calls()
            .into_iter()
            .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
            .collect()
    }

    fn changed(id: &str) -> Result<PanelChangedDto, UiError> {
        Ok(PanelChangedDto {
            monitor_id: id.to_owned(),
        })
    }

    fn click(button: MouseButton, button_state: MouseButtonState) -> TrayIconEvent {
        TrayIconEvent::Click {
            id: TrayIconId::new(TRAY_ID),
            position: PhysicalPosition::new(10.0, 10.0),
            rect: Rect::default(),
            button,
            button_state,
        }
    }

    #[test]
    fn the_tray_icon_id_is_stable() {
        assert_eq!(TRAY_ID, "ddc-control");
    }

    #[test]
    fn a_shortcut_sets_the_selected_monitor_and_names_it() {
        let (osd, backend) = osd_with([rtk_monitor(), dell_monitor()]);

        let outcome = brightness_shortcut(&osd, Some(MonitorId::new(DELL_ID)), 75);

        assert_eq!(outcome, changed(DELL_ID));
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
        let (osd, backend) = osd_with([rtk_monitor(), dell_monitor()]);

        let outcome = brightness_shortcut(&osd, None, 25);

        assert_eq!(outcome, changed(RTK_ID));
        assert_eq!(
            writes(&backend),
            [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 25)]
        );
    }

    #[test]
    fn the_zero_and_full_shortcuts_reach_the_ends_of_the_range() {
        let (osd, backend) = osd_with([rtk_monitor()]);

        assert_eq!(brightness_shortcut(&osd, None, 0), changed(RTK_ID));
        assert_eq!(brightness_shortcut(&osd, None, 100), changed(RTK_ID));

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

    #[test]
    fn the_release_of_a_left_click_toggles_the_popup() {
        assert!(is_left_click(&click(
            MouseButton::Left,
            MouseButtonState::Up
        )));
    }

    #[test]
    fn a_press_or_another_button_does_not_toggle_the_popup() {
        for (button, state) in [
            (MouseButton::Left, MouseButtonState::Down),
            (MouseButton::Right, MouseButtonState::Up),
            (MouseButton::Right, MouseButtonState::Down),
            (MouseButton::Middle, MouseButtonState::Up),
        ] {
            assert!(
                !is_left_click(&click(button, state)),
                "{button:?} {state:?}"
            );
        }
    }

    #[test]
    fn hovering_or_a_double_click_does_not_toggle_the_popup() {
        let id = TrayIconId::new(TRAY_ID);
        let position = PhysicalPosition::new(10.0, 10.0);
        let rect = Rect::default();
        for event in [
            TrayIconEvent::Enter {
                id: id.clone(),
                position,
                rect,
            },
            TrayIconEvent::Move {
                id: id.clone(),
                position,
                rect,
            },
            TrayIconEvent::Leave {
                id: id.clone(),
                position,
                rect,
            },
            TrayIconEvent::DoubleClick {
                id: id.clone(),
                position,
                rect,
                button: MouseButton::Left,
            },
        ] {
            assert!(!is_left_click(&event), "{event:?}");
        }
    }
}
