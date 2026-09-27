//! Presentation of the popup, as plain code on the core's `MonitorControl`
//! port (D-2026-09-26-tray-app-3): which controls it shows and in which
//! order, what "all settings" lists, what an error means to the UI, and the
//! brightness shortcuts with the monitor they act on. Names, aliases, risks
//! and value lists all come from the core; nothing here knows the webview
//! or the DDC/CI transport.

use ddc_core::domain::{
    Capabilities, Confirm, DdcError, Feature, FeatureKind, MonitorId, VcpCode, VcpValue,
};
use ddc_core::ports::MonitorControl;

use crate::dto::{
    ControlDto, ErrorKind, FeatureDto, MonitorDto, OptionDto, Origin, PanelDto, ReadBackDto,
    UiError,
};

/// The popup's quick controls, in the order it shows them: brightness,
/// contrast, volume, input, colour preset, power.
pub const QUICK_CONTROLS: [VcpCode; 6] = [
    VcpCode::BRIGHTNESS,
    VcpCode::CONTRAST,
    VcpCode::AUDIO_VOLUME,
    VcpCode::INPUT_SOURCE,
    VcpCode::COLOR_PRESET,
    VcpCode::POWER_MODE,
];

/// The monitors to pick from, in enumeration order.
///
/// # Errors
///
/// The [`UiError`] of the enumeration's failure.
pub fn monitors<M: MonitorControl + ?Sized>(osd: &M) -> Result<Vec<MonitorDto>, UiError> {
    let monitors = osd.list_monitors().map_err(ui_error)?;
    Ok(monitors.iter().map(MonitorDto::new).collect())
}

/// The quick controls of monitor `id`, in [`QUICK_CONTROLS`] order. A
/// control whose reading fails is left out and the others still load —
/// volume, for one, answers on monitors whose capabilities omit it.
///
/// # Errors
///
/// `not_found` when the monitor is not reachable. When no quick control
/// can be read, the error of the first reading that failed: a monitor
/// listed from its EDID whose DDC/CI is mute fails its panel instead of
/// loading it empty, so the popup moves on to the next monitor.
pub fn load_panel<M: MonitorControl + ?Sized>(
    osd: &M,
    id: &MonitorId,
) -> Result<PanelDto, UiError> {
    let mut controls = Vec::with_capacity(QUICK_CONTROLS.len());
    let mut first_failure = None;
    for code in QUICK_CONTROLS {
        match osd.get_feature(id, code) {
            Ok(reading) => controls.push(ControlDto::new(&reading)),
            Err(error @ DdcError::MonitorNotFound(_)) => return Err(ui_error(error)),
            Err(error) => {
                first_failure.get_or_insert(error);
            }
        }
    }
    match first_failure {
        Some(error) if controls.is_empty() => Err(ui_error(error)),
        _ => Ok(PanelDto {
            monitor_id: id.to_string(),
            controls,
        }),
    }
}

/// The "all settings" entries of monitor `id`: every code its capabilities
/// declare that [is adjustable](is_adjustable), in code order, each read
/// once. A code that gives no value is still listed, with its status.
///
/// # Errors
///
/// The error that kept the capabilities from being read or parsed, or
/// `not_found` when the monitor stops being reachable.
pub fn load_features<M: MonitorControl + ?Sized>(
    osd: &M,
    id: &MonitorId,
) -> Result<Vec<FeatureDto>, UiError> {
    let capabilities = osd.capabilities(id).map_err(ui_error)?;
    capabilities
        .vcp
        .keys()
        .map(|code| capabilities.feature(VcpCode(*code)))
        .filter(is_adjustable)
        .map(|feature| {
            let outcome = osd
                .get_feature(id, feature.code)
                .map(|reading| reading.value);
            caps_entry(&feature, outcome)
        })
        .collect()
}

