//! End-to-end tests of `ddc-cli`: the built binary behind `--fake`, and
//! `run` in-process where a test must script the monitors or inspect the
//! backend's call log.

mod reads;
mod selection;
mod support;
mod writes;
