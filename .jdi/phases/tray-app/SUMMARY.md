# Phase 5: Tray app — Summary  (slug: tray-app)

**Status:** partial
**Tasks:** 5/8 complete, 0 blocked

## Executed tasks
- T-1: scaffold do crate `ddc-tray` (lib `ddc_tray` + bin `ddc-tray`) no workspace, com a config Tauri, a capability mínima, o `icon.svg` próprio, o conjunto de ícones gerado e o `tray.png`. Commits `e91a453` (build) e `4c93104` (docs).
- T-2: apresentação pura (`panel.rs`) genérica em `M: MonitorControl + ?Sized`, DTOs serde do contrato A-1 (`dto.rs`), 42 testes com o fake RTK e o golden `apps/ddc-tray/tests/fixtures/contract-rtk.json`. Commit `72d274e` (feat).
- T-3: os 7 comandos Tauri (`async fn` + `spawn_blocking`), o composition root (`compose_osd()`/`run()` com single-instance, blur/close escondem, `popup-shown`), o gate puro `popup.rs`, as permissões `allow-*` por comando e o teste de hardware `rtk_qhd_hdr`, rodado no RTK (brilho 100 → 90 → 100, conferido com `ddcutil`). Commit `e033137` (feat).
- T-4: módulos ES puros e sem dependências (`bridge.js` Tauri/demo, `demo-data.js`, `debounce.js`, `view-model.js`, `i18n/` en + pt-BR) e 63 testes `node --test` fora de `src/`, incluindo o RTK da demo `deepStrictEqual` ao golden. Commit `35c5ffd` (feat).
- T-5: bandeja com `tray.png`, tooltip e menu nativo bilíngue (`i18n.rs` + `menu.rs` puros), clique esquerdo no Windows alternando o popup ancorado pelo positioner com o gate de `popup.rs`, atalhos de brilho Safe em thread bloqueante com `panel-changed`, "Sair" → `app.exit(0)`, e `scripts/smoke-sni.sh`, que passa no KDE Wayland e falha nos 7 casos provocados. 26 testes novos. Commit `380a691` (feat).

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

## T-4 — Módulos JS puros (bridge demo, i18n, debounce, view-model) + `node --test`

### O que foi feito
Todos são ES modules sem dependências, com `window`/`navigator`/timers injetados. Nenhum teste fica em `src/`: os 5 arquivos de teste estão em `apps/ddc-tray/tests/ui/` e resolvem caminhos por `import.meta.url`.

- **`bridge.js`** (D-2026-09-26-tray-app-4, -8): `createBridge(window, { latencyMs, timers })`.
  - API única nos dois modos: `mode`, `listMonitors`, `selectMonitor`, `loadPanel`, `loadFeatures`, `probeFeatures`, `setFeature(id, code, value, { confirmed })`, `hidePopup`, `onPopupShown` e `onPanelChanged`.
  - **Tauri** (quando existe `window.__TAURI__.core.invoke`):
    - chama os 7 comandos do A-1 com args em camelCase (`monitorId`, `confirmed` sempre booleano);
    - normaliza rejeições para `{kind,message}`, e o que estiver fora do contrato (ex.: permissão negada) vira `unknown`;
    - entrega ao handler só o `payload` dos eventos.
  - **Demo** (`?demo=rtk|two-monitors|empty|error`; sem o parâmetro ou com valor desconhecido, `rtk`). Reproduz o que o core faz com a UI, com as mesmas mensagens do `DdcError`:
    - `dangerous` sem `confirmed` → `needs_confirmation`, sem registrar escrita;
    - valor fora das `options` ou acima do max → `invalid_value`;
    - monitor ausente → `not_found`; código ausente ou sem suporte → `unsupported`; `unresponsive` → `timeout`.
  - A resposta da demo é o read-back `{current,max}`. No NC, o max é o bruto: entrada 3, preset 11, energia 5.
  - Escritas aceitas vão para `window.__ddcDemo.writes` como `{monitorId,code,value,confirmed}`. O `__ddcDemo` também expõe `scenario`, `selected` e `emit(event, payload)`, para a T-6/T-7 simularem `popup-shown`/`panel-changed`.
  - No cenário `error`, todo comando rejeita com `backend_unavailable`, como o `AppState` sem backend da T-3.
  - Latência de 60 ms por comando (×8 na sondagem), pelos timers injetados. As respostas são cópias.
