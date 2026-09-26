# ddc-control

Control everything your monitor's physical OSD offers — brightness, contrast, input source, color preset, volume, power — from software, over DDC/CI, with the same Rust binary on Windows and Linux.

**Status:** pre-alpha. Phases 1 (`core-domain`), 2 (`ddc-backends`) and 3 (`cli`) are implemented: the `ddc-cli` binary lists monitors, shows their capabilities and reads their features on real hardware. Writes pass the same checks and are tested against the in-memory backend; they have not been validated on real hardware yet. The tray app comes next. See `.jdi/ROADMAP.md` (run `npx -y jdi-cli render` to regenerate it).

## Install

```sh
cargo install --path crates/ddc-cli --locked
```

On Linux your user needs access to `/dev/i2c-*` first: see [`docs/linux-ddc-setup.md`](docs/linux-ddc-setup.md). Never run `ddc-cli` with `sudo`.

## Usage

```text
ddc-cli list                     monitors reachable over DDC/CI, numbered from 1
ddc-cli caps [--refresh]         the monitor's parsed capabilities string
ddc-cli get <vcp>                read a feature
ddc-cli set <vcp> <value> [--yes]  write a feature and read it back
```

Global options go before or after the subcommand:

- `--monitor`/`-m <id|index>` picks the monitor. Without it, the only monitor is used; with none or several attached the command fails and lists them. The value is tried as an exact id, then, when it is all digits, as the index shown by `list` (an index out of range is an error, never a partial id match), then as a case-insensitive part of exactly one id. A part of several ids fails and lists the candidates. `list` ignores it.
- `--json` prints the result as JSON, numbers in decimal. Errors and warnings always go to stderr as text, so scripts parse stdout and check the exit code.

`<vcp>` is a code in decimal (`16`) or hex (`0x10`), or one of six shortcuts, case-insensitive:

| Shortcut | Code | Write |
|---|---|---|
| `brightness` | `0x10` | safe |
| `contrast` | `0x12` | safe |
| `preset` | `0x14` | safe |
| `volume` | `0x62` | safe |
| `input` | `0x60` | dangerous, needs `--yes` |
| `power` | `0xD6` | dangerous, needs `--yes` |

`<value>` is decimal or hex, up to 65535. Every write goes through the core's checks (see [Monitor-write safety](#monitor-write-safety)): a dangerous code, including any code not in the core's safe list, needs `--yes`, and there is never a prompt. The value must be one the monitor allows or within its maximum. The value read back is printed; if it differs from the one written, a warning says the monitor may have ignored the write, JSON reports `"applied": false`, and the exit code is still 0.

Example on the dev machine (Linux, the "RTK QHD HDR" monitor plus an LG TV whose DDC/CI is mute), read-only:

```text
$ ddc-cli list
1  GSM-LG-TV-SSCR2-01010101  (GSM LG TV SSCR2 01010101)
2  RTK-RTK-QHD-HDR-01010101  (RTK RTK QHD HDR 01010101)

$ ddc-cli get brightness
error: 2 monitors found; choose one with --monitor <id|index>:
  1  GSM-LG-TV-SSCR2-01010101  (GSM LG TV SSCR2 01010101)
  2  RTK-RTK-QHD-HDR-01010101  (RTK RTK QHD HDR 01010101)
e.g. --monitor 1 or --monitor GSM-LG-TV-SSCR2-01010101

$ ddc-cli --monitor rtk get brightness
0x10 brightness: 100 (0x64), max 100 (0x64)

$ ddc-cli -m 2 get volume
warning: 0x62 volume is not declared in capabilities, or they could not be read; showing what the monitor answered
0x62 volume: 30 (0x1E), max 100 (0x64)

$ ddc-cli -m rtk get brightness --json
{
  "monitor": "RTK-RTK-QHD-HDR-01010101",
  "code": 16,
  "name": "brightness",
  "current": 100,
  "max": 100,
  "declared_in_capabilities": true
}
```

The LG TV is listed because enumeration only reads EDID; its DDC/CI is mute, so reading or writing it fails with exit 6.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 2 | Invalid command line (from `clap`) |
| 3 | No single monitor matches: none attached, several without `--monitor`, no match, or an ambiguous match |
| 4 | The feature or value is not valid for the monitor |
| 5 | A dangerous write without `--yes` |
| 6 | The monitor did not answer in time, or the transport failed |

### Capabilities cache

Reading a capabilities string takes about 2.5 s on the dev monitor, and every `ddc-cli` run is a new process, so the first successful read per monitor is kept on disk, one file per monitor id:

