//! The USB switch follow's settings (D-2026-10-02-usb-switch-follow-4): the
//! devices learned from the switch, the monitor and the input to switch it
//! to, and whether the user turned the follow on. Kept in
//! `$XDG_CONFIG_HOME/ddc-control/usb-follow.json`, else under
//! `~/.config`, as versioned JSON. Only this module knows that format.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use ddc_core::domain::{MonitorId, UsbDeviceId};
use serde::{Deserialize, Serialize};

/// The inputs a follow can switch to: the standard MCCS codes of `0x60`
/// for DisplayPort 1 and 2 and HDMI 1 and 2. A fixed list, so nothing is
/// read from a monitor that may be on another input.
pub const FOLLOW_INPUTS: [u8; 4] = [0x0F, 0x10, 0x11, 0x12];

/// The file the settings live in, under the app's config directory.
pub const CONFIG_FILE: &str = "usb-follow.json";

/// The app's directory under the user's config directory.
const APP_DIR: &str = "ddc-control";

/// The format version this app reads and writes; another one is a format it
/// does not know, so it is neither read nor overwritten.
const VERSION: u64 = 1;

/// The follow's settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FollowConfig {
    /// Whether the user turned the follow on.
    pub enabled: bool,
    /// The devices learned from the switch.
    pub devices: BTreeSet<UsbDeviceId>,
    /// The monitor to switch, as it was when the switch was learned.
    pub monitor_id: Option<MonitorId>,
    /// The input to switch it to, one of [`FOLLOW_INPUTS`].
    pub target_input: Option<u8>,
}

/// A part of the settings the follow cannot run without.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// Nothing learned from the switch.
    Devices,
    /// No monitor recorded.
    Monitor,
    /// No input to switch to.
    TargetInput,
}

/// Why the follow cannot be turned on: what the settings lack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incomplete(pub Vec<Missing>);

impl fmt::Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Devices => "no learned devices",
            Self::Monitor => "no monitor",
            Self::TargetInput => "no target input",
        })
    }
}

impl fmt::Display for Incomplete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("learn the USB switch and pick the target input first (")?;
        for (index, missing) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{missing}")?;
        }
        f.write_str(")")
    }
}

impl std::error::Error for Incomplete {}

/// A code that is not one of [`FOLLOW_INPUTS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAFollowInput(pub u8);

impl fmt::Display for NotAFollowInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X} is not an input the follow switches to", self.0)
    }
}

impl std::error::Error for NotAFollowInput {}

/// The switch a fire makes: the monitor, and the input to put it on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The monitor recorded when the switch was learned.
    pub monitor_id: MonitorId,
    /// The input recorded, one of [`FOLLOW_INPUTS`].
    pub input: u8,
}

impl FollowConfig {
    /// What the follow still lacks to run, in the order the user sets it up.
    pub fn missing(&self) -> Vec<Missing> {
        [
            (self.devices.is_empty(), Missing::Devices),
            (self.monitor_id.is_none(), Missing::Monitor),
            (self.target_input.is_none(), Missing::TargetInput),
        ]
        .into_iter()
        .filter_map(|(lacking, part)| lacking.then_some(part))
        .collect()
    }

    /// The switch to make, only when the follow is on and complete: the one
    /// place a fire may get a monitor and an input from.
    pub fn active_target(&self) -> Option<Target> {
        if !self.enabled || self.devices.is_empty() {
            return None;
        }
        Some(Target {
            monitor_id: self.monitor_id.clone()?,
            input: self.target_input?,
        })
    }

    /// The settings after learning `devices` with `monitor` as the one to
    /// switch. Off whatever it was: learning never turns the follow on, and
    /// the user's consent covered the monitor recorded before.
    #[must_use]
    pub fn with_learned(&self, devices: BTreeSet<UsbDeviceId>, monitor: MonitorId) -> Self {
        Self {
            enabled: false,
            devices,
            monitor_id: Some(monitor),
            target_input: self.target_input,
        }
    }