- **`demo-data.js`:** cenários como dados.
  - `rtk` é o fake A-2 da T-2: os 6 controles e os 7 de "Todos os ajustes", com os valores registrados na T-2. A sondagem espelha o teste de probe de `panel/tests.rs` (9 entradas).
  - `two-monitors` = RTK + "DELL U2723QE", sem volume e com a entrada 0x1B sem nome no catálogo.
  - Os nomes são os do catálogo do core, literais. É fixture do contrato, não tabela de consulta: nenhum código da UI mapeia byte → nome.
- **`debounce.js`** (D-2026-09-26-tray-app-5): `createWriteQueue({ write, onResult, onError, delayMs = 80, timers })` → `push(key, value)`, `flush(key)`, `busy(key)`.
  - Janela de 80 ms, reiniciada a cada valor, com coalescência no último.
  - No máximo 1 escrita em voo por chave. Um valor que chega durante o voo sai logo que ela termina, se a janela dele já expirou; senão, sai no próprio timer.
  - `flush` envia na hora, ou logo depois da escrita em voo.
  - Um erro (rejeição ou `throw` síncrono) vai para `onError` e a fila segue: a próxima escrita sai antes dos callbacks.
- **`view-model.js`:** DTO → modelo de render.
  - `panelView` monta:
    - sliders com `percent`/`valueText` (`75%` com max 100; `40 de 80` nos demais);
    - `input` segmentado e `selects` (preset);
    - `power` com `choices` = as opções diferentes da atual (A-3).
  - `featuresView`/`featureView`: rótulo pela chave `feature.<alias>`, senão o nome MCCS do core, senão `Ajuste 0xNN`. Trazem `statusText`/`originText`, e `widget: null` quando não houve leitura.
  - `withReadBack`: o NC usa o byte baixo.
  - `statusView`: estados `loading|ready|empty|error`, com `retry` em empty/error. A dica de `/dev/i2c` → `docs/linux-ddc-setup.md` aparece só em Linux, e só em empty ou nos erros `backend_unavailable|transport|timeout`.
  - Também exporta `errorText`, `confirmView` (texto do efeito: entrada, energia ou genérico), `detectPlatform`, `pickMonitor`, `monitorPicker` e `hex`.
- **`i18n/`** (D-2026-09-26-tray-app-6): `en.js` e `pt-BR.js`, com 64 chaves cada: `state` 2, `action` 1, `hint` 1, `format` 4, `origin` 2, `reading` 2, `confirm` 6, `error` 8, `feature` 21, `value` 17.
  - `value.*` só existe para nomes que mudam com o idioma (Nativo, Usuário 1, Ligado, idiomas do OSD etc.). HDMI-1, 6500 K e sRGB caem no nome cru.
  - `index.js` exporta:
    - `resolveLocale`, que aceita string ou lista e usa a primeira tag `pt*`/`en*` (senão en);
    - `detectLocale(navigator)`, `translator(locale)` e `t(key, params, locale)`, com fallback para en e depois para a chave;
    - `translateOr`, `valueKey`, `featureKey` e `errorKey`.

### Testes (63, todos por igualdade)
- **`debounce.test.mjs` (13, com `mock.timers`):**
  - 5 inputs em menos de 80 ms → exatamente `[[0x10, 50]]`;
  - envio exatamente aos 80 ms (79 → nada);
  - a 2ª escrita espera a 1ª e leva só o último valor;
  - valor ainda na janela espera o próprio timer;
  - `flush` imediato e em voo;
  - `flush` vazio;
  - lanes independentes por código;
  - `onResult` com o read-back;
  - rejeição e `throw` síncrono não travam a lane;
  - `busy`.
- **`view-model.test.mjs` (17):**
  - sliders do golden e o caso max 80 → `40 de 80`/50%;
  - entrada com a atual selecionada; preset traduzido (`Nativo`, `Usuário 1`) com fallback ao nome cru; opção sem nome → `Valor 0x1B`;
  - energia com `choices` = 0x04/0x05;
  - slots vazios;
  - "Todos os ajustes" em pt-BR; feature sem chave → nome do core; `withReadBack`;
  - os 4 estados, a dica só em Linux e só nos kinds certos;
  - `errorText`, `confirmView`, `detectPlatform` (inclui "Darwin" ≠ Windows) e o seletor de monitor.
