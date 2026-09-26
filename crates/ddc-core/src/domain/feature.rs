use super::{VcpCode, VcpValue};

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

#[cfg(test)]
mod tests;