    /// The settings with `code` as the input to switch to.
    ///
    /// # Errors
    ///
    /// [`NotAFollowInput`] for a code off [`FOLLOW_INPUTS`].
    pub fn with_target_input(&self, code: u8) -> Result<Self, NotAFollowInput> {
        if !FOLLOW_INPUTS.contains(&code) {
            return Err(NotAFollowInput(code));
        }
        Ok(Self {
            target_input: Some(code),
            ..self.clone()
        })
    }

    /// The settings with the follow turned off when it is on, and on when it
    /// is off and complete.
    ///
    /// # Errors
    ///
    /// [`Incomplete`], with what is missing, when turning it on.
    pub fn enabled_toggled(&self) -> Result<Self, Incomplete> {
        let missing = self.missing();
        if !self.enabled && !missing.is_empty() {
            return Err(Incomplete(missing));
        }
        Ok(Self {
            enabled: !self.enabled,
            ..self.clone()
        })
    }
}

/// The path of the settings: `$XDG_CONFIG_HOME/ddc-control/usb-follow.json`
/// when that variable is an absolute path, else
/// `$HOME/.config/ddc-control/usb-follow.json` when `HOME` is one; `None`
/// when neither is.
pub fn config_path(xdg_config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let base =
        absolute(xdg_config_home).or_else(|| absolute(home).map(|home| home.join(".config")))?;
    Some(base.join(APP_DIR).join(CONFIG_FILE))
}

/// `value` as a path, when it is an absolute one; the XDG spec has a
/// relative `XDG_CONFIG_HOME` ignored.
fn absolute(value: Option<&OsStr>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|path| path.is_absolute())
}

/// What reading the settings found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    /// No file: the follow is off and nothing is learned.
    Missing,
    /// The settings in the file.
    Loaded(FollowConfig),
    /// A file this app cannot read as its settings — an unknown version,
    /// invalid JSON, an invalid value —, and why. The follow stays off and
    /// the file is left as it is.
    NotConfigured(String),
}

/// Reads and writes the settings file at a path handed in.
#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    /// The store of the file at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Where the file is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the settings. Never writes anything.
    pub fn load(&self) -> LoadOutcome {
        match fs::read(&self.path) {
            Ok(bytes) => match parse(&bytes) {
                Ok(config) => LoadOutcome::Loaded(config),
                Err(reason) => LoadOutcome::NotConfigured(reason),
            },
            Err(error) if error.kind() == ErrorKind::NotFound => LoadOutcome::Missing,
            Err(error) => LoadOutcome::NotConfigured(error.to_string()),
        }
    }

    /// Writes `config`, creating the directory: into a temporary file next
    /// to the settings, then renamed over them, so a crash leaves either the
    /// old settings or the new ones.
    ///
    /// # Errors
    ///
    /// The I/O error of creating the directory, writing or renaming.
    pub fn save(&self, config: &FollowConfig) -> io::Result<()> {
        let dir = self.path.parent().ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                "the settings path has no directory",
            )
        })?;
        fs::create_dir_all(dir)?;
        let mut text = serde_json::to_string_pretty(&ConfigFile::from(config))?;
        text.push('\n');
        let temporary = dir.join(format!(".{CONFIG_FILE}.{}.tmp", std::process::id()));
        let written = write_synced(&temporary, text.as_bytes())
            .and_then(|()| fs::rename(&temporary, &self.path));
        if written.is_err() {
            // Best effort: the error that matters is the one returned.
            let _ = fs::remove_file(&temporary);
        }
        written
    }
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// The settings as the file holds them (format version 1): the ids as
/// text, the input in decimal, the monitor and the input left out until
/// they are known.
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: u64,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    devices: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    monitor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_input: Option<u16>,
}

impl From<&FollowConfig> for ConfigFile {
    fn from(config: &FollowConfig) -> Self {
        Self {
            version: VERSION,
            enabled: config.enabled,
            devices: config.devices.iter().map(ToString::to_string).collect(),
            monitor_id: config.monitor_id.as_ref().map(|id| id.as_str().to_owned()),
            target_input: config.target_input.map(u16::from),
        }
    }
}

