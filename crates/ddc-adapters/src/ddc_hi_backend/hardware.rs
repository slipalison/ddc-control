//! The thin shell over `ddc-hi`: display enumeration, the handle seam, and
//! pure translations of `ddc-hi` values into core and adapter types.
//!
//! Only the worker thread ever calls into this module's shell (D-6).

use std::error::Error;

use ddc_core::domain::{VcpCode, VcpValue};
use ddc_hi::{Ddc, DdcHost, DisplayInfo, Handle};

use super::identity::{DisplayIdentity, EdidIdentity};
use super::worker::{DdcHandle, DisplaySource, HandleError};

/// The displays `ddc_hi::Display::enumerate` finds. It never probes DDC/CI
/// (D-4): a display with a readable EDID and a mute DDC/CI is listed and
/// fails later, on its first transaction.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DdcHiDisplays;

impl DisplaySource for DdcHiDisplays {
    type Handle = Handle;

    fn enumerate(&mut self) -> Vec<(DisplayIdentity, Handle)> {
        ddc_hi::Display::enumerate()
            .into_iter()
            .map(|display| (display_identity(&display.info), display.handle))
            .collect()
    }
}

impl DdcHandle for Handle {
    fn read_capabilities(&mut self) -> Result<Vec<u8>, HandleError> {
        self.capabilities_string().map_err(transaction_error)
    }

    fn read_vcp(&mut self, code: VcpCode) -> Result<VcpValue, HandleError> {
        let reply = self.get_vcp_feature(code.0).map_err(transaction_error)?;
        ensure_reply_answers(code, echoed_code(&reply))?;
        Ok(vcp_value(reply))
    }

    fn write_vcp(&mut self, code: VcpCode, value: u16) -> Result<(), HandleError> {
        self.set_vcp_feature(code.0, value)
            .map_err(transaction_error)
    }
}

/// How `ddc` 0.2.2 words a Get VCP Feature reply with result code 0x01: the
/// monitor does not support the VCP code.
const UNSUPPORTED_VCP_CODE: &str = "Unsupported VCP code";
/// What `ddc-i2c` puts before the text of a `ddc` protocol error.
const DDC_I2C_PROTOCOL_PREFIX: &str = "DDC/CI error: ";

/// A failed `ddc-hi` transaction as the worker sees it: the whole cause
/// chain as text, so no `ddc-hi` error type crosses the seam. The monitor's
/// "unsupported VCP code" reply is final; any other failure is transient
/// (D-2026-09-26-cli-4). On Windows `dxva2` decodes replies itself and
/// `ddc-winapi` hands back only an OS error, so every failure there stays
/// transient.
pub(crate) fn transaction_error(error: <Handle as DdcHost>::Error) -> HandleError {
    if error.chain().any(is_unsupported_reply) {
        HandleError::unsupported(error)
    } else {
        HandleError::new(error)
    }
}

/// Whether `cause` is the "unsupported VCP code" reply, bare or as
/// `ddc-i2c` wraps it.
// WHY text: `ddc-i2c` wraps `ddc::ErrorCode` in an error with no `source()`,
// so `downcast_ref::<ddc::ErrorCode>()` over the chain never finds it.
fn is_unsupported_reply(cause: &(dyn Error + 'static)) -> bool {
    let text = cause.to_string();
    let reply = text.strip_prefix(DDC_I2C_PROTOCOL_PREFIX).unwrap_or(&text);
    reply == UNSUPPORTED_VCP_CODE
}

/// The VCP code a Get VCP Feature reply says it answers, where the platform
/// backend passes it on. `ddc` 0.2.2, under `ddc-i2c`, stores the code the
/// monitor echoes in `ty` and never compares it with the request. `dxva2`
/// checks replies itself, and `ddc-winapi` puts the value type in `ty`.
#[cfg(target_os = "linux")]
fn echoed_code(reply: &ddc_hi::VcpValue) -> Option<u8> {
    Some(reply.ty)
}

#[cfg(not(target_os = "linux"))]
fn echoed_code(_: &ddc_hi::VcpValue) -> Option<u8> {
    None
}

/// Refuses a reply that answers another VCP code: a late reply to an earlier
/// request, left on the bus when that one gave up. Taking it would report,
/// and bound writes by, another feature's value; like any garbled reply it is
/// a transient failure, so the read is tried again.
pub(crate) fn ensure_reply_answers(code: VcpCode, echoed: Option<u8>) -> Result<(), HandleError> {
    match echoed {
        Some(other) if other != code.0 => Err(HandleError::new(format_args!(
            "reply answers VCP code {}, not {code}",
            VcpCode(other)
        ))),
        _ => Ok(()),
    }
}

/// A Get VCP Feature reply in core terms: maximum then current, each a
/// big-endian byte pair (VESA MCCS, D-5).
pub(crate) fn vcp_value(reply: ddc_hi::VcpValue) -> VcpValue {
    VcpValue {
        current: u16::from_be_bytes([reply.sh, reply.sl]),
        max: u16::from_be_bytes([reply.mh, reply.ml]),
    }
}

/// The capabilities reply as text. Every NUL goes: some scalers terminate
/// each fragment with one, and an inner NUL would split a token for the
/// core's parser.
pub(crate) fn capabilities_text(raw: &[u8]) -> String {
    let bytes: Vec<u8> = raw.iter().copied().filter(|&byte| byte != 0).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Identity of a display; EDID was read exactly when the manufacturer is
/// known.
pub(crate) fn display_identity(info: &DisplayInfo) -> DisplayIdentity {
    DisplayIdentity {
        description: info.id.clone(),
        edid: info
            .manufacturer_id
            .clone()
            .map(|manufacturer| EdidIdentity {
                manufacturer,
                model_name: info.model_name.clone(),
                model_id: info.model_id,
                serial_number: info.serial_number.clone(),
                serial: info.serial,
            }),
    }
}

#[cfg(test)]
mod tests;
