//! Read-only probe of the catalogued codes the capabilities do not declare
//! (D-2026-09-26-full-osd-control-2).

use ddc_adapters::{BackendCall, FakeMonitor};
use ddc_core::domain::mccs_catalog::catalog_codes;
use ddc_core::domain::{Confirm, DdcError, FeatureKind, MonitorId, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::support::{
    RED_BLACK_LEVEL, osd_with, rtk_id, rtk_monitor, rtk_monitor_without_capabilities,
    wrote_anything,
};

/// 0x20 — horizontal position.
const HORIZONTAL_POSITION: VcpCode = VcpCode(0x20);
/// 0x7E — trapezoid: its reply makes `ddc-i2c` 0.2.2 panic on the dev
/// monitor, which the real backend reports as a transport error.
const TRAPEZOID: VcpCode = VcpCode(0x7E);
/// The catalogued codes the dev monitor's capabilities leave out, in order.
const UNDECLARED: [u8; 11] = [
    0x1E, 0x20, 0x30, 0x62, 0x6C, 0x6E, 0x70, 0x7E, 0xC9, 0xE6, 0xF1,
];

fn panicked() -> DdcError {
    DdcError::Transport(
        "ddc-hi panicked: index out of bounds: the len is 11 but the index is 11".to_owned(),
    )
}

/// The dev monitor as the probe sees it: 0x62 and 0x6C answer, 0x7E makes
/// the transport fail, 0x20 times out, and every other undeclared code is
/// refused as unsupported.
fn probed_rtk_monitor() -> FakeMonitor {
    rtk_monitor()
        .with_vcp_failure(TRAPEZOID, panicked())
        .with_vcp_failure(HORIZONTAL_POSITION, DdcError::Timeout)
}

fn reads_of(calls: &[BackendCall]) -> Vec<u8> {
    calls
        .iter()
        .filter_map(|call| match call {
            BackendCall::ReadVcp(_, code) => Some(code.0),
            _ => None,
        })
        .collect()
}

#[test]
fn probe_undeclared_features_reads_only_catalog_codes_absent_from_capabilities_and_distinguishes_unsupported_from_unresponsive_without_writing()
 {
    let (osd, backend) = osd_with([probed_rtk_monitor()]);

    let probed = osd.probe_undeclared_features(&rtk_id()).unwrap();

    let codes: Vec<u8> = probed.iter().map(|probe| probe.code.0).collect();
    assert_eq!(codes, UNDECLARED);
    let calls = backend.calls();
    assert_eq!(calls[0], BackendCall::ReadCapabilities(rtk_id()));
    assert_eq!(
        reads_of(&calls),
        UNDECLARED,
        "each code read once, in order"
    );
    assert_eq!(calls.len(), 1 + UNDECLARED.len());
    assert!(!wrote_anything(&backend));
    for probe in &probed {
        let code = probe.code;
        match code {
            VcpCode::AUDIO_VOLUME | RED_BLACK_LEVEL => {
                let reading = probe.outcome.as_ref().unwrap();
                let current = if code == RED_BLACK_LEVEL { 50 } else { 30 };
                assert_eq!(reading.value, VcpValue { current, max: 100 }, "{code}");
                assert!(!reading.declared_in_capabilities, "{code}");
                assert_eq!(reading.feature.kind, FeatureKind::Continuous, "{code}");
            }
            TRAPEZOID => assert_eq!(probe.outcome, Err(panicked())),
            HORIZONTAL_POSITION => assert_eq!(probe.outcome, Err(DdcError::Timeout)),
            _ => assert_eq!(
                probe.outcome,
                Err(DdcError::UnsupportedFeature(code)),
                "{code}"
            ),
        }
    }
}

/// A probed code's maximum is remembered: a safe write to it right after
/// needs no extra read (D-2026-09-26-full-osd-control-2).
#[test]
fn a_probed_code_is_written_without_reading_its_max_again() {
    let (osd, backend) = osd_with([probed_rtk_monitor()]);
    osd.probe_undeclared_features(&rtk_id()).unwrap();
    let calls_before = backend.calls().len();

    let written = osd.set_feature(&rtk_id(), RED_BLACK_LEVEL, 45, Confirm::No);

    assert_eq!(
        written,
        Ok(VcpValue {
            current: 45,
            max: 100
        })
    );
    assert_eq!(
        backend.calls()[calls_before..],
        [
            BackendCall::WriteVcp(rtk_id(), RED_BLACK_LEVEL, 45),
            BackendCall::ReadVcp(rtk_id(), RED_BLACK_LEVEL),
        ]
    );
}

#[test]
fn without_capabilities_the_whole_catalog_is_probed() {
    let (osd, backend) = osd_with([rtk_monitor_without_capabilities()]);

    let probed = osd.probe_undeclared_features(&rtk_id()).unwrap();

    let codes: Vec<VcpCode> = probed.iter().map(|probe| probe.code).collect();
    assert_eq!(codes, catalog_codes());
    assert_eq!(
        reads_of(&backend.calls()),
        catalog_codes()
            .iter()
            .map(|code| code.0)
            .collect::<Vec<_>>()
    );
    let brightness = probed
        .iter()
        .find(|probe| probe.code == VcpCode::BRIGHTNESS)
        .unwrap();
    assert_eq!(
        brightness
            .outcome
            .as_ref()
            .map(|reading| reading.value.current),
        Ok(50)
    );
    assert!(!wrote_anything(&backend));
}

#[test]
fn probing_a_monitor_that_is_not_reachable_is_not_found() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let ghost = MonitorId::new("ghost");

    let result = osd.probe_undeclared_features(&ghost);

    assert_eq!(result, Err(DdcError::MonitorNotFound(ghost.clone())));
    assert_eq!(backend.calls(), [BackendCall::ReadCapabilities(ghost)]);
}

/// A monitor unplugged mid-probe ends the probe at once instead of being
/// looked up again for every code left.
#[test]
fn a_monitor_that_vanishes_mid_probe_ends_the_probe_as_not_found() {
    let vanishing =
        rtk_monitor().with_vcp_failure(HORIZONTAL_POSITION, DdcError::MonitorNotFound(rtk_id()));
    let (osd, backend) = osd_with([vanishing]);

    let result = osd.probe_undeclared_features(&rtk_id());

    assert_eq!(result, Err(DdcError::MonitorNotFound(rtk_id())));
    assert_eq!(reads_of(&backend.calls()), [0x1E, 0x20]);
}
