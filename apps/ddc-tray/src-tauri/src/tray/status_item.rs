//! The tray icon on Linux (D-2026-09-27-tray-app-2): a StatusNotifierItem
//! of the app's own, over `ksni`. The AppIndicator behind Tauri's tray icon
//! only shows a menu there; this item also hears the host's `Activate` — a
//! left click, which toggles the popup — and `Scroll` — the wheel, which
//! steps the brightness ([`crate::scroll`]). The host draws its menu on a
//! right click.
//!
//! ksni calls the item on its own tasks, off the main thread: the popup is
//! toggled on the main thread, and DDC/CI runs on a blocking thread.

use std::sync::Arc;

use ksni::menu::StandardItem;
use ksni::{Category, Handle, Icon, MenuItem, Orientation, ToolTip, TrayMethods};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::{
    APP_NAME, TRAY_ID, brightness_changed, kwin_placement, on_main_thread, report_ui,
    run_menu_action, toggle_popup,
};
use crate::commands::{AppState, on_blocking_thread};
use crate::i18n::Locale;
use crate::menu::{MenuEntry, Platform, menu_entries};
use crate::scroll::{Push, WheelQueue, drain_wheel};
use crate::{diagnose, report};

/// The running item, kept to refresh its tooltip.
struct ItemHandle<R: Runtime>(Handle<StatusItem<R>>);

/// The app's StatusNotifierItem.
struct StatusItem<R: Runtime> {
    app: AppHandle<R>,
    locale: Locale,
    icon: Icon,
    wheel: Arc<WheelQueue>,
}

/// Registers the item in the background. It waits for a host that comes
/// up after the app, as at login; a D-Bus failure is only reported. On
/// KDE Plasma under Wayland, also has KWin place the popup.
pub(super) fn install<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<()> {
    tauri::async_runtime::spawn(kwin_placement::install());
    let item = StatusItem {
        app: app.clone(),
        locale,
        icon: tray_icon(),
        wheel: Arc::new(WheelQueue::new()),
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match item.assume_sni_available(true).spawn().await {
            Ok(handle) => {
                app.manage(ItemHandle(handle));
                diagnose("tray item started");
            }
            Err(error) => report("show the tray icon", &error),
        }
    });
    Ok(())
}

/// Unloads the popup's placement from KWin; the item goes with the process.
pub(super) fn uninstall<R: Runtime>(_app: &AppHandle<R>) {
    tauri::async_runtime::block_on(kwin_placement::uninstall());
}

/// Asks the host to read the tooltip again: it names the monitor selected.
pub(super) fn selection_changed<R: Runtime>(app: &AppHandle<R>) {
    let Some(item) = app.try_state::<ItemHandle<R>>() else {
        return;
    };
    let handle = item.0.clone();
    tauri::async_runtime::spawn(async move {
        handle.update(|_| ()).await;
    });
}

/// Wayland lets no app place its window: KWin's script places the popup
/// when it is mapped (see `kwin_placement`); elsewhere it opens where the
/// compositor puts it.
pub(super) fn place_popup<R: Runtime>(_popup: &WebviewWindow<R>) {}

impl<R: Runtime> ksni::Tray for StatusItem<R> {
    fn id(&self) -> String {
        TRAY_ID.to_owned()
    }

    fn title(&self) -> String {
        APP_NAME.to_owned()
    }

    fn category(&self) -> Category {
        Category::Hardware
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![self.icon.clone()]
    }

    fn tool_tip(&self) -> ToolTip {
        let monitor = self
            .app
            .try_state::<AppState>()
            .and_then(|state| state.selected_label());
        ToolTip {
            title: APP_NAME.to_owned(),
            description: self.locale.tooltip_description(monitor.as_deref()),
            ..ToolTip::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        diagnose("tray activated");
        on_main_thread(&self.app, toggle_popup);
    }

    fn scroll(&mut self, delta: i32, orientation: Orientation) {
        if orientation == Orientation::Vertical && self.wheel.push(delta) == Push::Start {
            write_wheel(&self.app, Arc::clone(&self.wheel));
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        menu_entries(self.locale, Platform::Linux)
            .into_iter()
            .map(menu_item)
            .collect()
    }
}

fn menu_item<R: Runtime>(entry: MenuEntry) -> MenuItem<StatusItem<R>> {
    match entry {
        MenuEntry::Item { action, label } => StandardItem {
            label: menu_label(&label),
            activate: Box::new(move |item: &mut StatusItem<R>| run_menu_action(&item.app, action)),
            ..StandardItem::default()
        }
        .into(),
        MenuEntry::Separator => MenuItem::Separator,
    }
}

/// `label` as dbusmenu shows it verbatim: an underscore there marks an
/// access key, and a doubled one is a plain underscore.
fn menu_label(label: &str) -> String {
    label.replace('_', "__")
}

/// Writes the wheel's notches on a blocking thread, one write per batch,
/// and tells the popup after each write. When the core cannot be reached
/// at all, the notches are dropped and the next one tries again.
fn write_wheel<R: Runtime>(app: &AppHandle<R>, wheel: Arc<WheelQueue>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(state) = app.try_state::<AppState>() else {
            wheel.discard();
            return;
        };
        let selected = state.selected();
        let writer = Arc::clone(&wheel);
        let emitter = app.clone();
        let ran = on_blocking_thread(&state, move |osd| {
            drain_wheel(osd, selected, &writer, |batch| match batch {
                Ok(Some(change)) => brightness_changed(&emitter, &change),
                Ok(None) => {}
                Err(error) => report_ui("step the brightness", &error),
            });
            Ok(())
        })
        .await;
        if let Err(error) = ran {
            wheel.discard();
            report_ui("step the brightness", &error);
        }
    });
}

/// The app's tray icon as a StatusNotifierItem pixmap.
fn tray_icon() -> Icon {
    let image = tauri::include_image!("icons/tray.png");
    Icon {
        width: i32::try_from(image.width()).unwrap_or(0),
        height: i32::try_from(image.height()).unwrap_or(0),
        data: argb_from_rgba(image.rgba()),
    }
}

/// ARGB32 in network byte order, the StatusNotifierItem pixmap format, from
/// RGBA pixels.
fn argb_from_rgba(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[red, green, blue, alpha]| [alpha, red, green, blue])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{argb_from_rgba, menu_label, tray_icon};

    #[test]
    fn a_pixmap_moves_the_alpha_of_each_pixel_first() {
        let rgba = [0x11, 0x22, 0x33, 0xFF, 0xAA, 0xBB, 0xCC, 0x00];

        assert_eq!(
            argb_from_rgba(&rgba),
            [0xFF, 0x11, 0x22, 0x33, 0x00, 0xAA, 0xBB, 0xCC]
        );
    }

    #[test]
    fn the_tray_icon_is_the_64_px_tray_png_as_argb() {
        let icon = tray_icon();

        assert_eq!((icon.width, icon.height), (64, 64));
        assert_eq!(icon.data.len(), 64 * 64 * 4);
        let image = tauri::include_image!("icons/tray.png");
        assert_eq!(icon.data, argb_from_rgba(image.rgba()));
    }

    #[test]
    fn a_menu_label_keeps_its_underscores_visible() {
        assert_eq!(menu_label("Brilho 25%"), "Brilho 25%");
        assert_eq!(menu_label("HDMI_1"), "HDMI__1");
    }
}
