# ddc-control

Control everything your monitor's physical OSD offers — brightness, contrast, input source, color preset, volume, power — from software, over DDC/CI, with the same Rust binary on Windows and Linux.

**Status:** pre-alpha. Project skeleton and workflow only; phase 1 (`core-domain`) not started. See `.jdi/ROADMAP.md` (run `npx -y jdi-cli render` to regenerate it).

## Planned shape

- `crates/ddc-core` — the hexagon: domain types, ports (`MonitorBackend`, `MonitorControl`), use cases. Depends on `thiserror` only.
- `crates/ddc-adapters` — driven adapters: `ddc-hi` (Windows `dxva2`, Linux `/dev/i2c-*`) and an in-memory fake.
- `crates/ddc-cli` — `list`, `caps`, `get`, `set`, named shortcuts, `--json`.
- `apps/ddc-tray` — Tauri 2 tray popup: monitor picker, sliders, input/preset/power, profiles, global hotkeys.

Architecture is locked to Hexagonal (Ports & Adapters) — see `.jdi/PROJECT.md` and `.jdi/decisions/`.

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
