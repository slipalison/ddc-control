use std::path::PathBuf;

use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::{Capabilities, DdcError, MonitorId, MonitorInfo, VcpCode};
use ddc_core::ports::MonitorControl;
use serde::Serialize;
use serde_json::Value;

use super::{
    QUICK_CONTROLS, brightness_for_percent, load_features, load_panel, monitors, probe_features,
    set_brightness_percent, ui_error,
};
use crate::dto::{
    ControlDto, ControlValueDto, ErrorKind, FeatureDto, FeatureStatus, MonitorDto, OptionDto,
    Origin, PanelDto, ReadBackDto, UiError,
};

/// Id the real backend derives for the dev monitor "RTK QHD HDR".
pub(crate) const RTK_ID: &str = "RTK-RTK-QHD-HDR-01010101";

/// Capabilities of the dev monitor, from the fixture the core's tests share.
pub(crate) const RTK_CAPS: &str =
    include_str!("../../../../../crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt")
        .trim_ascii_end();

/// 0x0C — colour temperature request, declared by the dev monitor.
const COLOR_TEMP: VcpCode = VcpCode(0x0C);

/// Set to regenerate the golden on purpose, after a deliberate contract
/// change: `DDC_TRAY_UPDATE_GOLDEN=1 cargo test -p ddc-tray --locked golden`.
const UPDATE_GOLDEN: &str = "DDC_TRAY_UPDATE_GOLDEN";

pub(crate) type Osd = SoftwareOsd<InMemoryMonitorBackend>;

pub(crate) fn rtk_id() -> MonitorId {
    MonitorId::new(RTK_ID)
}

fn rtk_info() -> MonitorInfo {
    MonitorInfo {
        id: rtk_id(),
        manufacturer: Some("RTK".to_owned()),
        model: Some("RTK QHD HDR".to_owned()),
        serial: Some("01010101".to_owned()),
    }
}

/// The contract's RTK scenario (plan A-2): the dev monitor's real
/// capabilities and identity, the six quick controls at fixed values —
/// volume answering although the capabilities omit it — and the seven "all
/// settings" codes. The demo bridge mirrors exactly these values.
pub(crate) fn rtk_monitor() -> FakeMonitor {
    FakeMonitor::new(rtk_info())
        .with_capabilities(RTK_CAPS)
        .with_value(VcpCode::BRIGHTNESS, 75, 100)
        .with_value(VcpCode::CONTRAST, 50, 100)
        .with_value(VcpCode::AUDIO_VOLUME, 30, 100)
        .with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x03)
        .with_value(VcpCode::COLOR_PRESET, 0x01, 0x0B)
        .with_value(VcpCode::POWER_MODE, 0x01, 0x05)
        .with_value(COLOR_TEMP, 70, 100)
        .with_value(VcpCode::RED_GAIN, 50, 100)
        .with_value(VcpCode::GREEN_GAIN, 48, 100)
        .with_value(VcpCode::BLUE_GAIN, 46, 100)
        .with_value(VcpCode::SHARPNESS, 5, 10)
        .with_value(VcpCode::OSD_LOCK, 0x02, 0x02)
        .with_value(VcpCode::OSD_LANGUAGE, 0x02, 0x0D)
}

/// A core wired to `monitors`, plus a handle on the same fake to inspect it.
pub(crate) fn osd_with(
    monitors: impl IntoIterator<Item = FakeMonitor>,
) -> (Osd, InMemoryMonitorBackend) {
    let backend = monitors
        .into_iter()
        .fold(InMemoryMonitorBackend::builder(), |builder, monitor| {
            builder.monitor(monitor)
        })
        .build();
    (SoftwareOsd::new(backend.clone()), backend)
}

fn writes(backend: &InMemoryMonitorBackend) -> Vec<BackendCall> {
    backend
        .calls()
        .into_iter()
        .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
        .collect()
}

fn continuous(current: u16, max: u16) -> ControlValueDto {
    ControlValueDto::Continuous { current, max }
}

