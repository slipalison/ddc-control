//! The tray icon where the notification area reports clicks (Windows):
//! Tauri's tray icon, whose left click toggles the popup anchored above it
//! by the positioner and whose right click opens the menu.

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime, WebviewWindow};
use tauri_plugin_positioner::{Position, WindowExt};

use super::{TRAY_ID, run_menu_action, toggle_popup};
use crate::i18n::Locale;
use crate::menu::{MenuAction, MenuEntry, Platform, menu_entries};
use crate::report;

/// Puts Tauri's tray icon in the notification area.
pub(super) fn install<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<()> {
    let menu = native_menu(app, &menu_entries(locale, Platform::Windows))?;
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

/// The tooltip here does not name the monitor: nothing to update.
pub(super) fn selection_changed<R: Runtime>(_app: &AppHandle<R>) {}

/// Places the popup above the icon, kept on the icon's screen.
pub(super) fn place_popup<R: Runtime>(popup: &WebviewWindow<R>) {
    if let Err(error) = popup.move_window_constrained(Position::TrayCenter) {
        report("anchor the popup to the tray icon", &error);
    }
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
    if let Some(action) = MenuAction::from_id(event.id().as_ref()) {
        run_menu_action(app, action);
    }
}

fn on_tray_icon_event<R: Runtime>(tray: &TrayIcon<R>, event: TrayIconEvent) {
    let app = tray.app_handle();
    tauri_plugin_positioner::on_tray_event(app, &event);
    if is_left_click(&event) {
        toggle_popup(app);
    }
}

#[cfg(test)]
mod tests {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent, TrayIconId};
    use tauri::{PhysicalPosition, Rect};

    use super::{TRAY_ID, is_left_click};

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
