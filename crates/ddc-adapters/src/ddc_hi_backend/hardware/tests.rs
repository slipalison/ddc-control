use std::error::Error as _;
use std::io;

use ddc::ErrorCode;
use ddc_core::domain::{DdcError, MonitorId, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;
use ddc_hi::{Backend, DisplayInfo};

use super::super::DdcHiBudgets;
use super::super::identity::{DisplayIdentity, EdidIdentity};
use super::super::retry::RetryPolicies;
use super::super::worker::{DdcHandle, DisplaySource, HandleError, VcpReply, WorkerClient};
use super::{capabilities_text, display_identity, transaction_error, vcp_reply, vcp_value};

fn reply(mh: u8, ml: u8, sh: u8, sl: u8) -> ddc_hi::VcpValue {
    ddc_hi::VcpValue {
        ty: 0,
        mh,
        ml,
        sh,
        sl,
    }
}

/// 0x70 at 80 of 100, as `ddc` 0.2.2 decodes it under `ddc-i2c`: the code
/// the monitor echoes sits in `ty`.
fn reply_for_blue_black_level() -> ddc_hi::VcpValue {
    ddc_hi::VcpValue {
        ty: 0x70,
        ..reply(0, 100, 0, 80)
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

/// On Linux the reply hands the worker the code the monitor echoed, which
/// `ddc` 0.2.2 never compares with the request
/// (D-2026-09-26-full-osd-control-10).
#[cfg(target_os = "linux")]
#[test]
fn a_linux_reply_carries_the_code_the_monitor_echoed() {
    let value = VcpValue {
        current: 80,
        max: 100,
    };

    assert_eq!(
        vcp_reply(reply_for_blue_black_level()),
        VcpReply {
            value,
            echoed: Some(VcpCode(0x70)),
        }
    );
}

/// On Windows `ddc-winapi` puts the value type in `ty`, not an echo, so the
/// reply carries none and the worker takes the value as it is.
#[cfg(not(target_os = "linux"))]
#[test]
fn a_reply_off_linux_carries_no_echo() {
    assert_eq!(vcp_reply(reply_for_blue_black_level()).echoed, None);
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

/// What `ddc-hi` hands back on Linux when a reply fails `ddc`'s checks with
/// `code`: `ddc-i2c`'s error around it, inside `anyhow`.
fn ddc_i2c_failure(code: ErrorCode) -> anyhow::Error {
    anyhow::Error::from(ddc_i2c::Error::<io::Error>::Ddc(code))
}

/// A Get VCP Feature reply with result code 0x01, as `ddc` 0.2.2 decodes it.
fn unsupported_code() -> ErrorCode {
    ErrorCode::Invalid("Unsupported VCP code".to_owned())
}

/// The monitor's "unsupported VCP code" reply as `ddc-hi` hands it back on
/// Linux.
fn unsupported_reply() -> anyhow::Error {
    ddc_i2c_failure(unsupported_code())
}

/// One display whose every transaction fails with the error its function
/// builds, converted by the production [`transaction_error`].
#[derive(Debug, Clone, Copy)]
struct BrokenBus(fn() -> anyhow::Error);

#[derive(Debug)]
struct BrokenHandle(fn() -> anyhow::Error);

impl DisplaySource for BrokenBus {
    type Handle = BrokenHandle;

    fn enumerate(&mut self) -> Vec<(DisplayIdentity, BrokenHandle)> {
        let identity = DisplayIdentity {
            description: "broken".to_owned(),
            edid: None,
        };
        vec![(identity, BrokenHandle(self.0))]
    }
}

impl DdcHandle for BrokenHandle {
    fn read_capabilities(&mut self) -> Result<Vec<u8>, HandleError> {
        Err(transaction_error(self.0()))
    }

    fn read_vcp(&mut self, _code: VcpCode) -> Result<VcpReply, HandleError> {
        Err(transaction_error(self.0()))
    }

    fn write_vcp(&mut self, _code: VcpCode, _value: u16) -> Result<(), HandleError> {
        Err(transaction_error(self.0()))
    }
}

/// Critic of DoD #14: a real `anyhow::Error` chain, through the same
/// conversion `impl DdcHandle for Handle` uses, reaches the port as plain
/// `Transport` text with every cause and no error object behind it.
#[test]
fn ddc_hi_error_chain_reaches_the_port_as_transport_text_only() {
    let no_backoff = RetryPolicies::without_backoff();
    let client =
        WorkerClient::spawn(BrokenBus(i2c_failure), DdcHiBudgets::default(), no_backoff).unwrap();
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

/// D-2026-09-26-cli-4: only the monitor's "unsupported VCP code" reply is
/// final. The real Linux chain hides `ddc`'s `ErrorCode` from a downcast,
/// which is why the reply is recognised by its text.
#[test]
fn transaction_error_classifies_only_the_unsupported_vcp_code_reply_as_final() {
    let linux = unsupported_reply();
    let bare = anyhow::Error::new(unsupported_code()).context("Get VCP Feature");

    assert!(
        linux
            .chain()
            .all(|cause| cause.downcast_ref::<ErrorCode>().is_none())
    );
    assert_eq!(
        transaction_error(linux),
        HandleError::unsupported("DDC/CI error: Unsupported VCP code")
    );
    assert_eq!(
        transaction_error(bare),
        HandleError::unsupported("Get VCP Feature: Unsupported VCP code")
    );
}

#[test]
fn transaction_error_keeps_every_other_ddc_hi_failure_transient() {
    let unrecognized = ErrorCode::Invalid("Unrecognized VCP error code 0x02".to_owned());
    let i2c_saying_unsupported =
        ddc_i2c::Error::<io::Error>::I2c(io::Error::other("Unsupported VCP code"));
    let failures = [
        ddc_i2c_failure(ErrorCode::InvalidOffset),
        ddc_i2c_failure(ErrorCode::InvalidChecksum),
        ddc_i2c_failure(unrecognized),
        anyhow::Error::from(i2c_saying_unsupported),
        i2c_failure(),
    ];

    for failure in failures {
        let chain = format!("{failure:#}");
        assert_eq!(transaction_error(failure), HandleError::new(chain));
    }
}

/// The real "unsupported VCP code" chain, through the conversion
/// `impl DdcHandle for Handle` uses, reaches the port as
/// `UnsupportedFeature` of the code asked for, never as `Transport`.
#[test]
fn unsupported_vcp_code_reply_reaches_the_port_as_unsupported_feature() {
    let no_backoff = RetryPolicies::without_backoff();
    let client = WorkerClient::spawn(
        BrokenBus(unsupported_reply),
        DdcHiBudgets::default(),
        no_backoff,
    )
    .unwrap();
    let id = MonitorId::new("broken");
    let code = VcpCode(0x8D);

    assert_eq!(
        client.read_vcp(&id, code),
        Err(DdcError::UnsupportedFeature(code))
    );
    assert_eq!(
        client.write_vcp(&id, code, 1),
        Err(DdcError::UnsupportedFeature(code))
    );
}
