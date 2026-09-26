use ddc_core::domain::VcpValue;
use ddc_hi::{Backend, DisplayInfo};

use super::super::identity::{DisplayIdentity, EdidIdentity};
use super::{capabilities_text, display_identity, vcp_value};

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
