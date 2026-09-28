# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Security scans on every pull request and push, through the shared pipeline: Gitleaks over the whole history, TruffleHog (verified secrets) over the commits of the push or pull request, Semgrep, Trivy on `Cargo.lock` and `package-lock.json`, an SPDX SBOM, and CodeQL for Rust, JavaScript/TypeScript and the GitHub Actions workflows. The results go to the Security tab, and an `error` finding fails the run.
- SonarQube Cloud analysis on CI, whose Quality Gate fails the run. It covers the Rust code (Sonar's own Rust rules, plus `cargo llvm-cov` coverage) and the popup's JS (coverage from `node --test`). `scripts/ci/sonar-coverage.sh` writes both reports, and runs locally too.
- One `esteira / Portao` check that fails when any job of the pipeline failed, or when Sonar is off without a written reason.
- An 80% line-coverage floor for the popup's JS modules: `npm run test:unit` writes `apps/ddc-tray/coverage/lcov.info`, and it measured 88.15%. Before, the CI job printed an error annotation for the missing report and passed.

### Changed

- CI is one call to the shared `pipeline.yml` of slipalison/github-workflows, instead of `versao`, `qualidade` and `lancar` called one by one. Job names gain the `esteira / ` prefix. v0.1.0 was released before this change, without the security scans, Sonar or the gate.
- `npm test` in `apps/ddc-tray` runs `test:unit` (the `node --test` suites, with coverage), then the Playwright suite.

### Removed

- The release rehearsal on `ensaio-release/*` branches. A draft release there would have to pass the Sonar gate, and SonarQube Cloud's free plan analyzes only `main` and pull requests. The packages of every pull request remain downloadable as run artifacts.

## [0.1.0] - 2026-09-28

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
- `ddc-tray` (`apps/ddc-tray`), a Tauri 2 system-tray app over the same core, backend and capabilities cache as `ddc-cli`, installed from the release packages (below) or run with `cargo run -p ddc-tray`.
  - A 360×560 popup, hidden on focus loss and Esc: brightness, contrast and volume sliders, color preset, input source, power mode, and an "All settings" section loaded on demand, with a read-only probe of the codes the capabilities leave out.
  - Sliders write 80 ms after the last move, coalesced, one write in flight per feature; the popup shows the value the monitor reads back.
  - Every dangerous change (input, power, dangerous "All settings" entries) goes through a confirmation dialog first; `Confirm::Yes` is built only in the app's command layer.
  - Several monitors: a selector; the popup opens on the last monitor it loaded, else the first whose panel loads. A panel fails when none of its quick controls can be read, so a monitor whose DDC/CI is mute is skipped and marked "(no DDC/CI)" in the selector.
  - On Linux the icon is a StatusNotifierItem of the app's own, over `ksni` (pure Rust on D-Bus; no AppIndicator library, at build or run time): a left click toggles the popup, the mouse wheel changes the brightness by 5% per notch (0–100%, notches that arrive during a write coalesced into one write), and a right click opens the menu. Its tooltip names the monitor the wheel and the menu act on.
  - On KDE Plasma under Wayland the popup opens next to the tray icon, inside the screen's work area, with no taskbar entry: the app loads a KWin script over D-Bus at start and unloads it when it quits, through **Quit** or on SIGTERM, SIGINT or SIGHUP (which quit the app the same way), waiting at most 2 s for KWin; only a SIGKILL or a crash leaves it loaded, matching only the dead process's popup, and the next start replaces it. The script is written only to `$XDG_RUNTIME_DIR`; without it none is loaded. Elsewhere the popup opens where the compositor puts it.
  - Tray menu in English or Brazilian Portuguese, from the system's locale: "Open panel" and "Quit", plus brightness shortcuts 0/25/50/75/100% on Linux. On Windows (Tauri's tray icon) a left click toggles the popup anchored above the icon (not built on Windows yet).
  - Lists (monitor selector, color preset, "All settings" lists) are drawn inside the popup, keyboard operable as a WAI-ARIA select-only combobox: a native select's menu is a window of its own, and the popup hid as soon as it opened.
  - Popup in English or Brazilian Portuguese, every text from locale keys; light and dark themes; keyboard operable; strict CSP, tested as Tauri embeds it, with any platform config merged; capabilities limited to the app's own commands; single instance.
  - The CSP check covers the effective policy of Linux, Windows and macOS alike, each merged as Tauri's build merges it for that target (`the_effective_csp_is_the_strict_policy_on_every_target`), and a `tauri.<platform>.conf.json` must not touch `app.security`: the policy lives only in `tauri.conf.json`, and the test fails a platform file that changes `csp` or sets a `devCsp`.
  - On Linux, a restart at launch with WebKitGTK's DMA-BUF renderer off (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) unless the variable is already set: that renderer crashed the app on NVIDIA under Wayland.
  - A browser demo of the popup (`?demo=rtk|two-monitors|empty|error`, and `&fail=write,features,probe` to make those commands time out as a mute monitor does, `events,hide` to have listening to the tray and hiding the popup refused as Tauri refuses a command) without Tauri, on a local dev server only (`localhost` or `127.0.0.1`; elsewhere, as inside the app should the Tauri API be missing, the popup reports the backend unavailable, never simulated values), checked by a Playwright + axe suite (no console errors, no critical/serious accessibility violations, no native `<select>` in any state, light and dark, also after a change, **All settings** and a probe that time out, and after listening to the tray or hiding the popup is refused; a mouse drag of the brightness slider writes nothing while it moves and once, the last value, after the release) that also takes the README screenshots; `node --test` suites for the UI modules, including one that fails any literal text the scripts put into the page instead of an i18n key, and one that fails any phrase in a string or template of the scripts outside the locale files and the demo's data; a demo-only pseudo-locale (`?pseudo=1`: every translated text between `⟦` and `⟧`, the monitor's own names, value names, codes and messages marked `translate="no"`) and a Playwright check, in 27 states and both themes (the failures and waits included, with a state for every toast the scripts show, which a `node --test` check keeps so, and for every confirmation dialog), that fails any visible text or readable attribute that is neither, or that has a letter outside the marks; a demo-only guard that logs a console error, which fails any Playwright test, for a text the page shows that no translation produced and that is not marked `translate="no"`; golden JSON contracts shared by the Rust and JS tests (the RTK, and a mute monitor's panel error); a StatusNotifierItem smoke script, whose `--activate` also clicks the icon over D-Bus and requires the popup to show and stay shown, and whose `--fake --scroll` rolls the wheel over the icon against a simulated monitor (`DDC_TRAY_FAKE=1`: the RTK of the contract, in memory, never a real monitor) and requires one brightness step per vertical notch and none for a horizontal one; `DDC_TRAY_DEBUG=1` diagnostics on stderr, including each brightness write from the tray (`ddc-tray: brightness 75 -> 80`); and `#[ignore]`d hardware tests: one safe brightness change, restored, and a read-only check that a mute monitor fails its panel, both refusing to run on the simulated monitor.
- Continuous integration (`.github/workflows/ci.yml`) on every pull request, every push to `main` and on demand, through the reusable workflows of [slipalison/github-workflows](https://github.com/slipalison/github-workflows):
  - `rust-linux` (ubuntu-22.04): `cargo fmt --check`, `cargo audit`, `cargo clippy --all-targets --all-features -D warnings`, the tests under `cargo llvm-cov` with the 80% line floor (82.93% on 2026-09-27), and `cargo build -p ddc-tray --release --locked`;
  - `rust-windows` (windows-latest): the same without the audit; the first build and test run of the Windows code, all 11 test binaries passing, `ddc-tray`'s included;
  - `node-ui`: `npm test` in `apps/ddc-tray` (161 `node --test` tests, 138 Playwright tests);
  - `versao`: the next version computed from the Conventional Commits, with no tag or release.
- `.cargo/audit.toml`: the `cargo audit` policy, four advisories ignored (RUSTSEC-2018-0005, RUSTSEC-2024-0370, RUSTSEC-2024-0320, RUSTSEC-2024-0429), each with the dependency path, why ddc-control does not reach the affected code and when to check it again.
- `npm test` in `apps/ddc-tray` runs the `node --test` suites (TAP), installs Playwright's Chromium and runs the Playwright suite. Only a GitHub-hosted runner gets the browser's system libraries (`--with-deps`); elsewhere the browser goes to the user's cache, never with `sudo`.
- Release packages built by CI on every pull request and push, from the same jobs that test: the deb, the rpm and the AppImage of `ddc-tray` on Linux, its MSI and NSIS installer on Windows, each named after the product in lower case with the version and no space (`ddc-control_0.1.0_amd64.deb`), in the artifacts `pacotes-rust-linux` and `pacotes-rust-windows`.
- `ddc-cli` release archives: `ddc-cli-x86_64-unknown-linux-gnu.tar.gz` and `ddc-cli-x86_64-pc-windows-msvc.zip`, each with the binary and `LICENSE` only.
- Automatic GitHub Release: a push to `main` that passes every quality job and brings a commit calling for a new version creates the tag `v<version>` and the release, with the seven packages attached, a `SHA256SUMS` of them and this file's section above the commit notes. A push to an `ensaio-release/*` branch creates the same release as a draft, with no tag; a pull request never publishes.
- The deb and the rpm install the udev `uaccess` rule of `docs/linux-ddc-setup.md` as `/usr/lib/udev/rules.d/60-ddc-control-i2c.rules` and load `i2c-dev` at boot through `/usr/lib/modules-load.d/ddc-control.conf`; their post-install loads the module and reloads udev, best effort. The AppImage cannot install them.
- The AppImage is built only from pinned tools: its entry point (`AppRun`), `linuxdeploy`, the AppImage output plugin and the AppImage runtime come from fixed releases with their sha256 checked, and CI fails if the bundler downloads anything else or if the AppImage does not start with the checked runtime.

### Changed

- Write risk comes from the catalog, and any code outside it stays dangerous. `0x0C` (color temperature) and `0x6C`/`0x6E`/`0x70` (black levels) can now be written without `--yes`. Read-only codes (`0x0B`, `0x52`, `0xAC`, `0xAE`, `0xB2`, `0xB6`, `0xC6`, `0xC8`, `0xC9`, `0xDF`, `0xFD`, `0xFF`) are refused as not writable (exit 4) instead of asking for `--yes` (exit 5).
- A non-continuous feature whose capabilities list no values (a preset or OSD language with unreadable capabilities, an input source a monitor does not list) no longer accepts any value up to the maximum it reports: it is checked against the catalog's value names, and refused when there are none (`0x02`).
- `0x1E` (auto setup) and `0xCA` (OSD/button control) are written only with their MCCS values, still with `--yes`: `00` Off, `01` Run, `02` Continuous for `0x1E`; `01` OSD disabled, `02` OSD enabled for `0xCA`, with the button byte always zero. Before, `0xCA` took any value up to the maximum the monitor reported.
- `set_feature` refuses read-only and `Table` features before reading anything, reads a maximum only for continuous features, and never reads a write-only feature: no maximum, no read-back, the value sent is returned.
- `Capabilities::feature` takes kind and access from the catalog, for declared and undeclared codes alike.
- `ddc-cli` labels every catalogued code by its catalog name in text and in JSON `name` (`0x87 sharpness:`); the six names of earlier versions are unchanged. The `SHORTCUTS` table and `shortcut_name` are gone.
- VCP reads and writes are retried 200 ms apart instead of 50 ms: read back to back, the RTK sometimes failed a read three times in a row 50 ms apart, which made a random code of `features --probe` read `not responding` in 3 of 4 runs; at 200 ms, 0 of 4. Three attempts still fit the 1 s VCP budget; a display with mute DDC/CI now fails a read in about 0.4 s instead of 0.1 s.
- The workspace version is `0.0.0`: CI stamps the version computed from the commits into the release build and the packages, and `ddc-cli --version` built locally says `0.0.0`.
- CI's `rust-linux` job runs on Ubuntu 22.04 instead of `ubuntu-latest`, so the Linux packages run on glibc 2.35 and newer (Ubuntu 22.04, Debian 12, Mint 21 and later).
- `ddc-tray` is no longer build-from-source only: it ships as the deb, rpm, AppImage, MSI and NSIS packages above.

### Fixed

- A panic inside `ddc-hi` (`ddc-i2c` 0.2.2 indexes out of bounds on the RTK's reply to `0x7E`) ended the backend's worker thread, and every later call in the process failed. Now only that call fails, as a transport error, and the worker keeps serving.
- On Linux a late reply to an earlier read could pass for the answer to the next code (`features --probe` showed `0x7E` with `0x70`'s value); a reply for another code is now refused and the read retried.
- `ddc-tray`'s test binaries would not start on Windows (`STATUS_ENTRYPOINT_NOT_FOUND`, tauri-apps/tauri#13419): tauri-build puts the Common Controls v6 manifest only in the app binary. The build script now hands the same manifest to the linker for every binary of the crate on `windows-msvc` targets. Fixed before the tray's first Windows build, whose CI run passed all of its tests.

### Removed

- The placeholder hello-world binary at the repository root.
