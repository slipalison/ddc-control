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

### Removed

- The placeholder hello-world binary at the repository root.