fn non_continuous(current: u8, options: &[(u8, Option<&'static str>)]) -> ControlValueDto {
    ControlValueDto::NonContinuous {
        current,
        options: options
            .iter()
            .map(|&(value, name)| OptionDto { value, name })
            .collect(),
    }
}

fn control(code: u8, key: &'static str, dangerous: bool, value: ControlValueDto) -> ControlDto {
    ControlDto {
        code,
        key: Some(key),
        dangerous,
        value,
    }
}

const RTK_INPUTS: [(u8, Option<&str>); 7] = [
    (0x01, Some("VGA-1")),
    (0x03, Some("DVI-1")),
    (0x04, Some("DVI-2")),
    (0x0F, Some("DisplayPort-1")),
    (0x10, Some("DisplayPort-2")),
    (0x11, Some("HDMI-1")),
    (0x12, Some("HDMI-2")),
];

const RTK_PRESETS: [(u8, Option<&str>); 7] = [
    (0x01, Some("sRGB")),
    (0x02, Some("Display Native")),
    (0x04, Some("5000 K")),
    (0x05, Some("6500 K")),
    (0x06, Some("7500 K")),
    (0x08, Some("9300 K")),
    (0x0B, Some("User 1")),
];

const RTK_POWER_MODES: [(u8, Option<&str>); 3] = [
    (0x01, Some("On")),
    (0x04, Some("Off (DPM)")),
    (0x05, Some("Off (write-only)")),
];

fn rtk_controls() -> Vec<ControlDto> {
    vec![
        control(0x10, "brightness", false, continuous(75, 100)),
        control(0x12, "contrast", false, continuous(50, 100)),
        control(0x62, "volume", false, continuous(30, 100)),
        control(0x60, "input", true, non_continuous(0x0F, &RTK_INPUTS)),
        control(0x14, "preset", false, non_continuous(0x01, &RTK_PRESETS)),
        control(0xD6, "power", true, non_continuous(0x01, &RTK_POWER_MODES)),
    ]
}

fn entry(
    code: u8,
    alias: Option<&'static str>,
    name: &'static str,
    dangerous: bool,
    origin: Origin,
    status: FeatureStatus,
    value: Option<ControlValueDto>,
) -> FeatureDto {
    FeatureDto {
        code,
        alias,
        name: Some(name),
        dangerous,
        origin,
        status,
        value,
    }
}

fn caps_ok(
    code: u8,
    alias: &'static str,
    name: &'static str,
    value: ControlValueDto,
) -> FeatureDto {
    let dangerous = code == 0xCA;
    let ok = FeatureStatus::Ok;
    entry(
        code,
        Some(alias),
        name,
        dangerous,
        Origin::Caps,
        ok,
        Some(value),
    )
}

fn rtk_features() -> Vec<FeatureDto> {
    let osd_values = [(0x01, Some("OSD disabled")), (0x02, Some("OSD enabled"))];
    let languages = [
        (0x01, Some("Chinese (traditional)")),
        (0x02, Some("English")),
        (0x03, Some("French")),
        (0x04, Some("German")),
        (0x06, Some("Japanese")),
        (0x0A, Some("Spanish")),
        (0x0D, Some("Chinese (simplified)")),
    ];
    vec![
        caps_ok(
            0x0C,
            "color-temp",
            "Color Temperature Request",
            continuous(70, 100),
        ),
        caps_ok(0x16, "red-gain", "Video Gain (Red)", continuous(50, 100)),
        caps_ok(
            0x18,
            "green-gain",
            "Video Gain (Green)",
            continuous(48, 100),
        ),
        caps_ok(0x1A, "blue-gain", "Video Gain (Blue)", continuous(46, 100)),
        caps_ok(0x87, "sharpness", "Sharpness", continuous(5, 10)),
        caps_ok(
            0xCA,
            "osd-lock",
            "OSD/Button Control",
            non_continuous(0x02, &osd_values),
        ),
        caps_ok(
            0xCC,
            "osd-language",
            "OSD Language",
            non_continuous(0x02, &languages),
        ),
    ]
}

#[test]
fn quick_controls_are_the_six_panel_codes_in_display_order() {
    let codes: Vec<u8> = QUICK_CONTROLS.iter().map(|code| code.0).collect();

    assert_eq!(codes, [0x10, 0x12, 0x62, 0x60, 0x14, 0xD6]);
}

#[test]
fn lists_monitors_labelled_by_model() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let dynamic: &(dyn MonitorControl + Send + Sync) = &osd;

    let listed = monitors(dynamic).unwrap();

    assert_eq!(
        listed,
        [MonitorDto {
            id: RTK_ID.to_owned(),
            label: "RTK QHD HDR".to_owned(),
            manufacturer: Some("RTK".to_owned()),
            model: Some("RTK QHD HDR".to_owned()),
        }]
    );
}

#[test]
fn lists_no_monitor_when_none_is_reachable() {
    let (osd, _) = osd_with([]);

    assert_eq!(monitors(&osd).unwrap(), []);
}

#[test]
fn loads_the_whole_rtk_panel_in_quick_control_order() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    assert_eq!(
        panel,
        PanelDto {
            monitor_id: RTK_ID.to_owned(),
            controls: rtk_controls(),
        }
    );
    assert_eq!(writes(&backend), []);
}

