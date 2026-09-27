# Phase 5: Tray app — Summary  (slug: tray-app)

**Status:** partial
**Tasks:** 3/8 complete, 0 blocked

## Executed tasks
- T-1: scaffold do crate `ddc-tray` (lib `ddc_tray` + bin `ddc-tray`) no workspace, com a config Tauri, a capability mínima, o `icon.svg` próprio, o conjunto de ícones gerado e o `tray.png`. Commits `e91a453` (build) e `4c93104` (docs).
- T-2: apresentação pura (`panel.rs`) genérica em `M: MonitorControl + ?Sized`, DTOs serde do contrato A-1 (`dto.rs`), 42 testes com o fake RTK e o golden `apps/ddc-tray/tests/fixtures/contract-rtk.json`. Commit `72d274e` (feat).
- T-3: os 7 comandos Tauri (`async fn` + `spawn_blocking`), o composition root (`compose_osd()`/`run()` com single-instance, blur/close escondem, `popup-shown`), o gate puro `popup.rs`, as permissões `allow-*` por comando e o teste de hardware `rtk_qhd_hdr`, rodado no RTK (brilho 100 → 90 → 100, conferido com `ddcutil`). Commit `e033137` (feat).

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

## T-3 — Comandos Tauri + composition root + ciclo do popup + teste de hardware RTK

### O que foi feito
- **`commands.rs`** (D-2026-09-26-tray-app-4/-7):
  - `AppState { osd: Result<SharedOsd, UiError>, selected: Mutex<Option<MonitorId>> }`, com `SharedOsd = Arc<dyn MonitorControl + Send + Sync>`. `AppState::new(Result<SharedOsd, DdcError>)` transforma a falha do backend em `backend_unavailable`, e o app segue de pé.
  - Helpers síncronos, genéricos e testados:
    - `write_feature(osd, &WriteRequest)`: `WriteRequest { monitor_id, code, value, confirmed }` → `Confirm`. É o único `Confirm::Yes` fora de testes. Devolve sempre o read-back.
    - `shortcut_target(osd, selected)`: o monitor selecionado; senão, o 1º da lista; sem nenhum, `not_found`.
    - `backend_unavailable(DdcError)`.
  - `on_blocking_thread(&AppState, call)`: tira o `Arc` do estado e roda `call` em `tauri::async_runtime::spawn_blocking`. Sem core, responde `backend_unavailable` sem rodar nada; se a tarefa para antes de responder, dá `transport`.
  - Os 7 comandos do A-1 (`list_monitors`, `select_monitor`, `load_panel`, `load_features`, `probe_features`, `set_feature`, `hide_popup`) são `#[tauri::command] pub async fn` de 1–3 linhas. Os que tocam DDC passam por `on_blocking_thread`; `select_monitor` só grava o estado e `hide_popup` só esconde a janela.
- **`lib.rs`** (composition root, D-2026-09-26-tray-app-3):
  - `pub fn compose_osd() -> Result<SharedOsd, DdcError>` tem o único `DdcHiMonitorBackend::new` e monta `SoftwareOsd<CachingMonitorBackend<_>>` com `default_cache_dir()`; sem diretório, fica `SoftwareOsd<DdcHiMonitorBackend>` sem cache, como o CLI.
  - `run()`:
    - `tauri_plugin_single_instance` é o 1º plugin, e o callback chama `show_popup`;
    - o `AppState` e o `PopupGate` entram no `setup`, depois do plugin, então a 2ª instância sai sem criar backend;
    - `on_window_event`: `Focused(false)` registra o instante no `PopupGate` e esconde; `CloseRequested` faz `prevent_close` e esconde;
    - `invoke_handler` com os 7 comandos.
  - `show_popup` faz `show` → `set_focus` → `emit(POPUP_SHOWN)`. Os erros de janela vão para o stderr (`report`) e não derrubam nada.
- **`popup.rs`** (puro; D-2026-09-26-tray-app-5): `PopupGate` guarda o instante do último hide por blur (`Mutex<Option<Instant>>`, com o tempo passado pelo chamador). `tray_clicked(Visibility, at)` dá:
  - `Hide` se o popup está aberto;
  - `Nothing` se ele se escondeu por blur há ≤ 300 ms (`BLUR_CLICK_WINDOW`);
  - `Show` nos outros casos.
  
  A T-5 liga isso ao clique da bandeja.