- **`bridge-demo.test.mjs` (17):**
  - os 4 cenários e o fallback do `?demo=`;
  - `needs_confirmation` sem escrita; escrita confirmada → `writes` = `[{monitorId, code: 0x60, value: 0x11, confirmed: true}]` e read-back `{17, 3}`;
  - `invalid_value` (101 e 0x02 em 0xD6), `not_found` e `unsupported`;
  - sondagem com 9 entradas; respostas são cópias;
  - eventos da demo com `unlisten`; a latência respeitada com `mock.timers`;
  - no modo Tauri: comandos + args exatos, sem `__ddcDemo`, normalização de erros e `payload` dos eventos.
- **`contract.test.mjs` (3):** `{monitors, panel, features}` do RTK da demo `deepStrictEqual` o `contract-rtk.json`; o cenário padrão também; no `two-monitors`, o RTK continua igual ao golden e o 2º monitor tem as mesmas chaves do contrato em cada nível.
- **`i18n-parity.test.mjs` (13):**
  - mesmas chaves nos 2 locales, nenhum valor vazio, mesmos placeholders por chave;
  - os 8 `error.*`;
  - as chaves `feature.*`/`value.*` são checadas contra o fonte do catálogo do core (`crates/ddc-core/src/domain/mccs_catalog.rs`, lido pelo teste, sem cópia em JS): todo `feature.*` é um alias do core, todo alias RW tem rótulo, e todo `value.*` é o slug de um nome do core;
  - `resolveLocale`, `detectLocale`, fallbacks de `t()`, placeholders e `translateOr`.

### Mutações (provadas e revertidas com `git checkout`, nada commitado)
- **Debounce sem coalescência.** O corpo de `push` virou `timers.setTimeout(() => void write(key, value), delayMs)`, ou seja, cada valor agenda a própria escrita. Resultado: `✖ five inputs within 80 ms become one write of the last value`, com `actual: [ [ 16, 10 ], [ 16, 20 ], [ 16, 30 ], [ 16, 40 ], [ 16, 50 ] ]` vs `expected: [ [ 16, 50 ] ]`.
- **Demo divergindo do golden** (brilho RTK 75 → 76 em `demo-data.js`): os 3 testes de `contract.test.mjs` falham (`ℹ pass 0`, `ℹ fail 3`).
- Depois de reverter: `# pass 63`, `# fail 0`.

### Desvios do plano
- **Verify da CONTEXT incompatível com o Node desta máquina (v24.18.0).** No Node ≥ 23, o reporter padrão do `node --test` é `spec` mesmo fora de TTY, então a saída traz `ℹ pass 63`, e o `grep -qE '^# pass …'` dos 2 Verify (testes JS e i18n) nunca casa. Rodados como estão, os dois imprimem FAIL; com `--test-reporter=tap`, os dois imprimem OK.
  - Proposta ao orquestrador: emendar os 2 Verify para `node --test --test-reporter=tap '…'`.
  - Não há arquivo de configuração que resolva isso sem flag experimental.
- **Chaves além das listadas no plano,** usadas pelo view-model: `state.*`, `action.retry`, `hint.i2c`, `format.*`, `origin.*`, `reading.*` e `confirm.*`. Também `feature.new-control-value`, para o teste "todo alias RW tem rótulo" ser exato. As chaves do HTML ficam para a T-6.
- **Extras da demo:** `__ddcDemo.selected` e `__ddcDemo.emit`, além de `writes`/`scenario`, para T-6/T-7 simularem eventos.
- **Nota para a T-6:** a fila de `debounce.js` é por chave genérica. Para não misturar monitores ao trocar a seleção, use a chave `` `${monitorId}:${code}` `` e leve o `monitorId` no valor. Nos testes, a chave é o código.
- Nenhum `cargo` foi rodado (pedido do orquestrador: a T-3 compilava em paralelo) e nenhum arquivo Rust foi tocado.
- Nenhum arquivo fora do `files_modified` foi tocado.

### Verificação
| Comando | Resultado |
|---|---|
| `node --test 'apps/ddc-tray/tests/ui/**/*.test.mjs'` (Verify como está) | FAIL: só o formato (`ℹ pass 63`, `ℹ fail 0`, reporter `spec`) |
| mesmo Verify com `--test-reporter=tap` | OK: `# tests 63`, `# pass 63`, `# fail 0`; `find apps/ddc-tray/src -name '*.test.*'` = 0 |
| `node --test 'apps/ddc-tray/tests/ui/i18n*.test.mjs'` (Verify como está) | FAIL: só o formato (`ℹ pass 13`) |
| mesmo Verify com `--test-reporter=tap` | OK: `# tests 13`, `# pass 13`, `# fail 0` |

