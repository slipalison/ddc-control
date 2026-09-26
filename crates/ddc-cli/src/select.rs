//! Choosing the monitor a command acts on (D-2026-09-25-cli-2).
//!
//! Without `--monitor`, the only monitor is used; zero or several are an
//! error. With `--monitor`, the value is tried as an exact id, then — when
//! it is all digits — as a 1-based index in enumeration order, else as a
//! case-insensitive part of exactly one id. Nothing is ever guessed: an
//! index out of range does not fall back to matching ids.

use std::fmt;

use ddc_core::domain::{MonitorId, MonitorInfo};

use crate::output::monitor_line;

/// A monitor together with its 1-based position in the enumeration.
pub type Indexed = (usize, MonitorInfo);

/// Why no single monitor could be chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    /// No monitor is reachable.
    NoMonitor,
    /// Several monitors are reachable and `--monitor` was not given.
    ChoiceNeeded(Vec<Indexed>),
    /// `--monitor` matched no monitor.
    NoMatch {
        /// The value given.
        wanted: String,
        /// Every monitor, to choose from.
        monitors: Vec<Indexed>,
    },
    /// `--monitor` is part of several ids.
    Ambiguous {
        /// The value given.
        wanted: String,
        /// The monitors whose id contains it.
        candidates: Vec<Indexed>,
    },
}

/// Picks the monitor to act on from `monitors`, in enumeration order.
pub fn select_monitor(
    monitors: &[MonitorInfo],
    wanted: Option<&str>,
) -> Result<MonitorId, SelectionError> {
    let indexed: Vec<Indexed> = monitors
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, m)| (i + 1, m))
        .collect();
    if indexed.is_empty() {
        return Err(SelectionError::NoMonitor);
    }
    match wanted {
        None => only_one(indexed),
        Some(wanted) => matching(indexed, wanted),
    }
}

fn only_one(mut indexed: Vec<Indexed>) -> Result<MonitorId, SelectionError> {
    match indexed.len() {
        1 => Ok(indexed.remove(0).1.id),
        _ => Err(SelectionError::ChoiceNeeded(indexed)),
    }
}

fn matching(indexed: Vec<Indexed>, wanted: &str) -> Result<MonitorId, SelectionError> {
    if let Some((_, exact)) = indexed.iter().find(|(_, m)| m.id.as_str() == wanted) {
        return Ok(exact.id.clone());
    }
    let is_index = !wanted.is_empty() && wanted.bytes().all(|b| b.is_ascii_digit());
    let found: Vec<Indexed> = if is_index {
        let index = wanted.parse::<usize>().ok();
        indexed
            .iter()
            .filter(|(i, _)| Some(*i) == index)
            .cloned()
            .collect()
    } else {
        let needle = wanted.to_lowercase();
        indexed
            .iter()
            .filter(|(_, m)| m.id.as_str().to_lowercase().contains(&needle))
            .cloned()
            .collect()
    };
    let wanted = wanted.to_owned();
    match <[Indexed; 1]>::try_from(found) {
        Ok([(_, only)]) => Ok(only.id),
        Err(none) if none.is_empty() => Err(SelectionError::NoMatch {
            wanted,
            monitors: indexed,
        }),
        Err(candidates) => Err(SelectionError::Ambiguous { wanted, candidates }),
    }
}

impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMonitor => f.write_str("no monitor found over DDC/CI"),
            Self::ChoiceNeeded(monitors) => {
                write!(f, "{} monitors found; ", monitors.len())?;
                write_choices(f, monitors)
            }
            Self::NoMatch { wanted, monitors } => {
                write!(f, "no monitor matches '{wanted}'; ")?;
                write_choices(f, monitors)
            }
            Self::Ambiguous { wanted, candidates } => {
                write!(f, "'{wanted}' matches {} monitors; ", candidates.len())?;
                write_choices(f, candidates)
            }
        }
    }
}

/// The hint, one `list`-style line per monitor, and an example taken from
/// the first of them.
fn write_choices(f: &mut fmt::Formatter<'_>, monitors: &[Indexed]) -> fmt::Result {
    f.write_str("choose one with --monitor <id|index>:")?;
    for (index, monitor) in monitors {
        write!(f, "\n  {}", monitor_line(*index, monitor))?;
    }
    if let Some((index, monitor)) = monitors.first() {
        write!(f, "\ne.g. --monitor {index} or --monitor {}", monitor.id)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
