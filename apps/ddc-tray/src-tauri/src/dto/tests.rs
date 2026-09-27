use ddc_core::domain::{
    Capabilities, DdcError, FeatureKind, FeatureReading, MonitorId, MonitorInfo, VcpCode, VcpValue,
};
use serde::Serialize;
use serde_json::{Value, json};

use super::{
    ControlDto, ControlValueDto, ErrorKind, FeatureDto, FeatureStatus, MonitorDto, OptionDto,
    Origin, PANEL_CHANGED, POPUP_SHOWN, PanelChangedDto, PanelDto, ReadBackDto, UiError,
};

const CAPS: &str = "(vcp(10 14(01 0B) 60(0F 11 1B) CA E6 F7(01 02)))";

fn caps() -> Capabilities {
    Capabilities::parse(CAPS).unwrap()
}

fn json_of(value: &impl Serialize) -> Value {
    serde_json::to_value(value).unwrap()
}

fn info(manufacturer: Option<&str>, model: Option<&str>) -> MonitorInfo {
    MonitorInfo {
        id: MonitorId::new("index-0"),
        manufacturer: manufacturer.map(str::to_owned),
        model: model.map(str::to_owned),
        serial: Some("01010101".to_owned()),
    }
}

fn value(current: u16, max: u16) -> VcpValue {
    VcpValue { current, max }
}

#[test]
fn event_names_are_the_contract_ones() {
    assert_eq!(POPUP_SHOWN, "popup-shown");
    assert_eq!(PANEL_CHANGED, "panel-changed");
}

#[test]
fn a_monitor_is_labelled_by_model_then_manufacturer_then_id() {
    let labels: Vec<String> = [
        info(Some("RTK"), Some("RTK QHD HDR")),
        info(Some("GSM"), None),
        info(None, None),
    ]
    .iter()
    .map(|monitor| MonitorDto::new(monitor).label)
    .collect();

    assert_eq!(labels, ["RTK QHD HDR", "GSM", "index-0"]);
}

#[test]
fn a_monitor_serializes_without_its_serial() {
    let monitor = MonitorDto::new(&info(Some("GSM"), None));

    assert_eq!(
        json_of(&monitor),
        json!({"id": "index-0", "label": "GSM", "manufacturer": "GSM", "model": null})
    );
}

#[test]
fn a_continuous_control_is_tagged_with_its_maximum() {
    let reading = FeatureReading {
        feature: caps().feature(VcpCode::BRIGHTNESS),
        value: value(75, 100),
        declared_in_capabilities: true,
    };

    assert_eq!(
        json_of(&ControlDto::new(&reading)),
        json!({
            "code": 16,
            "key": "brightness",
            "dangerous": false,
            "value": {"kind": "continuous", "current": 75, "max": 100}
        })
    );
}

#[test]
fn a_non_continuous_control_shows_the_low_byte_and_its_options() {
    let reading = FeatureReading {
        feature: caps().feature(VcpCode::INPUT_SOURCE),
        value: value(0x0211, 0x001B),
        declared_in_capabilities: true,
    };

    assert_eq!(
        json_of(&ControlDto::new(&reading)),
        json!({
            "code": 96,
            "key": "input",
            "dangerous": true,
            "value": {
                "kind": "nonContinuous",
                "current": 17,
                "options": [
                    {"value": 15, "name": "DisplayPort-1"},
                    {"value": 17, "name": "HDMI-1"},
                    {"value": 27, "name": null}
                ]
            }
        })
    );
}

#[test]
fn a_panel_serializes_in_camel_case() {
    let panel = PanelDto {
        monitor_id: "RTK-1".to_owned(),
        controls: Vec::new(),
    };

    assert_eq!(
        json_of(&panel),
        json!({"monitorId": "RTK-1", "controls": []})
    );
}

#[test]
fn declared_values_win_over_the_catalog() {
    let options = OptionDto::for_feature(&caps().feature(VcpCode::COLOR_PRESET));

    assert_eq!(
        options,
        [
            OptionDto {
                value: 0x01,
                name: Some("sRGB")
            },
            OptionDto {
                value: 0x0B,
                name: Some("User 1")
            },
        ]
    );
}

#[test]
fn undeclared_values_come_from_the_catalog() {
    let options = OptionDto::for_feature(&caps().feature(VcpCode::OSD_LOCK));

    assert_eq!(
        options,
        [
            OptionDto {
                value: 0x01,
                name: Some("OSD disabled")
            },
            OptionDto {
                value: 0x02,
                name: Some("OSD enabled")
            },
        ]
    );
}

#[test]
fn a_code_no_list_names_offers_its_declared_values_unnamed() {
    let options = OptionDto::for_feature(&caps().feature(VcpCode(0xF7)));

    assert_eq!(
        options,
        [
            OptionDto {
                value: 0x01,
                name: None
            },
            OptionDto {
                value: 0x02,
                name: None
            },
        ]
    );
}

