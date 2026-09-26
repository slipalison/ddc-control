use std::io::{self, Write};

use ddc_core::domain::{Capabilities, FeatureReading, MonitorId, MonitorInfo, VcpCode, VcpValue};
use serde_json::{Value, json};

use super::{Format, Printer, monitor_line};

const CAPS: &str =
    "(prot(monitor)type(LCD)model(FAKE)cmds(01 02 F3)vcp(10 14(01 0B) 60(0F 11))mccs_ver(2.2))";

/// Output and error streams captured by one command's printer.
struct Captured {
    out: String,
    err: String,
}

impl Captured {
    fn json(&self) -> Value {
        serde_json::from_str(&self.out).unwrap()
    }
}

fn capture(format: Format, print: impl FnOnce(&mut Printer<'_>)) -> Captured {
    let mut out = Vec::new();
    let mut err = Vec::new();
    print(&mut Printer::new(format, &mut out, &mut err));
    Captured {
        out: String::from_utf8(out).unwrap(),
        err: String::from_utf8(err).unwrap(),
    }
}

fn id() -> MonitorId {
    MonitorId::new("FAKE-OUT-1")
}

fn monitors() -> Vec<MonitorInfo> {
    vec![
        MonitorInfo {
            id: MonitorId::new("RTK-RTK-QHD-HDR-01010101"),
            manufacturer: Some("RTK".to_owned()),
            model: Some("RTK QHD HDR".to_owned()),
            serial: Some("01010101".to_owned()),
        },
        MonitorInfo {
            id: MonitorId::new("index-2"),
            manufacturer: None,
            model: None,
            serial: None,
        },
    ]
}

fn reading(code: VcpCode, current: u16, max: u16, declared: bool) -> FeatureReading {
    FeatureReading {
        feature: Capabilities::parse(CAPS).unwrap().feature(code),
        value: VcpValue { current, max },
        declared_in_capabilities: declared,
    }
}

#[test]
fn monitor_line_shows_index_id_and_known_details() {
    let [rtk, bare] = <[MonitorInfo; 2]>::try_from(monitors()).unwrap();

    assert_eq!(
        monitor_line(1, &rtk),
        "1  RTK-RTK-QHD-HDR-01010101  (RTK RTK QHD HDR 01010101)"
    );
    assert_eq!(monitor_line(2, &bare), "2  index-2");
}

#[test]
fn list_as_text_numbers_monitors_from_one() {
    let captured = capture(Format::Text, |p| p.list(&monitors()));

    assert_eq!(
        captured.out,
        "1  RTK-RTK-QHD-HDR-01010101  (RTK RTK QHD HDR 01010101)\n2  index-2\n"
    );
    assert!(captured.err.is_empty());
}

#[test]
fn list_as_json_has_one_object_per_monitor() {
    let captured = capture(Format::Json, |p| p.list(&monitors()));
    let listed = captured.json();

    assert_eq!(
        listed,
        json!([
            {
                "index": 1,
                "id": "RTK-RTK-QHD-HDR-01010101",
                "manufacturer": "RTK",
                "model": "RTK QHD HDR",
                "serial": "01010101"
            },
            {
                "index": 2,
                "id": "index-2",
                "manufacturer": null,
                "model": null,
                "serial": null
            }
        ])
    );
    assert!(captured.err.is_empty());
}

#[test]
fn an_empty_list_is_empty_output_and_a_warning() {
    let text = capture(Format::Text, |p| p.list(&[]));
    let json = capture(Format::Json, |p| p.list(&[]));

    assert!(text.out.is_empty());
    assert_eq!(json.json(), json!([]));
    for captured in [text, json] {
        assert_eq!(captured.err, "warning: no monitor found\n");
    }
}

#[test]
fn caps_as_text_lists_fields_and_codes_with_names_and_values() {
    let caps = Capabilities::parse(CAPS).unwrap();
    let captured = capture(Format::Text, |p| p.caps(&id(), &caps));

    assert_eq!(
        captured.out,
        "monitor: FAKE-OUT-1\n\
         protocol: monitor\n\
         type: LCD\n\
         model: FAKE\n\
         mccs version: 2.2\n\
         commands: 0x01 0x02 0xF3\n\
         vcp codes:\n\
         \x20 0x10 brightness\n\
         \x20 0x14 preset: 0x01 0x0B\n\
         \x20 0x60 input: 0x0F 0x11\n"
    );
    assert!(captured.err.is_empty());
}

#[test]
fn caps_as_text_skips_unknown_fields() {
    let caps = Capabilities::parse("(vcp(62))").unwrap();
    let captured = capture(Format::Text, |p| p.caps(&id(), &caps));

    assert_eq!(
        captured.out,
        "monitor: FAKE-OUT-1\nvcp codes:\n  0x62 volume\n"
    );
}

#[test]
fn caps_as_json_maps_every_field_in_decimal() {
    let caps = Capabilities::parse(CAPS).unwrap();
    let captured = capture(Format::Json, |p| p.caps(&id(), &caps));
    let parsed = captured.json();

    assert_eq!(parsed["monitor"], "FAKE-OUT-1");
    assert_eq!(parsed["protocol"], "monitor");
    assert_eq!(parsed["type"], "LCD");
    assert_eq!(parsed["model"], "FAKE");
    assert_eq!(parsed["mccs_version"], "2.2");
    assert_eq!(parsed["commands"], json!([1, 2, 243]));
    assert_eq!(
        parsed["vcp"],
        json!([
            { "code": 16, "values": null },
            { "code": 20, "values": [1, 11] },
            { "code": 96, "values": [15, 17] }
        ])
    );
}

#[test]
fn caps_as_json_keeps_unknown_fields_as_null() {
    let caps = Capabilities::parse("(vcp(62))").unwrap();
    let parsed = capture(Format::Json, |p| p.caps(&id(), &caps)).json();

    assert_eq!(
        parsed,
        json!({
            "monitor": "FAKE-OUT-1",
            "protocol": null,
            "type": null,
            "model": null,
            "mccs_version": null,
            "commands": [],
            "vcp": [{ "code": 98, "values": null }]
        })
    );
}

#[test]
fn get_as_text_shows_code_name_and_values() {
    let captured = capture(Format::Text, |p| {
        p.get(&id(), &reading(VcpCode::BRIGHTNESS, 50, 100, true));
    });

    assert_eq!(captured.out, "0x10 brightness: 50 (0x32), max 100 (0x64)\n");
    assert!(captured.err.is_empty());
}

/// The core reports unreadable capabilities as declaring nothing, so the
/// warning must not claim the monitor left the code out.
#[test]
fn get_warns_when_the_code_is_not_declared_in_capabilities() {
    for format in [Format::Text, Format::Json] {
        let captured = capture(format, |p| {
            p.get(&id(), &reading(VcpCode::AUDIO_VOLUME, 30, 100, false));
        });

        assert_eq!(
            captured.err,
            "warning: 0x62 volume is not declared in capabilities, or they could not \
             be read; showing what the monitor answered\n"
        );
    }
}

#[test]
fn get_as_json_has_the_reading_fields() {
    let captured = capture(Format::Json, |p| {
        p.get(&id(), &reading(VcpCode::AUDIO_VOLUME, 30, 100, false));
    });

    assert_eq!(
        captured.json(),
        json!({
            "monitor": "FAKE-OUT-1",
            "code": 98,
            "name": "volume",
            "current": 30,
            "max": 100,
            "declared_in_capabilities": false
        })
    );
}

/// The label and the JSON `name` are the catalog alias.
#[test]
fn a_catalogued_code_is_labelled_by_its_alias() {
    let text = capture(Format::Text, |p| {
        p.get(&id(), &reading(VcpCode::SHARPNESS, 5, 10, true));
    });
    let json = capture(Format::Json, |p| {
        p.get(&id(), &reading(VcpCode::SHARPNESS, 5, 10, true));
    });

    assert_eq!(text.out, "0x87 sharpness: 5 (0x05), max 10 (0x0A)\n");
    assert_eq!(json.json()["name"], "sharpness");
}

#[test]
fn a_code_without_an_alias_has_no_name() {
    let text = capture(Format::Text, |p| {
        p.get(&id(), &reading(VcpCode(0x8D), 1, 2, true));
    });
    let json = capture(Format::Json, |p| {
        p.get(&id(), &reading(VcpCode(0x8D), 1, 2, true));
    });

    assert_eq!(text.out, "0x8D: 1 (0x01), max 2 (0x02)\n");
    assert_eq!(json.json()["name"], Value::Null);
    assert_eq!(json.json()["code"], 141);
}

/// Text puts the meaning right after the raw value; JSON adds
/// `value_name` for a named value and `interpreted` for any other meaning
/// (D-2026-09-26-full-osd-control-3).
#[test]
fn get_shows_what_the_catalog_says_the_value_means() {
    let cases = [
        (
            VcpCode::COLOR_PRESET,
            0x01,
            0x0B,
            "0x14 preset: 1 (0x01) sRGB, max 11 (0x0B)\n",
            json!("sRGB"),
            Value::Null,
        ),
        (
            VcpCode::VERTICAL_FREQUENCY,
            14400,
            0xFFFF,
            "0xAE v-frequency: 14400 (0x3840) 144.00 Hz, max 65535 (0xFFFF)\n",
            Value::Null,
            json!("144.00 Hz"),
        ),
        (
            VcpCode::VCP_VERSION,
            0x0202,
            0xFFFF,
            "0xDF vcp-version: 514 (0x202) 2.2, max 65535 (0xFFFF)\n",
            Value::Null,
            json!("2.2"),
        ),
        (
            VcpCode(0xAC),
            3,
            0xFFFF,
            "0xAC h-frequency: 3 (0x03), max 65535 (0xFFFF)\n",
            Value::Null,
            Value::Null,
        ),
    ];
    for (code, current, max, line, value_name, interpreted) in cases {
        let text = capture(Format::Text, |p| {
            p.get(&id(), &reading(code, current, max, true));
        });
        let json = capture(Format::Json, |p| {
            p.get(&id(), &reading(code, current, max, true));
        })
        .json();

        assert_eq!(text.out, line, "{code}");
        assert_eq!(json["value_name"], value_name, "{code}");
        assert_eq!(json["interpreted"], interpreted, "{code}");
    }
}

/// A plain reading keeps exactly the JSON of the phase `cli`: the new
/// fields are left out, not null.
#[test]
fn a_reading_without_a_meaning_has_no_value_name_or_interpreted_field() {
    let json = capture(Format::Json, |p| {
        p.get(&id(), &reading(VcpCode::BRIGHTNESS, 50, 100, true));
    })
    .json();

    assert!(json.get("value_name").is_none(), "{json}");
    assert!(json.get("interpreted").is_none(), "{json}");
}

#[test]
fn set_as_text_shows_the_value_read_back() {
    let captured = capture(Format::Text, |p| {
        p.set(
            &id(),
            VcpCode::INPUT_SOURCE,
            0x11,
            VcpValue {
                current: 0x11,
                max: 0x12,
            },
        );
    });

    assert_eq!(
        captured.out,
        "0x60 input: 17 (0x11) HDMI-1, max 18 (0x12)\n"
    );
    assert!(captured.err.is_empty());
}

#[test]
fn set_as_json_names_the_value_read_back() {
    let captured = capture(Format::Json, |p| {
        p.set(
            &id(),
            VcpCode::COLOR_PRESET,
            0x05,
            VcpValue {
                current: 0x05,
                max: 0x0B,
            },
        );
    });

    assert_eq!(
        captured.json(),
        json!({
            "monitor": "FAKE-OUT-1",
            "code": 20,
            "name": "preset",
            "requested": 5,
            "current": 5,
            "max": 11,
            "value_name": "6500 K",
            "applied": true
        })
    );
}

/// A write-only feature is never read back, so `set` on one prints what
/// was sent, as `reset` does (D-2026-09-26-full-osd-control-8).
#[test]
fn a_write_only_feature_prints_only_what_was_sent() {
    fn via_sent(p: &mut Printer<'_>) {
        p.sent(&id(), VcpCode::RESTORE_FACTORY_COLOR, 1);
    }
    fn via_set(p: &mut Printer<'_>) {
        let sent = VcpValue { current: 1, max: 1 };
        p.set(&id(), VcpCode::RESTORE_FACTORY_COLOR, 1, sent);
    }
    for print in [via_sent as fn(&mut Printer<'_>), via_set] {
        let text = capture(Format::Text, print);
        let json = capture(Format::Json, print);

        assert_eq!(
            text.out,
            "0x08: sent 1 (0x01) Reset (write-only, not read back)\n"
        );
        assert!(text.err.is_empty(), "{}", text.err);
        assert_eq!(
            json.json(),
            json!({
                "monitor": "FAKE-OUT-1",
                "code": 8,
                "name": null,
                "requested": 1,
                "value_name": "Reset",
                "read_back": false
            })
        );
    }
}

#[test]
fn set_as_json_reports_an_applied_write() {
    let captured = capture(Format::Json, |p| {
        p.set(
            &id(),
            VcpCode::BRIGHTNESS,
            60,
            VcpValue {
                current: 60,
                max: 100,
            },
        );
    });

    assert_eq!(
        captured.json(),
        json!({
            "monitor": "FAKE-OUT-1",
            "code": 16,
            "name": "brightness",
            "requested": 60,
            "current": 60,
            "max": 100,
            "applied": true
        })
    );
    assert!(captured.err.is_empty());
}

#[test]
fn a_write_the_monitor_ignored_is_not_applied_and_warned() {
    let read_back = VcpValue {
        current: 50,
        max: 100,
    };
    let text = capture(Format::Text, |p| {
        p.set(&id(), VcpCode::BRIGHTNESS, 60, read_back);
    });
    let json = capture(Format::Json, |p| {
        p.set(&id(), VcpCode::BRIGHTNESS, 60, read_back);
    });

    assert_eq!(text.out, "0x10 brightness: 50 (0x32), max 100 (0x64)\n");
    assert_eq!(json.json()["applied"], false);
    assert_eq!(json.json()["requested"], 60);
    assert_eq!(json.json()["current"], 50);
    for captured in [text, json] {
        assert_eq!(
            captured.err,
            "warning: the monitor accepted 0x10 brightness = 60 (0x3C) but reads back \
             50 (0x32); it may have ignored the write\n"
        );
    }
}

#[test]
fn errors_go_to_the_error_stream_in_both_formats() {
    for format in [Format::Text, Format::Json] {
        let captured = capture(format, |p| p.error("monitor X not found"));

        assert!(captured.out.is_empty());
        assert_eq!(captured.err, "error: monitor X not found\n");
    }
}

/// A stream that refuses every write, like a closed pipe.
struct Broken;

impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

#[test]
fn failing_streams_are_ignored() {
    let caps = Capabilities::parse(CAPS).unwrap();
    for format in [Format::Text, Format::Json] {
        let (mut out, mut err) = (Broken, Broken);
        let mut printer = Printer::new(format, &mut out, &mut err);

        printer.list(&monitors());
        printer.list(&[]);
        printer.caps(&id(), &caps);
        printer.get(&id(), &reading(VcpCode::AUDIO_VOLUME, 30, 100, false));
        printer.set(
            &id(),
            VcpCode::BRIGHTNESS,
            60,
            VcpValue {
                current: 50,
                max: 100,
            },
        );
        printer.error("still no panic");
    }
    assert!(Broken.flush().is_err());
}
