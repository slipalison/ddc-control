use super::{MonitorId, VcpCode};

/// Everything that can go wrong while controlling a monitor.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DdcError {
    /// No monitor with this id is currently reachable.
    #[error("monitor {0} not found")]
    MonitorNotFound(MonitorId),
    /// The monitor does not support this feature, or the core lacks the
    /// information needed to operate on it safely.
    #[error("feature {0} is not supported")]
    UnsupportedFeature(VcpCode),
    /// The value is above the maximum known for the feature.
    #[error("value {value} for feature {code} exceeds its maximum {max}")]
    InvalidValue {
        /// Feature being written.
        code: VcpCode,
        /// Rejected value.
        value: u16,
        /// Known maximum.
        max: u16,
    },
    /// The value is not one of the feature's allowed discrete values.
    #[error("value {value} is not an allowed value for feature {code}")]
    ValueNotAllowed {
        /// Feature being written.
        code: VcpCode,
        /// Rejected value.
        value: u16,
    },
    /// A dangerous feature was written without explicit user confirmation.
    #[error("writing feature {0} is dangerous and was not confirmed")]
    DangerousWriteNotConfirmed(VcpCode),
    /// The monitor did not answer in time.
    #[error("monitor did not respond in time")]
    Timeout,
    /// The transport failed or returned malformed data.
    #[error("transport error: {0}")]
    Transport(String),
}

#[cfg(test)]
mod tests {
    use super::DdcError;
    use crate::domain::{MonitorId, VcpCode};

    #[test]
    fn error_messages_name_the_feature_and_values() {
        let cases = [
            (
                DdcError::MonitorNotFound(MonitorId::new("m1")),
                "monitor m1 not found",
            ),
            (
                DdcError::UnsupportedFeature(VcpCode(0x62)),
                "feature 0x62 is not supported",
            ),
            (
                DdcError::InvalidValue {
                    code: VcpCode::BRIGHTNESS,
                    value: 101,
                    max: 100,
                },
                "value 101 for feature 0x10 exceeds its maximum 100",
            ),
            (
                DdcError::ValueNotAllowed {
                    code: VcpCode::INPUT_SOURCE,
                    value: 2,
                },
                "value 2 is not an allowed value for feature 0x60",
            ),
            (
                DdcError::DangerousWriteNotConfirmed(VcpCode::POWER_MODE),
                "writing feature 0xD6 is dangerous and was not confirmed",
            ),
            (DdcError::Timeout, "monitor did not respond in time"),
            (DdcError::Transport("nak".into()), "transport error: nak"),
        ];
        for (error, message) in cases {
            assert_eq!(error.to_string(), message);
        }
    }
}