- **Permissões:** `build.rs` usa `tauri_build::try_build` com `AppManifest::commands(COMMANDS)` (sem `expect`/`panic`: erro → stderr + `ExitCode::FAILURE`). A capability `default` ganhou os 7 `allow-*` explícitos, além de `core:event:default`.
- **Hardware** (`tests/rtk_qhd_hdr.rs`, D-2026-09-26-tray-app-9): `rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored`, com `#[ignore]` e inerte sem `DDC_HW_TESTS=1`.
  - Passa por `compose_osd()` e pelos helpers de produção, sem nomear o adapter.
  - Confere o painel `0x10 0x12 0x62 0x60 0x14 0xD6`, com 7 entradas, 7 presets e 3 modos de energia.
  - Faz UMA escrita de brilho `confirmed: false` (original +10, ou −10 se não couber no max).
  - O guard `RestoreBrightness` (`Drop`) regrava o original, também com `confirmed: false`, e confere o read-back e uma leitura nova = original. Se o teste já está em pânico, ele só restaura e imprime.

### Testes novos (24, todos por igualdade)
- `commands` (17):
  - 0x60 e 0xD6 sem `confirmed` → `needs_confirmation`, sem `WriteVcp`;
  - 0x60 com `confirmed` → `WriteVcp(0x11)`, read-back `{17, 3}`, e a última chamada é o `ReadVcp`;
  - brilho sem confirmação → grava;
  - `ignoring_writes_to` → devolve 75 (o lido), não 40;
  - 0x05 fora da lista e 101 acima do max → `invalid_value`, sem escrita;
  - monitor ausente → `not_found`;
  - `backend_unavailable` (mapeamento, estado e `on_blocking_thread` sem rodar);
  - `on_blocking_thread` com o fake (lista + escrita);
  - tarefa que para → `transport`;
  - seleção;
  - alvo do atalho: o selecionado (sem nenhuma chamada ao backend), o 1º da lista, ou `not_found`.
- `popup` (7): janela de 300 ms; clique sem blur → `Show`; aberto → `Hide`; 0/120/300 ms → `Nothing`; 301 ms → `Show`; clique com carimbo anterior ao blur → `Nothing`; só o último blur conta.

### Mutação (provada, sem commit)
`WriteRequest::confirmation()` fixo em `Confirm::Yes` derruba 2 testes, e o arquivo foi restaurado em seguida (17/17 verdes):
```
test commands::tests::an_unconfirmed_input_switch_needs_confirmation_and_writes_nothing ... FAILED
test commands::tests::an_unconfirmed_power_change_needs_confirmation_and_writes_nothing ... FAILED
  left: Ok(ReadBackDto { current: 17, max: 3 })
 right: Err(NeedsConfirmation)
  left: Ok(ReadBackDto { current: 4, max: 5 })
 right: Err(NeedsConfirmation)
test result: FAILED. 15 passed; 2 failed; 0 ignored; 0 measured; 49 filtered out
```

### Saída do teste de hardware
Brilho antes: `ddcutil --bus 5 getvcp 10` → `current value = 100, max value = 100`.

`DDC_HW_TESTS=1 cargo test -p ddc-tray --locked -- --ignored rtk_qhd_hdr --test-threads=1 --nocapture`. A saída abaixo é verbatim; só o dump `{:#?}` do painel (≈ 180 linhas) foi resumido entre colchetes.
```
running 1 test
test rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored ... list_monitors: 1.172890463s
monitors: [ MonitorDto { id: "GSM-LG-TV-SSCR2-01010101", label: "LG TV SSCR2", .. },
            MonitorDto { id: "RTK-RTK-QHD-HDR-01010101", label: "RTK QHD HDR", manufacturer: Some("RTK"), model: Some("RTK QHD HDR") } ]
load_panel: 513.226003ms
panel: PanelDto { monitor_id: "RTK-RTK-QHD-HDR-01010101", controls: [
  [0x10 brightness Continuous 100/100; 0x12 contrast Continuous 50/100; 0x62 volume Continuous 30/100;
   0x60 input dangerous NonContinuous current 0x0F, options VGA-1 DVI-1 DVI-2 DisplayPort-1 DisplayPort-2 HDMI-1 HDMI-2;
   0x14 preset NonContinuous current 0x01, options sRGB, Display Native, 5000 K, 6500 K, 7500 K, 9300 K, User 1;
   0xD6 power dangerous NonContinuous current 0x01, options On, Off (DPM), Off (write-only)] ] }
write 0x10: 145.721976ms
brightness 100 -> 90 (max 100): read back Ok(ReadBackDto { current: 90, max: 100 })
restore 0x10: 145.804355ms
restored brightness: Ok(ReadBackDto { current: 100, max: 100 }); fresh read: Ok(FeatureReading { feature: Feature { code: VcpCode(16), kind: Continuous, access: ReadWrite, risk: Safe, allowed_values: None }, value: VcpValue { current: 100, max: 100 }, declared_in_capabilities: true })
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.07s
```
Brilho depois: `ddcutil --bus 5 getvcp 10` → `current value = 100, max value = 100` (voltou ao original). Nada `dangerous` foi gravado: entrada e energia só foram lidas.

