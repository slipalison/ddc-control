//! `ddc-cli` — the command-line driving adapter of `ddc-control`.
//!
//! Turns command-line arguments into calls on the core's `MonitorControl`
//! port and renders the results as text or JSON. Every rule about monitors
//! — which writes are dangerous, which values are valid — lives in the
//! core; this crate only parses, selects a monitor and prints.

#![forbid(unsafe_code)]

pub mod args;

pub use args::{Cli, Command};
