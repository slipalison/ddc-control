//! Binary entry point of `ddc-tray`: starts the tray app and turns a
//! startup failure into a non-zero exit code.

#![forbid(unsafe_code)]
// Release builds on Windows must not open a console window next to the tray.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match ddc_tray::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ddc-tray: {error}");
            ExitCode::FAILURE
        }
    }
}