/// The settings in `bytes`, or why they are not settings this app reads.
fn parse(bytes: &[u8]) -> Result<FollowConfig, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| format!("not valid JSON: {error}"))?;
    match value.get("version") {
        Some(version) if version.as_u64() == Some(VERSION) => {}
        Some(version) => return Err(format!("version {version} is not one this app reads")),
        None => return Err("no version".to_owned()),
    }
    let file: ConfigFile =
        serde_json::from_value(value).map_err(|error| format!("invalid settings: {error}"))?;
    let config = FollowConfig {
        enabled: file.enabled,
        devices: file
            .devices
            .iter()
            .map(|text| text.parse().map_err(|error| format!("{error}")))
            .collect::<Result<_, _>>()?,
        monitor_id: file.monitor_id.map(monitor_id).transpose()?,
        target_input: file.target_input.map(target_input).transpose()?,
    };
    if config.enabled && !config.missing().is_empty() {
        return Err(format!("enabled, but {}", Incomplete(config.missing())));
    }
    Ok(config)
}

fn monitor_id(text: String) -> Result<MonitorId, String> {
    if text.trim().is_empty() {
        return Err("an empty monitor id".to_owned());
    }
    Ok(MonitorId::new(text))
}

fn target_input(code: u16) -> Result<u8, String> {
    u8::try_from(code)
        .ok()
        .filter(|code| FOLLOW_INPUTS.contains(code))
        .ok_or_else(|| format!("target input {code} is not one of {FOLLOW_INPUTS:?}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};

    use ddc_core::domain::{MonitorId, UsbDeviceId};
    use tempfile::TempDir;

    use super::{CONFIG_FILE, ConfigStore, FollowConfig, LoadOutcome, config_path};
    use crate::fixture::rtk_id;

    pub(super) fn devices() -> BTreeSet<UsbDeviceId> {
        ["046d:c31c:KB0001", "046d:c077"]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect()
    }

    pub(super) fn complete(enabled: bool) -> FollowConfig {
        FollowConfig {
            enabled,
            devices: devices(),
            monitor_id: Some(rtk_id()),
            target_input: Some(0x10),
        }
    }

    /// A store in a directory that does not exist yet, under `home`.
    pub(super) fn store_in(home: &TempDir) -> ConfigStore {
        ConfigStore::new(home.path().join("ddc-control").join(CONFIG_FILE))
    }

    fn listing(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn round_trips_through_a_tempdir() {
        let home = TempDir::new().unwrap();
        let store = store_in(&home);

        store.save(&complete(true)).unwrap();
        assert_eq!(store.load(), LoadOutcome::Loaded(complete(true)));

        let saved: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(store.path()).unwrap()).unwrap();
        assert_eq!(
            saved,
            serde_json::json!({
                "version": 1,
                "enabled": true,
                "devices": ["046d:c077", "046d:c31c:KB0001"],
                "monitor_id": "RTK-RTK-QHD-HDR-01010101",
                "target_input": 16,
            })
        );
        let learned_only = FollowConfig {
            devices: devices(),
            ..FollowConfig::default()
        };
        store.save(&learned_only).unwrap();
        assert_eq!(store.load(), LoadOutcome::Loaded(learned_only));
        assert_eq!(listing(&home.path().join("ddc-control")), [CONFIG_FILE]);
    }

    #[test]
    fn path_uses_xdg_config_home_then_home_config() {
        let xdg = TempDir::new().unwrap();
        let home = TempDir::new().unwrap();
        let (xdg_dir, home_dir) = (xdg.path().as_os_str(), home.path().as_os_str());
        let in_xdg = xdg.path().join("ddc-control").join("usb-follow.json");
        let in_home = home.path().join(".config/ddc-control/usb-follow.json");

        assert_eq!(
            config_path(Some(xdg_dir), Some(home_dir)),
            Some(in_xdg.clone())
        );
        assert_eq!(config_path(Some(xdg_dir), None), Some(in_xdg));
        for unusable in ["", "relative/config"] {
            let unusable = Some(OsStr::new(unusable));
            assert_eq!(config_path(unusable, Some(home_dir)), Some(in_home.clone()));
            assert_eq!(config_path(None, unusable), None);
        }
        assert_eq!(config_path(None, Some(home_dir)), Some(in_home));
        assert_eq!(config_path(None, None), None::<PathBuf>);
    }

    #[test]
    fn missing_file_is_disabled_default() {
        let home = TempDir::new().unwrap();

        assert_eq!(store_in(&home).load(), LoadOutcome::Missing);

        let default = FollowConfig::default();
        assert!(!default.enabled);
        assert_eq!(default.devices, BTreeSet::new());
        assert_eq!(default.active_target(), None);
        assert_eq!((default.monitor_id, default.target_input), (None, None));
        assert!(listing(home.path()).is_empty(), "loading wrote something");
    }

    #[test]
    fn unknown_version_is_not_configured_and_not_overwritten() {
        let home = TempDir::new().unwrap();
        let store = store_in(&home);
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        let future = br#"{"version":2,"enabled":true,"devices":["046d:c077"],"monitor_id":"RTK-RTK-QHD-HDR-01010101","target_input":16,"rules":[]}"#;
        fs::write(store.path(), future).unwrap();

        let outcome = store.load();

        assert_eq!(
            outcome,
            LoadOutcome::NotConfigured("version 2 is not one this app reads".to_owned())
        );
        assert_eq!(fs::read(store.path()).unwrap(), future);
        assert_eq!(listing(store.path().parent().unwrap()), [CONFIG_FILE]);
    }

    #[test]
    fn learn_never_sets_enabled() {
        let monitor = MonitorId::new("DEL-U2720Q-7");
        let relearned: BTreeSet<UsbDeviceId> = ["1a2c:2124".parse().unwrap()].into();

        let from_on = complete(true).with_learned(relearned.clone(), monitor.clone());
        let from_off = FollowConfig::default().with_learned(devices(), rtk_id());

        assert_eq!(
            from_on,
            FollowConfig {
                enabled: false,
                devices: relearned,
                monitor_id: Some(monitor),
                target_input: Some(0x10),
            }
        );
        assert_eq!(
            from_off,
            FollowConfig {
                enabled: false,
                devices: devices(),
                monitor_id: Some(rtk_id()),
                target_input: None,
            }
        );
        assert_eq!(from_on.active_target(), None);
    }
}

/// The rules of the settings beyond the tests the phase's definition of
/// done names: what a file may not hold, and the transitions.
#[cfg(test)]
mod rule_tests {
    use std::fs;

    use tempfile::TempDir;

    use super::tests::{complete, devices, store_in};
    use super::{
        CONFIG_FILE, FollowConfig, Incomplete, LoadOutcome, Missing, NotAFollowInput, Target,
    };
    use crate::fixture::rtk_id;

    #[test]
    fn files_this_app_cannot_read_are_not_configured_and_left_as_they_are() {
        let cases: [(&str, &str); 10] = [
            ("not json", "not valid JSON: "),
            (r#"{"enabled":false}"#, "no version"),
            (
                r#"{"version":"1"}"#,
                r#"version "1" is not one this app reads"#,
            ),
            (
                r#"{"version":1,"devices":"046d:c077"}"#,
                "invalid settings: ",
            ),
            (
                r#"{"version":1,"devices":["zz"]}"#,
                r#""zz" is not a USB device id"#,
            ),
            (r#"{"version":1,"monitor_id":" "}"#, "an empty monitor id"),
            (
                r#"{"version":1,"target_input":19}"#,
                "target input 19 is not one of [15, 16, 17, 18]",
            ),
            (
                r#"{"version":1,"target_input":272}"#,
                "target input 272 is not one of",
            ),
            (r#"{"version":1,"target_input":-1}"#, "invalid settings: "),
            (
                r#"{"version":1,"enabled":true,"devices":["046d:c077"]}"#,
                "enabled, but learn the USB switch and pick the target input first \
                 (no monitor, no target input)",
            ),
        ];
        for (text, reason) in cases {
            let home = TempDir::new().unwrap();
            let store = store_in(&home);
            fs::create_dir_all(store.path().parent().unwrap()).unwrap();
            fs::write(store.path(), text).unwrap();

            let outcome = store.load();

            assert!(
                matches!(&outcome, LoadOutcome::NotConfigured(why) if why.starts_with(reason)),
                "{text}: {outcome:?}"
            );
            assert_eq!(fs::read_to_string(store.path()).unwrap(), text);
        }
    }

    #[test]
    fn a_version_1_file_may_leave_out_what_is_not_known_yet() {
        let home = TempDir::new().unwrap();
        let store = store_in(&home);
        fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        fs::write(store.path(), r#"{"version":1,"later":"ignored"}"#).unwrap();

        assert_eq!(store.load(), LoadOutcome::Loaded(FollowConfig::default()));
    }

    #[test]
    fn turning_on_needs_devices_a_monitor_and_an_input() {
        let nothing = FollowConfig::default().enabled_toggled();
        assert_eq!(
            nothing,
            Err(Incomplete(vec![
                Missing::Devices,
                Missing::Monitor,
                Missing::TargetInput
            ]))
        );
        assert_eq!(
            nothing.unwrap_err().to_string(),
            "learn the USB switch and pick the target input first \
             (no learned devices, no monitor, no target input)"
        );
        let learned = FollowConfig::default().with_learned(devices(), rtk_id());
        assert_eq!(
            learned.enabled_toggled(),
            Err(Incomplete(vec![Missing::TargetInput]))
        );

        assert_eq!(complete(false).enabled_toggled(), Ok(complete(true)));
        assert_eq!(complete(true).enabled_toggled(), Ok(complete(false)));
    }

    #[test]
    fn only_the_four_follow_inputs_can_be_the_target() {
        for code in [0x0F, 0x10, 0x11, 0x12] {
            let chosen = complete(true).with_target_input(code).unwrap();
            assert_eq!(chosen.target_input, Some(code));
            assert!(
                chosen.enabled,
                "choosing the input keeps the follow as it was"
            );
        }
        for code in [0x00, 0x01, 0x03, 0x0E, 0x13, 0x60, 0xFF] {
            assert_eq!(
                complete(false).with_target_input(code),
                Err(NotAFollowInput(code))
            );
        }
        assert_eq!(
            NotAFollowInput(0x13).to_string(),
            "0x13 is not an input the follow switches to"
        );
    }

    #[test]
    fn the_target_is_active_only_when_on_and_complete() {
        assert_eq!(
            complete(true).active_target(),
            Some(Target {
                monitor_id: rtk_id(),
                input: 0x10
            })
        );
        assert_eq!(complete(false).active_target(), None);
        for lacking in [
            FollowConfig {
                devices: Default::default(),
                ..complete(true)
            },
            FollowConfig {
                monitor_id: None,
                ..complete(true)
            },
            FollowConfig {
                target_input: None,
                ..complete(true)
            },
        ] {
            assert_eq!(lacking.active_target(), None, "{lacking:?}");
        }
    }

    #[test]
    fn a_save_that_cannot_write_fails_and_leaves_no_temporary_file() {
        let home = TempDir::new().unwrap();
        let store = store_in(&home);
        let dir = store.path().parent().unwrap();
        fs::create_dir_all(store.path()).unwrap();

        assert!(store.save(&complete(true)).is_err());

        let names: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, [CONFIG_FILE]);
        let blocked = super::ConfigStore::new(store.path().join(CONFIG_FILE).join("x"));
        fs::write(store.path().join(CONFIG_FILE), "").unwrap();
        assert!(blocked.save(&complete(true)).is_err());
    }

    #[test]
    fn a_save_that_cannot_write_its_temporary_file_keeps_the_old_bytes() {
        let home = TempDir::new().unwrap();
        let store = store_in(&home);
        store.save(&complete(false)).unwrap();
        let before = fs::read(store.path()).unwrap();
        let dir = store.path().parent().unwrap();
        let temporary = dir.join(format!(".{CONFIG_FILE}.{}.tmp", std::process::id()));
        fs::create_dir(&temporary).unwrap();

        assert!(store.save(&complete(true)).is_err());

        assert_eq!(fs::read(store.path()).unwrap(), before);
        assert_eq!(store.load(), LoadOutcome::Loaded(complete(false)));
    }
}
