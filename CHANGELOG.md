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
  - Failed transactions are retried up to 3 times within the budget, 200 ms apart for VCP reads and writes and 500 ms apart for capabilities reads, then answer `Transport`. A VCP code the monitor answers as unsupported (DDC/CI result code `0x01`) is not retried and answers `UnsupportedFeature` (Linux; `dxva2` on Windows keeps that reply to itself). When one more enumeration still fits the rest of the budget (capabilities reads, with the defaults), the monitor is looked up again first, so an unplugged one answers `MonitorNotFound`. VCP reads and writes skip that check, so a display with mute DDC/CI fails in about 0.4 s.
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
- MCCS feature catalog in `ddc-core` (`domain::mccs_catalog`): one static table of the 39 VCP codes observed on the "RTK QHD HDR" (28 declared, 9 answered without being declared, `0xE6`/`0xF1` found by a full read-only scan), each with its MCCS name, CLI name, kind, access, risk and value names, plus the lookups built on it (`catalog_entry`, `catalog_codes`, `value_name`, `value_for_name`, `code_for_alias`, `hertz`, `version_from_u16`, `interpret`).
- `MonitorControl::probe_undeclared_features` and `ProbedFeature`: reads, once each, every catalogued code the capabilities do not declare, telling a code the monitor refuses (`UnsupportedFeature`) from one it does not answer (`Transport`/`Timeout`); one failure never stops the probe, and nothing is written.
- `ddc-cli features [--probe] [--json]`: every declared feature with its current value, kind, access, risk and meaning; `--probe` adds the catalogued codes the capabilities leave out, marked `not supported by this monitor` or `not responding` (`probe_status` in JSON).
- `ddc-cli` names: `<vcp>` takes any catalog name (`sharpness`, `osd-language`, `red-black-level`, `v-frequency`…), and `<value>` takes value names, ignoring case and punctuation (`set preset srgb`, `set preset 6500k`, `set input hdmi-1 --yes`). A name that is no value of the feature being set exits 2 before any monitor is touched.
- `get` and `set` show what a value means (`1 (0x01) sRGB`, `514 (0x202) 2.2`); JSON gains `value_name`/`interpreted` only when there is one.
- `ddc-cli reset <factory|brightness-contrast|geometry|color> [--yes]`, the same write as `set <code> reset --yes`: `0x01` to `0x04`/`0x05`/`0x06`/`0x08`, refused without `--yes` (exit 5), reported as sent since the reset codes are write-only.

### Changed

- Write risk comes from the catalog, and any code outside it stays dangerous. `0x0C` (color temperature) and `0x6C`/`0x6E`/`0x70` (black levels) can now be written without `--yes`. Read-only codes (`0x0B`, `0x52`, `0xAC`, `0xAE`, `0xB2`, `0xB6`, `0xC6`, `0xC8`, `0xC9`, `0xDF`, `0xFD`, `0xFF`) are refused as not writable (exit 4) instead of asking for `--yes` (exit 5).
- A non-continuous feature whose capabilities list no values (a preset or OSD language with unreadable capabilities, an input source a monitor does not list) no longer accepts any value up to the maximum it reports: it is checked against the catalog's value names, and refused when there are none (`0x02`).
- `0x1E` (auto setup) and `0xCA` (OSD/button control) are written only with their MCCS values, still with `--yes`: `00` Off, `01` Run, `02` Continuous for `0x1E`; `01` OSD disabled, `02` OSD enabled for `0xCA`, with the button byte always zero. Before, `0xCA` took any value up to the maximum the monitor reported.
- `set_feature` refuses read-only and `Table` features before reading anything, reads a maximum only for continuous features, and never reads a write-only feature: no maximum, no read-back, the value sent is returned.
- `Capabilities::feature` takes kind and access from the catalog, for declared and undeclared codes alike.
- `ddc-cli` labels every catalogued code by its catalog name in text and in JSON `name` (`0x87 sharpness:`); the six names of earlier versions are unchanged. The `SHORTCUTS` table and `shortcut_name` are gone.
- VCP reads and writes are retried 200 ms apart instead of 50 ms: read back to back, the RTK sometimes failed a read three times in a row 50 ms apart, which made a random code of `features --probe` read `not responding` in 3 of 4 runs; at 200 ms, 0 of 4. Three attempts still fit the 1 s VCP budget; a display with mute DDC/CI now fails a read in about 0.4 s instead of 0.1 s.

### Fixed

- A panic inside `ddc-hi` (`ddc-i2c` 0.2.2 indexes out of bounds on the RTK's reply to `0x7E`) ended the backend's worker thread, and every later call in the process failed. Now only that call fails, as a transport error, and the worker keeps serving.
- On Linux a late reply to an earlier read could pass for the answer to the next code (`features --probe` showed `0x7E` with `0x70`'s value); a reply for another code is now refused and the read retried.

### Removed

- The placeholder hello-world binary at the repository root.