#[test]
fn volume_is_shown_although_the_capabilities_omit_it() {
    let (osd, _) = osd_with([rtk_monitor()]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    let capabilities = Capabilities::parse(RTK_CAPS).unwrap();
    assert!(!capabilities.declares(VcpCode::AUDIO_VOLUME));
    let volume: Vec<&ControlDto> = panel.controls.iter().filter(|c| c.code == 0x62).collect();
    assert_eq!(
        volume,
        [&control(0x62, "volume", false, continuous(30, 100))]
    );
}

#[test]
fn input_and_preset_offer_the_seven_declared_values_and_power_three() {
    let (osd, _) = osd_with([rtk_monitor()]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    let value_of = |code: u8| {
        panel
            .controls
            .iter()
            .find(|control| control.code == code)
            .map(|control| control.value.clone())
    };
    assert_eq!(value_of(0x60), Some(non_continuous(0x0F, &RTK_INPUTS)));
    assert_eq!(value_of(0x14), Some(non_continuous(0x01, &RTK_PRESETS)));
    assert_eq!(value_of(0xD6), Some(non_continuous(0x01, &RTK_POWER_MODES)));
}

#[test]
fn a_control_whose_read_times_out_is_left_out_and_the_rest_still_load() {
    let monitor = rtk_monitor().with_vcp_failure(VcpCode::CONTRAST, DdcError::Timeout);
    let (osd, _) = osd_with([monitor]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    let mut expected = rtk_controls();
    expected.remove(1);
    assert_eq!(panel.controls, expected);
}

#[test]
fn an_unreachable_monitor_is_not_found() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let missing = MonitorId::new("NOPE");
    let not_found = UiError {
        kind: ErrorKind::NotFound,
        message: "monitor NOPE not found".to_owned(),
    };

    assert_eq!(load_panel(&osd, &missing), Err(not_found.clone()));
    assert_eq!(load_features(&osd, &missing), Err(not_found.clone()));
    assert_eq!(probe_features(&osd, &missing), Err(not_found));
}

#[test]
fn options_come_from_the_capabilities_named_by_the_catalog() {
    let monitor = FakeMonitor::new(rtk_info())
        .with_capabilities("(vcp(10 60(0F 11 1B)))")
        .with_value(VcpCode::INPUT_SOURCE, 0x11, 0x1B);
    let (osd, _) = osd_with([monitor]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    let options = [
        (0x0F, Some("DisplayPort-1")),
        (0x11, Some("HDMI-1")),
        (0x1B, None),
    ];
    assert_eq!(
        panel.controls,
        [control(0x60, "input", true, non_continuous(0x11, &options))]
    );
}

#[test]
fn options_fall_back_to_the_catalog_without_capabilities() {
    let monitor = FakeMonitor::new(rtk_info()).with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x12);
    let (osd, _) = osd_with([monitor]);

    let panel = load_panel(&osd, &rtk_id()).unwrap();

    assert_eq!(
        panel.controls,
        [control(
            0x60,
            "input",
            true,
            non_continuous(0x0F, &RTK_INPUTS)
        )]
    );
}

#[test]
fn all_settings_list_the_declared_adjustable_codes_beyond_the_panel() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let features = load_features(&osd, &rtk_id()).unwrap();

    let codes: Vec<u8> = features.iter().map(|feature| feature.code).collect();
    assert_eq!(codes, [0x0C, 0x16, 0x18, 0x1A, 0x87, 0xCA, 0xCC]);
    assert_eq!(features, rtk_features());
    assert_eq!(writes(&backend), []);
}

#[test]
fn all_settings_keep_a_code_that_gives_no_value_with_its_status() {
    let monitor = rtk_monitor()
        .with_vcp_failure(COLOR_TEMP, DdcError::UnsupportedFeature(COLOR_TEMP))
        .with_vcp_failure(VcpCode::SHARPNESS, DdcError::Timeout);
    let (osd, _) = osd_with([monitor]);

    let features = load_features(&osd, &rtk_id()).unwrap();

    let mut expected = rtk_features();
    expected[0] = entry(
        0x0C,
        Some("color-temp"),
        "Color Temperature Request",
        false,
        Origin::Caps,
        FeatureStatus::Unsupported,
        None,
    );
    expected[4] = entry(
        0x87,
        Some("sharpness"),
        "Sharpness",
        false,
        Origin::Caps,
        FeatureStatus::Unresponsive,
        None,
    );
    assert_eq!(features, expected);
}

#[test]
fn all_settings_fail_when_the_capabilities_cannot_be_read() {
    let monitor = FakeMonitor::new(rtk_info()).with_value(VcpCode::BRIGHTNESS, 75, 100);
    let (osd, _) = osd_with([monitor]);

    let features = load_features(&osd, &rtk_id());

    assert_eq!(
        features,
        Err(UiError {
            kind: ErrorKind::Transport,
            message: "transport error: capabilities string unavailable".to_owned(),
        })
    );
}

#[test]
fn a_probe_lists_the_undeclared_adjustable_codes_with_their_status() {
    let monitor = rtk_monitor()
        .with_value(VcpCode(0x1E), 0x00, 0x02)
        .with_value(VcpCode(0x6C), 50, 100)
        .with_value(VcpCode::FIRMWARE_LEVEL, 0x0102, 0xFFFF)
        .with_vcp_failure(VcpCode(0x7E), DdcError::Transport("nak".to_owned()));
    let (osd, backend) = osd_with([monitor]);

    let probed = probe_features(&osd, &rtk_id()).unwrap();

    let auto_setup = [
        (0x00, Some("Off")),
        (0x01, Some("Run")),
        (0x02, Some("Continuous")),
    ];
    let (probe, ok, unsupported) = (Origin::Probe, FeatureStatus::Ok, FeatureStatus::Unsupported);
    let expected = vec![
        entry(
            0x1E,
            Some("auto-setup"),
            "Auto Setup",
            true,
            probe,
            ok,
            Some(non_continuous(0x00, &auto_setup)),
        ),
        entry(
            0x20,
            Some("h-position"),
            "Horizontal Position",
            true,
            probe,
            unsupported,
            None,
        ),
        entry(
            0x30,
            Some("v-position"),
            "Vertical Position",
            true,
            probe,
            unsupported,
            None,
        ),
        entry(
            0x6C,
            Some("red-black-level"),
            "Video Black Level (Red)",
            false,
            probe,
            ok,
            Some(continuous(50, 100)),
        ),
        entry(
            0x6E,
            Some("green-black-level"),
            "Video Black Level (Green)",
            false,
            probe,
            unsupported,
            None,
        ),
        entry(
            0x70,
            Some("blue-black-level"),
            "Video Black Level (Blue)",
            false,
            probe,
            unsupported,
            None,
        ),
        entry(
            0x7E,
            Some("trapezoid"),
            "Trapezoid",
            true,
            probe,
            FeatureStatus::Unresponsive,
            None,
        ),
        entry(
            0xE6,
            None,
            "Manufacturer specific (0xE6)",
            true,
            probe,
            unsupported,
            None,
        ),
        entry(
            0xF1,
            None,
            "Manufacturer specific (0xF1)",
            true,
            probe,
            unsupported,
            None,
        ),
    ];
    assert_eq!(probed, expected);
    assert_eq!(writes(&backend), []);
}

#[test]
fn every_core_error_maps_to_its_stable_kind() {
    let cases = [
        (
            DdcError::MonitorNotFound(MonitorId::new("m1")),
            ErrorKind::NotFound,
            "monitor m1 not found",
        ),
        (
            DdcError::UnsupportedFeature(VcpCode::AUDIO_VOLUME),
            ErrorKind::Unsupported,
            "feature 0x62 is not supported",
        ),
        (
            DdcError::InvalidValue {
                code: VcpCode::BRIGHTNESS,
                value: 101,
                max: 100,
            },
            ErrorKind::InvalidValue,
            "value 101 for feature 0x10 exceeds its maximum 100",
        ),
        (
            DdcError::ValueNotAllowed {
                code: VcpCode::INPUT_SOURCE,
                value: 2,
            },
            ErrorKind::InvalidValue,
            "value 2 is not an allowed value for feature 0x60",
        ),
        (
            DdcError::DangerousWriteNotConfirmed(VcpCode::POWER_MODE),
            ErrorKind::NeedsConfirmation,
            "writing feature 0xD6 is dangerous and was not confirmed",
        ),
        (
            DdcError::Timeout,
            ErrorKind::Timeout,
            "monitor did not respond in time",
        ),
        (
            DdcError::Transport("nak".to_owned()),
            ErrorKind::Transport,
            "transport error: nak",
        ),
    ];
    for (error, kind, message) in cases {
        let expected = UiError {
            kind,
            message: message.to_owned(),
        };
        assert_eq!(ui_error(error), expected);
    }
}

#[test]
fn shortcut_percentages_of_a_maximum_of_100_are_the_percentages() {
    let values: Vec<u16> = [0, 25, 50, 75, 100]
        .into_iter()
        .map(|percent| brightness_for_percent(100, percent))
        .collect();

    assert_eq!(values, [0, 25, 50, 75, 100]);
}

#[test]
fn shortcut_percentages_scale_to_a_maximum_of_80() {
    let values: Vec<u16> = [0, 25, 50, 75, 100]
        .into_iter()
        .map(|percent| brightness_for_percent(80, percent))
        .collect();

    assert_eq!(values, [0, 20, 40, 60, 80]);
}

#[test]
fn percentages_round_to_the_nearest_step_and_stop_at_the_maximum() {
    assert_eq!(brightness_for_percent(80, 33), 26);
    assert_eq!(brightness_for_percent(255, 50), 128);
    assert_eq!(brightness_for_percent(100, 150), 100);
    assert_eq!(brightness_for_percent(u16::MAX, 255), u16::MAX);
    assert_eq!(brightness_for_percent(0, 50), 0);
}

#[test]
fn a_brightness_shortcut_writes_the_percentage_and_returns_the_read_back() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let read_back = set_brightness_percent(&osd, &rtk_id(), 25);

    assert_eq!(
        read_back,
        Ok(ReadBackDto {
            current: 25,
            max: 100
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 25)]
    );
}