- Windows: `%LOCALAPPDATA%\ddc-control\caps\`
- Elsewhere: `$XDG_CACHE_HOME/ddc-control/caps/` when that variable is an absolute path, else `~/.cache/ddc-control/caps/`

With the cache warm, a `get` on the dev monitor takes about 1.15 s (the enumeration) instead of about 3.7 s. Failed reads are never cached, and neither are replies the core cannot parse, such as a truncated string; a cached file the core cannot parse is ignored. `caps --refresh` reads the monitor again, and only a successful read replaces the file: a refresh the monitor refuses exits 6 and leaves the previous file for later runs. If none of those directories can be determined, `ddc-cli` runs without a cache.

The cache is keyed by monitor id, and ids built without an EDID serial (the device description on Windows, `index-N`, a `#N` suffix) can move to another monitor after a hotplug, so one monitor's capabilities could be served for another. Risk is decided per code and continuous maxima are read from the monitor, so the worst case is a write checked against the wrong list of allowed values. Run `caps --refresh` after changing monitors.

Some monitors, the dev one included, refuse a capabilities read issued right after the previous one, and keep refusing for a few hundred milliseconds after each failed attempt. Capabilities reads are therefore retried 500 ms apart, where VCP reads and writes wait 50 ms. On the dev monitor, back-to-back `caps --refresh` runs then succeed, taking about 1 s longer when the first attempt is refused. If a refresh still fails, wait a few seconds and run it again; the cached file keeps being used meanwhile.

### Known limitations

- Until the MCCS feature catalog exists (phase `full-osd-control`), a code whose capabilities entry lists no values, or that the capabilities do not declare at all, counts as continuous: a write is checked against the maximum the monitor reports. For a discrete feature outside the capabilities, such as an input source on a monitor that does not list its inputs, that maximum says little about which values are valid. Dangerous codes still need `--yes`.

## Layout

