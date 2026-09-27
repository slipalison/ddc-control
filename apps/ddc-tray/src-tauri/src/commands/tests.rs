use std::sync::Arc;

use ddc_adapters::{BackendCall, InMemoryMonitorBackend};
use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};
use tauri::async_runtime::block_on;

use super::{
    AppState, SharedOsd, WriteRequest, backend_unavailable, on_blocking_thread, write_feature,
};
use crate::dto::{ErrorKind, MonitorDto, ReadBackDto, UiError};
use crate::fixture::{RTK_ID, rtk_id, rtk_monitor};
use crate::panel;
use crate::panel::tests::{DELL_ID, Osd, dell_monitor, osd_with};

/// HDMI-1: an input the dev monitor's capabilities declare.
const HDMI_1: u16 = 0x11;
/// 0x05 is not in the dev monitor's declared input list.
const UNDECLARED_INPUT: u16 = 0x05;

fn request(code: VcpCode, value: u16, confirmed: bool) -> WriteRequest {
    WriteRequest {
        monitor_id: rtk_id(),
        code,
        value,
        confirmed,
    }
}

fn writes(backend: &InMemoryMonitorBackend) -> Vec<BackendCall> {
    backend
        .calls()
        .into_iter()
        .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
        .collect()
}

fn state_over(osd: Osd) -> AppState {
    let shared: SharedOsd = Arc::new(osd);
    AppState::new(Ok(shared))
}

fn no_backend() -> AppState {
    AppState::new(Err(DdcError::Transport("no I2C bus".to_owned())))
}

fn unavailable() -> UiError {
    UiError {
        kind: ErrorKind::BackendUnavailable,
        message: "transport error: no I2C bus".to_owned(),
    }
}

