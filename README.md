# ddc-control

Control everything your monitor's physical OSD offers — brightness, contrast, input source, color preset, volume, power — from software, over DDC/CI, with the same Rust binary on Windows and Linux.

**Status:** pre-alpha. Phase 1 (`core-domain`) is implemented: a cargo workspace with two library crates and no binary yet — nothing talks to a real monitor until phase `ddc-backends`, and there is nothing to run until phase `cli`. See `.jdi/ROADMAP.md` (run `npx -y jdi-cli render` to regenerate it).

## Layout

- `crates/ddc-core` — the hexagon. Domain types (`VcpCode`, `VcpValue`, `MonitorId`, `Feature`, `Risk`, `Confirm`, `DdcError`), `Capabilities` with its own MCCS capabilities-string parser, the ports `MonitorBackend` (driven) and `MonitorControl` (driving), and `SoftwareOsd`, which implements `MonitorControl` on top of any `MonitorBackend`. Depends on `thiserror` only.
- `crates/ddc-adapters` — driven adapters. Today only `InMemoryMonitorBackend`, a scripted fake the core's use-case tests run against.

Still to come:

- `crates/ddc-adapters` — a `ddc-hi` adapter (Windows `dxva2`, Linux `/dev/i2c-*`), phase `ddc-backends`.
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
cargo install cargo-llvm-cov --locked
git config core.hooksPath .githooks             # once per clone — see below
```

Quality gates (the reviewer runs exactly these):

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'
```

Linux: load `i2c-dev` and make sure your user can open `/dev/i2c-*` (group `i2c` or a udev rule). Never run the tool with `sudo`.

## Commits

Atomic commits are enforced by hooks in `.githooks/` (activate with `git config core.hooksPath .githooks`):

- `commit-msg` — Conventional Commits header (`type(scope): subject`, ≤ 72 chars). `feat`/`fix`/`refactor`/`perf`/`test` require a scope equal to a roadmap phase slug. A commit may not mix code (`Cargo.*`, `src/`, `crates/`, `apps/`, `.github/`) with `.jdi/` state, nor touch two phases. Large task commits get a warning.
- `pre-commit` (from JDI) — rejects code changes when no active phase (`CONTEXT.md` + `PLAN.md`) is in the index, and rejects commits of generated `.jdi/` views.

Humans may bypass one commit with `JDI_ALLOW_MIXED=1` / `JDI_GATE_DISABLE=1`. Agents never do.

## Workflow

This repo is driven by [JDI](https://github.com/slipalison/jdi-cli) inside Claude Code: `/jdi-next` routes to the right step (`/jdi-discuss` → `/jdi-plan` → `/jdi-do` → `/jdi-verify` → `/jdi-ship`). Specialists live in `.jdi/agents/` with Claude Code copies in `.claude/agents/`.

## License

MIT — see `LICENSE`.
