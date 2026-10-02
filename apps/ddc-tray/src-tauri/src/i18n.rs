//! The tray menu's words in the languages the app speaks
//! (D-2026-09-26-tray-app-6). The popup translates itself in the webview;
//! the tray menu is native, so its few labels live here, one [`Labels`]
//! per locale — a locale missing a label does not compile.

use std::collections::BTreeSet;

use ddc_core::domain::UsbDeviceId;

/// A language the tray speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// English, the fallback.
    En,
    /// Brazilian Portuguese.
    PtBr,
}

/// Every label the tray shows, in one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Labels {
    /// Hover text of the tray icon.
    pub tooltip: &'static str,
    /// Menu item that shows the popup.
    pub open_panel: &'static str,
    /// Word before the percentage of a brightness shortcut.
    pub brightness: &'static str,
    /// Checkable menu item that starts the app with the system.
    pub autostart: &'static str,
    /// Menu item that ends the app.
    pub quit: &'static str,
    /// What the wheel over the icon does, in the tooltip of the
    /// StatusNotifierItem.
    pub scroll_hint: &'static str,
    /// Checkable menu item that turns the USB switch follow on.
    pub follow: &'static str,
    /// Menu item that learns the devices the USB switch moves.
    pub learn: &'static str,
    /// Word before the devices learned, in the menu's summary line.
    pub learned: &'static str,
    /// What the summary line says when nothing is learned.
    pub nothing_learned: &'static str,
}

const EN: Labels = Labels {
    tooltip: "DDC Control — monitor settings",
    open_panel: "Open panel",
    brightness: "Brightness",
    autostart: "Start with system",
    quit: "Quit",
    scroll_hint: "Scroll to change the brightness",
    follow: "Follow USB switch",
    learn: "Learn USB switch",
    learned: "Learned",
    nothing_learned: "none",
};

const PT_BR: Labels = Labels {
    tooltip: "DDC Control — ajustes do monitor",
    open_panel: "Abrir painel",
    brightness: "Brilho",
    autostart: "Iniciar com o sistema",
    quit: "Sair",
    scroll_hint: "Role para mudar o brilho",
    follow: "Seguir o switch USB",
    learn: "Aprender o switch USB",
    learned: "Aprendidos",
    nothing_learned: "nenhum",
};

impl Locale {
    /// The locale for a language tag such as `pt-BR`, `pt_BR.UTF-8` or
    /// `en-US`: any Portuguese tag gets pt-BR, anything else English — the
    /// same rule as the popup's `resolveLocale`.
    pub fn from_tag(tag: &str) -> Self {
        if tag.to_ascii_lowercase().starts_with("pt") {
            Self::PtBr
        } else {
            Self::En
        }
    }

    /// The tray's labels in this locale.
    pub fn labels(self) -> &'static Labels {
        match self {
            Self::En => &EN,
            Self::PtBr => &PT_BR,
        }
    }

    /// The label of the shortcut that sets the brightness to `percent`.
    pub fn brightness_label(self, percent: u8) -> String {
        format!("{} {percent}%", self.labels().brightness)
    }

    /// The menu's summary of what was learned from the USB switch: the
    /// vendor and product ids of each device, or that there is none.
    pub fn learned_summary(self, devices: &BTreeSet<UsbDeviceId>) -> String {
        let labels = self.labels();
        let names: Vec<String> = devices.iter().map(UsbDeviceId::vendor_product).collect();
        let listed = if names.is_empty() {
            labels.nothing_learned.to_owned()
        } else {
            names.join(", ")
        };
        format!("{}: {listed}", labels.learned)
    }

    /// The text under the tooltip's title: the monitor the tray acts on,
    /// when the popup selected one, and what the wheel does.
    pub fn tooltip_description(self, monitor: Option<&str>) -> String {
        let hint = self.labels().scroll_hint;
        match monitor {
            Some(monitor) => format!("{monitor} · {hint}"),
            None => hint.to_owned(),
        }
    }
}

