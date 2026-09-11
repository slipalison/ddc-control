<!-- jdi:registry -->
## R-2026-09-11-ddc-control (2026-09-11)
**Type:** specialist (doer + reviewer)
**Slug:** ddc-control
**Stack:** Rust stable 1.98 workspace (ddc-core / ddc-adapters / ddc-cli / apps/ddc-tray) — Hexagonal (D-1), core deps = thiserror only (D-2)
**Files:** .jdi/agents/jdi-doer-ddc-control.md, .jdi/agents/jdi-reviewer-ddc-control.md
**Runtime copies:** .claude/agents/jdi-doer-ddc-control.md, .claude/agents/jdi-reviewer-ddc-control.md (frontmatter achatado para o Claude Code — `jdi-cli` não copia `.jdi/agents/` para `.claude/agents/`; regenerar a cópia sempre que o canônico mudar)
**Gates validated at bootstrap:** cargo build/test/fmt/clippy/llvm-cov + `cargo check --target x86_64-unknown-linux-gnu` no scaffold raiz; toolchain (clippy, rustfmt, llvm-tools-preview, cargo-llvm-cov 0.9.1, target linux-gnu) instalado na máquina de dev
**Frontend gate:** has_frontend=false (tray popup não é web app roteável; phase `tray-app` pode reverter via D-XX)
<!-- /jdi:registry -->
<!-- jdi:specialists -->
| Rust (core + adapters + CLI + Tauri) | jdi-doer-ddc-control | **/* | executor for files matching glob |
<!-- /jdi:specialists -->
<!-- jdi:reviewers -->
| jdi-reviewer-ddc-control | **/* | /jdi-verify | yes, if BLOCKED |
<!-- /jdi:reviewers -->
