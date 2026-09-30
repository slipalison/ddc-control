//! What the tray menu holds on each platform, and what each item does
//! (D-2026-09-26-tray-app-5). Pure: the tray builds the native menu from
//! [`menu_entries`] and maps a clicked item back with
//! [`MenuAction::from_id`].
//!
//! On Windows the menu offers "Open panel", "Start with system" and "Quit".
//! On Linux the StatusNotifierItem's menu also carries the brightness
//! shortcuts (D-2026-09-27-tray-app-2): there the icon is a status item
//! whose menu the desktop draws, next to its click and wheel. "Start with
//! system" is a checkable item, marked exactly when the OS has the startup
//! entry (D-2026-09-30-input-switch-autostart-9, -10).

use crate::autostart::Autostart;
use crate::i18n::Locale;

/// The brightness shortcuts, in percent of the monitor's maximum.
pub const BRIGHTNESS_STEPS: [u8; 5] = [0, 25, 50, 75, 100];

const OPEN_PANEL_ID: &str = "open-panel";
const AUTOSTART_ID: &str = "autostart";
const QUIT_ID: &str = "quit";
const BRIGHTNESS_ID_PREFIX: &str = "brightness-";

/// The desktop the tray runs on, as far as the menu cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// The notification area, through Tauri's tray icon.
    Windows,
    /// A StatusNotifierItem host (KDE Plasma, GNOME with the AppIndicator
    /// extension), through the app's own item.
    Linux,
}

impl Platform {
    /// The platform this binary was built for.
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// What a tray menu item does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Show the popup.
    OpenPanel,
    /// Set the brightness of the shortcut target to this percentage.
    Brightness(u8),
    /// Turn "Start with system" on or off.
    Autostart,
    /// End the app.
    Quit,
}

impl MenuAction {
    /// The id of the menu item, stable across locales.
    pub fn id(self) -> String {
        match self {
            Self::OpenPanel => OPEN_PANEL_ID.to_owned(),
            Self::Brightness(percent) => format!("{BRIGHTNESS_ID_PREFIX}{percent}"),
            Self::Autostart => AUTOSTART_ID.to_owned(),
            Self::Quit => QUIT_ID.to_owned(),
        }
    }

    /// The action of the menu item `id`; `None` for an id the tray menu
    /// never shows, a brightness off the [`BRIGHTNESS_STEPS`] included.
    pub fn from_id(id: &str) -> Option<Self> {
        [Self::OpenPanel, Self::Autostart, Self::Quit]
            .into_iter()
            .chain(BRIGHTNESS_STEPS.map(Self::Brightness))
            .find(|action| action.id() == id)
    }
}

/// A line of the tray menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    /// An item the user can pick.
    Item {
        /// What picking it does.
        action: MenuAction,
        /// What it reads, in the menu's locale.
        label: String,
    },
    /// An item the user can pick, with a mark that says whether what it
    /// controls is on.
    Check {
        /// What picking it does.
        action: MenuAction,
        /// What it reads, in the menu's locale.
        label: String,
        /// Whether the mark is on.
        checked: bool,
    },
    /// A divider between groups of items.
    Separator,
}

/// The tray menu, top to bottom, for `platform` in `locale`; the mark of
/// "Start with system" follows `autostart`, the OS's own state.
pub fn menu_entries(locale: Locale, platform: Platform, autostart: Autostart) -> Vec<MenuEntry> {
    let labels = locale.labels();
    let mut entries = vec![item(MenuAction::OpenPanel, labels.open_panel)];
    if platform == Platform::Linux {
        entries.push(MenuEntry::Separator);
        entries.extend(BRIGHTNESS_STEPS.map(|percent| {
            item(
                MenuAction::Brightness(percent),
                locale.brightness_label(percent),
            )
        }));
    }
    entries.push(MenuEntry::Separator);
    entries.push(MenuEntry::Check {
        action: MenuAction::Autostart,
        label: labels.autostart.to_owned(),
        checked: autostart == Autostart::Enabled,
    });
    entries.push(item(MenuAction::Quit, labels.quit));
    entries
}

