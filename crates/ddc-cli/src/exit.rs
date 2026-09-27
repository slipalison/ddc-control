//! Exit codes and the errors that lead to them (D-2026-09-25-cli-5).

use std::fmt;
use std::process::ExitCode;

use ddc_core::domain::{DdcError, MonitorId};

use crate::select::SelectionError;

/// The closed table of exit codes. Scripts may rely on these values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Exit {
    /// The command succeeded.
    Success = 0,
    /// The command line is invalid. clap exits with this code on its own.
    Usage = 2,
    /// No single reachable monitor matches the selection.
    Monitor = 3,
    /// The feature or the value is not valid for the monitor.
    Invalid = 4,
    /// A dangerous write was not confirmed with `--yes`.
    Unconfirmed = 5,
    /// The monitor failed to answer, or answered garbage.
    Transport = 6,
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(exit as u8)
    }
}

/// Everything that makes a command fail after its arguments parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// An argument clap accepted does not fit the feature it goes with,
    /// such as a value name of another feature. Found before any monitor
    /// is touched.
    Usage(String),
    /// No single monitor could be chosen.
    Selection(SelectionError),
    /// The core refused or failed the operation.
    Ddc {
        /// The monitor it was about, once one was chosen.
        monitor: Option<MonitorId>,
        /// What went wrong.
        error: DdcError,
    },
}

impl CliError {
    /// The exit code this error ends the process with.
    pub fn exit(&self) -> Exit {
        let error = match self {
            Self::Usage(_) => return Exit::Usage,
            Self::Selection(_) => return Exit::Monitor,
            Self::Ddc { error, .. } => error,
        };
        match error {
            DdcError::MonitorNotFound(_) => Exit::Monitor,
            DdcError::UnsupportedFeature(_)
            | DdcError::InvalidValue { .. }
            | DdcError::ValueNotAllowed { .. } => Exit::Invalid,
            DdcError::DangerousWriteNotConfirmed(_) => Exit::Unconfirmed,
            DdcError::Timeout | DdcError::Transport(_) => Exit::Transport,
        }
    }
}

impl From<SelectionError> for CliError {
    fn from(error: SelectionError) -> Self {
        Self::Selection(error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (monitor, error) = match self {
            Self::Usage(message) => return f.write_str(message),
            Self::Selection(selection) => return selection.fmt(f),
            Self::Ddc { monitor, error } => (monitor, error),
        };
        match (monitor, error) {
            (_, DdcError::MonitorNotFound(id)) => {
                write!(
                    f,
                    "monitor {id} is not reachable; `list` shows the ones that are"
                )
            }
            (Some(id), error) => write!(f, "{id}: {}", describe(error)),
            (None, error) => f.write_str(&describe(error)),
        }
    }
}

/// The core's message, except for an unconfirmed write, which also says
/// how to confirm it.
fn describe(error: &DdcError) -> String {
    match error {
        DdcError::DangerousWriteNotConfirmed(code) => format!(
            "writing feature {code} is classified as dangerous; \
             repeat the command with --yes to confirm it"
        ),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests;
