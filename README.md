# ddc-control

Control everything your monitor's physical OSD offers — brightness, contrast, input source, color preset, volume, power — from software, over DDC/CI, with the same Rust binary on Windows and Linux.

**Status:** pre-alpha. Phases 1 (`core-domain`) and 2 (`ddc-backends`) are implemented: a cargo workspace with two library crates that can already talk to real monitors, but no binary yet. There is nothing to run until phase `cli`. See `.jdi/ROADMAP.md` (run `npx -y jdi-cli render` to regenerate it).

## Layout

- `crates/ddc-core` — the hexagon. Domain types (`VcpCode`, `VcpValue`, `MonitorId`, `Feature`, `Risk`, `Confirm`, `DdcError`), `Capabilities` with its own MCCS capabilities-string parser, the ports `MonitorBackend` (driven) and `MonitorControl` (driving), and `SoftwareOsd`, which implements `MonitorControl` on top of any `MonitorBackend`. Depends on `thiserror` only.
- `crates/ddc-adapters` — driven adapters:
  - `DdcHiMonitorBackend`, the real backend over [`ddc-hi`](https://docs.rs/ddc-hi) 0.4 (Windows `dxva2`, Linux `/dev/i2c-*`). It sits behind the crate feature `ddc-hi`, which is on by default; `--no-default-features` builds only the fake.
    - **One worker thread** owns every monitor handle and runs every DDC/CI transaction. The Windows handles cannot cross threads, so this is what makes the backend `Send + Sync` without `unsafe`.
    - **Budgets.** Callers wait at most a per-operation budget: 1 s for a VCP read or write, 8 s for a capabilities read, 5 s for an enumeration (`DdcHiBudgets`, overridable with `with_budgets`). Past the budget they get `Timeout`. A job whose caller already gave up never reaches the monitor.
    - **No `enumerate()` needed first.** A monitor id the backend has not seen yet (a fresh backend, a saved id, a monitor plugged in later) costs one enumeration under the enumeration budget; then the call runs under its own budget. An id still missing after that answers `MonitorNotFound`.
    - **Retries.** A failed transaction is retried up to 3 times, 50 ms apart, within the budget. If it still fails, the answer is `Transport`, which names the attempts. Only when one more enumeration (as long as the last one) still fits the rest of the budget is the monitor looked up again first, so an unplugged one answers `MonitorNotFound`. With the default budgets that happens for capabilities reads (8 s), not for VCP reads and writes (1 s, against a ~1.1 s enumeration): a display with mute DDC/CI answers `Transport` in about 100 ms and does not hold up calls to other monitors.
    - **`MonitorId` scheme.** With EDID (Linux), the id is `manufacturer-model-serial`, sanitized to ASCII letters, digits and single dashes. The dev monitor is `RTK-RTK-QHD-HDR-01010101`. Without EDID (Windows), the id is the sanitized device description, else `index-N`. A repeated id in one enumeration gets `#2`, `#3`…; that suffix follows enumeration order, so it may change across hotplugs.
    - **Enumeration** only reads EDID and never probes DDC/CI. A display with mute DDC/CI is listed, and fails on its first read.
  - `InMemoryMonitorBackend`, a scripted fake the core's use-case tests run against.

Still to come:

- `crates/ddc-cli` — `list`, `caps`, `get`, `set`, named shortcuts, `--json`, phase `cli`.
- `apps/ddc-tray` — Tauri 2 tray popup: monitor picker, sliders, input/preset/power, profiles, global hotkeys, phase `tray-app`.

Architecture is locked to Hexagonal (Ports & Adapters) — see `.jdi/PROJECT.md` and `.jdi/decisions/`.

## Monitor-write safety

Every write goes through `MonitorControl::set_feature`, and the core — never an adapter — enforces, in order:

1. **Risk.** Each VCP code has a `Risk`. Brightness, contrast, color preset, RGB gains, volume, sharpness and OSD language are `Safe`. Everything else — factory resets, input source, OSD lock, power mode, the manufacturer-specific range `0xE0`–`0xFF`, and any code not yet classified — is `Dangerous` and needs `Confirm::Yes`, checked before the monitor is touched at all.
2. **No blind writes.** A feature whose capabilities list discrete values accepts only those values. Any other feature is checked against a known maximum (from an earlier read, or read on demand when the capabilities declare the code); with no known maximum the write is refused.
3. **Read-back.** After the write the value is read back once, and that reading is what `set_feature` returns — some monitors acknowledge writes they silently drop.

Reads are never filtered by the capabilities string: monitors answer codes they do not declare (the dev monitor answers `0x62` volume), so `get_feature` always reads and reports `declared_in_capabilities` instead. A capabilities string that cannot be read or parsed does not block reads either: the monitor is treated as declaring no code, so a write needs the maximum from an earlier read. The failure is remembered per monitor, and only an explicit `capabilities` request asks the monitor again.

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
cargo check -p ddc-core -p ddc-adapters --locked --target x86_64-unknown-linux-gnu
cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc
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
