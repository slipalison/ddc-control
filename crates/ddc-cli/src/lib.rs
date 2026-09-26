//! `ddc-cli` — the command-line driving adapter of `ddc-control`.
//!
//! Turns command-line arguments into calls on the core's `MonitorControl`
//! port and renders the results as text or JSON. Every rule about monitors
//! — which writes are dangerous, which values are valid — lives in the
//! core; this crate only parses, selects a monitor and prints.

#![forbid(unsafe_code)]

pub mod args;
pub mod exit;
pub mod fixture;
pub mod output;
mod run;
pub mod select;

pub use args::{Cli, Command};
pub use exit::{CliError, Exit};
pub use run::{report_startup_failure, run};