/// The entries a probe adds to "all settings": the adjustable codes of the
/// catalog the capabilities do not declare, in catalog order, each with its
/// status. Only reads; asked for explicitly, since each silent code costs up
/// to the backend's timeout.
///
/// # Errors
///
/// `not_found` when the monitor is not reachable, before or during the
/// probe.
pub fn probe_features<M: MonitorControl + ?Sized>(
    osd: &M,
    id: &MonitorId,
) -> Result<Vec<FeatureDto>, UiError> {
    let probed = osd.probe_undeclared_features(id).map_err(ui_error)?;
    // A probed code is one the capabilities do not declare, so its feature is
    // exactly what capabilities declaring nothing say about it.
    let undeclared = Capabilities::default();
    Ok(probed
        .into_iter()
        .map(|probe| {
            let outcome = probe.outcome.map(|reading| reading.value);
            (undeclared.feature(probe.code), outcome)
        })
        .filter(|(feature, _)| is_adjustable(feature))
        .map(|(feature, outcome)| FeatureDto::new(&feature, Origin::Probe, &outcome))
        .collect())
}

/// The UI's view of a core error: a stable kind plus the core's message.
pub fn ui_error(error: DdcError) -> UiError {
    let kind = match &error {
        DdcError::MonitorNotFound(_) => ErrorKind::NotFound,
        DdcError::UnsupportedFeature(_) => ErrorKind::Unsupported,
        DdcError::InvalidValue { .. } | DdcError::ValueNotAllowed { .. } => ErrorKind::InvalidValue,
        DdcError::DangerousWriteNotConfirmed(_) => ErrorKind::NeedsConfirmation,
        DdcError::Timeout => ErrorKind::Timeout,
        DdcError::Transport(_) => ErrorKind::Transport,
    };
    UiError {
        kind,
        message: error.to_string(),
    }
}

/// The brightness value `percent` percent of `max` stands for, rounded to
/// the nearest step; a percentage above 100 counts as 100.
pub fn brightness_for_percent(max: u16, percent: u8) -> u16 {
    let scaled = (u32::from(max) * u32::from(percent.min(100)) + 50) / 100;
    u16::try_from(scaled).unwrap_or(max)
}

/// Sets monitor `id`'s brightness to `percent` percent of the maximum it
/// reports, and returns the value read back. Brightness is safe to write,
/// so no confirmation is given.
///
/// # Errors
///
/// The [`UiError`] of the reading of the maximum or of the write.
pub fn set_brightness_percent<M: MonitorControl + ?Sized>(
    osd: &M,
    id: &MonitorId,
    percent: u8,
) -> Result<ReadBackDto, UiError> {
    let max = osd
        .get_feature(id, VcpCode::BRIGHTNESS)
        .map_err(ui_error)?
        .value
        .max;
    let value = brightness_for_percent(max, percent);
    osd.set_feature(id, VcpCode::BRIGHTNESS, value, Confirm::No)
        .map(ReadBackDto::from)
        .map_err(ui_error)
}

/// The monitor a tray shortcut acts on: the one the popup last selected,
/// else the first one listed.
///
/// # Errors
///
/// The enumeration's [`UiError`], or `not_found` when no monitor is
/// reachable.
pub fn shortcut_target<M: MonitorControl + ?Sized>(
    osd: &M,
    selected: Option<MonitorId>,
) -> Result<MonitorId, UiError> {
    if let Some(id) = selected {
        return Ok(id);
    }
    osd.list_monitors()
        .map_err(ui_error)?
        .into_iter()
        .next()
        .map(|monitor| monitor.id)
        .ok_or_else(|| UiError {
            kind: ErrorKind::NotFound,
            message: "no monitor is reachable".to_owned(),
        })
}

/// Whether "all settings" lists `feature`: a code the quick controls do not
/// already show, that can be read and written and, when non-continuous,
/// lists its values somewhere — the UI cannot offer a pick among nothing.
fn is_adjustable(feature: &Feature) -> bool {
    let has_values =
        feature.kind != FeatureKind::NonContinuous || !OptionDto::for_feature(feature).is_empty();
    !QUICK_CONTROLS.contains(&feature.code)
        && feature.is_readable()
        && feature.ensure_writable().is_ok()
        && has_values
}

/// The "all settings" entry of a declared `feature`; a monitor gone while
/// reading it ends the whole listing.
fn caps_entry(
    feature: &Feature,
    outcome: Result<VcpValue, DdcError>,
) -> Result<FeatureDto, UiError> {
    match outcome {
        Err(error @ DdcError::MonitorNotFound(_)) => Err(ui_error(error)),
        outcome => Ok(FeatureDto::new(feature, Origin::Caps, &outcome)),
    }
}

#[cfg(test)]
pub(crate) mod tests;
