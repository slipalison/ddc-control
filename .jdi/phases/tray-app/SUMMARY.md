# Phase 5: Tray app — Summary  (slug: tray-app)

**Status:** partial
**Tasks:** 2/8 complete, 0 blocked

## Executed tasks
- T-1: scaffold do crate `ddc-tray` (lib `ddc_tray` + bin `ddc-tray`) no workspace, com a config Tauri, a capability mínima, o `icon.svg` próprio, o conjunto de ícones gerado e o `tray.png`. Commits `e91a453` (build) e `4c93104` (docs).
- T-2: apresentação pura (`panel.rs`) genérica em `M: MonitorControl + ?Sized`, DTOs serde do contrato A-1 (`dto.rs`), 42 testes com o fake RTK e o golden `apps/ddc-tray/tests/fixtures/contract-rtk.json`. Commit `72d274e` (feat).

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

## T-2 — Apresentação pura (`panel.rs`) + DTOs (`dto.rs`) + golden do contrato

### O que foi feito
- **`panel.rs`** (D-2026-09-26-tray-app-3): funções genéricas em `M: MonitorControl + ?Sized` (um teste passa `&(dyn MonitorControl + Send + Sync)`, o formato que a T-3 vai guardar no `AppState`). Não cita o runtime do app, nem em comentário.
  - `QUICK_CONTROLS` = `0x10 0x12 0x62 0x60 0x14 0xD6`, nessa ordem.
  - `monitors(osd)`: devolve `[MonitorDto]`.
  - `load_panel(osd, id)`: um `get_feature` por controle. Uma leitura que falha tira só aquele controle, e `MonitorNotFound` vira `not_found`.
  - `load_features(osd, id)`: códigos declarados no caps, em ordem de código, filtrados por `is_adjustable`:
    - fora dos 6 rápidos;
    - `is_readable()` e `ensure_writable()` do core, ou seja, RW e não `Table`;
    - NC só com lista (caps, senão catálogo).
    
    No RTK dá exatamente `0x0C 0x16 0x18 0x1A 0x87 0xCA 0xCC`. Caps ilegíveis devolvem o erro do core (kind `transport`/`timeout`), e um código sem valor continua na lista com `status`.
  - `probe_features(osd, id)`: `probe_undeclared_features` com o mesmo filtro. A feature de um código sondado vem de `Capabilities::default().feature(code)`, que equivale ao caps real, porque o código não é declarado.
  - `ui_error(DdcError) → UiError`: os dois `InvalidValue`/`ValueNotAllowed` → `invalid_value`.
  - `brightness_for_percent(max, pct)`: `(max·min(pct,100) + 50) / 100`.
  - `set_brightness_percent(osd, id, pct)`: lê o max, grava com `Confirm::No` e devolve o read-back.
- **`dto.rs`** (D-2026-09-26-tray-app-4, A-1): tudo `Serialize` em camelCase, montado a partir de tipos do core.
  - `MonitorDto {id,label,manufacturer,model}`: `label` = modelo, senão fabricante, senão id.
  - `PanelDto {monitorId,controls}` e `ControlDto {code,key,dangerous,value}`.
  - `ControlValueDto` com `#[serde(tag = "kind")]`:
    - `continuous {current,max}`;
    - `nonContinuous {current: SL, options}`.
  - `OptionDto {value,name}` (lista do caps, senão do catálogo; `name` do catálogo ou `null`).
  - `FeatureDto {code,alias,name,dangerous,origin,status,value|null}`.
  - `ReadBackDto {current,max}`, `PanelChangedDto {monitorId}` e as constantes `POPUP_SHOWN`/`PANEL_CHANGED`.
  - `UiError {kind,message}` com `ErrorKind` em snake_case, incluindo `backend_unavailable` para a T-3.
