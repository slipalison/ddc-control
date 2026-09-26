//! The thin shell over `ddc-hi`: display enumeration, the handle seam, and
//! pure translations of `ddc-hi` values into core and adapter types.
//!
//! Only the worker thread ever calls into this module's shell (D-6).

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
        self.get_vcp_feature(code.0)
            .map(vcp_value)
            .map_err(transaction_error)
    }

    fn write_vcp(&mut self, code: VcpCode, value: u16) -> Result<(), HandleError> {
        self.set_vcp_feature(code.0, value)
            .map_err(transaction_error)
    }
}

/// A failed `ddc-hi` transaction as the worker sees it: the whole cause
/// chain as text, so no `ddc-hi` error type crosses the seam.
pub(crate) fn transaction_error(error: <Handle as DdcHost>::Error) -> HandleError {
    HandleError::new(error)
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
