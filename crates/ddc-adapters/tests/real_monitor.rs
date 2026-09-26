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
//! budgets of D-7 (enumerate < 5 s, capabilities < 8 s, VCP < 1 s).

#![cfg(feature = "ddc-hi")]

use std::time::{Duration, Instant};

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};
use ddc_core::ports::MonitorBackend;

/// Capabilities string of the dev monitor, verbatim — same fixture as the
/// core's use-case tests.
const RTK_QHD_HDR_CAPS: &str = "(prot(monitor)type(LCD)model(RTK)cmds(01 02 03 07 0C E3 F3)vcp(02 04 05 06 08 0B 0C 10 12 14(01 02 04 05 06 08 0B) 16 18 1A 52 60(01 03 04 0F 10 11 12) 87 AC AE B2 B6 C6 C8 CA CC(01 02 03 04 06 0A 0D) D6(01 04 05) DF FD FF)mswhql(1)asset_eep(40)mccs_ver(2.2))";
/// EDID key of the dev monitor: empty serial descriptor, placeholder
/// numeric serial.
const RTK_ID: &str = "RTK-RTK-QHD-HDR-01010101";
/// 0xDF — VCP version.
const VCP_VERSION: VcpCode = VcpCode(0xDF);
/// VCP version reply of an MCCS 2.2 monitor.
const MCCS_2_2: u16 = 0x0202;
/// Default VCP budget (D-7).
const VCP_BUDGET: Duration = Duration::from_secs(1);
/// Scheduling slack on top of a budget.
const SLACK: Duration = Duration::from_millis(250);
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

/// Every other display (on the dev machine: a TV with a readable EDID and
/// a mute DDC/CI) is listed without probing (D-4); reading it comes back
/// within the budget, as a transport failure or a timeout, and the worker
/// still serves the dev monitor afterwards.
#[test]
#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]
fn hardware_mute_displays_fail_fast_and_worker_recovers() {
    if !hardware_enabled() {
        return;
    }
    let backend = ddc_adapters::DdcHiMonitorBackend::new().unwrap();
    let (dev, others) = enumerate_split(&backend).unwrap();
    let dev = dev.expect("RTK QHD HDR attached");

    for other in &others {
        let started = Instant::now();
        let result = backend.read_vcp(&other.id, VcpCode::BRIGHTNESS);
        let took = started.elapsed();
        println!("read_vcp 0x10 on {}: {result:?} in {took:?}", other.id);
        assert!(took <= VCP_BUDGET + SLACK, "{} took {took:?}", other.id);
        assert!(
            matches!(
                result,
                Ok(_) | Err(DdcError::Transport(_) | DdcError::Timeout)
            ),
            "{result:?}"
        );
    }
    let after = timed("read_vcp 0x10 on the dev monitor afterwards", || {
        backend.read_vcp(&dev.id, VcpCode::BRIGHTNESS)
    });

    assert!(after.is_ok(), "{after:?}");
}