- `crates/ddc-core` — the hexagon. Domain types (`VcpCode`, `VcpValue`, `MonitorId`, `Feature`, `Risk`, `Confirm`, `DdcError`), `Capabilities` with its own MCCS capabilities-string parser, the ports `MonitorBackend` (driven) and `MonitorControl` (driving), and `SoftwareOsd`, which implements `MonitorControl` on top of any `MonitorBackend`. Depends on `thiserror` only.
- `crates/ddc-adapters` — driven adapters:
  - `DdcHiMonitorBackend`, the real backend over [`ddc-hi`](https://docs.rs/ddc-hi) 0.4 (Windows `dxva2`, Linux `/dev/i2c-*`). It sits behind the crate feature `ddc-hi`, which is on by default; `--no-default-features` builds only the fake.
    - **One worker thread** owns every monitor handle and runs every DDC/CI transaction. The Windows handles cannot cross threads, so this is what makes the backend `Send + Sync` without `unsafe`.
    - **Budgets.** Callers wait at most a per-operation budget: 1 s for a VCP read or write, 8 s for a capabilities read, 5 s for an enumeration (`DdcHiBudgets`, overridable with `with_budgets`). Past the budget they get `Timeout`. A job whose caller already gave up never reaches the monitor.
    - **No `enumerate()` needed first.** A monitor id the backend has not seen yet (a fresh backend, a saved id, a monitor plugged in later) costs one enumeration under the enumeration budget; then the call runs under its own budget. An id still missing after that answers `MonitorNotFound`.
    - **Retries.** A failed transaction is retried up to 3 times within the budget: 50 ms apart for VCP reads and writes, 500 ms apart for capabilities reads, which the dev monitor refuses for a few hundred milliseconds after a failed one. If it still fails, the answer is `Transport`, which names the attempts. Only when one more enumeration (as long as the last one) still fits the rest of the budget is the monitor looked up again first, so an unplugged one answers `MonitorNotFound`. With the default budgets that happens for capabilities reads (8 s), not for VCP reads and writes (1 s, against a ~1.1 s enumeration): a display with mute DDC/CI answers `Transport` in about 100 ms and does not hold up calls to other monitors.
    - **`MonitorId` scheme.** With EDID (Linux), the id is `manufacturer-model-serial`, sanitized to ASCII letters, digits and single dashes. The dev monitor is `RTK-RTK-QHD-HDR-01010101`. Without EDID (Windows), the id is the sanitized device description, else `index-N`. A repeated id in one enumeration gets `#2`, `#3`…; that suffix follows enumeration order, so it may change across hotplugs.
    - **Enumeration** only reads EDID and never probes DDC/CI. A display with mute DDC/CI is listed, and fails on its first read.
  - `CachingMonitorBackend`, a decorator for any backend that keeps capabilities strings on disk, one file per monitor id, in a directory its caller passes in. Only successful reads the core can parse are stored, and cache I/O never fails a call. `invalidate` makes reads reach the monitor until one succeeds and replaces the file; until then the file is kept for later runs. `default_cache_dir()` resolves the per-user directory above.
  - `InMemoryMonitorBackend`, a scripted fake the core's use-case tests run against.
- `crates/ddc-cli` — the `ddc-cli` binary, a driving adapter: clap arguments, monitor selection, text and JSON output, and the exit-code table. `main.rs` is only the composition root that wires `DdcHiMonitorBackend`, `CachingMonitorBackend` and `SoftwareOsd`.

Still to come:

- `apps/ddc-tray` — Tauri 2 tray popup: monitor picker, sliders, input/preset/power, profiles, global hotkeys, phase `tray-app`.

Architecture is locked to Hexagonal (Ports & Adapters) — see `.jdi/PROJECT.md` and `.jdi/decisions/`.

## Monitor-write safety

Every write goes through `MonitorControl::set_feature`, and the core — never an adapter — enforces, in order:

1. **Risk.** Each VCP code has a `Risk`. Brightness, contrast, color preset, RGB gains, volume, sharpness and OSD language are `Safe`. Everything else — factory resets, input source, OSD lock, power mode, the manufacturer-specific range `0xE0`–`0xFF`, and any code not yet classified — is `Dangerous` and needs `Confirm::Yes`, checked before the monitor is touched at all.
2. **No blind writes.** A feature whose capabilities list discrete values accepts only those values. Any other feature is checked against its maximum: the one from an earlier read, else one read from the monitor right before the write, whether the capabilities declare the code or not (the dev monitor answers `0x62` volume without declaring it). If that read fails, the write is refused with the read's error and nothing is written.
3. **Read-back.** After the write the value is read back once, and that reading is what `set_feature` returns — some monitors acknowledge writes they silently drop.

Reads are never filtered by the capabilities string: monitors answer codes they do not declare (the dev monitor answers `0x62` volume), so `get_feature` always reads and reports `declared_in_capabilities` instead. A capabilities string that cannot be read or parsed does not block reads or writes either: the monitor is treated as declaring no code, so a write reads the feature's maximum first. The failure is remembered per monitor, and only an explicit `capabilities` request asks the monitor again.

## Dev setup

```sh
rustup toolchain install stable
rustup component add clippy rustfmt llvm-tools-preview
rustup target add x86_64-unknown-linux-gnu      # cross `cargo check` of the pure-Rust crates from Windows
rustup target add x86_64-pc-windows-msvc        # cross `cargo check` of the Windows backend from Linux
cargo install cargo-llvm-cov --locked
git config core.hooksPath .githooks             # once per clone — see below
```

On Linux the real backend needs the libudev headers to build (`systemd-devel` on Fedora, `libudev-dev` and `pkg-config` on Debian/Ubuntu).

Quality gates (the reviewer runs exactly these):

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'
```

Cross-platform checks: the Windows one also proves, through a static assertion, that `DdcHiMonitorBackend` is `Send + Sync` on Windows.

```sh
cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu
cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc
cargo check -p ddc-cli --locked --target x86_64-pc-windows-msvc
```

Hardware tests are read-only, `#[ignore]`d, and do nothing unless `DDC_HW_TESTS=1`. They expect the dev monitor "RTK QHD HDR" to be attached. CI and reviewers never run them.

```sh
DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1 --nocapture
```

Linux: load `i2c-dev` and make sure your user can open `/dev/i2c-*` (a udev `uaccess` rule or group `i2c`). Never run the tool with `sudo`. Step by step: [`docs/linux-ddc-setup.md`](docs/linux-ddc-setup.md).

## Commits

Atomic commits are enforced by hooks in `.githooks/` (activate with `git config core.hooksPath .githooks`):

- `commit-msg` — Conventional Commits header (`type(scope): subject`, ≤ 72 chars). `feat`/`fix`/`refactor`/`perf`/`test` require a scope equal to a roadmap phase slug. A commit may not mix code (`Cargo.*`, `src/`, `crates/`, `apps/`, `.github/`) with `.jdi/` state, nor touch two phases. Large task commits get a warning.
- `pre-commit` (from JDI) — rejects code changes when no active phase (`CONTEXT.md` + `PLAN.md`) is in the index, and rejects commits of generated `.jdi/` views.

Humans may bypass one commit with `JDI_ALLOW_MIXED=1` / `JDI_GATE_DISABLE=1`. Agents never do.

## Workflow

This repo is driven by [JDI](https://github.com/slipalison/jdi-cli) inside Claude Code: `/jdi-next` routes to the right step (`/jdi-discuss` → `/jdi-plan` → `/jdi-do` → `/jdi-verify` → `/jdi-ship`). Specialists live in `.jdi/agents/` with Claude Code copies in `.claude/agents/`.

## License

MIT — see `LICENSE`.