## T-5 — Bandeja: ícone, menu bilíngue, eventos, positioner + smoke SNI

### O que foi feito
- **`i18n.rs`** (puro; D-2026-09-26-tray-app-6):
  - `Locale { En, PtBr }`; `Locale::from_tag` usa a mesma regra do `resolveLocale` da UI: tag que começa com `pt` (sem diferenciar maiúsculas) → pt-BR, qualquer outra → en.
  - `Labels { tooltip, open_panel, brightness, quit }`, um `const` por locale. A paridade vem por construção: um locale sem um rótulo não compila.
  - `brightness_label(pct)` → `Brightness 25%` / `Brilho 25%`.
- **`menu.rs`** (puro; D-2026-09-26-tray-app-5):
  - `Platform { Windows, Linux }` e `Platform::current()` (`cfg!(windows)`).
  - `MenuAction { OpenPanel, Brightness(u8), Quit }`, com ids estáveis e iguais nos 2 idiomas (`open-panel`, `brightness-N`, `quit`).
  - `from_id` é derivado de `id()`: só aceita ids que o menu mostra de fato (`brightness-025`, `brightness-+25`, `brightness-10` → `None`).
  - `menu_entries(locale, platform)`: no Windows, Abrir painel │ Sair; no Linux, Abrir painel │ Brilho 0/25/50/75/100% │ Sair (│ = separador). `BRIGHTNESS_STEPS = [0, 25, 50, 75, 100]`.
- **`tray.rs`** (glue do Tauri, com um helper genérico testado):
  - `install(app)`: `sys_locale::get_locale()` só aqui; `TrayIconBuilder::with_id("ddc-control")` com `tauri::include_image!("icons/tray.png")` (decodificado na compilação, sem caminho de erro em runtime), tooltip do locale, o menu de `menu_entries`, `show_menu_on_left_click(false)`, `on_menu_event` e `on_tray_icon_event`.
  - Eventos da bandeja: todo evento passa por `tauri_plugin_positioner::on_tray_event`. `Click { Left, Up }` (`is_left_click`) chama `toggle_popup`, que consulta o `PopupGate` com `Visibility` do `is_visible()`: `Show` → `open_panel`, `Hide` → `hide()`, `Nothing` → nada.
  - `open_panel`: `move_window_constrained(Position::TrayCenter)` e depois `show_popup`. O erro de posição só vai para o log no Windows; no SNI/Wayland, onde o positioner nunca recebe o retângulo do ícone, ele é esperado e ignorado.
  - Atalho de brilho: `tauri::async_runtime::spawn` → `on_blocking_thread` da T-3 (`spawn_blocking`; sem backend, dá `backend_unavailable`) → `brightness_shortcut(osd, selected, pct)` = `shortcut_target` + `panel::set_brightness_percent` (Safe, `Confirm::No`) → `emit(PANEL_CHANGED, PanelChangedDto { monitorId })`. Um erro só vai para o stderr (`ddc-tray: could not …: Kind: mensagem`).
  - "Sair" → `app.exit(0)`.
- **`lib.rs`** (composition root): `pub mod i18n/menu/tray`; `tauri_plugin_positioner::init()` como 2º plugin, depois do single-instance (obrigatório: sem ele, o `on_tray_event` do positioner entra em pânico no `state::<Tray>()`); `tray::install(app.handle())?` no `setup`, depois do `AppState` e do `PopupGate`.
- **`apps/ddc-tray/scripts/smoke-sni.sh <bin>`** (bash, `set -euo pipefail`, 100755):
  - sem watcher → falha com mensagem clara; lista os itens antes, sobe o app com o stderr num temporário e sonda a cada 250 ms, com prazo de 15 s em ms (`date +%s%N`; o `SECONDS` do bash tem granularidade de segundo e dava 14,6 s);
  - aceita item `:1.N/caminho` e nome bem conhecido (`serviço[/caminho]`), e confere o PID via `GetConnectionUnixProcessID`;
  - exige o processo vivo 2 s depois do registro; falha com status e mensagem explícitos se o app sai antes do registro (com a dica "outra instância?") ou depois dele; falha se o stderr tiver `panicked`, antes e depois do SIGTERM;
  - `trap cleanup EXIT`: SIGTERM, até 5 s de espera, SIGKILL e `wait`. O app é encerrado em qualquer caminho.

