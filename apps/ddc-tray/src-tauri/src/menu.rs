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
//! entry (D-2026-09-30-input-switch-autostart-9, -10). On Linux the menu
//! also carries the USB switch follow: its check, the item that learns the
//! switch, the input to switch to and a line saying what was learned
//! (D-2026-10-02-usb-switch-follow-5).

use crate::autostart::Autostart;
use crate::follow_config::{FOLLOW_INPUTS, FollowConfig};
use crate::i18n::{Locale, input_label};

/// The brightness shortcuts, in percent of the monitor's maximum.
pub const BRIGHTNESS_STEPS: [u8; 5] = [0, 25, 50, 75, 100];

const OPEN_PANEL_ID: &str = "open-panel";
const AUTOSTART_ID: &str = "autostart";
const QUIT_ID: &str = "quit";
const BRIGHTNESS_ID_PREFIX: &str = "brightness-";
const FOLLOW_ID: &str = "follow-usb";
const LEARN_ID: &str = "learn-usb";
const FOLLOW_INPUT_ID_PREFIX: &str = "follow-input-";

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
    /// Turn the USB switch follow on or off.
    Follow,
    /// Learn the devices the USB switch moves.
    Learn,
    /// Make this code of `0x60` the input the follow switches to.
    FollowInput(u8),
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
            Self::Follow => FOLLOW_ID.to_owned(),
            Self::Learn => LEARN_ID.to_owned(),
            Self::FollowInput(code) => format!("{FOLLOW_INPUT_ID_PREFIX}{code:02x}"),
            Self::Quit => QUIT_ID.to_owned(),
        }
    }

    /// The action of the menu item `id`; `None` for an id the tray menu
    /// never shows, a brightness off the [`BRIGHTNESS_STEPS`] or an input
    /// off the [`FOLLOW_INPUTS`] included.
    pub fn from_id(id: &str) -> Option<Self> {
        [
            Self::OpenPanel,
            Self::Autostart,
            Self::Follow,
            Self::Learn,
            Self::Quit,
        ]
        .into_iter()
        .chain(BRIGHTNESS_STEPS.map(Self::Brightness))
        .chain(FOLLOW_INPUTS.map(Self::FollowInput))
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
    /// A line the user reads but cannot pick.
    Note {
        /// What it reads, in the menu's locale.
        label: String,
    },
    /// A divider between groups of items.
    Separator,
}

/// The tray menu, top to bottom, for `platform` in `locale`; the mark of
/// "Start with system" follows `autostart`, the OS's own state, and the
/// follow's items follow `follow`, its settings.
pub fn menu_entries(
    locale: Locale,
    platform: Platform,
    autostart: Autostart,
    follow: &FollowConfig,
) -> Vec<MenuEntry> {
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
    if platform == Platform::Linux {
        entries.push(MenuEntry::Separator);
        entries.extend(follow_entries(locale, follow));
        entries.push(MenuEntry::Separator);
    }
    entries.push(item(MenuAction::Quit, labels.quit));
    entries
}

