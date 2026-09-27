# Phase 5: Tray app — Summary  (slug: tray-app)

**Status:** partial
**Tasks:** 1/8 complete, 0 blocked

## Executed tasks
- T-1: scaffold do crate `ddc-tray` (lib `ddc_tray` + bin `ddc-tray`) no workspace, com a config Tauri, a capability mínima, o `icon.svg` próprio, o conjunto de ícones gerado e o `tray.png`. Commits `e91a453` (build) e `4c93104` (docs).

## Blocked tasks
- nenhuma

## T-1 — Scaffold do crate `ddc-tray`

### O que foi feito
- **Workspace:** `apps/ddc-tray/src-tauri` entrou em `members`. O `crates/` não foi tocado, e o `ddc-core` continua só com `thiserror` (D-2).
- **Deps (D-2026-09-26-tray-app-2):** `tauri 2.12.0` (`tray-icon`, `image-png`), `tauri-build 2.7.0`; `tauri-plugin-positioner 2.4.0` (`tray-icon`), `tauri-plugin-single-instance 2.5.0`; `serde` (derive), `serde_json` e `sys-locale 0.3.2` (A-5); `ddc-core`/`ddc-adapters` via workspace e `[lints] workspace = true`.
- **`Cargo.lock`:** 440 pacotes novos; nenhuma versão existente mudou (lock comparado antes e depois).
- **`main.rs`:** `windows_subsystem` só no release; erro vira `eprintln!` + `ExitCode::FAILURE`, sem `expect`.
- **`lib.rs`:** `#![forbid(unsafe_code)]` compila com `generate_context!`; `run() -> Result<(), tauri::Error>` mínimo. **`build.rs`:** `tauri_build::build()`.
- **`tauri.conf.json`:** sem `version`; `frontendDist: "../src"`, sem `devUrl`; `withGlobalTauri`; janela `popup` 360×560 oculta, sem decoração, não redimensionável, `skipTaskbar`, `alwaysOnTop`; `bundle.active: false` com `bundle.icon` completo (32, 128, 128@2x, icns, ico); CSP `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`.
- **Capability `default`:** janela `popup`, só `core:event:default`.
- **Ícones:** `icon.svg` próprio (monitor em gradiente ciano→azul→violeta com sol branco), conferido em 16/22/24/32 px sobre barras claras e escuras; conjunto desktop gerado por `npx -y @tauri-apps/cli@2.12.0 icon icons/icon.svg -o icons`; `tray.png` 64×64 RGBA 8 bits (`magick ... -depth 8 -strip PNG32:`); comandos de regeneração comentados no SVG.
- **`src/index.html`:** placeholder sem texto.

### Desvios do plano
- `apps/ddc-tray/src-tauri/.gitignore` (`/gen/schemas`, recriado a cada build pelo `tauri-build`) fora do `files_modified`.
- `icons/android/` e `icons/ios/` não commitados (app só desktop).
- Sem `transparent` na janela (risco de artefatos no webkit2gtk); arredondamento no Linux fica para a T-6.
- CSP definida pelo doer a partir da nota do plano.
- O harness bloqueou a escrita deste SUMMARY pelo subagente; o orquestrador gravou o conteúdo relatado pelo doer.

### Verificação
| Comando | Resultado |
|---|---|
| `cargo fmt --all --check` | OK |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | OK (só o aviso future-incompat de `nom v3.2.1`, pré-existente, via `ddc-hi`) |
| `cargo test --workspace --locked` | 247 passed, 0 failed |
| `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` | OK |
| Verify release build | OK (11,7 MB, ~49 s) |
| Verify forbid / CSP / capabilities / fmt+clippy | OK / OK / OK / OK |
| `cargo llvm-cov ... --fail-under-lines 80` | TOTAL lines 95.89% |
| `timeout -s TERM 5 target/release/ddc-tray` | exit 124 (vivo até o SIGTERM), 0 `panicked` |

## Tests
- Total: 247
- Passing: 247
- Coverage: 95.89% (`cargo llvm-cov --summary-only`, TOTAL)