### Smoke do ciclo do popup (manual, KDE Plasma Wayland, NVIDIA RTX 4090)
- **Achado:** na 1ª tentativa, a 2ª execução fez a 1ª mostrar o popup, e a 1ª morreu (exit 1) com `Gdk-Message: Error 71 (Erro de protocolo) dispatching to Wayland display.`
  - Reproduziu 3/3 vezes; com `WEBKIT_DISABLE_DMABUF_RENDERER=1`, o problema sumiu.
  - É o renderer DMA-BUF do WebKitGTK 2.54 com o driver NVIDIA 615, e não a lógica do app. Até a T-2 nenhuma janela era mostrada, por isso não tinha aparecido.
- **Correção em `run()`** (só Linux): se a variável não está definida, o processo faz `exec` de si mesmo com `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
  - Usa `CommandExt::exec`, que é safe, porque `std::env::set_var` é `unsafe` na edição 2024 e o crate tem `forbid(unsafe_code)`.
  - O PID e os argumentos são os mesmos, então o PID que o smoke SNI da T-5 confere não muda.
  - Um valor escolhido pelo usuário é respeitado.
- **Depois da correção** (debug e release): 1ª instância com `env -u WEBKIT_DISABLE_DMABUF_RENDERER`; `/proc/<pid>/environ` mostra a variável no mesmo PID. A 2ª instância sai com 0 em ~80 ms, a 1ª continua viva depois do `show` até o SIGTERM (exit 143), e o stderr tem 0 `panicked`.
- Que o blur esconde o popup e que `popup-shown` chega à UI ainda não foi observado: o `index.html` ainda é o placeholder, sem JS. Fica para a validação manual no KDE da T-5/T-6 (R-1).

### Desvios do plano
- **`apps/ddc-tray/src-tauri/.gitignore`** (fora do `files_modified`) ganhou `/permissions/autogenerated`. O `tauri-build` regenera ali os 7 `.toml` de permissão a cada build, como o `/gen/schemas` que a T-1 já ignora. Um build do zero, sem o diretório, recria os arquivos antes de lê-los (conferido).
- O **re-exec sem DMA-BUF** em `run()` não estava no plano. Ele entrou no mesmo commit da T-3 porque sem ele "a 2ª execução mostra o popup" derrubava o app nesta máquina.
- O estado entra no `.setup()`, e não antes do `Builder`: assim a 2ª instância, que o plugin encerra no setup dele, não chega a criar o worker do backend.
- `hide_popup` devolve `()`: uma falha ao esconder não tem kind na D-4 e vai para o stderr. `select_monitor` devolve `Result<(), UiError>`, que o Tauri exige num `async fn` com `State<'_>`, e é sempre `Ok`.
- O `writes()` de `panel/tests.rs` é privado, então `commands/tests.rs` tem o seu (6 linhas), para não mexer em `panel/tests.rs`.
- Cobertura de `commands.rs` em 52.14%: faltam só as linhas 166–232, que são os 7 wrappers `#[tauri::command]` (exercitá-los exigiria a feature `test` do tauri no `Cargo.toml`, fora do `files_modified`). `lib.rs` está em 0% (cola do Tauri, R-3). Os helpers e o `popup.rs` estão em 100%.

### Verificação
| Comando | Resultado |
|---|---|
| Verify `DdcHiMonitorBackend::new` único (`grep … | wc -l` = 1, em `lib.rs`) | OK |
| Verify single-instance (`grep -q tauri_plugin_single_instance lib.rs`) | OK |
| Verify hardware (`--ignored --list` com `rtk_qhd_hdr` + `DDC_HW_TESTS` em `tests/`) | OK |
| Verify `panel.rs` puro / `forbid` / capabilities | OK / OK / OK |
| Gate 5.7 `Confirm::Yes` fora de `commands*`/testes | nenhum |
| Gate 5.9 `#[tauri::command]` sem `async fn` | nenhum |
| Gate 5.4 adapter construído fora de `main.rs`/`lib.rs` | nenhum |
| Gate 5.6 `unwrap`/`expect`/`panic!` fora de testes em `apps/` | nenhum |
| `cargo fmt --all --check` | OK |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | OK (só o aviso future-incompat de `nom v3.2.1`, pré-existente) |
| `cargo test -p ddc-tray --locked` | 66 passed, 0 failed, 1 ignored (hardware) |
| `cargo test --workspace --locked` | 313 passed, 0 failed, 8 ignored (hardware) |
| `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` | OK |
| `cargo build -p ddc-tray --release --locked` | OK |
| `cargo llvm-cov ... --fail-under-lines 80` | TOTAL lines 90.77%; `popup.rs` 100%, `dto.rs` 100%, `panel.rs` 99.04%, `commands.rs` 52.14%, `lib.rs` 0% |

## Tests
- Total: 313
- Passing: 313
- Coverage: 90.77% (`cargo llvm-cov --summary-only`, TOTAL lines)
