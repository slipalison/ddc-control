//! "Start with system": whether the OS launches the app when the user logs
//! in (D-2026-09-30-input-switch-autostart-8, -9, -11).
//!
//! The state is the OS's own startup entry — `~/.config/autostart/*.desktop`
//! on Linux, `HKCU\...\Run` on Windows — read each time it is shown and never
//! kept in a config of the app. Nothing turns it on by itself: the entry is
//! written only when the user toggles it in the tray menu.

use std::fmt;

use tauri::{AppHandle, Builder, Manager, Runtime};
use tauri_plugin_autostart::AutoLaunchManager;

/// Whether the OS starts the app with the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Autostart {
    /// The startup entry exists.
    Enabled,
    /// There is no startup entry. What a fresh install has.
    Disabled,
}

impl Autostart {
    /// The state a toggle moves to.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Enabled => Self::Disabled,
            Self::Disabled => Self::Enabled,
        }
    }
}

/// Why the OS entry could not be read or changed, as the plugin's message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutostartError(String);

impl AutostartError {
    fn new(message: impl fmt::Display) -> Self {
        Self(message.to_string())
    }
}

impl fmt::Display for AutostartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AutostartError {}

/// The OS's startup entry for this app: the seam the menu reaches it
/// through, so the tray logic never needs a real `~/.config/autostart`.
pub trait AutostartEntry: Send + Sync {
    /// Whether the OS entry exists now. Only reads.
    ///
    /// # Errors
    ///
    /// The OS's refusal to tell.
    fn state(&self) -> Result<Autostart, AutostartError>;

    /// Makes the OS entry exist ([`Autostart::Enabled`]) or not
    /// ([`Autostart::Disabled`]). Removing an entry that is not there is not
    /// an error.
    ///
    /// # Errors
    ///
    /// The OS's refusal to write or remove the entry.
    fn set(&self, state: Autostart) -> Result<(), AutostartError>;
}

/// Flips the entry and answers the state the OS reports afterwards. The
/// caller reports the error: a failed flip leaves the entry as it was.
///
/// # Errors
///
/// Whatever the entry refused, reading or writing.
pub fn toggle(entry: &dyn AutostartEntry) -> Result<Autostart, AutostartError> {
    entry.set(entry.state()?.toggled())?;
    entry.state()
}

/// The state to show: the OS's, or [`Autostart::Disabled`] — with the reason
/// handed to `report` — when the OS will not tell.
pub fn state_or_report(
    entry: &dyn AutostartEntry,
    report: impl FnOnce(&str, &dyn fmt::Display),
) -> Autostart {
    entry.state().unwrap_or_else(|error| {
        report("read the start-with-system entry", &error);
        Autostart::Disabled
    })
}

/// [`toggle`], for a click: a failure goes to `report` and the answer is the
/// state the OS has after it, so the menu shows what is true.
pub fn toggle_or_report(
    entry: &dyn AutostartEntry,
    report: impl Fn(&str, &dyn fmt::Display),
) -> Autostart {
    match toggle(entry) {
        Ok(state) => state,
        Err(error) => {
            report("change the start-with-system entry", &error);
            state_or_report(entry, &report)
        }
    }
}

/// The plugin's manager as an [`AutostartEntry`].
#[derive(Debug, Clone)]
pub struct PluginEntry<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> PluginEntry<R> {
    /// The entry of the app behind `app`, which [`register`] must have set
    /// up.
    #[must_use]
    pub fn new(app: &AppHandle<R>) -> Self {
        Self { app: app.clone() }
    }

    fn with_manager<T>(
        &self,
        op: impl FnOnce(&AutoLaunchManager) -> Result<T, tauri_plugin_autostart::Error>,
    ) -> Result<T, AutostartError> {
        let manager = self
            .app
            .try_state::<AutoLaunchManager>()
            .ok_or_else(|| AutostartError::new("the autostart plugin is not registered"))?;
        op(&manager).map_err(AutostartError::new)
    }
}

impl<R: Runtime> AutostartEntry for PluginEntry<R> {
    fn state(&self) -> Result<Autostart, AutostartError> {
        self.with_manager(AutoLaunchManager::is_enabled)
            .map(|enabled| {
                if enabled {
                    Autostart::Enabled
                } else {
                    Autostart::Disabled
                }
            })
    }

    fn set(&self, state: Autostart) -> Result<(), AutostartError> {
        self.with_manager(|manager| match state {
            Autostart::Enabled => manager.enable(),
            Autostart::Disabled => manager.disable(),
        })
    }
}

/// Registers the autostart plugin on `builder`: the only place it is
/// configured. It takes no arguments, so the OS starts the app as it would
/// any other start, and the plugin answers nothing to the webview, which has
/// no permission for it.
#[must_use]
pub fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.plugin(tauri_plugin_autostart::Builder::new().build())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::Display;
    use std::sync::Mutex;

    use tauri::Manager;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    use tauri_plugin_autostart::AutoLaunchManager;

    use super::{Autostart, AutostartEntry, AutostartError, register, toggle, toggle_or_report};

    /// An entry held in memory, as the OS would hold it.
    struct FakeEntry {
        state: Mutex<Autostart>,
        refuses: Option<&'static str>,
    }

    impl FakeEntry {
        fn new(state: Autostart) -> Self {
            Self {
                state: Mutex::new(state),
                refuses: None,
            }
        }

        fn refusing(state: Autostart, reason: &'static str) -> Self {
            Self {
                refuses: Some(reason),
                ..Self::new(state)
            }
        }

        fn held(&self) -> Autostart {
            *self.state.lock().unwrap()
        }
    }

    impl AutostartEntry for FakeEntry {
        fn state(&self) -> Result<Autostart, AutostartError> {
            Ok(self.held())
        }

        fn set(&self, state: Autostart) -> Result<(), AutostartError> {
            if let Some(reason) = self.refuses {
                return Err(AutostartError::new(reason));
            }
            *self.state.lock().unwrap() = state;
            Ok(())
        }
    }

    #[test]
    fn toggle_enables_a_disabled_entry() {
        let entry = FakeEntry::new(Autostart::Disabled);

        assert_eq!(toggle(&entry), Ok(Autostart::Enabled));
        assert_eq!(entry.held(), Autostart::Enabled);
    }

    #[test]
    fn toggle_disables_an_enabled_entry() {
        let entry = FakeEntry::new(Autostart::Enabled);

        assert_eq!(toggle(&entry), Ok(Autostart::Disabled));
        assert_eq!(entry.held(), Autostart::Disabled);
    }

    #[test]
    fn a_failed_toggle_is_reported_and_leaves_the_os_state_as_it_was() {
        for before in [Autostart::Disabled, Autostart::Enabled] {
            let entry = FakeEntry::refusing(before, "permission denied");
            let reports = RefCell::new(Vec::new());
            let report = |action: &str, error: &dyn Display| {
                reports.borrow_mut().push(format!("{action}: {error}"));
            };

            assert_eq!(
                toggle(&entry),
                Err(AutostartError::new("permission denied"))
            );
            assert_eq!(toggle_or_report(&entry, report), before);

            assert_eq!(entry.held(), before);
            assert_eq!(
                *reports.borrow(),
                ["change the start-with-system entry: permission denied"]
            );
        }
    }

    #[test]
    fn the_app_builder_registers_the_autostart_plugin() {
        let app = register(mock_builder())
            .build(mock_context(noop_assets()))
            .unwrap();

        assert!(app.try_state::<AutoLaunchManager>().is_some());
    }
}
