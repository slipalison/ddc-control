use super::{DdcError, VcpCode, VcpValue};

/// How a feature's value is interpreted (MCCS feature type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    /// A value on a range `0..=max` (brightness, contrast, gains).
    Continuous,
    /// One of a discrete set of values (input source, color preset).
    NonContinuous,
    /// A multi-byte table value.
    Table,
}

/// Which directions a feature can be accessed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// The feature can only be read.
    ReadOnly,
    /// The feature can only be written.
    WriteOnly,
    /// The feature can be read and written.
    ReadWrite,
}

/// Consequence class of writing a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    /// Harmless and easy to revert from the OSD (brightness, volume, ...).
    Safe,
    /// Can blank the screen, lock the OSD or wipe settings; a write needs
    /// explicit user confirmation.
    Dangerous,
}

/// Explicit user confirmation for a write. An enum rather than a `bool` so
/// call sites read as a decision, not a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confirm {
    /// The user explicitly confirmed the write.
    Yes,
    /// No confirmation was given.
    No,
}

/// A controllable monitor feature and the rules that govern writing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature {
    /// VCP code of the feature.
    pub code: VcpCode,
    /// How the value is interpreted.
    pub kind: FeatureKind,
    /// Allowed access directions.
    pub access: Access,
    /// Consequence class of writing it.
    pub risk: Risk,
    /// Discrete values the monitor declares for a non-continuous feature.
    pub allowed_values: Option<Vec<u8>>,
}

impl Feature {
    /// Whether validating a write needs the feature's maximum: true unless the
    /// feature lists its allowed values, since scalers often misreport the
    /// maximum of non-continuous features.
    pub fn requires_known_max(&self) -> bool {
        self.allowed_values.is_none()
    }

    /// Checks `value` before it is written. A listed feature accepts only its
    /// listed values; any other feature needs a `known_max` — without one the
    /// write is refused rather than sent blind.
    pub fn validate_write(&self, value: u16, known_max: Option<u16>) -> Result<(), DdcError> {
        let code = self.code;
        if self.access == Access::ReadOnly {
            return Err(DdcError::UnsupportedFeature(code));
        }
        if let Some(allowed) = &self.allowed_values {
            let listed = u8::try_from(value).is_ok_and(|byte| allowed.contains(&byte));
            return if listed {
                Ok(())
            } else {
                Err(DdcError::ValueNotAllowed { code, value })
            };
        }
        let max = known_max.ok_or(DdcError::UnsupportedFeature(code))?;
        if value > max {
            return Err(DdcError::InvalidValue { code, value, max });
        }
        Ok(())
    }
}

/// The result of reading a feature from a monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureReading {
    /// The feature that was read.
    pub feature: Feature,
    /// The value the monitor reported.
    pub value: VcpValue,
    /// Whether the monitor's capabilities string declares this code. Monitors
    /// answer some codes they do not declare, so this informs, never filters.
    pub declared_in_capabilities: bool,
}

/// Seed risk classification of a VCP code.
///
/// Safe: brightness, contrast, color preset, RGB gains, volume, sharpness and
/// OSD language. Everything else — factory resets, input source, OSD lock,
/// power mode, the manufacturer-specific range `0xE0..=0xFF`, and any code not
/// yet classified — is `Dangerous`, so an unknown code always needs
/// confirmation before it is written.
pub fn risk_for_code(code: VcpCode) -> Risk {
    match code {
        VcpCode::BRIGHTNESS
        | VcpCode::CONTRAST
        | VcpCode::COLOR_PRESET
        | VcpCode::RED_GAIN
        | VcpCode::GREEN_GAIN
        | VcpCode::BLUE_GAIN
        | VcpCode::AUDIO_VOLUME
        | VcpCode::SHARPNESS
        | VcpCode::OSD_LANGUAGE => Risk::Safe,
        _ => Risk::Dangerous,
    }
}

/// Refuses a write to a dangerous code that the user did not confirm. Needs
/// only the code, so it runs before anything is read from the monitor.
pub fn authorize_write(code: VcpCode, confirm: Confirm) -> Result<(), DdcError> {
    match (risk_for_code(code), confirm) {
        (Risk::Dangerous, Confirm::No) => Err(DdcError::DangerousWriteNotConfirmed(code)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