fn item(action: MenuAction, label: impl Into<String>) -> MenuEntry {
    MenuEntry::Item {
        action,
        label: label.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{BRIGHTNESS_STEPS, MenuAction, MenuEntry, Platform, menu_entries};
    use crate::autostart::Autostart;
    use crate::i18n::Locale;

    fn item(action: MenuAction, label: &str) -> MenuEntry {
        MenuEntry::Item {
            action,
            label: label.to_owned(),
        }
    }

    fn autostart_item(label: &str, checked: bool) -> MenuEntry {
        MenuEntry::Check {
            action: MenuAction::Autostart,
            label: label.to_owned(),
            checked,
        }
    }

    fn actions(entries: &[MenuEntry]) -> Vec<MenuAction> {
        entries
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Item { action, .. } | MenuEntry::Check { action, .. } => Some(*action),
                MenuEntry::Separator => None,
            })
            .collect()
    }

    #[test]
    fn the_brightness_steps_are_the_five_quarters() {
        assert_eq!(BRIGHTNESS_STEPS, [0, 25, 50, 75, 100]);
    }

    #[test]
    fn windows_offers_open_panel_start_with_system_and_quit() {
        assert_eq!(
            menu_entries(Locale::En, Platform::Windows, Autostart::Disabled),
            vec![
                item(MenuAction::OpenPanel, "Open panel"),
                MenuEntry::Separator,
                autostart_item("Start with system", false),
                item(MenuAction::Quit, "Quit"),
            ]
        );
    }

    #[test]
    fn windows_speaks_brazilian_portuguese() {
        assert_eq!(
            menu_entries(Locale::PtBr, Platform::Windows, Autostart::Disabled),
            vec![
                item(MenuAction::OpenPanel, "Abrir painel"),
                MenuEntry::Separator,
                autostart_item("Iniciar com o sistema", false),
                item(MenuAction::Quit, "Sair"),
            ]
        );
    }

    #[test]
    fn linux_adds_the_brightness_shortcuts_between_open_and_start_with_system() {
        assert_eq!(
            menu_entries(Locale::En, Platform::Linux, Autostart::Disabled),
            vec![
                item(MenuAction::OpenPanel, "Open panel"),
                MenuEntry::Separator,
                item(MenuAction::Brightness(0), "Brightness 0%"),
                item(MenuAction::Brightness(25), "Brightness 25%"),
                item(MenuAction::Brightness(50), "Brightness 50%"),
                item(MenuAction::Brightness(75), "Brightness 75%"),
                item(MenuAction::Brightness(100), "Brightness 100%"),
                MenuEntry::Separator,
                autostart_item("Start with system", false),
                item(MenuAction::Quit, "Quit"),
            ]
        );
    }

    #[test]
    fn linux_speaks_brazilian_portuguese() {
        assert_eq!(
            menu_entries(Locale::PtBr, Platform::Linux, Autostart::Disabled),
            vec![
                item(MenuAction::OpenPanel, "Abrir painel"),
                MenuEntry::Separator,
                item(MenuAction::Brightness(0), "Brilho 0%"),
                item(MenuAction::Brightness(25), "Brilho 25%"),
                item(MenuAction::Brightness(50), "Brilho 50%"),
                item(MenuAction::Brightness(75), "Brilho 75%"),
                item(MenuAction::Brightness(100), "Brilho 100%"),
                MenuEntry::Separator,
                autostart_item("Iniciar com o sistema", false),
                item(MenuAction::Quit, "Sair"),
            ]
        );
    }

    #[test]
    fn the_autostart_item_is_checked_exactly_when_the_os_entry_exists_on_both_platforms() {
        for platform in [Platform::Windows, Platform::Linux] {
            for (state, checked) in [(Autostart::Enabled, true), (Autostart::Disabled, false)] {
                let marks: Vec<bool> = menu_entries(Locale::En, platform, state)
                    .into_iter()
                    .filter_map(|entry| match entry {
                        MenuEntry::Check {
                            action: MenuAction::Autostart,
                            checked,
                            ..
                        } => Some(checked),
                        _ => None,
                    })
                    .collect();

                assert_eq!(marks, [checked], "{platform:?} {state:?}");
            }
        }
    }

    #[test]
    fn the_ids_are_stable_and_locale_free() {
        assert_eq!(MenuAction::OpenPanel.id(), "open-panel");
        assert_eq!(MenuAction::Brightness(0).id(), "brightness-0");
        assert_eq!(MenuAction::Brightness(75).id(), "brightness-75");
        assert_eq!(MenuAction::Autostart.id(), "autostart");
        assert_eq!(MenuAction::Quit.id(), "quit");
    }

    #[test]
    fn the_autostart_action_maps_back_from_its_id() {
        assert_eq!(
            MenuAction::from_id(&MenuAction::Autostart.id()),
            Some(MenuAction::Autostart)
        );
    }

    #[test]
    fn every_menu_item_maps_back_from_its_id_on_both_platforms() {
        for platform in [Platform::Windows, Platform::Linux] {
            for action in actions(&menu_entries(Locale::En, platform, Autostart::Enabled)) {
                assert_eq!(MenuAction::from_id(&action.id()), Some(action));
            }
        }
    }

    #[test]
    fn the_ids_of_the_menu_are_unique() {
        let mut ids: Vec<String> = actions(&menu_entries(
            Locale::En,
            Platform::Linux,
            Autostart::Enabled,
        ))
        .into_iter()
        .map(MenuAction::id)
        .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn an_id_the_menu_never_shows_maps_to_nothing() {
        for id in [
            "",
            "open",
            "Quit",
            "Autostart",
            "autostart-",
            "brightness-",
            "brightness-10",
            "brightness-101",
            "brightness-256",
            "brightness--25",
            "brightness-25%",
            "brightness-025",
            "brightness-+25",
            " quit",
        ] {
            assert_eq!(MenuAction::from_id(id), None, "{id}");
        }
    }

    #[test]
    fn the_platform_is_the_build_target() {
        let expected = if cfg!(windows) {
            Platform::Windows
        } else {
            Platform::Linux
        };
        assert_eq!(Platform::current(), expected);
    }
}
