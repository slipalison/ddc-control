//! Read-only checks of the real `ddc-hi` backend against the dev monitor
//! "RTK QHD HDR" (MCCS 2.2). Ignored by default, and inert unless
//! `DDC_HW_TESTS=1`:
//!
//! ```text
//! DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1 --nocapture
//! ```
//!
//! Every test only enumerates and reads; none changes a monitor setting.
//! Each prints how long each operation took, as evidence for the default
//! budgets of D-7 (enumerate < 5 s, capabilities < 8 s, VCP < 1 s), for
//! D-2026-09-26-ddc-backends-1 (no enumeration inside a VCP budget) and for
//! D-2026-09-26-cli-4 (a refused code is unsupported, not a transport error).

#![cfg(feature = "ddc-hi")]

use std::time::{Duration, Instant};

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

/// Capabilities string of the dev monitor, verbatim — the fixture the
/// core's parser and use-case tests read.
const RTK_QHD_HDR_CAPS: &str =
    include_str!("../../ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt").trim_ascii_end();
/// EDID key of the dev monitor: empty serial descriptor, placeholder
/// numeric serial.
const RTK_ID: &str = "RTK-RTK-QHD-HDR-01010101";
/// 0xDF — VCP version.
const VCP_VERSION: VcpCode = VcpCode(0xDF);
/// VCP version reply of an MCCS 2.2 monitor.
const MCCS_2_2: u16 = 0x0202;
/// 0x8D audio mute and 0xDC display mode: the dev monitor neither declares
/// them nor supports them, and says so with result code 0x01.
const REFUSED_BY_DEV_MONITOR: [VcpCode; 2] = [VcpCode(0x8D), VcpCode(0xDC)];
/// Well under the default VCP budget of 1 s (D-7): a mute display fails
/// after its three attempts (~100 ms), with no presence check that cannot
/// fit the budget (D-2026-09-26-ddc-backends-1).
const FAST_FAILURE: Duration = Duration::from_millis(500);
/// A Get VCP on the dev monitor takes ~45 ms; far less than the ~1.1 s
/// enumeration that used to queue behind a failure on another display.
const NO_STALL: Duration = Duration::from_millis(250);
const REASON: &str = "needs the dev monitor attached; run with DDC_HW_TESTS=1";

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

/// Reads `code` from `id`, printing and returning how long it took.
fn timed_read(
    backend: &impl MonitorBackend,
    id: &MonitorId,
    code: VcpCode,
) -> (Result<VcpValue, DdcError>, Duration) {
    let started = Instant::now();
    let result = backend.read_vcp(id, code);
    let took = started.elapsed();
    println!("read_vcp {code} on {id}: {result:?} in {took:?}");
    (result, took)
}

fn is_dev_monitor(info: &MonitorInfo) -> bool {
    info.manufacturer.as_deref() == Some("RTK") && info.model.as_deref() == Some("RTK QHD HDR")
}

/// Enumerates, then splits the dev monitor from every other display.
fn enumerate_split(
    backend: &impl MonitorBackend,
) -> Result<(Option<MonitorInfo>, Vec<MonitorInfo>), DdcError> {
    let monitors = timed("enumerate", || backend.enumerate())?;
    println!("enumerated: {monitors:#?}");
    let (mut dev, others): (Vec<_>, Vec<_>) = monitors.into_iter().partition(is_dev_monitor);
    Ok((dev.pop(), others))
}

#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_enumerates_the_rtk_monitor_by_edid_id() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();

    let (dev, _) = enumerate_split(&backend).unwrap();

    let dev = dev.expect("RTK QHD HDR attached");
    assert_eq!(dev.id, MonitorId::new(RTK_ID));
    assert_eq!(dev.serial.as_deref(), Some("01010101"));
}

#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_capabilities_match_the_core_fixture() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();
    let dev = enumerate_split(&backend)
        .unwrap()
        .0
        .expect("RTK QHD HDR attached");

    let caps = timed("read_capabilities", || backend.read_capabilities(&dev.id));

    assert_eq!(caps.unwrap(), RTK_QHD_HDR_CAPS);
}

#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_reads_brightness_and_vcp_version() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();
    let dev = enumerate_split(&backend)
        .unwrap()
        .0
        .expect("RTK QHD HDR attached");

    let brightness = timed("read_vcp 0x10", || {
        backend.read_vcp(&dev.id, VcpCode::BRIGHTNESS)
    })
    .unwrap();
    let version = timed("read_vcp 0xDF", || backend.read_vcp(&dev.id, VCP_VERSION)).unwrap();

    println!("brightness: {brightness:?}, version: {version:?}");
    assert!(
        brightness.max > 0 && brightness.current <= brightness.max,
        "{brightness:?}"
    );
    assert_eq!(version.current, MCCS_2_2);
}

/// A fresh backend answers its very first call without an `enumerate()`
/// first: the unknown id is looked up under the enumeration budget, then
/// read under the VCP budget (D-2026-09-26-ddc-backends-1).
#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_first_read_on_a_fresh_backend_needs_no_enumerate() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();

    let (brightness, _) = timed_read(&backend, &MonitorId::new(RTK_ID), VcpCode::BRIGHTNESS);

    let brightness = brightness.unwrap();
    assert!(
        brightness.max > 0 && brightness.current <= brightness.max,
        "{brightness:?}"
    );
}

/// Every other display (on the dev machine: a TV with a readable EDID and
/// a mute DDC/CI) is listed without probing (D-4). Reading it either
/// answers (a second working monitor) or fails as a transport error well
/// under the VCP budget, and the dev monitor answers right afterwards,
/// with no enumeration queued in front of it
/// (D-2026-09-26-ddc-backends-1).
#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_other_displays_fail_within_budget_and_worker_recovers() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();
    let (dev, others) = enumerate_split(&backend).unwrap();
    let dev = dev.expect("RTK QHD HDR attached");
    if others.is_empty() {
        println!("no other display on this machine — nothing to check (D-4 not exercised)");
        return;
    }

    for other in &others {
        let (result, took) = timed_read(&backend, &other.id, VcpCode::BRIGHTNESS);
        assert!(
            matches!(result, Ok(_) | Err(DdcError::Transport(_))),
            "{result:?}"
        );
        assert!(took < FAST_FAILURE, "{} took {took:?}", other.id);
    }
    let (after, took) = timed_read(&backend, &dev.id, VcpCode::BRIGHTNESS);

    assert!(after.is_ok(), "{after:?}");
    assert!(took < NO_STALL, "the dev monitor waited {took:?}");
}

/// A code the dev monitor answers as unsupported is reported as
/// `UnsupportedFeature`, not as a transport failure, and the dev monitor
/// keeps answering afterwards (D-2026-09-26-cli-4).
#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_code_the_dev_monitor_refuses_is_unsupported() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();
    let dev = MonitorId::new(RTK_ID);

    for code in REFUSED_BY_DEV_MONITOR {
        let (refused, _) = timed_read(&backend, &dev, code);
        assert_eq!(refused, Err(DdcError::UnsupportedFeature(code)));
    }
    let (after, _) = timed_read(&backend, &dev, VcpCode::BRIGHTNESS);

    assert!(after.is_ok(), "{after:?}");
}