#[test]
fn an_unconfirmed_input_switch_needs_confirmation_and_writes_nothing() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let refused = write_feature(&osd, &request(VcpCode::INPUT_SOURCE, HDMI_1, false));

    assert_eq!(
        refused.map_err(|error| error.kind),
        Err(ErrorKind::NeedsConfirmation)
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn an_unconfirmed_power_change_needs_confirmation_and_writes_nothing() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let refused = write_feature(&osd, &request(VcpCode::POWER_MODE, 0x04, false));

    assert_eq!(
        refused.map_err(|error| error.kind),
        Err(ErrorKind::NeedsConfirmation)
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn a_confirmed_input_switch_is_written_and_answers_the_read_back() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let written = write_feature(&osd, &request(VcpCode::INPUT_SOURCE, HDMI_1, true));

    assert_eq!(
        written,
        Ok(ReadBackDto {
            current: HDMI_1,
            max: 0x03
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(
            rtk_id(),
            VcpCode::INPUT_SOURCE,
            HDMI_1
        )]
    );
    assert_eq!(
        backend.calls().last(),
        Some(&BackendCall::ReadVcp(rtk_id(), VcpCode::INPUT_SOURCE))
    );
}

#[test]
fn a_safe_write_needs_no_confirmation() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let written = write_feature(&osd, &request(VcpCode::BRIGHTNESS, 40, false));

    assert_eq!(
        written,
        Ok(ReadBackDto {
            current: 40,
            max: 100
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 40)]
    );
}

#[test]
fn a_write_the_monitor_ignores_answers_what_it_kept_not_what_was_asked() {
    let monitor = rtk_monitor().ignoring_writes_to(VcpCode::BRIGHTNESS);
    let (osd, backend) = osd_with([monitor]);

    let written = write_feature(&osd, &request(VcpCode::BRIGHTNESS, 40, false));

    assert_eq!(
        written,
        Ok(ReadBackDto {
            current: 75,
            max: 100
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 40)]
    );
}

#[test]
fn a_value_outside_the_declared_list_is_invalid_and_never_written() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let refused = write_feature(
        &osd,
        &request(VcpCode::INPUT_SOURCE, UNDECLARED_INPUT, true),
    );

    assert_eq!(
        refused.map_err(|error| error.kind),
        Err(ErrorKind::InvalidValue)
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn a_value_above_the_maximum_is_invalid_and_never_written() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let refused = write_feature(&osd, &request(VcpCode::BRIGHTNESS, 101, false));

    assert_eq!(
        refused.map_err(|error| error.kind),
        Err(ErrorKind::InvalidValue)
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn a_write_to_a_missing_monitor_is_not_found() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let mut missing = request(VcpCode::BRIGHTNESS, 40, false);
    missing.monitor_id = MonitorId::new("NOPE");

    let refused = write_feature(&osd, &missing);

    assert_eq!(
        refused.map_err(|error| error.kind),
        Err(ErrorKind::NotFound)
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn a_backend_that_could_not_start_is_unavailable() {
    assert_eq!(
        backend_unavailable(DdcError::Transport("no I2C bus".to_owned())),
        unavailable()
    );
}

#[test]
fn without_a_backend_the_state_has_no_core_and_answers_unavailable() {
    let state = no_backend();

    assert_eq!(state.osd().err(), Some(unavailable()));
}

#[test]
fn without_a_backend_every_blocking_call_answers_unavailable_without_running() {
    let state = no_backend();

    let listed = block_on(on_blocking_thread(&state, |osd| panel::monitors(osd)));
    let written = block_on(on_blocking_thread(&state, |osd| {
        write_feature(osd, &request(VcpCode::BRIGHTNESS, 40, false))
    }));

    assert_eq!(listed, Err(unavailable()));
    assert_eq!(written, Err(unavailable()));
}

#[test]
fn a_blocking_call_runs_on_the_shared_core() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let state = state_over(osd);

    let listed = block_on(on_blocking_thread(&state, |osd| panel::monitors(osd)));
    let written = block_on(on_blocking_thread(&state, |osd| {
        write_feature(osd, &request(VcpCode::BRIGHTNESS, 40, false))
    }));

    assert_eq!(
        listed,
        Ok(vec![MonitorDto {
            id: RTK_ID.to_owned(),
            label: "RTK QHD HDR".to_owned(),
            manufacturer: Some("RTK".to_owned()),
            model: Some("RTK QHD HDR".to_owned()),
        }])
    );
    assert_eq!(
        written,
        Ok(ReadBackDto {
            current: 40,
            max: 100
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 40)]
    );
}

#[test]
fn a_blocking_call_that_stops_before_answering_is_a_transport_error() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let state = state_over(osd);

    let stopped = block_on(on_blocking_thread(&state, |_| -> Result<(), UiError> {
        std::panic::resume_unwind(Box::new("scaler went away"))
    }));

    assert_eq!(
        stopped.map_err(|error| error.kind),
        Err(ErrorKind::Transport)
    );
}

#[test]
fn nothing_is_selected_at_start_and_the_last_selection_wins() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let state = state_over(osd);
    assert_eq!(state.selected(), None);

    state.select(MonitorId::new(DELL_ID));
    state.select(rtk_id());

    assert_eq!(state.selected(), Some(rtk_id()));
}

#[test]
fn the_selected_label_is_the_name_the_popup_was_given_for_it() {
    let (osd, _) = osd_with([rtk_monitor(), dell_monitor(30, 100)]);
    let listed = panel::monitors(&osd).unwrap();
    let state = state_over(osd);
    assert_eq!(state.selected_label(), None);

    state.remember_listed(&listed);
    assert_eq!(state.selected_label(), None, "nothing is selected yet");
    state.select(rtk_id());
    assert_eq!(state.selected_label().as_deref(), Some("RTK QHD HDR"));
    state.select(MonitorId::new(DELL_ID));
    assert_eq!(state.selected_label().as_deref(), Some("U2720Q"));
    state.select(MonitorId::new("GONE"));
    assert_eq!(
        state.selected_label(),
        None,
        "a monitor the popup never listed"
    );
}

#[test]
fn a_new_listing_replaces_the_names_remembered() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let state = state_over(osd);
    state.select(rtk_id());
    state.remember_listed(&[MonitorDto::new(&MonitorInfo {
        id: rtk_id(),
        manufacturer: Some("RTK".to_owned()),
        model: None,
        serial: None,
    })]);
    assert_eq!(state.selected_label().as_deref(), Some("RTK"));

    state.remember_listed(&[]);

    assert_eq!(state.selected_label(), None);
}
