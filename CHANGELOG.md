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
- Ports `MonitorBackend` (driven) and `MonitorControl` (driving), and `SoftwareOsd`, which implements `MonitorControl`: capabilities cached per monitor, reads never filtered or blocked by the capabilities (unreadable ones count as declaring nothing), and writes guarded by risk confirmation and value validation, then read back. A write whose maximum is not known yet reads the feature once first, whether or not the capabilities declare the code; if that read fails, nothing is written.
- `InMemoryMonitorBackend` in `ddc-adapters`: a scripted fake monitor backend with a call log, used by the core's use-case tests.
- `DdcHiMonitorBackend` in `ddc-adapters`: the real DDC/CI backend over `ddc-hi` 0.4.1 (`dxva2` on Windows, `/dev/i2c-*` on Linux, no `nvapi`/macOS backends), behind the default-on crate feature `ddc-hi`.
  - It runs on a single worker thread that owns every monitor handle, so it is `Send + Sync` without `unsafe`; a static assertion checks this on every target.
  - Callers wait per-operation budgets (`DdcHiBudgets`: 1 s VCP read/write, 8 s capabilities, 5 s enumeration) and get `Timeout` past them. A job whose caller gave up never reaches the monitor.
  - No call needs an `enumerate()` first: an unknown monitor id costs one enumeration under the enumeration budget, then the call runs under its own budget; an id still missing answers `MonitorNotFound`.
  - Failed transactions are retried up to 3 times within the budget, 50 ms apart for VCP reads and writes and 500 ms apart for capabilities reads, then answer `Transport`. A VCP code the monitor answers as unsupported (DDC/CI result code `0x01`) is not retried and answers `UnsupportedFeature` (Linux; `dxva2` on Windows keeps that reply to itself). When one more enumeration still fits the rest of the budget (capabilities reads, with the defaults), the monitor is looked up again first, so an unplugged one answers `MonitorNotFound`. VCP reads and writes skip that check, so a display with mute DDC/CI fails in about 100 ms.
  - `MonitorId`s come from EDID (`manufacturer-model-serial`), else from the device description, else `index-N`, with a `#N` suffix on repeats. Enumeration never probes DDC/CI.
  - VCP replies map to `current`/`max` in VESA MCCS byte order. Capabilities replies lose every NUL byte and are decoded lossily.
- `FakeMonitor::with_transient_capabilities_failures`, to script capabilities reads that fail before succeeding.
- Read-only hardware tests (`crates/ddc-adapters/tests/real_monitor.rs`), `#[ignore]`d and gated by `DDC_HW_TESTS=1`.
- `docs/linux-ddc-setup.md`: libudev build headers, loading `i2c-dev`, udev `uaccess` or `i2c` group permissions, why never to use `sudo`, and the proprietary NVIDIA driver caveat.
- `ddc-cli`, the command-line binary (`crates/ddc-cli`): `list`, `caps [--refresh]`, `get <vcp>` and `set <vcp> <value> [--yes]`, with `--monitor` selection (exact id, `list` index, or a unique case-insensitive part of an id; never a guess) and `--json` output.
  - `<vcp>` takes decimal, `0x` hex, or the shortcuts `brightness`, `contrast`, `input`, `preset`, `volume` and `power`.
  - Dangerous writes need `--yes`, decided by the core; there is no prompt. The value read back after a write is printed, with a warning when the monitor ignored it. A code the capabilities do not declare can be written too, such as `volume` on the dev monitor: the core reads its maximum first.
  - Exit codes: 0 success, 2 usage, 3 monitor selection, 4 invalid feature or value, 5 unconfirmed dangerous write, 6 transport or timeout. A code the monitor refuses as unsupported exits 4, not 6, for `get` and `set`.
- `CachingMonitorBackend` and `default_cache_dir()` in `ddc-adapters`: capabilities strings kept on disk per monitor id (`%LOCALAPPDATA%\ddc-control\caps`, `$XDG_CACHE_HOME/ddc-control/caps` or `~/.cache/ddc-control/caps`), so a warm `ddc-cli get` skips the ~2.5 s capabilities read. Only successful reads the core can parse are stored. `invalidate` backs `caps --refresh`: the file is replaced only by a successful re-read, so a refused refresh keeps it.
- `FakeMonitor::with_vcp_failure`, to script a VCP code whose reads and writes fail with a given error.

### Removed

- The placeholder hello-world binary at the repository root.
