//! The contract between the popup and the Rust side
//! (D-2026-09-26-tray-app-4): what commands return and events carry, in
//! camelCase, each shape built from core types. The goldens
//! `apps/ddc-tray/tests/fixtures/contract-rtk.json` (the RTK) and
//! `contract-mute.json` (a mute monitor's panel error) pin them for the UI.

use ddc_core::domain::mccs_catalog::{catalog_entry, value_name};
use ddc_core::domain::{
    DdcError, Feature, FeatureKind, FeatureReading, MonitorInfo, Risk, VcpValue,
};
use serde::Serialize;

/// Event sent when the popup is shown, so the UI revalidates what it shows.
pub const POPUP_SHOWN: &str = "popup-shown";

/// Event sent after a tray shortcut changed a monitor; carries a
/// [`PanelChangedDto`].
pub const PANEL_CHANGED: &str = "panel-changed";

/// A reachable monitor, as the monitor picker shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
    /// Stable id every other command takes.
    pub id: String,
    /// What the picker displays: the model, else the manufacturer, else the id.
    pub label: String,
    /// EDID manufacturer code, when known.
    pub manufacturer: Option<String>,
    /// EDID model name, when known.
    pub model: Option<String>,
}

impl MonitorDto {
    /// The picker entry of `monitor`.
    pub fn new(monitor: &MonitorInfo) -> Self {
        let label = monitor
            .model
            .clone()
            .or_else(|| monitor.manufacturer.clone())
            .unwrap_or_else(|| monitor.id.to_string());
        Self {
            id: monitor.id.to_string(),
            label,
            manufacturer: monitor.manufacturer.clone(),
            model: monitor.model.clone(),
        }
    }
}

/// The quick controls of one monitor, in the order the popup shows them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelDto {
    /// The monitor the controls belong to.
    pub monitor_id: String,
    /// One entry per control whose reading succeeded.
    pub controls: Vec<ControlDto>,
}

/// One quick control and its current value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlDto {
    /// VCP code.
    pub code: u8,
    /// The catalog alias (`brightness`, `input`...), the UI's label key.
    pub key: Option<&'static str>,
    /// Whether a write needs the user's confirmation first.
    pub dangerous: bool,
    /// The value read from the monitor.
    pub value: ControlValueDto,
}

impl ControlDto {
    /// The control showing `reading`.
    pub fn new(reading: &FeatureReading) -> Self {
        let feature = &reading.feature;
        Self {
            code: feature.code.0,
            key: catalog_entry(feature.code).and_then(|entry| entry.alias),
            dangerous: feature.risk == Risk::Dangerous,
            value: ControlValueDto::new(feature, reading.value),
        }
    }
}

/// A value as the UI edits it: a slider up to `max`, or a pick among
/// `options`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ControlValueDto {
    /// A value on `0..=max`.
    Continuous {
        /// Current value.
        current: u16,
        /// Maximum the monitor reports.
        max: u16,
    },
    /// One value of a closed list.
    NonContinuous {
        /// The low byte (SL) of the reading, where MCCS puts the value.
        current: u8,
        /// The values the feature accepts.
        options: Vec<OptionDto>,
    },
}

impl ControlValueDto {
    /// `value` of `feature`, shaped by the feature's kind. A table value
    /// has no shape of its own here; no code the popup lists is one.
    pub fn new(feature: &Feature, value: VcpValue) -> Self {
        match feature.kind {
            FeatureKind::NonContinuous => {
                let [_, current] = value.current.to_be_bytes();
                Self::NonContinuous {
                    current,
                    options: OptionDto::for_feature(feature),
                }
            }
            FeatureKind::Continuous | FeatureKind::Table => Self::Continuous {
                current: value.current,
                max: value.max,
            },
        }
    }
}

/// One value a non-continuous feature accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionDto {
    /// The byte written to select it.
    pub value: u8,
    /// Its MCCS name, when the catalog names it.
    pub name: Option<&'static str>,
}