#[test]
fn a_brightness_shortcut_scales_to_the_monitor_maximum() {
    let monitor = FakeMonitor::new(rtk_info())
        .with_capabilities(RTK_CAPS)
        .with_value(VcpCode::BRIGHTNESS, 60, 80);
    let (osd, backend) = osd_with([monitor]);

    let read_back = set_brightness_percent(&osd, &rtk_id(), 50);

    assert_eq!(
        read_back,
        Ok(ReadBackDto {
            current: 40,
            max: 80
        })
    );
    assert_eq!(
        writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 40)]
    );
}

#[test]
fn a_brightness_shortcut_returns_what_the_monitor_kept() {
    let monitor = rtk_monitor().ignoring_writes_to(VcpCode::BRIGHTNESS);
    let (osd, _) = osd_with([monitor]);

    let read_back = set_brightness_percent(&osd, &rtk_id(), 25);

    assert_eq!(
        read_back,
        Ok(ReadBackDto {
            current: 75,
            max: 100
        })
    );
}

#[test]
fn a_brightness_shortcut_on_an_unreachable_monitor_writes_nothing() {
    let (osd, backend) = osd_with([rtk_monitor()]);

    let read_back = set_brightness_percent(&osd, &MonitorId::new("NOPE"), 50);

    assert_eq!(
        read_back.map_err(|error| error.kind),
        Err(ErrorKind::NotFound)
    );
    assert_eq!(writes(&backend), []);
}

