use super::mccs_catalog::catalog_entry;
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
    /// Refuses any write to a feature that cannot take one: a `Table`
    /// feature, since a write carries a single value
    /// (D-2026-09-26-full-osd-control-5), then a read-only one. Needs no
    /// value, so it runs before anything is read from the monitor.
    pub fn ensure_writable(&self) -> Result<(), DdcError> {
        if self.kind == FeatureKind::Table || self.access == Access::ReadOnly {
            return Err(DdcError::UnsupportedFeature(self.code));
        }
        Ok(())
    }

    /// Whether the monitor can be asked for the feature's value: false only
    /// for a write-only feature, which is then never read — not for its
    /// maximum, not back after a write (D-2026-09-26-full-osd-control-8).
    pub fn is_readable(&self) -> bool {
        self.access != Access::WriteOnly
    }

    /// Whether validating a write needs the feature's maximum: only for a
    /// continuous feature that does not list its values. A non-continuous
    /// value is one of a closed set, never anything up to a maximum
    /// (D-2026-09-26-full-osd-control-1).
    pub fn requires_known_max(&self) -> bool {
        self.kind == FeatureKind::Continuous && self.allowed_values.is_none()
    }

    /// Checks `value` before it is written: the feature must be writable
    /// ([`ensure_writable`](Self::ensure_writable)); a feature with a value
    /// list — the one the capabilities declare, else the catalog's — accepts
    /// only its listed values; any other feature needs a `known_max`, and
    /// without one the write is refused rather than sent blind.
    pub fn validate_write(&self, value: u16, known_max: Option<u16>) -> Result<(), DdcError> {
        let code = self.code;
        self.ensure_writable()?;
        if let Some(listed) = self.lists(value) {
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

    /// Whether `value` is in the feature's value list: the capabilities'
    /// list, else the catalog's value names. `None` when neither lists any.
    fn lists(&self, value: u16) -> Option<bool> {
        let byte = u8::try_from(value).ok();
        if let Some(allowed) = &self.allowed_values {
            return Some(byte.is_some_and(|byte| allowed.contains(&byte)));
        }
        let named = catalog_entry(self.code)
            .map(|entry| entry.values)
            .filter(|values| !values.is_empty())?;
        Some(byte.is_some_and(|byte| named.iter().any(|(listed, _)| *listed == byte)))
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

/// The result of reading one code the capabilities do not declare, as part
/// of a probe: a failure concerns only this code
/// (D-2026-09-26-full-osd-control-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedFeature {
    /// The code that was read.
    pub code: VcpCode,
    /// The reading, or why this code gave none:
    /// [`DdcError::UnsupportedFeature`] when the monitor said it does not
    /// support the code, [`DdcError::Transport`] or [`DdcError::Timeout`]
    /// when it did not answer.
    pub outcome: Result<FeatureReading, DdcError>,
}

/// Risk of writing `code`, as the [MCCS catalog](super::mccs_catalog) says.
///
/// Any code outside the catalog — the rest of the manufacturer-specific range
/// `0xE0..=0xFF` included — is `Dangerous`, so an unknown code always needs
/// confirmation before it is written.
pub fn risk_for_code(code: VcpCode) -> Risk {
    catalog_entry(code).map_or(Risk::Dangerous, |entry| entry.risk)
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
