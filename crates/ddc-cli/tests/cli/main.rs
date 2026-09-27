//! End-to-end tests of `ddc-cli`: the built binary behind `--fake`, and
//! `run` in-process where a test must script the monitors or inspect the
//! backend's call log.

mod exit_codes;
mod features;
mod reads;
mod reset;
mod selection;
mod support;
mod writes;