### Testes novos (26, todos por igualdade)
- `i18n` (6): tags pt (`pt-BR`, `pt_BR.UTF-8`, `pt`, `PT-br`, `pt-PT`) → pt-BR; outras (`en-US`, `C`, `POSIX`, `de-DE`, `es_ES.UTF-8`, `""`) → en; rótulos en e pt-BR exatos; rótulo de brilho com o percentual; nenhum rótulo vazio.
- `menu` (10): os passos 0/25/50/75/100; o menu exato do Windows e do Linux em en e pt-BR; ids estáveis; ida e volta `MenuAction` ↔ id para todo item das 2 plataformas; 7 ids únicos; 12 ids que o menu não mostra → `None`; `Platform::current()`.
- `tray` (10, com o fake RTK da T-2):
  - atalho no monitor selecionado (`WriteVcp(DEL, 0x10, 75)`, devolve `{monitorId: DEL}`);
  - sem seleção, no 1º listado (`WriteVcp(RTK, 0x10, 25)`);
  - 0% e 100% → 0 e 100;
  - sem monitor → `not_found` "no monitor is reachable", sem escrita;
  - selecionado que sumiu → `not_found` "monitor GONE not found", sem escrita;
  - `Timeout` na leitura do max → `timeout`, sem escrita;
  - `Click{Left,Up}` alterna; `Left/Down`, `Right`, `Middle`, `Enter`, `Move`, `Leave` e `DoubleClick` não alternam;
  - id do ícone estável.

### Smoke SNI (KDE Plasma Wayland, Fedora 44)
Verify da CONTEXT, como está:
```
$ cargo build -p ddc-tray --release --locked -q && bash apps/ddc-tray/scripts/smoke-sni.sh target/release/ddc-tray && echo OK
smoke-sni: started target/release/ddc-tray as PID 248086
smoke-sni: org.kde.StatusNotifierWatcher lists :1.2354/org/ayatana/NotificationItem/tray_icon_tray_app_ddc_control, owned by PID 248086
smoke-sni: OK — PID 248086 registered its tray item, was alive 2 s later and never panicked
OK
```
O PID conferido é o do processo lançado, então o re-exec sem DMA-BUF (D-2026-09-26-tray-app-10) mantém o PID, como previsto.

Falhas provocadas (script final, todas com exit 1 e o app encerrado; os wrappers ficaram no scratchpad, fora do repo):

| Caso | Saída |
|---|---|
| `/bin/true` (sai cedo) | `FAIL: the app exited before registering a tray item with status 0 — is another instance already running?` |
| sem watcher (`dbus-run-session -- bash smoke-sni.sh target/release/ddc-tray`) | `FAIL: no org.kde.StatusNotifierWatcher on the session bus: run this in a desktop session with a StatusNotifierItem host (KDE Plasma, or GNOME with the AppIndicator extension)` |
| outra instância já rodando | `FAIL: the app exited before registering a tray item with status 0 — is another instance already running?` (a 1ª recebeu o popup pelo single-instance; SIGTERM → 143) |
| processo vivo que nunca registra (`exec sleep 60`) | `FAIL: no StatusNotifierItem owned by PID 238416 within 15 s` (15,1 s) |
| item registrado por um PID filho (wrapper sem `exec`) | `FAIL: no StatusNotifierItem owned by PID 240026 within 15 s`: prova a checagem de PID |
| morre ~2 s depois de subir (wrapper com `exec` + SIGTERM agendado) | registra `:1.2380/…` e depois `FAIL: the app exited within 2 s of registering its tray item with status 143` |
| `panicked` no stderr (wrapper escreve e faz `exec` do app) | registra `:1.2391/…` e depois `FAIL: the app's stderr says 'panicked'`, com o fim do stderr |
| sem argumento / não executável | `FAIL: usage: …` / `FAIL: not an executable file: Cargo.toml` |