#[test]
fn a_feature_without_any_list_offers_nothing() {
    assert_eq!(
        OptionDto::for_feature(&caps().feature(VcpCode::BRIGHTNESS)),
        []
    );
    assert_eq!(OptionDto::for_feature(&caps().feature(VcpCode(0xE6))), []);
}

#[test]
fn a_reading_status_says_why_there_is_no_value() {
    let statuses = [
        FeatureStatus::of(&Ok(value(1, 2))),
        FeatureStatus::of(&Err(DdcError::UnsupportedFeature(VcpCode(0x20)))),
        FeatureStatus::of(&Err(DdcError::Timeout)),
        FeatureStatus::of(&Err(DdcError::Transport("nak".to_owned()))),
    ];

    assert_eq!(
        statuses,
        [
            FeatureStatus::Ok,
            FeatureStatus::Unsupported,
            FeatureStatus::Unresponsive,
            FeatureStatus::Unresponsive,
        ]
    );
}

#[test]
fn a_declared_feature_that_answered_serializes_its_value() {
    let feature = caps().feature(VcpCode::OSD_LOCK);

    let entry = FeatureDto::new(&feature, Origin::Caps, &Ok(value(0x02, 0x02)));

    assert_eq!(
        json_of(&entry),
        json!({
            "code": 202,
            "alias": "osd-lock",
            "name": "OSD/Button Control",
            "dangerous": true,
            "origin": "caps",
            "status": "ok",
            "value": {
                "kind": "nonContinuous",
                "current": 2,
                "options": [
                    {"value": 1, "name": "OSD disabled"},
                    {"value": 2, "name": "OSD enabled"}
                ]
            }
        })
    );
}

#[test]
fn a_probed_feature_without_value_serializes_null() {
    let feature = Capabilities::default().feature(VcpCode(0xE6));

    let entry = FeatureDto::new(&feature, Origin::Probe, &Err(DdcError::Timeout));

    assert_eq!(
        json_of(&entry),
        json!({
            "code": 230,
            "alias": null,
            "name": "Manufacturer specific (0xE6)",
            "dangerous": true,
            "origin": "probe",
            "status": "unresponsive",
            "value": null
        })
    );
}

#[test]
fn an_uncatalogued_feature_has_neither_alias_nor_name() {
    let feature = caps().feature(VcpCode(0xF7));

    let entry = FeatureDto::new(
        &feature,
        Origin::Caps,
        &Err(DdcError::UnsupportedFeature(VcpCode(0xF7))),
    );

    assert_eq!(
        entry,
        FeatureDto {
            code: 0xF7,
            alias: None,
            name: None,
            dangerous: true,
            origin: Origin::Caps,
            status: FeatureStatus::Unsupported,
            value: None,
        }
    );
}

#[test]
fn a_read_back_is_the_value_the_monitor_reports() {
    let read_back = ReadBackDto::from(value(40, 80));

    assert_eq!(
        read_back,
        ReadBackDto {
            current: 40,
            max: 80
        }
    );
    assert_eq!(json_of(&read_back), json!({"current": 40, "max": 80}));
}

#[test]
fn panel_changed_carries_the_monitor_id() {
    let payload = PanelChangedDto {
        monitor_id: "RTK-1".to_owned(),
    };

    assert_eq!(json_of(&payload), json!({"monitorId": "RTK-1"}));
}

#[test]
fn error_kinds_serialize_as_the_stable_codes() {
    let kinds = [
        ErrorKind::NotFound,
        ErrorKind::Unsupported,
        ErrorKind::InvalidValue,
        ErrorKind::NeedsConfirmation,
        ErrorKind::Timeout,
        ErrorKind::Transport,
        ErrorKind::BackendUnavailable,
    ];

    let codes: Vec<Value> = kinds.iter().map(json_of).collect();

    assert_eq!(
        codes,
        [
            "not_found",
            "unsupported",
            "invalid_value",
            "needs_confirmation",
            "timeout",
            "transport",
            "backend_unavailable",
        ]
    );
}

#[test]
fn an_error_serializes_its_kind_and_message() {
    let error = UiError {
        kind: ErrorKind::NeedsConfirmation,
        message: "writing feature 0x60 is dangerous and was not confirmed".to_owned(),
    };

    assert_eq!(
        json_of(&error),
        json!({
            "kind": "needs_confirmation",
            "message": "writing feature 0x60 is dangerous and was not confirmed"
        })
    );
}

#[test]
fn a_table_value_keeps_the_continuous_shape() {
    let mut feature = caps().feature(VcpCode(0xE6));
    feature.kind = FeatureKind::Table;

    assert_eq!(
        ControlValueDto::new(&feature, value(3, 9)),
        ControlValueDto::Continuous { current: 3, max: 9 }
    );
}