/// The name of the input `code` of `0x60` in the follow's menu: the
/// standard MCCS names, brand names the same in every locale, else the code.
pub fn input_label(code: u8) -> String {
    match code {
        0x0F => "DisplayPort 1".to_owned(),
        0x10 => "DisplayPort 2".to_owned(),
        0x11 => "HDMI 1".to_owned(),
        0x12 => "HDMI 2".to_owned(),
        other => format!("0x{other:02X}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Labels, Locale, input_label};

    #[test]
    fn portuguese_tags_get_brazilian_portuguese() {
        for tag in ["pt-BR", "pt_BR.UTF-8", "pt", "PT-br", "pt-PT"] {
            assert_eq!(Locale::from_tag(tag), Locale::PtBr, "{tag}");
        }
    }

    #[test]
    fn any_other_tag_gets_english() {
        for tag in ["en-US", "en", "C", "POSIX", "de-DE", "es_ES.UTF-8", ""] {
            assert_eq!(Locale::from_tag(tag), Locale::En, "{tag}");
        }
    }

    #[test]
    fn english_labels_are_exact() {
        assert_eq!(
            *Locale::En.labels(),
            Labels {
                tooltip: "DDC Control — monitor settings",
                open_panel: "Open panel",
                brightness: "Brightness",
                autostart: "Start with system",
                quit: "Quit",
                scroll_hint: "Scroll to change the brightness",
                follow: "Follow USB switch",
                learn: "Learn USB switch",
                learned: "Learned",
                nothing_learned: "none",
            }
        );
    }

    #[test]
    fn brazilian_portuguese_labels_are_exact() {
        assert_eq!(
            *Locale::PtBr.labels(),
            Labels {
                tooltip: "DDC Control — ajustes do monitor",
                open_panel: "Abrir painel",
                brightness: "Brilho",
                autostart: "Iniciar com o sistema",
                quit: "Sair",
                scroll_hint: "Role para mudar o brilho",
                follow: "Seguir o switch USB",
                learn: "Aprender o switch USB",
                learned: "Aprendidos",
                nothing_learned: "nenhum",
            }
        );
    }

    #[test]
    fn a_brightness_label_carries_the_percentage() {
        assert_eq!(Locale::En.brightness_label(25), "Brightness 25%");
        assert_eq!(Locale::PtBr.brightness_label(0), "Brilho 0%");
        assert_eq!(Locale::PtBr.brightness_label(100), "Brilho 100%");
    }

    #[test]
    fn the_tooltip_names_the_selected_monitor_and_the_wheel() {
        assert_eq!(
            Locale::En.tooltip_description(Some("RTK QHD HDR")),
            "RTK QHD HDR · Scroll to change the brightness"
        );
        assert_eq!(
            Locale::PtBr.tooltip_description(Some("RTK QHD HDR")),
            "RTK QHD HDR · Role para mudar o brilho"
        );
        assert_eq!(
            Locale::PtBr.tooltip_description(None),
            "Role para mudar o brilho"
        );
    }

    #[test]
    fn the_autostart_label_is_set_in_every_locale_and_differs_between_them() {
        let english = Locale::En.labels().autostart;
        let portuguese = Locale::PtBr.labels().autostart;

        assert_eq!(english, "Start with system");
        assert_eq!(portuguese, "Iniciar com o sistema");
        assert_ne!(english, portuguese);
    }

    #[test]
    fn no_label_is_empty_in_any_locale() {
        for locale in [Locale::En, Locale::PtBr] {
            let Labels {
                tooltip,
                open_panel,
                brightness,
                autostart,
                quit,
                scroll_hint,
                follow,
                learn,
                learned,
                nothing_learned,
            } = *locale.labels();
            for label in [
                tooltip,
                open_panel,
                brightness,
                autostart,
                quit,
                scroll_hint,
                follow,
                learn,
                learned,
                nothing_learned,
            ] {
                assert!(!label.trim().is_empty(), "{locale:?}");
            }
        }
    }

    #[test]
    fn follow_labels_are_literal_in_en_and_pt_br() {
        let (en, pt) = (Locale::En.labels(), Locale::PtBr.labels());
        assert_eq!(
            (en.follow, pt.follow),
            ("Follow USB switch", "Seguir o switch USB")
        );
        assert_eq!(
            (en.learn, pt.learn),
            ("Learn USB switch", "Aprender o switch USB")
        );
        assert_eq!(input_label(0x0F), "DisplayPort 1");
        assert_eq!(input_label(0x10), "DisplayPort 2");
        assert_eq!(input_label(0x11), "HDMI 1");
        assert_eq!(input_label(0x12), "HDMI 2");
        assert_eq!(input_label(0x13), "0x13");

        let learned: BTreeSet<_> = ["046d:c31c:KB0001", "046d:c077"]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect();
        assert_eq!(
            Locale::En.learned_summary(&learned),
            "Learned: 046d:c077, 046d:c31c"
        );
        assert_eq!(
            Locale::PtBr.learned_summary(&learned),
            "Aprendidos: 046d:c077, 046d:c31c"
        );
        assert_eq!(
            Locale::En.learned_summary(&BTreeSet::new()),
            "Learned: none"
        );
        assert_eq!(
            Locale::PtBr.learned_summary(&BTreeSet::new()),
            "Aprendidos: nenhum"
        );
    }
}