impl OptionDto {
    /// The values `feature` accepts: the list its capabilities declare,
    /// else the catalog's, each named by the catalog. Empty when neither
    /// lists any.
    pub fn for_feature(feature: &Feature) -> Vec<Self> {
        let values: Vec<u8> = match &feature.allowed_values {
            Some(declared) => declared.clone(),
            None => catalog_entry(feature.code)
                .map(|entry| entry.values.iter().map(|(value, _)| *value).collect())
                .unwrap_or_default(),
        };
        values
            .into_iter()
            .map(|value| Self {
                value,
                name: value_name(feature.code, value),
            })
            .collect()
    }
}

/// Where an "all settings" entry was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    /// The capabilities declare the code.
    Caps,
    /// A probe found the code although the capabilities omit it.
    Probe,
}

/// Whether reading an "all settings" entry gave a value, and if not, why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FeatureStatus {
    /// The monitor answered.
    Ok,
    /// The monitor said it does not support the code.
    Unsupported,
    /// The monitor did not answer, or not in time.
    Unresponsive,
}

impl FeatureStatus {
    /// The status of a reading that gave `outcome`.
    pub fn of(outcome: &Result<VcpValue, DdcError>) -> Self {
        match outcome {
            Ok(_) => Self::Ok,
            Err(DdcError::UnsupportedFeature(_)) => Self::Unsupported,
            Err(_) => Self::Unresponsive,
        }
    }
}

/// One entry of the "all settings" section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureDto {
    /// VCP code.
    pub code: u8,
    /// The catalog alias, the UI's label key when present.
    pub alias: Option<&'static str>,
    /// The MCCS name, the label when there is no alias key.
    pub name: Option<&'static str>,
    /// Whether a write needs the user's confirmation first.
    pub dangerous: bool,
    /// Where the entry was found.
    pub origin: Origin,
    /// Whether reading it gave a value.
    pub status: FeatureStatus,
    /// The value read; `None` unless `status` is [`FeatureStatus::Ok`].
    pub value: Option<ControlValueDto>,
}

impl FeatureDto {
    /// The entry of `feature`, found by `origin`, whose reading gave
    /// `outcome`.
    pub fn new(feature: &Feature, origin: Origin, outcome: &Result<VcpValue, DdcError>) -> Self {
        let entry = catalog_entry(feature.code);
        Self {
            code: feature.code.0,
            alias: entry.and_then(|entry| entry.alias),
            name: entry.and_then(|entry| entry.name),
            dangerous: feature.risk == Risk::Dangerous,
            origin,
            status: FeatureStatus::of(outcome),
            value: outcome
                .as_ref()
                .ok()
                .map(|value| ControlValueDto::new(feature, *value)),
        }
    }
}

/// The value read back from the monitor right after a write — never the
/// value asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadBackDto {
    /// Current value.
    pub current: u16,
    /// Maximum the monitor reports.
    pub max: u16,
}

impl From<VcpValue> for ReadBackDto {
    fn from(value: VcpValue) -> Self {
        Self {
            current: value.current,
            max: value.max,
        }
    }
}

/// Payload of [`PANEL_CHANGED`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelChangedDto {
    /// The monitor whose values changed.
    pub monitor_id: String,
}

/// An error as the UI shows it: a stable `kind` it translates, plus the
/// core's message as detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiError {
    /// What went wrong, as a stable code.
    pub kind: ErrorKind,
    /// The core's description, in English.
    pub message: String,
}

/// The stable error codes of the contract (D-2026-09-26-tray-app-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// No monitor with this id is reachable.
    NotFound,
    /// The monitor or the core does not support the feature.
    Unsupported,
    /// The value is not one the feature accepts.
    InvalidValue,
    /// The write is dangerous and needs the user's confirmation.
    NeedsConfirmation,
    /// The monitor did not answer in time.
    Timeout,
    /// The DDC/CI transport failed.
    Transport,
    /// No DDC/CI backend could be started on this machine.
    BackendUnavailable,
}

#[cfg(test)]
mod tests;