- **Fake RTK (A-2)**, em `panel/tests.rs` como `pub(crate)` (`RTK_ID`, `RTK_CAPS`, `rtk_monitor()`, `osd_with()`), para a T-3 reaproveitar:
  - identidade e caps reais: id `RTK-RTK-QHD-HDR-01010101`, `RTK` / `RTK QHD HDR` / `01010101`, e a fixture do core;
  - controles rápidos: brilho 75/100, contraste 50/100, volume 30/100 (fora do caps), entrada 0x0F/3, preset 0x01/11 e energia 0x01/5 (max NC iguais aos do hardware);
  - **valores fixados aqui, que a demo da T-4 espelha:** `0x0C` 70/100, `0x16` 50/100, `0x18` 48/100, `0x1A` 46/100, `0x87` 5/10, `0xCA` 0x02/2 ("OSD enabled"), `0xCC` 0x02/13 ("English").
- **Golden** `apps/ddc-tray/tests/fixtures/contract-rtk.json` (277 linhas): `{monitors, panel, features}` serializados do fake RTK.
  - O teste compara como `serde_json::Value`. Se divergir, falha com a primeira linha diferente (golden vs atual) e mostra como regenerar.
  - Só se regenera de propósito, com `DDC_TRAY_UPDATE_GOLDEN=1 cargo test -p ddc-tray --locked golden`.
  - O golden foi gerado assim uma vez e conferido campo a campo contra A-1/A-2.
- **Testes (42, todos por igualdade):**
  - `panel` (23):
    - o painel RTK inteiro, e o volume presente com o caps sem 0x62;
    - 0x60 e 0x14 com as 7 opções e 0xD6 com 3;
    - `Timeout` no contraste → 5 controles;
    - `not_found` em `load_panel`/`load_features`/`probe_features`;
    - opções vindas do caps (com um byte sem nome → `null`) e do catálogo (sem caps);
    - a lista A-4 exata, e os códigos sem valor com `unsupported`/`unresponsive`;
    - caps ilegíveis → `transport`;
    - a sonda do RTK com 9 entradas exatas: 0x62 e 0xC9 RO ficam de fora, e nada é gravado;
    - os 7 `DdcError` → kind + mensagem;
    - percentuais 0/25/50/75/100 com max 100 e com max 80, arredondamento e limite;
    - atalho de brilho: grava 25 (e 40 com max 80), devolve o que o monitor manteve com `ignoring_writes_to`, e não grava nada num monitor ausente;
    - o golden.
  - `dto` (19): o JSON exato de cada forma, os 7 kinds e os nomes de evento.

### Desvios do plano
- `ControlDto.key` e `FeatureDto.alias`/`name` são `Option` (`null` para um código fora do catálogo). Os 6 controles rápidos sempre têm alias, então o golden não traz nenhum `null`.
- O fixture RTK é `pub(crate)` em `panel/tests.rs` (`#[cfg(test)] pub(crate) mod tests`) para a T-3 reaproveitar. Não criei um arquivo de suporte fora do `files_modified`.
- `ErrorKind::BackendUnavailable` e as constantes/`PanelChangedDto` de eventos ficaram prontos no `dto.rs`, mas ainda não são usados em código de produção (T-3/T-5).
- Nenhum outro arquivo fora do `files_modified` foi tocado.

### Verificação
| Comando | Resultado |
|---|---|
| Verify da CONTEXT de `panel.rs` (`test -f … && ! grep -nE '\btauri(::|_)' …`) | OK (e `grep -i tauri` em `panel.rs` = 0 linhas) |
| `cargo fmt --all --check` | OK |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | OK (só o aviso future-incompat de `nom v3.2.1`, pré-existente) |
| `cargo test -p ddc-tray --locked` | 42 passed, 0 failed |
| `cargo test --workspace --locked` | 289 passed, 0 failed, 7 ignored (hardware) |
| `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` | OK |
| `cargo llvm-cov ... --fail-under-lines 80` | TOTAL lines 96.21%; `panel.rs` 99.04%, `dto.rs` 100% |

## Tests
- Total: 289
- Passing: 289
- Coverage: 96.21% (`cargo llvm-cov --summary-only`, TOTAL lines)
