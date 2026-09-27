use std::process::ExitCode;

use clap::Parser;
use ddc_core::domain::{DdcError, MonitorId, VcpCode};

use super::{CliError, Exit};
use crate::args::Cli;
use crate::select::SelectionError;

fn on_monitor(error: DdcError) -> CliError {
    CliError::Ddc {
        monitor: Some(MonitorId::new("RTK-1")),
        error,
    }
}

fn every_ddc_error() -> [(DdcError, Exit); 7] {
    [
        (
            DdcError::MonitorNotFound(MonitorId::new("RTK-1")),
            Exit::Monitor,
        ),
        (
            DdcError::UnsupportedFeature(VcpCode::SHARPNESS),
            Exit::Invalid,
        ),
        (
            DdcError::InvalidValue {
                code: VcpCode::BRIGHTNESS,
                value: 101,
                max: 100,
            },
            Exit::Invalid,
        ),
        (
            DdcError::ValueNotAllowed {
                code: VcpCode::COLOR_PRESET,
                value: 7,
            },
            Exit::Invalid,
        ),
        (
            DdcError::DangerousWriteNotConfirmed(VcpCode::INPUT_SOURCE),
            Exit::Unconfirmed,
        ),
        (DdcError::Timeout, Exit::Transport),
        (DdcError::Transport("nak".to_owned()), Exit::Transport),
    ]
}

#[test]
fn exit_codes_follow_the_closed_table() {
    let table = [
        (Exit::Success, 0),
        (Exit::Usage, 2),
        (Exit::Monitor, 3),
        (Exit::Invalid, 4),
        (Exit::Unconfirmed, 5),
        (Exit::Transport, 6),
    ];
    for (exit, code) in table {
        assert_eq!(exit as u8, code);
        assert_eq!(ExitCode::from(exit), ExitCode::from(code));
    }
}

#[test]
fn usage_is_the_code_clap_exits_with() {
    let usage = Cli::try_parse_from(["ddc-cli", "get", "foo"]).unwrap_err();

    assert_eq!(usage.exit_code(), i32::from(Exit::Usage as u8));
}

#[test]
fn every_core_error_maps_to_its_exit_code() {
    for (error, exit) in every_ddc_error() {
        assert_eq!(on_monitor(error.clone()).exit(), exit, "{error:?}");
        let unscoped = CliError::Ddc {
            monitor: None,
            error: error.clone(),
        };
        assert_eq!(unscoped.exit(), exit, "{error:?}");
    }
}

#[test]
fn every_selection_error_is_a_monitor_error() {
    let errors = [
        SelectionError::NoMonitor,
        SelectionError::ChoiceNeeded(Vec::new()),
        SelectionError::NoMatch {
            wanted: "x".to_owned(),
            monitors: Vec::new(),
        },
        SelectionError::Ambiguous {
            wanted: "x".to_owned(),
            candidates: Vec::new(),
        },
    ];
    for error in errors {
        assert_eq!(CliError::from(error).exit(), Exit::Monitor);
    }
}

#[test]
fn every_message_names_the_monitor_or_the_code() {
    let expected = [
        "monitor RTK-1 is not reachable; `list` shows the ones that are",
        "RTK-1: feature 0x87 is not supported",
        "RTK-1: value 101 for feature 0x10 exceeds its maximum 100",
        "RTK-1: value 7 is not an allowed value for feature 0x14",
        "RTK-1: writing feature 0x60 is classified as dangerous; \
         repeat the command with --yes to confirm it",
        "RTK-1: monitor did not respond in time",
        "RTK-1: transport error: nak",
    ];
    for ((error, _), message) in every_ddc_error().into_iter().zip(expected) {
        assert_eq!(on_monitor(error).to_string(), message);
    }
}

#[test]
fn an_error_before_a_monitor_is_chosen_has_no_prefix() {
    let error = CliError::Ddc {
        monitor: None,
        error: DdcError::Transport("no i2c bus".to_owned()),
    };

    assert_eq!(error.to_string(), "transport error: no i2c bus");
}

#[test]
fn a_selection_error_keeps_its_own_message() {
    let error = CliError::from(SelectionError::NoMonitor);

    assert_eq!(error.to_string(), "no monitor found over DDC/CI");
}

#[test]
fn a_usage_error_found_while_running_exits_2_with_its_own_message() {
    let error = CliError::Usage("0x10 brightness has no value named 'srgb'".to_owned());

    assert_eq!(error.exit(), Exit::Usage);
    assert_eq!(
        error.to_string(),
        "0x10 brightness has no value named 'srgb'"
    );
}