/// The follow's items: its check, marked exactly when it is on, the item
/// that learns the switch, a check per input with the recorded one marked,
/// and a line saying what was learned.
fn follow_entries(locale: Locale, follow: &FollowConfig) -> Vec<MenuEntry> {
    let labels = locale.labels();
    let mut entries = vec![
        MenuEntry::Check {
            action: MenuAction::Follow,
            label: labels.follow.to_owned(),
            checked: follow.enabled,
        },
        item(MenuAction::Learn, labels.learn),
    ];
    entries.extend(FOLLOW_INPUTS.map(|code| MenuEntry::Check {
        action: MenuAction::FollowInput(code),
        label: input_label(code),
        checked: follow.target_input == Some(code),
    }));
    entries.push(MenuEntry::Note {
        label: locale.learned_summary(&follow.devices),
    });
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
    use crate::follow_config::FollowConfig;
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

    fn check(action: MenuAction, label: &str, checked: bool) -> MenuEntry {
        MenuEntry::Check {
            action,
            label: label.to_owned(),
            checked,
        }
    }

    fn note(label: &str) -> MenuEntry {
        MenuEntry::Note {
            label: label.to_owned(),
        }
    }

    fn actions(entries: &[MenuEntry]) -> Vec<MenuAction> {
        entries
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Item { action, .. } | MenuEntry::Check { action, .. } => Some(*action),
                MenuEntry::Note { .. } | MenuEntry::Separator => None,
            })
            .collect()
    }

    /// Settings with the keyboard and the mouse learned, the RTK to switch
    /// to DisplayPort 2, and the follow on as `enabled` says.
    fn followed(enabled: bool) -> FollowConfig {
        FollowConfig {
            enabled,
            devices: ["046d:c31c:KB0001", "046d:c077"]
                .iter()
                .map(|text| text.parse().unwrap())
                .collect(),
            monitor_id: Some(crate::fixture::rtk_id()),
            target_input: Some(0x10),
        }
    }

    /// The entries of the menu for `platform` with `follow` as the settings.
    fn entries(platform: Platform, follow: &FollowConfig) -> Vec<MenuEntry> {
        menu_entries(Locale::En, platform, Autostart::Disabled, follow)
    }

    #[test]
    fn the_brightness_steps_are_the_five_quarters() {
        assert_eq!(BRIGHTNESS_STEPS, [0, 25, 50, 75, 100]);
    }

    #[test]
    fn windows_offers_open_panel_start_with_system_and_quit() {
        assert_eq!(
            menu_entries(
                Locale::En,
                Platform::Windows,
                Autostart::Disabled,
                &FollowConfig::default()
            ),
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
            menu_entries(
                Locale::PtBr,
                Platform::Windows,
                Autostart::Disabled,
                &FollowConfig::default()
            ),
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
            menu_entries(
                Locale::En,
                Platform::Linux,
                Autostart::Disabled,
                &FollowConfig::default()
            ),
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
                MenuEntry::Separator,
                check(MenuAction::Follow, "Follow USB switch", false),
                item(MenuAction::Learn, "Learn USB switch"),
                check(MenuAction::FollowInput(0x0F), "DisplayPort 1", false),
                check(MenuAction::FollowInput(0x10), "DisplayPort 2", false),
                check(MenuAction::FollowInput(0x11), "HDMI 1", false),
                check(MenuAction::FollowInput(0x12), "HDMI 2", false),
                note("Learned: none"),
                MenuEntry::Separator,
                item(MenuAction::Quit, "Quit"),
            ]
        );
    }

    #[test]
    fn linux_speaks_brazilian_portuguese() {
        assert_eq!(
            menu_entries(
                Locale::PtBr,
                Platform::Linux,
                Autostart::Disabled,
                &FollowConfig::default()
            ),
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
                MenuEntry::Separator,
                check(MenuAction::Follow, "Seguir o switch USB", false),
                item(MenuAction::Learn, "Aprender o switch USB"),
                check(MenuAction::FollowInput(0x0F), "DisplayPort 1", false),
                check(MenuAction::FollowInput(0x10), "DisplayPort 2", false),
                check(MenuAction::FollowInput(0x11), "HDMI 1", false),
                check(MenuAction::FollowInput(0x12), "HDMI 2", false),
                note("Aprendidos: nenhum"),
                MenuEntry::Separator,
                item(MenuAction::Quit, "Sair"),
            ]
        );
    }

    #[test]
    fn the_autostart_item_is_checked_exactly_when_the_os_entry_exists_on_both_platforms() {
        for platform in [Platform::Windows, Platform::Linux] {
            for (state, checked) in [(Autostart::Enabled, true), (Autostart::Disabled, false)] {
                let marks: Vec<bool> = menu_entries(Locale::En, platform, state, &followed(true))
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
            let entries = menu_entries(Locale::En, platform, Autostart::Enabled, &followed(true));
            for action in actions(&entries) {
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
            &followed(true),
        ))
        .into_iter()
        .map(MenuAction::id)
        .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 14);
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
            "follow",
            "follow-usb-",
            "learn",
            "follow-input-",
            "follow-input-0F",
            "follow-input-13",
            "follow-input-0e",
            "follow-input-60",
            "follow-input-f",
            "follow-input-010",
        ] {
            assert_eq!(MenuAction::from_id(id), None, "{id}");
        }
    }

    #[test]
    fn follow_check_is_marked_exactly_when_enabled() {
        for enabled in [true, false] {
            let marks: Vec<(MenuAction, bool)> = entries(Platform::Linux, &followed(enabled))
                .into_iter()
                .filter_map(|entry| match entry {
                    MenuEntry::Check {
                        action: action @ (MenuAction::Follow | MenuAction::FollowInput(_)),
                        checked,
                        ..
                    } => Some((action, checked)),
                    _ => None,
                })
                .collect();

            assert_eq!(
                marks,
                [
                    (MenuAction::Follow, enabled),
                    (MenuAction::FollowInput(0x0F), false),
                    (MenuAction::FollowInput(0x10), true),
                    (MenuAction::FollowInput(0x11), false),
                    (MenuAction::FollowInput(0x12), false),
                ],
                "enabled: {enabled}"
            );
        }
    }

    #[test]
    fn follow_items_are_in_the_linux_menu() {
        let menu = entries(Platform::Linux, &followed(true));

        assert_eq!(
            menu[9..],
            [
                MenuEntry::Separator,
                check(MenuAction::Follow, "Follow USB switch", true),
                item(MenuAction::Learn, "Learn USB switch"),
                check(MenuAction::FollowInput(0x0F), "DisplayPort 1", false),
                check(MenuAction::FollowInput(0x10), "DisplayPort 2", true),
                check(MenuAction::FollowInput(0x11), "HDMI 1", false),
                check(MenuAction::FollowInput(0x12), "HDMI 2", false),
                note("Learned: 046d:c077, 046d:c31c"),
                MenuEntry::Separator,
                item(MenuAction::Quit, "Quit"),
            ]
        );
        assert_eq!(menu[8], autostart_item("Start with system", false));
    }

    #[test]
    fn follow_items_are_not_in_the_windows_menu() {
        for follow in [FollowConfig::default(), followed(true)] {
            assert_eq!(
                entries(Platform::Windows, &follow),
                vec![
                    item(MenuAction::OpenPanel, "Open panel"),
                    MenuEntry::Separator,
                    autostart_item("Start with system", false),
                    item(MenuAction::Quit, "Quit"),
                ]
            );
        }
    }

    #[test]
    fn follow_ids_round_trip_through_from_id() {
        let follow_actions = [
            (MenuAction::Follow, "follow-usb"),
            (MenuAction::Learn, "learn-usb"),
            (MenuAction::FollowInput(0x0F), "follow-input-0f"),
            (MenuAction::FollowInput(0x10), "follow-input-10"),
            (MenuAction::FollowInput(0x11), "follow-input-11"),
            (MenuAction::FollowInput(0x12), "follow-input-12"),
        ];
        for (action, id) in follow_actions {
            assert_eq!(action.id(), id);
            assert_eq!(MenuAction::from_id(id), Some(action));
        }
        for refused in [0x00, 0x0E, 0x13, 0x60] {
            assert_eq!(
                MenuAction::from_id(&MenuAction::FollowInput(refused).id()),
                None
            );
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
