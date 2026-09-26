use std::error::Error as _;
use std::io;

use ddc_core::domain::{DdcError, MonitorId, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;
use ddc_hi::{Backend, DisplayInfo};

use super::super::DdcHiBudgets;
use super::super::identity::{DisplayIdentity, EdidIdentity};
use super::super::retry::RetryPolicies;
use super::super::worker::{DdcHandle, DisplaySource, HandleError, WorkerClient};
use super::{capabilities_text, display_identity, transaction_error, vcp_value};

fn reply(mh: u8, ml: u8, sh: u8, sl: u8) -> ddc_hi::VcpValue {
    ddc_hi::VcpValue {
        ty: 0,
        mh,
        ml,
        sh,
        sl,
    }
}

#[test]
fn maps_ddc_hi_vcp_reply_to_core_current_and_max() {
    assert_eq!(
        vcp_value(reply(0x01, 0x02, 0x03, 0x04)),
        VcpValue {
            current: 0x0304,
            max: 0x0102
        }
    );
    assert_eq!(
        vcp_value(reply(0, 100, 0, 50)),
        VcpValue {
            current: 50,
            max: 100
        }
    );
}

#[test]
fn capabilities_text_drops_every_nul_byte() {
    let raw = b"(prot(monitor)vcp(10\0 12))\0";

    assert_eq!(capabilities_text(raw), "(prot(monitor)vcp(10 12))");
}

#[test]
fn capabilities_text_replaces_invalid_utf8() {
    let raw = b"(model(\xFFRTK))";

    assert_eq!(capabilities_text(raw), "(model(\u{FFFD}RTK))");
}

#[test]
fn edid_backed_display_info_keeps_its_identity_fields() {
    let mut info = DisplayInfo::new(Backend::I2cDevice, "22533".to_owned());
    info.manufacturer_id = Some("RTK".to_owned());
    info.model_name = Some("RTK QHD HDR".to_owned());
    info.model_id = Some(0x8B8A);
    info.serial = Some(0x0101_0101);

    assert_eq!(
        display_identity(&info),
        DisplayIdentity {
            description: "22533".to_owned(),
            edid: Some(EdidIdentity {
                manufacturer: "RTK".to_owned(),
                model_name: Some("RTK QHD HDR".to_owned()),
                model_id: Some(0x8B8A),
                serial_number: None,
                serial: Some(0x0101_0101),
            }),
        }
    );
}

#[test]
fn display_info_without_manufacturer_carries_only_its_description() {
    let mut info = DisplayInfo::new(Backend::WinApi, "Generic PnP Monitor".to_owned());
    info.model_name = Some("ignored without EDID".to_owned());

    assert_eq!(
        display_identity(&info),
        DisplayIdentity {
            description: "Generic PnP Monitor".to_owned(),
            edid: None,
        }
    );
}

/// A `ddc-hi` failure with a cause chain. It is the type `ddc-hi` hands
/// the adapter, or passing it to [`transaction_error`] would not compile.
fn i2c_failure() -> anyhow::Error {
    anyhow::Error::new(io::Error::other("remote I/O error"))
        .context("DDC/CI I2C error")
        .context("Get VCP Feature")
}

/// `{:#}` of [`i2c_failure`]: every cause, outermost first.
const I2C_FAILURE_CHAIN: &str = "Get VCP Feature: DDC/CI I2C error: remote I/O error";

/// One display whose every transaction fails with [`i2c_failure`],
/// converted by the production [`transaction_error`].
#[derive(Debug, Clone, Copy)]
struct BrokenBus;

#[derive(Debug)]
struct BrokenHandle;

impl DisplaySource for BrokenBus {
    type Handle = BrokenHandle;

    fn enumerate(&mut self) -> Vec<(DisplayIdentity, BrokenHandle)> {
        let identity = DisplayIdentity {
            description: "broken".to_owned(),
            edid: None,
        };
        vec![(identity, BrokenHandle)]
    }
}

impl DdcHandle for BrokenHandle {
    fn read_capabilities(&mut self) -> Result<Vec<u8>, HandleError> {
        Err(transaction_error(i2c_failure()))
    }

    fn read_vcp(&mut self, _code: VcpCode) -> Result<VcpValue, HandleError> {
        Err(transaction_error(i2c_failure()))
    }

    fn write_vcp(&mut self, _code: VcpCode, _value: u16) -> Result<(), HandleError> {
        Err(transaction_error(i2c_failure()))
    }
}

/// Critic of DoD #14: a real `anyhow::Error` chain, through the same
/// conversion `impl DdcHandle for Handle` uses, reaches the port as plain
/// `Transport` text with every cause and no error object behind it.
#[test]
fn ddc_hi_error_chain_reaches_the_port_as_transport_text_only() {
    let no_backoff = RetryPolicies::without_backoff();
    let client = WorkerClient::spawn(BrokenBus, DdcHiBudgets::default(), no_backoff).unwrap();
    let id = MonitorId::new("broken");

    let failures = [
        client.read_capabilities(&id).map(drop),
        client.read_vcp(&id, VcpCode::BRIGHTNESS).map(drop),
        client.write_vcp(&id, VcpCode::BRIGHTNESS, 10),
    ];

    let expected = format!("{I2C_FAILURE_CHAIN} (gave up after attempt 3 of 3)");
    for failure in failures {
        let error = failure.unwrap_err();
        assert!(error.source().is_none(), "{error:?}");
        assert_eq!(error, DdcError::Transport(expected.clone()));
    }
}
