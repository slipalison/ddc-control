# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cargo workspace with two crates, `crates/ddc-core` and `crates/ddc-adapters`, sharing workspace lints: `unsafe_code` denied; clippy `unwrap_used`, `expect_used` and `panic` warned, with `unwrap`/`expect` allowed in tests.
- `ddc-core` domain model: `VcpCode` with named MCCS codes, `VcpValue`, `MonitorId`, `MonitorInfo`, `Feature` (kind, access, risk, allowed values), `Confirm`, `FeatureReading` and `DdcError`.
- Seed risk table (`risk_for_code`): brightness, contrast, color preset, RGB gains, volume, sharpness and OSD language are `Safe`; every other code, including unclassified ones, is `Dangerous`.
- `Capabilities::parse`, an in-house, tolerant parser for MCCS capabilities strings, tested against the real string of the "RTK QHD HDR" dev monitor.
- Ports `MonitorBackend` (driven) and `MonitorControl` (driving), and `SoftwareOsd`, which implements `MonitorControl`: capabilities cached per monitor, reads never filtered or blocked by the capabilities (unreadable ones count as declaring nothing), and writes guarded by risk confirmation and value validation, then read back.
- `InMemoryMonitorBackend` in `ddc-adapters`: a scripted fake monitor backend with a call log, used by the core's use-case tests.
- `DdcHiMonitorBackend` in `ddc-adapters`: the real DDC/CI backend over `ddc-hi` 0.4.1 (`dxva2` on Windows, `/dev/i2c-*` on Linux, no `nvapi`/macOS backends), behind the default-on crate feature `ddc-hi`.
  - It runs on a single worker thread that owns every monitor handle, so it is `Send + Sync` without `unsafe`; a static assertion checks this on every target.
  - Callers wait per-operation budgets (`DdcHiBudgets`: 1 s VCP read/write, 8 s capabilities, 5 s enumeration) and get `Timeout` past them. A job whose caller gave up never reaches the monitor.
  - No call needs an `enumerate()` first: an unknown monitor id costs one enumeration under the enumeration budget, then the call runs under its own budget; an id still missing answers `MonitorNotFound`.
  - Failed transactions are retried up to 3 times, 50 ms apart, within the budget, then answer `Transport`. When one more enumeration still fits the rest of the budget (capabilities reads, with the defaults), the monitor is looked up again first, so an unplugged one answers `MonitorNotFound`. VCP reads and writes skip that check, so a display with mute DDC/CI fails in about 100 ms.
  - `MonitorId`s come from EDID (`manufacturer-model-serial`), else from the device description, else `index-N`, with a `#N` suffix on repeats. Enumeration never probes DDC/CI.
  - VCP replies map to `current`/`max` in VESA MCCS byte order. Capabilities replies lose every NUL byte and are decoded lossily.
- `FakeMonitor::with_transient_capabilities_failures`, to script capabilities reads that fail before succeeding.
- Read-only hardware tests (`crates/ddc-adapters/tests/real_monitor.rs`), `#[ignore]`d and gated by `DDC_HW_TESTS=1`.
- `docs/linux-ddc-setup.md`: libudev build headers, loading `i2c-dev`, udev `uaccess` or `i2c` group permissions, why never to use `sudo`, and the proprietary NVIDIA driver caveat.

### Removed

- The placeholder hello-world binary at the repository root.
