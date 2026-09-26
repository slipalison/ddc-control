//! Core of `ddc-control` — the hexagon.
//!
//! Holds the monitor-control domain model, the ports the outside world plugs
//! into, and the use cases that implement the driving port. It performs no I/O
//! and knows no transport: every monitor interaction goes through a driven port
//! implemented by an adapter crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod app;
pub mod domain;
pub mod ports;