/// What the golden pins: the three answers the popup needs to open on the
/// RTK scenario.
#[derive(Serialize)]
struct Contract {
    monitors: Vec<MonitorDto>,
    panel: PanelDto,
    features: Vec<FeatureDto>,
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/contract-rtk.json")
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap()
}

/// The first line where `actual` departs from `golden`, with its number.
fn first_difference(golden: &str, actual: &str) -> String {
    let golden: Vec<&str> = golden.lines().collect();
    let actual: Vec<&str> = actual.lines().collect();
    let line = golden
        .iter()
        .zip(&actual)
        .position(|(expected, got)| expected != got)
        .unwrap_or_else(|| golden.len().min(actual.len()));
    format!(
        "contract differs from the golden at line {}:\n  golden: {}\n  actual: {}\n\
         Regenerate only for a deliberate contract change: {UPDATE_GOLDEN}=1",
        line + 1,
        line_at(&golden, line),
        line_at(&actual, line)
    )
}

fn line_at<'a>(lines: &[&'a str], index: usize) -> &'a str {
    lines.get(index).copied().unwrap_or("<end of document>")
}

#[test]
fn rtk_contract_matches_the_golden() {
    let (osd, _) = osd_with([rtk_monitor()]);
    let contract = Contract {
        monitors: monitors(&osd).unwrap(),
        panel: load_panel(&osd, &rtk_id()).unwrap(),
        features: load_features(&osd, &rtk_id()).unwrap(),
    };
    if std::env::var_os(UPDATE_GOLDEN).is_some() {
        let document = serde_json::to_string_pretty(&contract).unwrap();
        std::fs::write(golden_path(), format!("{document}\n")).unwrap();
    }

    let golden: Value =
        serde_json::from_str(&std::fs::read_to_string(golden_path()).unwrap()).unwrap();
    let actual = serde_json::to_value(&contract).unwrap();

    assert!(
        actual == golden,
        "{}",
        first_difference(&pretty(&golden), &pretty(&actual))
    );
}
