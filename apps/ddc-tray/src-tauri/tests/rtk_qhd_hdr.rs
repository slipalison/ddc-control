//! The tray's path to the dev monitor "RTK QHD HDR" (MCCS 2.2), through the
//! composition root the app itself uses (D-2026-09-26-tray-app-9). Ignored
//! by default, and inert unless `DDC_HW_TESTS=1`:
//!
//! ```text
//! DDC_HW_TESTS=1 cargo test -p ddc-tray --locked -- --ignored rtk_qhd_hdr --test-threads=1 --nocapture
//! ```
//!
//! Only reads, plus ONE safe brightness write — never confirmed — that a
//! guard puts back to the original value and checks by reading it back,
//! even when an assertion fails first. Nothing dangerous (input, power,
//! OSD lock, resets) is ever written.

use std::time::Instant;

use ddc_core::domain::{MonitorId, VcpCode};
use ddc_core::ports::MonitorControl;
use ddc_tray::commands::{WriteRequest, write_feature};
use ddc_tray::compose_osd;
use ddc_tray::dto::{ControlDto, ControlValueDto, MonitorDto, PanelDto, ReadBackDto};
use ddc_tray::panel;

const REASON: &str = "needs the dev monitor attached; run with DDC_HW_TESTS=1";
/// The quick controls the popup shows for the dev monitor, in order.
const RTK_PANEL: [u8; 6] = [0x10, 0x12, 0x62, 0x60, 0x14, 0xD6];
/// How far the one test write moves brightness away from its value.
const STEP: u16 = 10;

fn hardware_enabled() -> bool {
    let enabled = std::env::var("DDC_HW_TESTS").is_ok_and(|value| value == "1");
    if !enabled {
        eprintln!("skipped: {REASON}");
    }
    enabled
}

fn timed<T>(label: &str, op: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let result = op();
    println!("{label}: {:?}", started.elapsed());
    result
}

fn is_dev_monitor(monitor: &MonitorDto) -> bool {
    monitor.manufacturer.as_deref() == Some("RTK")
        && monitor.model.as_deref() == Some("RTK QHD HDR")
}

fn control(panel: &PanelDto, code: u8) -> Option<&ControlDto> {
    panel.controls.iter().find(|control| control.code == code)
}

/// How many values the pick of `code` offers; `None` unless it is a pick.
fn option_count(panel: &PanelDto, code: u8) -> Option<usize> {
    match &control(panel, code)?.value {
        ControlValueDto::NonContinuous { options, .. } => Some(options.len()),
        ControlValueDto::Continuous { .. } => None,
    }
}

/// The `(current, max)` of the slider of `code`; `None` unless it is one.
fn slider(panel: &PanelDto, code: u8) -> Option<(u16, u16)> {
    match control(panel, code)?.value {
        ControlValueDto::Continuous { current, max } => Some((current, max)),
        ControlValueDto::NonContinuous { .. } => None,
    }
}

/// The brightness a safe test write can move to: 10 up, else 10 down.
fn nearby(current: u16, max: u16) -> u16 {
    if current + STEP <= max {
        current + STEP
    } else {
        current - STEP
    }
}

/// A safe, unconfirmed brightness write — the only kind this test sends.
fn brightness(monitor_id: &MonitorId, value: u16) -> WriteRequest {
    WriteRequest {
        monitor_id: monitor_id.clone(),
        code: VcpCode::BRIGHTNESS,
        value,
        confirmed: false,
    }
}

/// Puts brightness back to `original` when dropped — also when the test
/// fails before — and checks the monitor reads it back.
struct RestoreBrightness<'a> {
    osd: &'a (dyn MonitorControl + Send + Sync),
    monitor_id: MonitorId,
    original: u16,
}

impl Drop for RestoreBrightness<'_> {
    fn drop(&mut self) {
        let request = brightness(&self.monitor_id, self.original);
        let restored = timed("restore 0x10", || write_feature(self.osd, &request));
        let reread = self.osd.get_feature(&self.monitor_id, VcpCode::BRIGHTNESS);
        println!("restored brightness: {restored:?}; fresh read: {reread:?}");
        if std::thread::panicking() {
            return;
        }
        assert_eq!(restored.map(|back| back.current), Ok(self.original));
        assert_eq!(
            reread.map(|reading| reading.value.current),
            Ok(self.original)
        );
    }
}

#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored() {
    if !hardware_enabled() {
        return;
    }
    let osd = compose_osd().expect("the DDC/CI backend starts");
    let monitors = timed("list_monitors", || panel::monitors(&*osd)).unwrap();
    println!("monitors: {monitors:#?}");
    let dev = monitors
        .iter()
        .find(|monitor| is_dev_monitor(monitor))
        .expect("RTK QHD HDR attached");
    let monitor_id = MonitorId::new(dev.id.clone());

    let panel = timed("load_panel", || panel::load_panel(&*osd, &monitor_id)).unwrap();
    println!("panel: {panel:#?}");
    let codes: Vec<u8> = panel.controls.iter().map(|control| control.code).collect();
    assert_eq!(codes, RTK_PANEL);
    assert_eq!(option_count(&panel, 0x60), Some(7));
    assert_eq!(option_count(&panel, 0x14), Some(7));
    assert_eq!(option_count(&panel, 0xD6), Some(3));
    let (current, max) = slider(&panel, 0x10).expect("brightness is a slider");
    let target = nearby(current, max);

    let _restore = RestoreBrightness {
        osd: &*osd,
        monitor_id: monitor_id.clone(),
        original: current,
    };
    let written = timed("write 0x10", || {
        write_feature(&*osd, &brightness(&monitor_id, target))
    });
    println!("brightness {current} -> {target} (max {max}): read back {written:?}");

    assert_eq!(
        written,
        Ok(ReadBackDto {
            current: target,
            max
        })
    );
}