### Validação manual no KDE (release, via D-Bus, o mesmo caminho do Plasma)
- Watcher: `busctl --user get-property org.kde.StatusNotifierWatcher … RegisteredStatusNotifierItems` lista `:1.2400/org/ayatana/NotificationItem/tray_icon_tray_app_ddc_control`; `GetConnectionUnixProcessID` → `u 249170` = PID do app; `Status` `Active`.
- Menu (`com.canonical.dbusmenu.GetLayout`, com `LANG=pt_BR.UTF-8`): `Abrir painel`, separador, `Brilho 0%`, `Brilho 25%`, `Brilho 50%`, `Brilho 75%`, `Brilho 100%`, separador, `Sair`.
- `Event(2, "clicked")` (Abrir painel): o app continua vivo 3 s depois, sem nada no stderr além do `Gtk-Message … appmenu-gtk-module` de sempre (a ancoragem falha em silêncio, como previsto), e `/proc/<pid>/environ` tem `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
- `Event(10, "clicked")` (Sair): o processo termina com **status 0** (`wait`), 0 `panicked`.
- **Atalho de brilho não foi acionado no hardware, de propósito.** Sem seleção no popup, o alvo é o 1º monitor listado, que nesta máquina é a LG TV (`GSM-LG-TV-SSCR2-01010101`, pela saída da T-3), não o RTK autorizado. O caminho está coberto pelos testes com o fake. Brilho do RTK intocado: `ddcutil --bus 5 getvcp 10` continua `100`. Nota para a T-6 e o PR: o popup deve chamar `select_monitor` ao carregar, para os atalhos seguirem o monitor que o usuário vê.
- Nenhuma instância ficou rodando (`pgrep -fa 'target/(release|debug)/ddc-tray$'` vazio).

### Desvios do plano
- **`Position::TrayCenter` em vez de `TrayBottomCenter`.** No positioner 2.4.0, `TrayBottomCenter` põe o topo do popup no topo do ícone, crescendo para baixo, por cima da barra de tarefas do Windows. `TrayCenter` o põe acima do ícone (com fallback para baixo quando falta espaço em cima, no Windows), que é o que a D-2026-09-26-tray-app-5 trava ("ancorado acima dele"). O `move_window_constrained` mantém o popup dentro da tela do ícone.
- Os testes do `tray.rs` ficaram inline (`#[cfg(test)] mod tests { … }`), porque `src/tray/tests.rs` não estava no `files_modified`.
- Rótulo do tooltip: "DDC Control — monitor settings" / "DDC Control — ajustes do monitor" (o nome do produto não se traduz). No Linux/SNI o tooltip é ignorado pelo `tray-icon`.
- O helper `brightness_shortcut` mora em `tray.rs`, porque `commands.rs`/`panel.rs` não estavam no `files_modified`. Ele só compõe `shortcut_target` (T-3) e `set_brightness_percent` (T-2).
- O caminho do atalho reusa o `on_blocking_thread` da T-3 (`spawn_blocking` + `backend_unavailable`), em vez de um `spawn_blocking` próprio.
- O Windows continua sem compilação local (D-2026-09-26-tray-app-9): o clique, o `is_visible` e a ancoragem só são provados na `ci-crossbuild`.

### Verificação
| Comando | Resultado |
|---|---|
| Verify smoke SNI da CONTEXT | OK (acima) |
| Verify adapter único / `forbid` / single-instance / `panel.rs` puro | OK / OK / OK / OK |
| Gate 5.7 `Confirm::Yes` fora de `commands*` | nenhum |
| Gate 5.6 `unwrap`/`expect`/`panic!` fora de testes nos arquivos da T-5 | nenhum |
| `sys_locale` fora de `tray.rs` | nenhum |
| `cargo fmt --all --check` | OK |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | OK (só o aviso future-incompat de `nom v3.2.1`, pré-existente) |
| `cargo test -p ddc-tray --locked` | 92 passed, 0 failed, 1 ignored (hardware) |
| `cargo test --workspace --locked` | 339 passed, 0 failed, 8 ignored (hardware) |
| `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` | OK |
| `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` | OK |
| `cargo build -p ddc-tray --release --locked` | OK |
| `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'` | **TOTAL lines 88.70%** (2726 linhas, 308 sem cobertura); `i18n.rs` 100%, `menu.rs` 98.63% (só o ramo `Windows` de `Platform::current`), `tray.rs` 62.65% (o glue do Tauri), `popup.rs`/`dto.rs` 100%, `panel.rs` 99.04%, `commands.rs` 52.14%, `lib.rs` 0% |

## Tests
- Total: 339
- Passing: 339
- Coverage: 88.70% (`cargo llvm-cov --summary-only`, TOTAL lines)
- JS (`node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/**/*.test.mjs'`): 63 passing, 0 failing (medido na T-4; a T-5 não toca JS)
