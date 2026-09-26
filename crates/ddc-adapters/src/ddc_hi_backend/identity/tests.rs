use ddc_core::domain::{MonitorId, MonitorInfo};

use super::{DisplayIdentity, EdidIdentity, monitor_infos};

/// The dev monitor as its EDID reads: empty serial descriptor, placeholder
/// numeric serial.
fn rtk_dev_monitor() -> DisplayIdentity {
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
}

fn without_edid(description: &str) -> DisplayIdentity {
    DisplayIdentity {
        description: description.to_owned(),
        edid: None,
    }
}

fn ids(identities: &[DisplayIdentity]) -> Vec<String> {
    monitor_infos(identities)
        .into_iter()
        .map(|info| info.id.as_str().to_owned())
        .collect()
}

#[test]
fn derives_monitor_id_from_edid_and_disambiguates_duplicate_serials() {
    let mut labelled = rtk_dev_monitor();
    if let Some(edid) = labelled.edid.as_mut() {
        edid.serial_number = Some(" SN 42 ".to_owned());
    }

    let infos = monitor_infos(&[rtk_dev_monitor(), rtk_dev_monitor(), labelled]);

    assert_eq!(
        infos[0],
        MonitorInfo {
            id: MonitorId::new("RTK-RTK-QHD-HDR-01010101"),
            manufacturer: Some("RTK".to_owned()),
            model: Some("RTK QHD HDR".to_owned()),
            serial: Some("01010101".to_owned()),
        }
    );
    assert_eq!(infos[1].id.as_str(), "RTK-RTK-QHD-HDR-01010101#2");
    assert_eq!(infos[2].id.as_str(), "RTK-RTK-QHD-HDR-SN-42");
    assert_eq!(infos[2].serial.as_deref(), Some("SN 42"));
}

#[test]
fn derives_monitor_id_from_description_or_index_when_edid_absent() {
    let identities = [
        without_edid("Generic PnP Monitor"),
        without_edid("Generic PnP Monitor"),
        without_edid("  "),
    ];

    let infos = monitor_infos(&identities);

    assert_eq!(
        ids(&identities),
        ["Generic-PnP-Monitor", "Generic-PnP-Monitor#2", "index-2"]
    );
    assert!(
        infos.iter().all(|info| info.manufacturer.is_none()
            && info.model.is_none()
            && info.serial.is_none())
    );
}

#[test]
fn product_code_stands_in_for_a_missing_model_name_and_absent_serial_is_omitted() {
    let identity = DisplayIdentity {
        description: "7".to_owned(),
        edid: Some(EdidIdentity {
            manufacturer: "DEL".to_owned(),
            model_name: Some(" ".to_owned()),
            model_id: Some(0x40B1),
            serial_number: None,
            serial: None,
        }),
    };

    let infos = monitor_infos(&[identity]);

    assert_eq!(infos[0].id.as_str(), "DEL-40B1");
    assert_eq!(infos[0].model, None);
    assert_eq!(infos[0].serial, None);
}

#[test]
fn keys_keep_only_ascii_alphanumerics_joined_by_single_dashes() {
    assert_eq!(
        ids(&[
            without_edid(r"  \\.\DISPLAY1\Monitor0  "),
            without_edid("Écran — 27\"")
        ]),
        ["DISPLAY1-Monitor0", "cran-27"]
    );
}

#[test]
fn edid_that_sanitizes_to_nothing_falls_back_to_the_description() {
    let identity = DisplayIdentity {
        description: "i2c 5".to_owned(),
        edid: Some(EdidIdentity {
            manufacturer: "@@@".to_owned(),
            ..EdidIdentity::default()
        }),
    };

    assert_eq!(ids(&[identity]), ["i2c-5"]);
}
