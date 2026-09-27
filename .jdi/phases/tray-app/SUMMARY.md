# Phase 5: Tray app — Summary  (slug: tray-app)

**Status:** complete
**Tasks:** 8/8 complete, 0 blocked

## Executed tasks
- T-1: scaffold do crate `ddc-tray` (lib `ddc_tray` + bin `ddc-tray`) no workspace, com a config Tauri, a capability mínima, o `icon.svg` próprio, o conjunto de ícones gerado e o `tray.png`. Commits `e91a453` (build) e `4c93104` (docs).
- T-2: apresentação pura (`panel.rs`) genérica em `M: MonitorControl + ?Sized`, DTOs serde do contrato A-1 (`dto.rs`), 42 testes com o fake RTK e o golden `apps/ddc-tray/tests/fixtures/contract-rtk.json`. Commit `72d274e` (feat).
- T-3: os 7 comandos Tauri (`async fn` + `spawn_blocking`), o composition root (`compose_osd()`/`run()` com single-instance, blur/close escondem, `popup-shown`), o gate puro `popup.rs`, as permissões `allow-*` por comando e o teste de hardware `rtk_qhd_hdr`, rodado no RTK (brilho 100 → 90 → 100, conferido com `ddcutil`). Commit `e033137` (feat).
- T-4: módulos ES puros e sem dependências (`bridge.js` Tauri/demo, `demo-data.js`, `debounce.js`, `view-model.js`, `i18n/` en + pt-BR) e 63 testes `node --test` fora de `src/`, incluindo o RTK da demo `deepStrictEqual` ao golden. Commit `35c5ffd` (feat).
- T-5: bandeja com `tray.png`, tooltip e menu nativo bilíngue (`i18n.rs` + `menu.rs` puros), clique esquerdo no Windows alternando o popup ancorado pelo positioner com o gate de `popup.rs`, atalhos de brilho Safe em thread bloqueante com `panel-changed`, "Sair" → `app.exit(0)`, e `scripts/smoke-sni.sh`, que passa no KDE Wayland e falha nos 7 casos provocados. 26 testes novos. Commit `380a691` (feat).
- T-6: UI do popup em cartões Fluent com o gradiente do ícone: sliders, predefinição, entrada em chips, energia no cabeçalho, "Todos os ajustes" sob demanda com sondagem, diálogo `<dialog>` para todo `dangerous`, estados carregando/vazio/erro, claro/escuro, e o fallback de monitor pedido pelo orquestrador (lembra o último que respondeu e pula os mudos). 83 testes JS (+20). Commit `6e1273c` (feat).
- T-7: suíte Playwright do popup em `apps/ddc-tray/` (Gate 7): 18 testes × 2 temas = 36 verdes, com a CSP do Tauri injetada, console limpo e axe critical/serious `toEqual([])`, mais os screenshots claro/escuro (e o diálogo claro) em `docs/screenshots/`. O Verify do Gate 7 deu OK 3× seguidas. Commit `e677dd1` (test).
- T-8: bloco `frontend:` do PROJECT ligado (D-2026-09-26-tray-app-8), README (status na fase 5, seção "Tray app" com os 3 screenshots, build, deps Linux, uso, demo, testes, Known limitations), CHANGELOG `[Unreleased]` → Added e os passos 6–8 do tray em `docs/hardware-validation.md`. Achado: com o backend real, um monitor mudo carrega como painel vazio (a demo o pula); documentado como limitação, a correção de código fica com o orquestrador. Commit `c22aef0` (docs).

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

## T-6 — UI do popup: HTML, CSS Fluent-like, `app.js` e diálogo de confirmação

### O que foi feito
- **`index.html`** (D-2026-09-26-tray-app-6, -7):
  - nenhum texto literal: textos e atributos legíveis (`aria-label`, `title`) entram por `data-i18n`/`data-i18n-attr`, e o `<title>` também;
  - só `<link rel="stylesheet" href="styles.css">` e `<script type="module" src="app.js">`, sem `style`, `on*` ou script inline;
  - `<meta name="color-scheme" content="light dark">`, para os controles nativos seguirem o tema; o `lang` é ajustado no runtime.
- **Layout** (D-2026-09-26-tray-app-5). A janela é 360×560, e o RTK cabe sem rolar (506/506 px, contra 614/500 na 1ª versão).
  - **Cabeçalho fixo:**
    - o `icon.svg` do app em miniatura, o nome do monitor e uma linha secundária (`RTK · DDC/CI`);
    - o botão atualizar, que gira enquanto carrega, e o botão de energia, só ícone, em "perigo suave", na extrema direita;
    - com mais de um monitor, o título vira seletor: o `select` nativo fica transparente sobre o nome e o chevron, o que o deixa do tamanho do nome e com foco visível no conjunto.
  - **Cartão rápido:**
    - brilho, contraste e volume, cada um com ícone num tile, rótulo e pílula de valor `tabular-nums`;
    - o `input[type=range]` é nativo: trilha de 6 px com o gradiente ciano→azul→violeta revelado por `--fill` e polegar branco de 18 px com miolo de acento que cresce no hover, como no Windows 11, com anel de foco;
    - abaixo de um filete, a predefinição de cor num `select` estilizado.
  - **Entrada:** `radiogroup` de chips em 2 colunas. O ativo é preenchido com o gradiente profundo (branco ≥ 5,36:1 em toda a faixa) e leva um check.
    - Setas, Home e End só movem o foco (roving tabindex); Espaço ou Enter escolhe, e isso abre o diálogo.
  - **"Todos os ajustes":** `<details>` com chevron animado, carregado sob demanda.
    - Cada ajuste tem o controle do seu tipo (slider para contínuo, `select` em linha própria para NC), o código `0xNN` e a tag "Cuidado" nos `dangerous`. Uma leitura que falhou aparece como texto de status.
    - "Sondar ajustes ocultos" mostra spinner, o rótulo "Sondando…" e o aviso de que leva alguns segundos. Os códigos que respondem viram controles, e os mudos só são contados ("Códigos sem resposta: 7").
  - **Estados:**
    - skeleton com shimmer só na 1ª carga, sem animação com `prefers-reduced-motion`;
    - vazio com ilustração SVG própria, a dica de ativar o DDC/CI no menu do monitor e, só no Linux, a do `/dev/i2c`, com o caminho `docs/linux-ddc-setup.md` em `<code>`;
    - erro com ícone, o texto do `error.<kind>`, o detalhe e "Tentar de novo";
    - erro de escrita num aviso no rodapé, com "Tentar de novo".
  - **Teclado e eventos:**
    - Esc → `hide_popup`, exceto com o diálogo aberto;
    - `popup-shown` e `panel-changed` revalidam sem skeleton e sem mexer num slider em uso (lane ocupada ou ponteiro pressionado);
    - no Tauri, o menu de contexto do webview fica desligado.
  - **Read-back:** a UI mostra sempre o valor lido de volta. Se ele difere do pedido, a pílula recebe um anel âmbar de 1,2 s e o texto é anunciado via `aria-live=polite` ("Brilho: o monitor aplicou 70%.").
- **Diálogo de confirmação:**
  - `<dialog>` modal com foco inicial em Cancelar; Esc e clique no backdrop cancelam;
  - mostra o efeito (`confirmView`) e, na entrada, como voltar pelo botão do monitor; na energia, os modos diferentes do atual viram radios, e o texto acompanha a escolha;
  - o tom é de acento para a entrada e de perigo para energia e ajustes genéricos;
  - nada chega à fila de escrita antes de "Aplicar".
- **Escritas:**
  - a fila de `debounce.js` usa a chave `` `${monitorId}:${code}` `` com o `monitorId` no valor (nota da T-4);
  - cada widget guarda o monitor do painel a que pertence, então uma troca de monitor em andamento não desvia escritas para o monitor novo;
  - o slider Safe grava durante o arraste (80 ms, coalescendo) e faz `flush` no `change`; um contínuo `dangerous` só pré-visualiza e pergunta ao soltar.
- **`styles.css`:**
  - tokens em `:root`: cores, raios 12/8/6 px, espaços, sombras, durações 120/150/180 ms e easing;
  - escuro por `prefers-color-scheme` e `prefers-reduced-motion`;
  - fonte `"Segoe UI Variable Text", "Segoe UI", system-ui, -apple-system, "Noto Sans", Cantarell, sans-serif`;
  - rolagem interna fina e foco visível em tudo;
  - todo par texto/fundo foi conferido acima de 4,5:1 nos 2 temas;
  - num navegador largo (demo), o popup aparece numa moldura de 360×560.
- **`icons.js`:** 12 ícones de linha próprios na grade de 24, mais `brand` (o ícone do app) e `empty` (a ilustração).
  - São montados com `createElementNS`, sem parse de markup, com `aria-hidden` e ids de gradiente únicos por instância.
- **Fallback de monitor** (pedido do orquestrador depois da T-5):
  - **`view-model.js`** ganhou:
    - `monitorOrder` (o lembrado primeiro, depois a ordem do sistema);
    - `firstAnswering(ids, load, { stop })`, que tenta um de cada vez;
    - `storageOf`, `recallMonitor` e `rememberMonitor`, com `try/catch` e a chave `ddc-tray.last-monitor`;
    - `silent` no `monitorPicker`.
  - **`app.js`:**
    - ao carregar, tenta o lembrado e depois os demais, e só mostra erro se nenhum responder; o que falha continua no seletor como "LG TV SSCR2 (sem DDC/CI)";
    - escolhido à mão, um monitor que falha mostra o erro ("Sem resposta via DDC/CI" e a dica de ativar o DDC/CI), e "Tentar de novo" tenta aquele monitor; o seletor continua lá;
    - `select_monitor` só roda depois de um `load_panel` bem-sucedido, automático ou manual, e o botão de energia só aparece com o painel a que pertence.
  - **Demo:** o `two-monitors` lista a "LG TV SSCR2", com DDC/CI mudo (`timeout` em painel, features, sondagem e escrita), antes do RTK e da DELL. O golden do RTK não mudou.
- **i18n:** 23 chaves novas nos 2 locales, que passam de 64 para 87 chaves cada: `app.title`, `header.{monitor,meta,tagline,silent,noAnswer}`, `panel.quick`, `power.changeLabel`, `tag.dangerous`, `announce.readBack`, `more.{title,loading,empty,probe,probing,probeHint,probed,probeEmpty,probeSilent}`, `action.refresh`, `hint.ddc`, `confirm.recover.input` e `confirm.choose`.

### Testes (83 no total, +20; todos por igualdade)
- **`i18n-html.test.mjs` (11, novo):**
  - o `index.html` não tem texto literal nem atributo legível literal (`aria-label`, `title`, `alt`, `placeholder`, `aria-valuetext` etc.);
  - toda chave de `data-i18n`/`data-i18n-attr` e todo `t('…')` literal de `app.js`/`view-model.js` existe em en e pt-BR; `data-i18n-attr` só nomeia atributos legíveis, no formato `atributo:chave`;
  - `app.js` nunca atribui string literal a `textContent`;
  - o único script é o módulo `app.js`, sem `<style>`, `style=`, `on*=` ou `javascript:`;
  - nenhum `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, `setAttribute('style')`, `cssText`, `eval` ou `new Function`;
  - o único estilo tocado pelo script é `style.setProperty('--fill')`;
  - todo ícone pedido (`data-icon`, `CONTROL_ICONS`, `createIcon`/`tile`) existe em `ICONS`.
- **`view-model.test.mjs` (+8, total 25):**
  - `silent` no seletor;
  - a ordem de tentativa (lembrado, ausente, `null`, lista vazia);
  - `firstAnswering`: pula a TV e para no RTK, sem ler a DELL; o lembrado responde 1º e nada mais é lido; nenhum responde → `null` com as 2 falhas em ordem; `stop` interrompe;
  - memória em storage;
  - storage que recusa (getter de `localStorage` que lança, `getItem`/`setItem` que lançam, `null`).
  - O teste do `pickMonitor` saiu com a função.
- **`bridge-demo.test.mjs` (+1, total 18):** a TV é listada, mas `load_panel`, `load_features`, `probe_features` e `set_feature` dão `timeout`, sem escrita, e `select_monitor` responde `null`. A listagem do `two-monitors` agora tem os 3 monitores.
- **`contract.test.mjs`:** no `two-monitors`, o RTK e a DELL são localizados por id; o RTK continua igual ao golden, e todo monitor tem as chaves do contrato.

### Mutações (provadas e revertidas a partir de cópias, nada commitado)
| Mutação | Teste que falhou |
|---|---|
| `<h2 …>Entrada</h2>` literal no HTML | `the page has no literal text` |
| `aria-label="Atualizar"` literal | `the page has no literal readable attribute` |
| `data-i18n="more.titel"` | `every data-i18n and data-i18n-attr key exists in both locales` |
| `t('tag.danger')` no `app.js` | `every literal t('…') key … exists in both locales` |
| `node.innerHTML = ''` | `scripts never parse markup…` |
| `input.setAttribute('style', …)` no lugar do `setProperty` | `scripts never parse markup…` + `a slider fill is set as a custom property…` |
| `data-icon="gear"` | `every icon the page and the app ask for exists` |
| `break` depois da 1ª falha em `firstAnswering` | `the first monitor that answers is shown…` + `only when no monitor answers…` |

Depois de reverter: `cmp` idêntico às cópias e `# pass 83`, `# fail 0`.

### Validação no navegador (scripts descartáveis no scratchpad, fora do repo)
- **Chromium do Playwright 1.63.0**, 360×560, `deviceScaleFactor` 2, claro e escuro, pt-BR, com a CSP exata do `tauri.conf.json` injetada por header:
  - os 5 caminhos (`/`, `?demo=rtk`, `two-monitors`, `empty`, `error`) com 0 console error/warning e 0 `pageerror`, inclusive com os diálogos abertos, "Todos os ajustes" aberto e a sondagem.
- **Roteiro de comportamento** (todos `assert` verdes, 0 erro de console):
  - **Teclado:** Tab → atualizar → energia → brilho; `ArrowRight` gera exatamente 1 escrita `{0x10, 76, confirmed:false}`, e o `aria-valuetext` vira `76%`.
  - **Coalescência:** 5 `input` seguidos geram 1 escrita `{0x12, 50}`.
  - **Entrada:** o clique abre o diálogo com o foco em Cancelar; Esc e Cancelar deixam 0 escritas e a entrada inalterada; Aplicar grava `{0x60, 0x11, confirmed:true}` e marca HDMI-1 pelo read-back. No radiogroup, a seta só move o foco, e Espaço abre o diálogo.
  - **Energia:** radios `[0x04 ✓, 0x05]`; escolher 0x05 atualiza o texto; Aplicar grava `{0xD6, 5, confirmed:true}`.
  - **"Todos os ajustes":** mudar 0xCA abre o diálogo, e Cancelar volta o `select` para "Ativado"; mudar 0xCC (Safe) grava `{0xCC, 3, confirmed:false}`.
  - **Eventos:** Esc fora do diálogo não dá erro; `popup-shown` e `panel-changed` revalidam sem skeleton e com "Todos os ajustes" ainda aberto.
  - **Read-back divergente** (bridge da demo alterado só no teste para limitar a 70): `End` no brilho → pílula e slider em 70%, anúncio `Brilho: o monitor aplicou 70%.` e o destaque na pílula.
  - **Skeleton e movimento:** skeleton na 1ª carga (demo com latência de 1,5 s); com `reducedMotion: 'reduce'`, o shimmer fica `display: none`.
  - **Fallback:** o `two-monitors` abre o RTK, e o seletor mostra `["LG TV SSCR2 (sem DDC/CI)","RTK QHD HDR","DELL U2723QE"]`. Escolher a TV mostra erro, com o seletor e sem o botão de energia. Escolher a DELL grava `localStorage` e `__ddcDemo.selected` = DELL, e depois de recarregar a página a DELL abre direto.
- **axe-core 4.10.3** (`wcag2a`, `wcag2aa`, `wcag21aa` e `best-practice`) em 9 estados × 2 temas (padrão, RTK, "Todos os ajustes" + sondagem, diálogo de entrada, diálogo de energia, `two-monitors`, TV em erro, vazio e erro): 0 violações. Os "incomplete" de contraste são o chip com gradiente, conferido à mão (≥ 5,36:1).
- **WebKitGTK 2.54 do sistema** (o motor do Tauri no Linux), via PyGObject `OffscreenWindow` com `WEBKIT_DISABLE_DMABUF_RENDERER=1`: RTK, diálogo e "Todos os ajustes", claro e escuro, renderizam igual ao Chromium (backdrop com desfoque, trilha, polegar, rolagem fina), sem mensagens de console. O WebKit do Playwright não sobe nesta máquina: ele pede libs de host com nomes do Ubuntu, e instalar exigiria sudo.
- **Screenshots de trabalho** (não commitadas; os oficiais são da T-7), em `/tmp/claude-1000/-home-slipalison-repos-ddc-control/0d3f145e-b932-4d13-9d1c-2a0d820f1d21/scratchpad/t6/shots/`: `{light,dark}-{root,rtk,rtk-more,rtk-probing,rtk-probed,rtk-dialog-input,rtk-dialog-power,rtk-en,two,two-tv,two-tv-focus,two-dell,empty,error,skeleton}.png`, `light-{focus-slider,focus-chip,readback}.png` e `gtk-{light,dark}-rtk-{0,1,2}.png`.

### Desvios do plano
- **Energia no cabeçalho** (botão de ícone "perigo suave", sempre com o diálogo) e **predefinição dentro do cartão rápido**, em vez de uma linha própria para cada um. Com as 7 entradas do RTK, o conteúdo passava 114 px dos 560. Assim tudo cabe sem rolar, e a rolagem só aparece com "Todos os ajustes" aberto. O estado atual da energia ("Ligado") deixou de aparecer, e a chave `power.change` saiu.
- **Fallback de monitor** (pedido do orquestrador): tocou `demo-data.js`, fora do `files_modified`, e o `two-monitors` agora lista 3 monitores (a TV muda + RTK + DELL). O nome do cenário foi mantido porque é `critical_path` do Gate 7 (T-7/T-8). O `pickMonitor` do view-model saiu, substituído por `monitorOrder`, e o teste foi ajustado.
- **Radiogroup:** as setas movem só o foco, sem selecionar, ao contrário do padrão APG, porque cada seleção é uma troca `dangerous` confirmada. O axe não aponta nada.
- **`select` NC em "Todos os ajustes" em linha própria**, com largura total, para não truncar rótulos como "Ajuste automático" com a tag "Cuidado".
- **A dica do `/dev/i2c`** também aparece no erro de um monitor mudo (`timeout`), pela regra do `statusView` da T-4 (kinds `backend_unavailable|transport|timeout`). Junto dela vem a dica de ativar o DDC/CI, que é a mais provável nesse caso.
- **O detalhe do erro** (`error.message`, em inglês, vindo do core) aparece em mono pequeno abaixo do texto traduzido, como diagnóstico.
- **Nenhum `cargo` foi rodado e nenhum arquivo Rust foi tocado** (pedido do orquestrador: a T-5 rodava em paralelo). O binário atual embute o `index.html` antigo; a UI nova entra no próximo `cargo build`, porque o `frontendDist` é embutido na compilação.

### Verificação
| Comando | Resultado |
|---|---|
| Verify `node --test` da CONTEXT (`--test-reporter=tap`, `# pass ≥ 20`, `# fail 0`, nada de `*.test.*` em `src/`) | OK: `# tests 83`, `# pass 83`, `# fail 0` |
| Verify i18n da CONTEXT (`tests/ui/i18n*.test.mjs`) | OK: `# tests 24`, `# pass 24`, `# fail 0` |
| 5 `critical_paths` no Chromium com a CSP do Tauri, claro e escuro | 0 erro de console, 0 `pageerror` |
| axe-core, 9 estados × 2 temas | 0 violações |
| WebKitGTK 2.54 (RTK, diálogo, "Todos os ajustes"), claro e escuro | renderiza igual, 0 mensagens de console |
| `node --check apps/ddc-tray/src/app.js` | OK |
| Golden `contract-rtk.json` | intocado |

## T-7 — Suíte Playwright (Gate 7) + screenshots claro/escuro

### O que foi feito
- **Pacote** (D-2026-09-26-tray-app-2, -8):
  - `apps/ddc-tray/package.json`: `private`, `"type": "module"`, só `devDependencies` exatas (`@playwright/test` 1.63.0 e `@axe-core/playwright` 4.13.0) e `scripts.test = "playwright test"`;
  - `package-lock.json` gerado por `npm install --ignore-scripts`, com 5 pacotes: os 2 acima, `playwright`, `playwright-core` 1.63.0 e `axe-core` 4.13.0;
  - `.gitignore` com `node_modules/`, `test-results/` e `playwright-report/`;
  - nada entra em `src/`, que é o `frontendDist` embutido: o popup continua sem npm de runtime.
- **`playwright.config.mjs`:**
  - `testDir: tests/e2e`, então os `tests/ui/*.test.mjs` do `node --test` ficam de fora;
  - `webServer` `python3 -m http.server 1420 --directory src` com `reuseExistingServer: true`, e o stdout/stderr do servidor descartados para o log de requisições não poluir o `--reporter=list`;
  - `baseURL` `http://localhost:1420`, viewport 360×560, Chromium e `locale: 'pt-BR'`;
  - projetos `light` e `dark` por `colorScheme`;
  - `retries: 0` (um flaky não fica mascarado), `forbidOnly: true`, e trace e screenshot só em falha.
- **`tests/e2e/support.mjs`** (compartilhado):
  - toda resposta sai com a CSP lida do `tauri.conf.json` (D-2026-09-26-tray-app-7), via `page.route`, então o que o webview recusaria falha aqui também;
  - a fixture `page` junta `console.error` e `pageerror` em `pageErrors` e, no teardown, exige `toEqual([])` em todo teste;
  - `expectAccessible`: axe com as tags `wcag2a`, `wcag2aa` e `wcag21aa`. Critical/serious → `expect(...).toEqual([])` com `{id, impact, help, targets}` no diff; moderate/minor → anotação no teste e uma linha no stdout (o reviewer classifica como WARN);
  - os textos saem das chaves via `translator('pt-BR')` de `src/i18n/index.js`, sem strings duplicadas nos specs;
  - `loadEnds`: um `MutationObserver` no botão atualizar espera o fim da carga que uma ação dispara (determinístico, sem `waitForTimeout`);
  - locators por papel e nome acessível (`slider`, `radiogroup`/`radio`, `combobox`, `dialog`, botões).

### Testes (18 por tema, 36 no total; mais 2 de screenshot por tema, em `skip` sem `SCREENSHOTS=1`)
- **`scenarios.spec.mjs` (8):**
  - os 5 `critical_paths` (`/`, `?demo=rtk`, `two-monitors`, `empty` e `error`), cada um com o estado esperado visível:
    - RTK: 75/50/30 com o `aria-valuetext`, 7 entradas com DisplayPort-1 marcada, preset sRGB e o botão de energia;
    - `two-monitors`: o seletor com 3 monitores, parado no RTK;
    - `empty`: o título, a dica de DDC/CI e "Tentar de novo";
    - `error`: `error.backend_unavailable` com o detalhe;
    - no Linux, a dica `<code>docs/linux-ddc-setup.md</code>`;
  - com os diálogos de entrada e de energia abertos: texto do efeito, foco em Cancelar e 0 escritas;
  - "Todos os ajustes" aberto e sondado: 7 ajustes, 2 encontrados e "Códigos sem resposta: 7";
  - em todos: axe critical/serious `toEqual([])` e `pageErrors` `toEqual([])`, ambos como assert explícito no corpo do teste.
- **`keyboard.spec.mjs` (2):**
  - Tab (só Tab) chega ao brilho; `ArrowRight` muda para 76 e o `aria-valuetext` para `76%`; `__ddcDemo.writes` fica exatamente em `[{monitorId: RTK, code: 0x10, value: 76, confirmed: false}]`, conferido de novo depois de mais uma janela de debounce e latência (`2×80 + 60` ms, constantes importadas de `src/`);
  - no radiogroup, a seta só move o foco (DisplayPort-2 focado, DisplayPort-1 ainda marcado, sem diálogo); Espaço abre o diálogo com foco em Cancelar; Esc cancela, com 0 escritas.
- **`confirm.spec.mjs` (5):**
  - outra entrada abre o diálogo (texto do efeito, a nota de como voltar e foco em Cancelar) com 0 escritas;
  - Cancelar e Esc deixam 0 escritas e DisplayPort-1 marcada;
  - Aplicar → exatamente `[{code: 0x60, value: 0x11, confirmed: true}]`, e HDMI-1 marcada sem `is-pending` (o chip só é marcado quando o read-back chega);
  - **read-back divergente:** um `bridge.js` alterado só no teste, via `route`, com uma guarda que falha se a linha alterada mudar em `src/`, faz o monitor manter a entrada. A escrita confirmada sai, mas a UI mantém DisplayPort-1 e anuncia "Entrada: o monitor aplicou DisplayPort-1.";
  - energia: o diálogo lista os 2 outros modos, com "Em espera (DPM)" marcado; Cancelar → 0 escritas; escolher "Desligado (botão de energia)" atualiza o texto, e Aplicar → `[{code: 0xD6, value: 0x05, confirmed: true}]`.
- **`fallback.spec.mjs` (3):**
  - a TV muda vem 1º na lista e o popup abre no RTK sem erro. O seletor mostra `["LG TV SSCR2 (sem DDC/CI)", "RTK QHD HDR", "DELL U2723QE"]` na ordem do sistema, sem aviso e sem toast, e `__ddcDemo.selected` = RTK;
  - escolher a TV mostra `error.timeout`, a dica de DDC/CI, "Sem resposta via DDC/CI" e "Tentar de novo", com o seletor ainda lá e sem o botão de energia; o axe passa nesse estado;
    - "Tentar de novo" tenta a TV de novo e continua em erro com a TV. Um `refresh` abriria o RTK, então isso distingue os dois caminhos;
    - `selected` continua RTK: um monitor mudo nunca vira alvo dos atalhos;
  - da TV em erro, escolher a DELL mostra o painel dela (brilho 40, sem volume, entrada `Valor 0x1B`), e `selected` e o `localStorage` passam a ser a DELL; depois do `reload`, ela abre direto.
- **`screenshots.spec.mjs` (2 por tema, A-7):** só com `SCREENSHOTS=1`, em `en-US` (o README é em inglês), `reducedMotion: 'reduce'`, `animations: 'disabled'` e `deviceScaleFactor: 2`. Gera:
  - `docs/screenshots/tray-popup-light.png` e `tray-popup-dark.png`, PNG 720×1120;
  - `tray-popup-dialog-light.png`, o diálogo de entrada, para o README, só no tema claro.

### Mutações (provadas e revertidas com `git checkout`, `src/` limpo depois; nada commitado)
| Mutação | Resultado |
|---|---|
| **Sem o diálogo de confirmação** (`app.js`: `if (entry.dto.dangerous && !(await confirmChange(…)))` → `if (false)`, e a energia sem `askDialog`, gravando o 1º modo) | `confirm`: **10 failed** (os 5 × 2 temas). O 1º falha em `expect(confirmDialog).toBeVisible()`, "element(s) not found"; a energia falha em `toHaveCount(2)`, com 0 recebidos |
| Slider sem a fila de debounce (`bridge.setFeature` direto no `input` e no `change`) | `keyboard`: o teste de 1 escrita falha, porque recebe 2 escritas `{0x10, 76}`; o do radiogroup passa |
| `console.error("mutant")` no `start()` | `scenarios`: falha com `"console.error: mutant"` no diff de `pageErrors` |
| `document.body.setAttribute("style", …)` no `start()` | `scenarios`: falha com o console error "Applying inline style violates … 'style-src 'self''", ou seja, a CSP injetada está ativa |
| Botão atualizar sem `aria-label` (`index.html`) | `scenarios`: falha no assert do axe, com `{id: "button-name", impact: "critical", targets: ["#refresh"]}` |

### Achados de UI
- Nenhum bug encontrado; `apps/ddc-tray/src/` ficou intocado.

### Desvios do plano
- **Arquivos além do `files_modified`,** todos de teste ou de docs:
  - `tests/e2e/support.mjs`: fixtures e helpers comuns, para não duplicar CSP, coleta de erros e axe em 5 specs;
  - `tests/e2e/fallback.spec.mjs`: o fallback de monitor pedido pelo orquestrador, sem arquivo nomeado no plano;
  - `docs/screenshots/tray-popup-dialog-light.png`: opcional, pedido pelo orquestrador.
- **CSP do Tauri injetada como header** em toda resposta. Não foi pedido, mas fortalece o gate: uma regressão de CSP (estilo ou script inline) vira console error, como provado pela 4ª mutação. O axe continua funcionando sob a CSP, porque o Playwright injeta o script via CDP.
- **Testes além do pedido:**
  - o read-back divergente, que prova "a UI mostra o read-back" com valores diferentes do pedido;
  - a confirmação da energia (A-3);
  - o radiogroup por teclado;
  - "Todos os ajustes" sondado sob axe;
  - DELL + memória no fallback.
- **Idioma:** a suíte roda em pt-BR, com os textos vindos das chaves, e os screenshots são em inglês, para o README (en) da T-8.
- **`"type": "module"` no `package.json`** passa a valer também para os `src/*.js` no Node. Antes, o Node detectava a sintaxe ESM. Sem efeito prático: `node --test` segue com 83/83, e o Tauri não lê o `package.json` (`frontendDist` estático).
- **O lock não tem o `fsevents`,** opcional do `playwright`, só para macOS: o npm 11 no Linux não o registra. No macOS, o `npm ci` só não instala um opcional. Fica como nota para a `ci-crossbuild`.

### Verificação
| Comando | Resultado |
|---|---|
| Verify do Gate 7 da CONTEXT (`npm ci … && npx playwright test --reporter=list`, `grep` de passed/failed/flaky), 3× seguidas | **OK, OK, OK**: `36 passed`, `4 skipped` em cada, em 6,0 s, 6,3 s e 5,4 s |
| `npx playwright test --repeat-each=5` | `180 passed`, `20 skipped` (20,7 s) |
| `npx playwright test --repeat-each=3 --workers=24` | `108 passed`, `12 skipped` (12,7 s) |
| axe moderate/minor nas 3 execuções | 0 (`grep -c 'axe moderate'` = 0) |
| `SCREENSHOTS=1 npx playwright test screenshots` | `3 passed`, `1 skipped` (o diálogo no tema escuro) |
| Verify dos screenshots da CONTEXT (`test -s` + `file` → `PNG image`) | OK |
| Verify `node --test` e i18n da CONTEXT (`--test-reporter=tap`) | OK: `# pass 83`, `# fail 0` |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` | limpos; 339 passed, 0 failed, 8 ignored (hardware) |

## T-8 — Docs: bloco `frontend:` do PROJECT (D-8), README, CHANGELOG, roteiro de hardware

### O que foi feito
- **`.jdi/PROJECT.md`**: só o bloco `frontend:` mudou. `has_frontend: true`, `frontend_url: http://localhost:1420`, `dev_command: python3 -m http.server 1420 --directory apps/ddc-tray/src`, `critical_paths` = `/`, `/?demo=rtk`, `/?demo=two-monitors`, `/?demo=empty` e `/?demo=error`. Um comentário cita a D-2026-09-26-tray-app-8 e diz que o Gate 7 roda a suíte `apps/ddc-tray` (`npm ci --ignore-scripts && npx playwright test`). O bloco foi conferido com `yaml.safe_load`.
- **`README.md`** (en):
  - status na fase 5, com a ressalva de que a única escrita em hardware até aqui é a do teste do tray (brilho 100 → 90 → 100 na T-3);
  - seção **Tray app**:
    - os 3 screenshots (claro, escuro e o diálogo) numa tabela HTML com `width="240"`, avisando que são da demo;
    - **Build and run** (`cargo run -p ddc-tray` e o build release), com as deps Linux por distro (Fedora `webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel systemd-devel`; Debian/Ubuntu `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libudev-dev pkg-config`) e o link do `/dev/i2c` → `docs/linux-ddc-setup.md`;
    - **Using it**: o menu no Linux, os cliques no Windows, o debounce, o read-back, a confirmação de todo `dangerous`, "All settings" e a sondagem, o seletor com o fallback, os estados, en/pt-BR;
    - **Browser demo**: os 4 cenários do `?demo=`;
    - **Tests**: `cargo test`, `node --test` e Playwright, mais `SCREENSHOTS=1`;
    - **Known limitations of the tray app**: sem clique e sem posicionamento no Linux/Wayland, GNOME só com a extensão AppIndicator, DMA-BUF desligado (D-2026-09-26-tray-app-10; valor do usuário respeitado), monitor mudo = painel vazio (achado abaixo), Windows nunca compilado (D-2026-09-26-tray-app-9), sem autostart/perfis/hotkeys (`profiles-hotkeys`) e sem instalador (`release-packaging`);
  - **Layout** com `apps/ddc-tray` (`src-tauri/`, `src/`, `tests/`), e o "Still to come" passa a ser CI, pacotes e `profiles-hotkeys`;
  - **Monitor-write safety** ganhou um parágrafo: o diálogo faz o papel do `--yes`, e `Confirm::Yes` só existe em `commands.rs`;
  - **Dev setup** ganhou:
    - as deps do Tauri, porque `cargo build --workspace` agora as exige;
    - a linha do Playwright nos quality gates;
    - o motivo de o tray ficar fora do check Windows;
    - o comando de hardware do tray.
- **`CHANGELOG.md`**: um item `ddc-tray` em `[Unreleased]` → Added, com sub-itens:
  - popup e controles;
  - debounce e read-back;
  - confirmação;
  - vários monitores;
  - menu por plataforma;
  - i18n, tema, teclado, CSP, capabilities e single-instance;
  - DMA-BUF;
  - demo, Playwright/axe, `node --test`, golden, smoke SNI e o teste de hardware.
- **`docs/hardware-validation.md`**: título generalizado ("Hardware validation"), e a introdução aponta para a seção nova.
  - **Tray app — phase `tray-app`**: sair do tray antes dos passos 6 e 7.
  - **6. smoke SNI**: comando, as 3 linhas esperadas (formato conferido no script) e os casos de FAIL.
  - **7. teste de hardware**: com `ddcutil --bus 5 getvcp 10` antes e depois, os 4 passos do teste (conferidos no código) e os números da T-3.
  - **8. o popup à mão** (opcional): só escritas Safe desfeitas, atalhos só com o RTK carregado, diálogos só com Cancelar, "All settings" e a sondagem só para ler.
  - **Never do through the popup**: nunca Aplicar entrada, energia ou qualquer entrada **Caution** (`0xCA`, `0x1E`, `0x20`, `0x30`, `0x7E`, `0xE6`, `0xF1`…).

### Conferido no código antes de afirmar
- Menu, rótulos e cliques: `menu.rs`, `i18n.rs` e `tray.rs` (`show_menu_on_left_click(false)`, `Click{Left,Up}`, `Position::TrayCenter`, erro de ancoragem logado só no Windows).
- Re-exec DMA-BUF (`var_os(...).is_some()` → qualquer valor do usuário é mantido) e single-instance: `lib.rs`.
- Textos em inglês: `src/i18n/en.js`. Fallback de monitor: `app.js` (`refresh`/`firstAnswering`/`showLoaded`). Cancelar restaura o widget: `requestChange`.
- Deps Linux: pacotes `rpm -q` instalados (`webkit2gtk4.1-devel`, `libappindicator-gtk3-devel`, `librsvg2-devel`, `systemd-devel`). O `ldd` do release linka webkit2gtk-4.1, gtk-3, soup-3, javascriptcoregtk e udev; `libxdo-devel` não está instalado e não faz falta. Os nomes Debian/Ubuntu não foram testados, e o README diz isso.
- Linhas do smoke: `echo`s de `smoke-sni.sh`. Passos do teste de hardware: `tests/rtk_qhd_hdr.rs`.

### Achado (fora do escopo da T-8; nenhum código foi tocado)
**Com o backend real, um monitor com DDC/CI mudo não é pulado.** `panel::load_panel` descarta cada controle cuja leitura falha e só devolve erro em `MonitorNotFound`. Um monitor listado mas mudo, como a LG TV do dev (que responde `Transport`, exit 6 no CLI), devolve `Ok` com 0 controles. Então, no `app.js`, o `firstAnswering` o aceita como "respondeu":
- o popup pode abrir nele com o painel vazio (no dev a TV vem 1ª na lista);
- o `select_monitor` o torna alvo dos atalhos de brilho;
- ele fica lembrado para a próxima abertura.

A demo diverge: o `silent` do `bridge.js` rejeita com `timeout`, então o `two-monitors` e o `fallback.spec` provam um caminho que o app real não percorre. Nenhum teste Rust cobre o caso "todos os controles falham".

- **Prova** (teste temporário em `panel/tests.rs`, com os 6 códigos rápidos em `Transport`; o arquivo foi restaurado da cópia e `git status` ficou limpo):
  ```
  T8 PROBE load_panel(mute TV) = Ok(PanelDto { monitor_id: "GSM-LG-TV-SSCR2-01010101", controls: [] })
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out
  ```
- **Correção proposta ao orquestrador** (fora do `files_modified` da T-8, por isso não foi feita):
  - em `load_panel`, quando nenhum controle é lido, devolver `ui_error` da 1ª falha (`timeout`/`transport`), em vez de `Ok` vazio;
  - um teste Rust com um fake mudo;
  - retirar do README e do `docs/hardware-validation.md` (passo 8) a limitação "A monitor whose DDC/CI is mute opens as an empty panel".

  A demo e o golden não mudam.
- **Documentação honesta enquanto isso:** o README não afirma que o app real pula a TV. A demo afirma ("skipped and marked (no DDC/CI)"), porque é o que ela faz.

### Desvios do plano
- O screenshot do diálogo entrou no README (pedido "se couber"), e a tabela usa HTML para os 3 caberem lado a lado.
- O heading "Known limitations of the tray app" tem nome próprio para não colidir com a âncora do "Known limitations" do CLI.
- Além do pedido:
  - o passo 8 (popup à mão) no roteiro de hardware;
  - o parágrafo em "Monitor-write safety";
  - a linha do Playwright nos quality gates;
  - o título do roteiro generalizado.
- Um único commit de docs: o hook aceitou `.jdi/PROJECT.md` com README/CHANGELOG/docs, porque nada disso é código.

### Verificação
| Comando | Resultado |
|---|---|
| Test do plano (`grep -q 'has_frontend: true' … && grep -q '^## \[Unreleased\]' CHANGELOG.md`) | `PLAN TEST: OK` |
| `yaml.safe_load` do bloco `frontend:` | `{'has_frontend': True, 'frontend_url': 'http://localhost:1420', …, 'critical_paths': ['/', '/?demo=rtk', '/?demo=two-monitors', '/?demo=empty', '/?demo=error']}` |
| `cargo test --workspace --locked` | 339 passed, 0 failed, 8 ignored (hardware) |
| `node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/**/*.test.mjs'` | `# tests 83`, `# pass 83`, `# fail 0` |
| `cd apps/ddc-tray && npm ci --ignore-scripts … && npx playwright test --reporter=list` | `36 passed`, `4 skipped` (5,7 s) |
| `cargo llvm-cov --workspace --locked --summary-only --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'` | TOTAL lines **88.70%** (2726 linhas, 308 sem cobertura) |

## Tests
- Total: 383 (Rust, `cargo test --workspace --locked`) + 9 ignored (hardware, `DDC_HW_TESTS=1`), medido na iteração 4 (iteração 3: 371 + 9; iteração 2: 344 + 9; T-8: 339 + 8)
- Passing: 383
- Coverage: 83.22% (`cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`, TOTAL lines, medido na iteração 4; iteração 3: 83.98%; iteração 2: 88.79%; T-8: 88.70%). Sem o filtro (Verify literal do PROJECT): 82.78%.
- JS (`node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/**/*.test.mjs'`): 124 passing, 0 failing, 0 cancelled, 0 skipped, 0 todo (iteração 4 = iteração 3; iteração 2: 84; T-8: 83)
- Playwright (`cd apps/ddc-tray && npm ci --ignore-scripts && npx playwright test --reporter=list`): 52 passing, 0 failing, 6 skipped (screenshots sem `SCREENSHOTS=1`), iteração 4 (iteração 2: 36 + 4). Com `SCREENSHOTS=1`, o `screenshots.spec.mjs` roda 6 de 6 (iteração 3: 4 + 2 pulados).

## Iteração 2 — correções da review

Modo fix (REVIEW iter 1 = BLOCKED). As 8 tasks já estavam `completed` e nenhuma foi reaberta. O trabalho foi o B-1 e o W-4. O CONTEXT.md não foi tocado.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `b110026` | fix | B-1: `load_panel` devolve o 1º erro de leitura que não seja `MonitorNotFound` quando nenhum dos 6 controles rápidos é lido. A falha parcial continua como antes (D-2026-09-26-tray-app-4/-5). São 2 testes novos por igualdade. |
| `e4ed39f` | fix | Demo alinhada ao Rust: a TV muda do `two-monitors` passa a rejeitar com `transport` e a mensagem real, em vez de `timeout`. Entram o golden novo `contract-mute.json`, o teste Rust `mute_contract_matches_the_golden` e o teste JS que exige a mesma resposta da demo. `bridge-demo.test.mjs` e `fallback.spec.mjs` foram ajustados ao kind real. |
| `aae9e95` | test | Teste de hardware `rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads`, `#[ignore]`, gated por `DDC_HW_TESTS=1`, só leituras. |
| `f298cee` | refactor | W-4: a decisão do re-exec DMA-BUF virou `dmabuf_switch_to_set(Option<&OsStr>) -> Option<&str>`, função pura. Há 2 testes, sem `set_var`, que cobrem valor ausente, `1`, `0`, vazio e não-UTF-8. |
| `2b8b67f` | docs | Saiu do README a limitação "mute opens as an empty panel", e as linhas ~310 e ~352 foram ajustadas. Também mudaram: a tabela da demo, os goldens (agora 2), a nota de dev setup, o CHANGELOG (vários monitores, goldens e testes de hardware) e `docs/hardware-validation.md` (passo 7 com os 2 testes e a saída esperada; passo 8 espera o popup no RTK). |

### B-1: o que mudou
- **Rust (`panel.rs`):** o laço guarda `first_failure` (`get_or_insert`). No fim, se `controls` está vazio, devolve `Err(ui_error(first_failure))`; senão, `Ok` com os controles lidos. `MonitorNotFound` continua curto-circuitando.
- **Testes Rust (`panel/tests.rs`):**
  - `a_mute_monitor_fails_its_panel_instead_of_loading_it_empty`: fake `mute_tv()` com os 256 códigos VCP em `Transport("DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)")`, que é a mensagem real da LG TV medida na `ddc-backends`. Espera `Err(UiError { kind: Transport, message: "transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)" })` e nenhuma escrita.
  - `a_panel_with_no_control_fails_with_its_first_reading_error`: brilho em `Transport("nak")`, contraste em `Timeout` e o resto sem valor (`UnsupportedFeature`). Espera exatamente `Err(UiError { kind: Transport, message: "transport error: nak" })`, o que prova que é o 1º erro.
  - `a_control_whose_read_times_out_is_left_out_and_the_rest_still_load` (falha parcial, 5 controles) continua verde e sem mudança.
- **Contrato mudo:** `apps/ddc-tray/tests/fixtures/contract-mute.json` (`{ monitor, loadPanel }`) foi gerado de propósito, só ele, com `DDC_TRAY_UPDATE_GOLDEN=1 cargo test -p ddc-tray --locked --lib -- panel::tests::mute_contract_matches_the_golden`. O `contract-rtk.json` **não mudou**: o `git status` depois da geração mostrava só o arquivo novo. Os helpers de golden viraram `assert_matches_golden(file, &contract)`, compartilhados pelos 2 testes.
- **Demo:** `demo-data.js` `muteTv()` ganhou `mute: { kind: 'transport', message: <a mesma> }`; `bridge.js` `answering()` rejeita com esse erro. O `contract.test.mjs` exige `monitors[0]` do `two-monitors` === `muteGolden.monitor` e `loadPanel(TV)` rejeitando `deepStrictEqual` a `muteGolden.loadPanel`.
- **`fallback.spec.mjs`:** o heading esperado passou de `error.timeout` para `error.transport`. Continua verde pelo caminho certo: a demo rejeita a TV com o mesmo `{kind,message}` que o Rust devolve, então `firstAnswering` a pula, marca "(no DDC/CI)" e abre no RTK. Escolher a TV mostra o erro de transporte com `hint.ddc`, `header.noAnswer` e "Try again".

### Mutações que provam os testes
1. **Antes da correção** (testes novos contra o `load_panel` antigo):
   ```
   test panel::tests::a_panel_with_no_control_fails_with_its_first_reading_error ... FAILED
   test panel::tests::a_mute_monitor_fails_its_panel_instead_of_loading_it_empty ... FAILED
     left: Ok(PanelDto { monitor_id: "RTK-RTK-QHD-HDR-01010101", controls: [] })
    right: Err(UiError { kind: Transport, message: "transport error: nak" })
     left: Ok(PanelDto { monitor_id: "GSM-LG-TV-SSCR2-01010101", controls: [] })
    right: Err(UiError { kind: Transport, message: "transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)" })
   test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 92 filtered out
   ```
2. **Sobre o HEAD final**: só o braço `Some(error) if controls.is_empty() => Err(ui_error(error))` foi trocado por `Ok(PanelDto { .. })`. O arquivo foi restaurado da cópia e o `git status` ficou limpo.
   ```
   test panel::tests::a_mute_monitor_fails_its_panel_instead_of_loading_it_empty ... FAILED
   test panel::tests::a_panel_with_no_control_fails_with_its_first_reading_error ... FAILED
   test panel::tests::mute_contract_matches_the_golden ... FAILED
   test result: FAILED. 94 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
   ```
3. **Demo voltando a `timeout`**: `kind: 'transport'` virou `'timeout'` em `demo-data.js`, depois restaurado. O resultado foi `not ok 3 - the demo mute TV is listed first and fails its panel exactly as the Rust side does`, com `# pass 3`, `# fail 1`.

### Validação real no hardware (só leituras + a escrita Safe restaurada do teste existente)
Nenhum `ddc-tray`/`ddc-cli` rodando antes (`pgrep` vazio). O app e o popup **não** foram abertos na tela.

```
$ ddcutil --bus 5 getvcp 10
VCP code 0x10 (Brightness                    ): current value =   100, max value =   100
$ DDC_HW_TESTS=1 cargo test -p ddc-tray --locked --test rtk_qhd_hdr -- --ignored --test-threads=1 --nocapture
test rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads ... list_monitors: 1.139824288s
monitors: [ GSM-LG-TV-SSCR2-01010101 "LG TV SSCR2" (GSM), RTK-RTK-QHD-HDR-01010101 "RTK QHD HDR" (RTK) ]
load_panel GSM-LG-TV-SSCR2-01010101: 4.830290219s
LG TV SSCR2: Err(UiError { kind: Transport, message: "transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)" })
load_panel RTK: 756.917794ms
RTK panel codes: [10, 12, 62, 60, 14, D6]
ok
test rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored ... list_monitors: 1.218785784s
load_panel: 513.08197ms
write 0x10: 145.660721ms
brightness 100 -> 90 (max 100): read back Ok(ReadBackDto { current: 90, max: 100 })
restore 0x10: 145.81669ms
restored brightness: Ok(ReadBackDto { current: 100, max: 100 }); fresh read: Ok(FeatureReading { ... value: VcpValue { current: 100, max: 100 }, declared_in_capabilities: true })
ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.84s
$ ddcutil --bus 5 getvcp 10
VCP code 0x10 (Brightness                    ): current value =   100, max value =   100
```

- A LG TV, listada em 1º, agora falha o painel com `Transport`. A mensagem é **idêntica** à do `contract-mute.json`, então o golden e a demo reproduzem o hardware literalmente.
- O RTK carrega os 6 controles. O brilho fica em 100 → 90 → 100, e o `ddcutil` mostra 100 antes e depois. Nada Dangerous foi escrito.

<details><summary>Saída completa do teste de hardware (sem as 2 linhas de aviso future-incompat do <code>nom</code>)</summary>

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running tests/rtk_qhd_hdr.rs (target/debug/deps/rtk_qhd_hdr-e8ebe46ec3c2a18e)

running 2 tests
test rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads ... list_monitors: 1.139824288s
monitors: [
    MonitorDto {
        id: "GSM-LG-TV-SSCR2-01010101",
        label: "LG TV SSCR2",
        manufacturer: Some(
            "GSM",
        ),
        model: Some(
            "LG TV SSCR2",
        ),
    },
    MonitorDto {
        id: "RTK-RTK-QHD-HDR-01010101",
        label: "RTK QHD HDR",
        manufacturer: Some(
            "RTK",
        ),
        model: Some(
            "RTK QHD HDR",
        ),
    },
]
load_panel GSM-LG-TV-SSCR2-01010101: 4.830290219s
LG TV SSCR2: Err(UiError { kind: Transport, message: "transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)" })
load_panel RTK: 756.917794ms
RTK panel codes: [10, 12, 62, 60, 14, D6]
ok
test rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored ... list_monitors: 1.218785784s
monitors: [
    MonitorDto {
        id: "GSM-LG-TV-SSCR2-01010101",
        label: "LG TV SSCR2",
        manufacturer: Some(
            "GSM",
        ),
        model: Some(
            "LG TV SSCR2",
        ),
    },
    MonitorDto {
        id: "RTK-RTK-QHD-HDR-01010101",
        label: "RTK QHD HDR",
        manufacturer: Some(
            "RTK",
        ),
        model: Some(
            "RTK QHD HDR",
        ),
    },
]
load_panel: 513.08197ms
panel: PanelDto {
    monitor_id: "RTK-RTK-QHD-HDR-01010101",
    controls: [
        ControlDto {
            code: 16,
            key: Some(
                "brightness",
            ),
            dangerous: false,
            value: Continuous {
                current: 100,
                max: 100,
            },
        },
        ControlDto {
            code: 18,
            key: Some(
                "contrast",
            ),
            dangerous: false,
            value: Continuous {
                current: 50,
                max: 100,
            },
        },
        ControlDto {
            code: 98,
            key: Some(
                "volume",
            ),
            dangerous: false,
            value: Continuous {
                current: 30,
                max: 100,
            },
        },
        ControlDto {
            code: 96,
            key: Some(
                "input",
            ),
            dangerous: true,
            value: NonContinuous {
                current: 15,
                options: [
                    OptionDto {
                        value: 1,
                        name: Some(
                            "VGA-1",
                        ),
                    },
                    OptionDto {
                        value: 3,
                        name: Some(
                            "DVI-1",
                        ),
                    },
                    OptionDto {
                        value: 4,
                        name: Some(
                            "DVI-2",
                        ),
                    },
                    OptionDto {
                        value: 15,
                        name: Some(
                            "DisplayPort-1",
                        ),
                    },
                    OptionDto {
                        value: 16,
                        name: Some(
                            "DisplayPort-2",
                        ),
                    },
                    OptionDto {
                        value: 17,
                        name: Some(
                            "HDMI-1",
                        ),
                    },
                    OptionDto {
                        value: 18,
                        name: Some(
                            "HDMI-2",
                        ),
                    },
                ],
            },
        },
        ControlDto {
            code: 20,
            key: Some(
                "preset",
            ),
            dangerous: false,
            value: NonContinuous {
                current: 1,
                options: [
                    OptionDto {
                        value: 1,
                        name: Some(
                            "sRGB",
                        ),
                    },
                    OptionDto {
                        value: 2,
                        name: Some(
                            "Display Native",
                        ),
                    },
                    OptionDto {
                        value: 4,
                        name: Some(
                            "5000 K",
                        ),
                    },
                    OptionDto {
                        value: 5,
                        name: Some(
                            "6500 K",
                        ),
                    },
                    OptionDto {
                        value: 6,
                        name: Some(
                            "7500 K",
                        ),
                    },
                    OptionDto {
                        value: 8,
                        name: Some(
                            "9300 K",
                        ),
                    },
                    OptionDto {
                        value: 11,
                        name: Some(
                            "User 1",
                        ),
                    },
                ],
            },
        },
        ControlDto {
            code: 214,
            key: Some(
                "power",
            ),
            dangerous: true,
            value: NonContinuous {
                current: 1,
                options: [
                    OptionDto {
                        value: 1,
                        name: Some(
                            "On",
                        ),
                    },
                    OptionDto {
                        value: 4,
                        name: Some(
                            "Off (DPM)",
                        ),
                    },
                    OptionDto {
                        value: 5,
                        name: Some(
                            "Off (write-only)",
                        ),
                    },
                ],
            },
        },
    ],
}
write 0x10: 145.660721ms
brightness 100 -> 90 (max 100): read back Ok(ReadBackDto { current: 90, max: 100 })
restore 0x10: 145.81669ms
restored brightness: Ok(ReadBackDto { current: 100, max: 100 }); fresh read: Ok(FeatureReading { feature: Feature { code: VcpCode(16), kind: Continuous, access: ReadWrite, risk: Safe, allowed_values: None }, value: VcpValue { current: 100, max: 100 }, declared_in_capabilities: true })
ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.84s
```
</details>

### W-1..W-4
- **W-1 (`cargo audit`):** nada aqui. O `audit.toml`, com cada ignore justificado, fica para a phase `ci-crossbuild`. Os itens são RUSTSEC-2024-0429 `glib 0.18.5` e RUSTSEC-2024-0370 `proc-macro-error`, via Tauri/gtk-rs, e RUSTSEC-2018-0005 `serde_yaml 0.7.5` e RUSTSEC-2024-0320 `yaml-rust`, via `ddc-hi` → `mccs-db`.
- **W-2 / W-3:** nada a fazer. O orquestrador já corrigiu o Verify de capabilities (`9bc74c0`), e o W-3 era só registro.
- **W-4:** feito (`f298cee`). A cobertura de `lib.rs` foi de 0% para 17.78% de linhas. O resto é cola do Tauri (`run`, eventos de janela), prevista no R-3.

### Verify do CONTEXT.md (todos, menos o smoke SNI, que é do reviewer)
| # | Critério | Resultado |
|---|---|---|
| 1 | fmt + clippy `-D warnings` | `OK` (exit 0) |
| 2 | build release `ddc-tray` | `OK` |
| 3 | só o composition root constrói `DdcHiMonitorBackend` | `OK` |
| 4 | `panel.rs` puro sobre `MonitorControl` (≥5 `pub fn …<M: MonitorControl + ?Sized>`) | `OK` |
| 5 | `#![forbid(unsafe_code)]` em lib/main, sem `allow(unsafe_code)`/`unsafe {` | `OK` |
| 6 | `node --test` por módulo + total ≥20, 0 falhas, nada em `src/` | `OK` (84 pass, 0 fail) |
| 7 | i18n paridade/HTML | `OK` |
| 8 | CSP | `OK` |
| 9 | capabilities | `OK` |
| 10 | single-instance como 1º `.plugin(` | `OK` |
| 11 | smoke SNI | não rodado (é do reviewer) |
| 12 | teste de hardware existe, compila, gated, sem `confirmed: true`/`Confirm::Yes` | `OK` (lista os 2 testes `rtk_qhd_hdr*`) |
| 13 | Gate 7 Playwright (exit 0 + 10 `critical_paths` ✓) | `OK`: `36 passed`, `4 skipped` |
| 14 | TODO/FIXME em arquivos não-Rust do tray | `OK` |
| 15 | screenshots claro/escuro | `OK` |
| baseline | TODO/FIXME em `*.rs` sem issue | `OK` |

### Gates (números finais)
| Gate | Comando | Resultado |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| testes Rust | `cargo test --workspace --locked` | **344 passed**, 0 failed, 9 ignored (hardware). Antes: 339 + 8. Entraram +5: 2 do B-1, 1 do golden mudo e 2 do W-4, mais 1 ignored de hardware. |
| cross-check | `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` e `--target x86_64-unknown-linux-gnu` | exit 0 / exit 0 |
| JS | `node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/**/*.test.mjs'` | `# tests 84`, `# pass 84`, `# fail 0` |
| Playwright | `npx playwright test --reporter=list` | `36 passed`, `4 skipped` (screenshots sem `SCREENSHOTS=1`) |
| cobertura | `cargo llvm-cov --workspace --locked --summary-only --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'` | TOTAL lines **88.79%** (2747 linhas, 308 sem cobertura). `panel.rs` 99.08%, `lib.rs` 17.78%, `commands.rs` 52.14%, `tray.rs` 62.65%. |

### Desvios e observações
- **Arquivos fora do `files_modified` original:** todos foram pedidos pelo orquestrador nesta iteração.
  - `dto.rs`: só o `//!`, que agora cita os 2 goldens.
  - `tests/ui/bridge-demo.test.mjs`: o teste da TV muda passou a esperar `transport`.
  - Arquivo novo: `tests/fixtures/contract-mute.json`.
  - Docs: README, CHANGELOG e `docs/hardware-validation.md`.
- **Nome do teste de hardware:** `rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads`, em vez do exemplo `…_listed_before_the_rtk_…`. O teste não exige a ordem de enumeração, que o backend não garante; só imprime a ordem. Nesta máquina a TV vem em 1º.
- **Custo de tentar um monitor mudo:** o `load_panel` da TV leva **4.83 s**, e antes da correção levava o mesmo, já que as 6 leituras falhavam do mesmo jeito. Na 1ª abertura, sem monitor lembrado, o popup fica ~5 s em "carregando" antes de cair no RTK. Nas seguintes, o RTK lembrado é tentado primeiro. Isso está documentado no README e no passo 8 do roteiro.
  - Uma otimização possível seria desistir após a 1ª falha de transporte. Ela não foi feita, porque mudaria a semântica de falha parcial da D-4. Fica como sugestão para uma phase futura.
- **Atalhos de brilho antes de o popup carregar um painel:** `shortcut_target` cai no 1º monitor listado, a TV nesta máquina, e o atalho falha só no stderr. O comportamento é anterior à phase e o README já o descreve ("before that, the first monitor listed"). Não mexi.
  - O `app.js` chama `refresh()` ao carregar (linha 132), então o webview oculto provavelmente seleciona o RTK logo após o start. Não verifiquei isso no app real, porque a instrução foi não abrir o app.
- **Validação manual sugerida pela review:** o popup abrir no RTK com a TV marcada "(no DDC/CI)" não foi feita à mão, pela mesma instrução. Fica para o humano/PR. O passo 8 de `docs/hardware-validation.md` descreve o que conferir.
- **Commit de docs:** `2b8b67f` recebeu um `--amend` local, antes de qualquer push, para incluir uma frase do dev setup do README ("of the tray's two [hardware tests]…").

## Iteração 3 — feedback do usuário + DoD endurecido

Modo fix, cadeia autônoma do `/jdi-issue`. O usuário testou a versão instalada e reportou: (1) "sempre que eu seleciono um select list o painel fecha"; (2) "valide se é possível deixar o painel direto no tray seria melhor". O orquestrador travou D-2026-09-27-tray-app-1..4 e reescreveu o DoD (`896daf7`). O CONTEXT.md não foi tocado. As 8 tasks do PLAN continuam `completed`; o trabalho foi A–E abaixo, mais o W do DoD critic (reuso do servidor do Playwright).

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `8d26af0` | test | B: `reuseExistingServer: false` no `playwright.config.mjs`. Provado: com um `http.server` antigo na 1420 a suíte agora falha (`Process from config.webServer was not able to start. Exit code: 1`) em vez de validar outra UI. |
| `480d816` | fix | A: dropdown in-page (D-2026-09-27-tray-app-1) no seletor de monitor, na predefinição de cor e nas NC de "Todos os ajustes"; nenhum `<select>` em `src/`. |
| `144966a` | feat | C: StatusNotifierItem próprio no Linux via `ksni` 0.3.6 (D-2026-09-27-tray-app-2): clique esquerdo, roda, menu, tooltip; `scroll.rs`; `DDC_TRAY_DEBUG`; `smoke-sni.sh --activate`. O corpo recebeu um `--amend` local, antes de qualquer push, para corrigir a contagem de testes (18 → 19). |
| `bf76e7b` | feat | D: ancoragem no KDE Wayland por script do KWin carregado via D-Bus (D-2026-09-27-tray-app-3). Funcionou; entrou. |
| `50c9d68` | docs | E: README, CHANGELOG `[Unreleased]`, `docs/hardware-validation.md` (passos 6 e 8). |

### A — dropdown in-page
- `src/dropdown.js`: redutor puro `dropdownReduce(state, event)` (padrão WAI-ARIA select-only combobox) + `placeList` puro + ligação ao DOM `createDropdown`. Enter/Espaço/↑/↓/Alt+↓ abrem; ↑/↓/Home/End/PageUp/PageDown navegam, pulando opções desabilitadas; Enter/Espaço/Alt+↑ escolhem; Esc fecha sem mudar, com `preventDefault` (o popup não se esconde; o 2º Esc esconde); Tab e clique fora fecham sem mudar; foco volta ao botão (`div role="combobox"` com `aria-haspopup="listbox"`, `aria-expanded`, `aria-controls`, `aria-activedescendant`; lista `role="listbox"` com `role="option"` e `aria-selected` no valor atual).
- A lista é `position: fixed`, medida no tamanho natural e colocada abaixo do botão, ou acima quando há mais espaço, sempre 8 px dentro dos limites do `.app` (o popup de 360×560), com rolagem interna. Só usa as custom properties `--dropdown-*` (CSP intacta; o `i18n-html.test.mjs` passou a cobrir o `dropdown.js`).
- Achado durante os testes: o `scroll` que o Playwright dispara ao rolar o botão para a vista chega um frame depois e fechava a lista recém-aberta. Troquei "fecha em qualquer scroll" por "acompanha o botão e só fecha quando ele sai da área visível" (teste e2e próprio).
- Bridge demo ganhou `__ddcDemo.hides` (contador de `hide_popup`), usado para provar que o 1º Esc não esconde o popup.
- Testes: `tests/ui/dropdown.test.mjs` (28, redutor e posicionamento); `tests/e2e/dropdown.spec.mjs` (8 por tema, 16 ✓): escolher outra predefinição → exatamente 1 escrita `0x14` e o valor lido de volta na UI; monitor que mantém a predefinição → a UI mostra o valor lido de volta e anuncia; Esc → 0 escritas e `hides = 0`, 2º Esc → `hides = 1`; teclado (Alt+↓, End, Enter) → 1 escrita; axe sem critical/serious com a lista aberta + clique fora → 0 escritas; lista perto do fim abre para cima dentro da janela; NC perigosa (`0xCA`) pede confirmação e Cancelar restaura; scroll acompanha e fecha. `fallback.spec`/`scenarios.spec` migrados para o combobox.
- Screenshots regenerados e conferidos (Read): claro, escuro, diálogo, e um novo `tray-popup-list-light.png` (lista aberta). A regeneração é determinística (3 execuções, SHA-1 iguais).

### C — bandeja Linux via `ksni`
- `Cargo.toml`: `tray-icon` e `tauri-plugin-positioner` só em `cfg(not(target_os = "linux"))`; `ksni` 0.3.6 (feature `tokio`, o runtime do Tauri) só no Linux. O `Cargo.lock` ganhou três crates: `ksni` 0.3.6, `pastey` 0.2.3 e `tokio-macros` 2.7.2 (este vem da feature `macros` do `tokio`, puxada pelo `ksni` com `tokio`; correção da iteração 4, W-4 — a versão anterior desta frase omitia o `tokio-macros`). zbus 5.19 (já na árvore via single-instance) resolve tokio/async-io em tempo de execução (`use_tokio()` checa `Handle::try_current()`), então o single-instance continua igual.
- `tray.rs` virou a parte comum (toggle, ações do menu, atalhos); `tray/status_item.rs` (Linux) e `tray/notification_area.rs` (Windows, código antigo). Callbacks do ksni: `activate` → `run_on_main_thread(toggle_popup)` (mesmo gate blur↔clique); `scroll` (vertical) → `WheelQueue::push`; quando pede `Start`, um writer roda em `spawn_blocking` (`on_blocking_thread`) e emite `panel-changed` a cada escrita; menu = `menu_entries(locale, Linux)`; `assume_sni_available(true)` (espera um host que suba depois, como no login).
- `scroll.rs` (Rust puro sobre `MonitorControl`): notch = 120 (o `angleDelta.y` que o Plasma envia, conferido nas strings do `org.kde.plasma.systemtray.so`: `Plasmoid.scroll(..., wheel.angleDelta.y, "vertical")`), ±5% por notch, 0–100%, sem escrita no limite, resto de notch acumulado e zerado ao inverter o sentido; coalescência: um writer por vez, as notches que chegam durante a escrita viram UMA escrita seguinte. 19 testes com `scroll` no nome (limites, `i32::MAX/MIN`, rajada → 1 escrita, notches durante a escrita → 1 escrita seguinte, sem monitor/timeout → nada escrito e o writer para).
- Tooltip: título "DDC Control" + "‹monitor› · Role para mudar o brilho" (o `AppState` guarda os rótulos do último `list_monitors`; `select_monitor` pede ao host para reler o tooltip).
- Diagnóstico `DDC_TRAY_DEBUG=1` (só `1` liga; testado): `tray item started`, `tray activated`, `popup shown`, `popup focused`, `popup hidden`, `popup placement …`.
- **Evidências (KDE Plasma 6.7.5 Wayland):**
  - D-Bus do item: `Id "ddc-control"`, `Title "DDC Control"`, `Category "Hardware"`, `ItemIsMenu false`, `ToolTip "DDC Control" / "RTK QHD HDR · Role para mudar o brilho"`; menu (`GetLayout`): Abrir painel | Brilho 0/25/50/75/100% | Sair.
  - `Activate` → `tray activated`, `popup shown`, `popup focused` (o KWin deu foco mesmo com a ativação vindo do D-Bus); 2º `Activate` → `popup hidden`.
  - AppIndicator fora: `cargo tree -p ddc-tray -e normal` sem `tray-icon`/`libappindicator`/`positioner`; `strings` do binário: 0 ocorrências de `libayatana-appindicator3.so.1` (antes: 1); `ldd … | grep -i appindicator`: vazio (antes também: a lib era carregada por `dlopen`, por isso conferi também o `/proc/<pid>/maps` do app rodando, com 0 linhas `appindicator|ayatana`).
  - Provas negativas do `smoke-sni.sh --activate`: binário da iter 2 (AppIndicator) → `Call failed: Método inexistente "Activate"` → `FAIL: … did not answer org.kde.StatusNotifierItem.Activate`, exit 1; mutante com `activate()` sem toggle → `FAIL: no 'ddc-tray: popup shown' on stderr within 5 s of Activate`, exit 1. O smoke sem `--activate` segue passando.
  - A roda NÃO foi acionada no monitor real (só testes com o fake); nenhuma escrita durante smoke/sessões.
- Windows: o check `--target x86_64-pc-windows-msvc` do `ddc-tray` continua parando no `tauri-winres` (D-9). Para não commitar o módulo Windows às cegas, compilei-o no Linux num experimento descartável (tray-icon ligado e `mod notification_area` sem `cfg`): compila e seus 3 testes passam; árvore restaurada com `git checkout`.

### D — ancoragem no KDE Wayland: FUNCIONOU (entrou)
- Tentativa (a) da D-3, ~30 min: `org.kde.kwin.Scripting.loadScript(caminho, "ddc-tray-anchor")` + `/Scripting/Script<id> org.kde.kwin.Script.run`. O script (`src-tauri/kwin/anchor.js`) escuta `workspace.windowAdded` — no Wayland o GTK desmapeia a janela ao esconder, então cada exibição é um `windowAdded` — e, só para a janela com `pid` do app + `resourceClass "ddc-tray"` + `caption "DDC Control"` (conferidos ao vivo), centraliza no ponteiro e limita 8 px dentro de `clientArea(MaximizeArea, screenAt(ponteiro))`. Também aplica o que o `tauri.conf.json` pede e o Wayland ignora (sem entrada na barra de tarefas/alternador, acima das outras) — conferido ao vivo que o KWin ignorava `skipTaskbar`/`alwaysOnTop`.
- `print()` do script não chega ao journal (categoria de debug do KWin desligada). Prova objetiva por outro canal: um script-sonda descartável chama `callDBus` para um nome inexistente e eu leio os argumentos com `dbus-monitor`.
  - Sem o script: popup em `{"x":1356,"y":601}` (centro do HDMI-A-2 3072×1728).
  - Com o script, ponteiro em `{"x":2338,"y":471}`: `{"x":2158,"y":191,"width":360,"height":560}` (= ponteiro − 180/280), `skipTaskbar=true keepAbove=true active=true`; escondido → a janela some da lista; mostrado de novo → reposicionado igual.
  - Ciclo de vida no app real: `popup placement loaded into KWin as script 1`, `isScriptLoaded ddc-tray-anchor = true`, arquivo em `$XDG_RUNTIME_DIR/ddc-tray-kwin-anchor.js` com o PID; ao sair por "Sair" (`dbusmenu.Event`): `popup placement unloaded from KWin`, `isScriptLoaded = false`, arquivo removido.
- Não consegui mover o ponteiro para o ícone (sem injeção de entrada no Wayland, e não instalo pacotes), então o caso "clique no ícone do painel do topo" foi provado pela lógica: `tests/ui/kwin-anchor.test.mjs` roda o `anchor.js` num `vm` com um KWin falso (9 testes: painel do topo real do dev, área y=34 → popup em y=42; painel embaixo; cantos; tela fora da origem; arredondamento; só a janela do próprio PID/classe/título; flags). 4 mutações do `anchor.js` (margem 0, sem checar PID, `windowActivated`, sem centralizar em y) foram pegas e revertidas. Rust: `applies()` (só `XDG_SESSION_TYPE=wayland` + `XDG_CURRENT_DESKTOP` com `KDE`), substituição do PID, e título/classe presos ao `tauri.conf.json`/nome do pacote.
- Fallback silencioso: fora do KDE Wayland nada é carregado; falha de D-Bus/KWin vira só linha de diagnóstico. XWayland não foi usado; layer-shell ficou documentado como alternativa não feita.
- Validação visual humana (clique real no ícone) segue em "Deferred to PR review".

### Verify do CONTEXT.md (todos, rodados literalmente, `bash` a partir da raiz)
| # | Critério | Resultado |
|---|---|---|
| 1 | fmt + clippy `-D warnings` | `OK` |
| 2 | build release `ddc-tray` | `OK` |
| 3 | só o composition root constrói `DdcHiMonitorBackend` | `OK` |
| 4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| 5 | `#![forbid(unsafe_code)]` | `OK` |
| 6 | `node --test` por módulo (inclui `dropdown`) + total ≥20, 0 fail, 0 cancelled | `OK` (124 pass) |
| 7 | i18n paridade/HTML | `OK` |
| 8 | CSP (base + por plataforma) | `OK` |
| 9 | capabilities | `OK` |
| 10 | single-instance como 1ª chamada do builder | `OK` |
| 11 | smoke `--activate` (PID próprio, vivo, popup mostrado após `Activate`, sem `panicked`) | `OK` (instância do usuário encerrada antes e reaberta depois, conforme o protocolo do orquestrador) |
| 12 | teste de hardware `#[ignore]` gated | `OK` |
| 13 | Gate 7 Playwright (`reuseExistingServer: false`, 10 `critical_paths`, ≥4 ✓ do dropdown) | `OK` (52 passed, 6 skipped; 16 ✓ no `dropdown.spec`) |
| 14 | nenhum `<select>` nativo em `src/` | `OK` |
| 15 | `ksni` no Cargo.toml + ≥3 testes `scroll` | `OK` (`test result: ok. 19 passed`) |
| 16 | TODO/FIXME em arquivos não-Rust do tray | `OK` |
| 17 | screenshots regenerados byte a byte iguais | `OK` |
| PROJECT | `cargo test --workspace --locked` | `OK` |
| PROJECT | TODO/FIXME em `*.rs` (Verify novo da D-4) | `OK` |
| PROJECT | cobertura ≥80% de linhas | 83.54% (literal) / 83.98% (forma do gate) |

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked` | **371 passed**, 0 failed, 9 ignored (iter 2: 344) |
| cross-check `ddc-core ddc-adapters ddc-cli` em `x86_64-unknown-linux-gnu` e `x86_64-pc-windows-msvc` | exit 0 / exit 0 |
| `node --test` (todos) | 124 pass, 0 fail, 0 cancelled (iter 2: 84) |
| Playwright | 52 passed, 6 skipped (iter 2: 36 + 4) |
| cobertura (gate) | TOTAL lines **83.98%** (iter 2: 88.79%). `scroll.rs` 100%, `popup.rs` 100%, `panel.rs` 99.08%; cola: `tray.rs` 59.64%, `commands.rs` 54.60%, `kwin_placement.rs` 36.72%, `lib.rs` 27.64%, `status_item.rs` 24.31% |

### Desvios e observações
- **Arquivos novos fora do PLAN original** (todos pedidos nesta iteração): `src/dropdown.js`, `src-tauri/src/scroll.rs` (+`scroll/tests.rs`), `src-tauri/src/tray/{status_item,notification_area,kwin_placement}.rs`, `src-tauri/kwin/anchor.js`, `tests/ui/{dropdown,kwin-anchor}.test.mjs`, `tests/e2e/dropdown.spec.mjs`, `docs/screenshots/tray-popup-list-light.png`.
- **Contrato:** `select_monitor` passou a receber `AppHandle` (para atualizar o tooltip) e `list_monitors` guarda os rótulos no `AppState`; os argumentos vistos pela UI e os goldens não mudaram.
- **Dependência:** `zbus` virou dependência direta no Linux (para o `loadScript`), sem crate novo no lock (a D-3 pedia "sem pacote novo": nenhum pacote de sistema nem crate novo além dos três da D-2: `ksni`, `pastey` e `tokio-macros` — W-4, corrigido na iteração 4).
- **Cobertura caiu de 88.79% para 83.98%**: a cola nova (callbacks do ksni, D-Bus do KWin) só roda com sessão gráfica; toda a lógica foi extraída e está em 100% (`scroll.rs`) ou testada em JS (`anchor.js`).
- **GNOME:** a roda segue a convenção do Plasma (120 por notch, positivo = para longe do usuário); a extensão AppIndicator do GNOME manda outras unidades, então lá a roda pode não andar ou andar ao contrário. Não testado; documentado nas Known limitations, sem código especulativo.
- **SIGTERM** (o smoke encerra assim) deixa o script do KWin carregado; ele só casa com o PID morto e o próximo start o substitui (o `docs/hardware-validation.md` mostra como descarregar à mão). Depois dos meus smokes descarreguei-o (`unloadScript` → `b true`).
- Ao esconder pelo ícone aparecem duas linhas `popup hidden` (o hide gera um `Focused(false)`, que esconde de novo uma janela já escondida). Inofensivo; mantido.
- **Instância do usuário:** encerrada (`pkill -x ddc-tray`) antes de cada smoke/sessão e reaberta logo depois com `setsid -f /home/slipalison/.local/bin/ddc-tray`, conforme o protocolo do orquestrador; nada foi instalado nem alterado em `~/.local`. Popups mostrados na tela: só os necessários (smokes `--activate`, sessão de toggle, 2 sessões da ancoragem).

## Iteração 4 — DoD critic + warnings

Modo fix, cadeia autônoma do `/jdi-issue`, iter 4 do loop. A iter 3 foi aprovada pelo reviewer (APPROVED_PENDING_MANUAL, W-1..W-4), mas o DoD critic achou 8 linhas ocas. O orquestrador reescreveu o CONTEXT.md e travou a D-2026-09-27-tray-app-5 (`9fde585`). Não editei o CONTEXT. As 8 tasks do PLAN continuam `completed`. O trabalho foi: D-5 (monitor simulado + smoke da roda), Gate 7 em runtime, W-2, W-3, W-4 e a observação de acoplamento. W-1 (`cargo audit`) segue com a `ci-crossbuild`, como a review recomendou.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `79f5a6a` | refactor | D-5: fixture do RTK sai de `panel/tests.rs` para o módulo de produção `src-tauri/src/fixture.rs` (`RTK_ID`, `RTK_CAPS`, `COLOR_TEMP`, `rtk_id`, `rtk_info`, `rtk_monitor`); todos os testes importam de lá, sem cópia. Goldens intocados (`git status` limpo em `tests/fixtures`). |
| `bb4b1ca` | refactor | Item 6 (observação do reviewer): `shortcut_target` sai de `commands.rs` (que importa `tauri`) para `panel.rs`; `scroll.rs` e `tray.rs` não dependem mais do módulo de comandos. Os 3 testes foram junto, e as 3 cópias do monitor "DEL-U2720Q-7" dos testes viraram um helper `dell_monitor(current, max)` em `panel/tests.rs`. |
| `b943ca5` | feat | D-5: `DDC_TRAY_FAKE=1` (só `1`) → `compose_osd()` monta `SoftwareOsd` sobre `InMemoryMonitorBackend` com `fixture::rtk_monitor()` e SEMPRE avisa no stderr (`ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched`); nunca por padrão; `DdcHiMonitorBackend::new` continua único, em `lib.rs`. `BrightnessChange {monitor_id, before, after}` (em `panel.rs`, puro) é devolvido pelo atalho e pela roda; com `DDC_TRAY_DEBUG=1` cada escrita de brilho da bandeja imprime `ddc-tray: brightness <antes> -> <depois>` (valor lido antes → valor lido de volta). |
| `2686c09` | test | D-5: `smoke-sni.sh --fake` / `--scroll` e `--activate` com "continua mostrado". Detalhes abaixo. Teste Rust prende o texto que o script espera ao `SIMULATION_NOTICE` do app. |
| `e49de89` | test | Gate 7: `expectNoNativeSelect(page)` em `tests/e2e/support.mjs` (`page.locator('select')` com contagem 0) roda no `open()` (todo estado assentado), antes de cada `expectAccessible()` e no teardown do fixture `page` (fim de todo teste). Também renomeei a chave `only` do teste de fallback do i18n: o Verify novo do `node --test` a lia como teste focado e falhava no HEAD `9fde585`. |
| `d2fedba` | fix | W-3: `script_path(runtime_dir)` → `None` sem `XDG_RUNTIME_DIR` (ou com caminho vazio/relativo); o script não é carregado e o popup abre onde o compositor quiser. Sem fallback para `/tmp`. 2 testes puros. |
| `c2ff8aa` | fix | W-2: SIGTERM/SIGINT/SIGHUP → mesmo `app.exit(0)` do "Sair" (`stop_signals.rs`, `tokio::signal::unix` no runtime do Tauri); um 2º sinal durante a saída sai na hora (exit 1). Descarregar o script espera no máximo 2 s pelo D-Bus (`tokio::time::timeout` em conexão + `unloadScript`). Só descarrega o que o app carregou (`LoadedScript` no estado do app), e não chama mais `org.kde.KWin` fora do KDE Wayland. A remoção do arquivo virou `remove_script` (NotFound conta como removido), com falha reportada no stderr e comentário do porquê. Um load recusado pelo KWin não deixa o arquivo para trás. |
| `e24c130` | docs | README, CHANGELOG e `docs/hardware-validation.md` (passo 6): ciclo de vida do script (Sair/sinais, 2 s, só em `XDG_RUNTIME_DIR`), `DDC_TRAY_FAKE=1`, linha de brilho, flags do smoke, checagem de `<select>` em runtime. |
| `83b4c59` | test | Achado ao rodar o Verify novo dos screenshots: ele exige `SCREENSHOTS=1` sem nada pulado, mas o spec pulava o diálogo e a lista no tema escuro (`2 skipped` → Verify falhava). Agora todo screenshot sai nos dois temas: 2 PNGs novos (`tray-popup-dialog-dark.png`, `tray-popup-list-dark.png`, conferidos com Read), os 4 antigos idênticos byte a byte (3 regenerações, SHA-1 iguais). README cita os novos. |

### D-5 — monitor simulado e smoke da roda
- `smoke-sni.sh [--fake] [--activate] [--scroll] <bin>`, opções em qualquer ordem:
  - `--fake` sobe com `DDC_TRAY_FAKE=1` e exige no stderr o aviso do monitor simulado antes de qualquer outra coisa.
  - `--scroll` só roda com `--fake` (senão `FAIL: --scroll writes the brightness, so it only runs with --fake`, exit 1 — provado). Chama `org.kde.StatusNotifierItem.Scroll` no item do PRÓPRIO PID: `120 Vertical` → exige `ddc-tray: brightness 75 -> 80` em 5 s; `120 Horizontal` → exige NENHUMA linha `ddc-tray: brightness ` em 1,5 s; `-120 Vertical` → exige `brightness 80 -> 75` (prova que o app seguia vivo e ouvindo durante a janela do horizontal, e o sinal da roda).
  - `--activate` agora exige, além do `popup shown` em 5 s, nenhum `popup hidden` nos 1,5 s seguintes.
  - Achado: `busctl` lia `-120` como opção; o script passa `--` antes do verbo.
- Saída do `--fake --scroll` (binário release limpo):
  ```text
  smoke-sni: the app serves the simulated monitor
  smoke-sni: the app printed 'ddc-tray: brightness 75 -> 80' after a vertical Scroll of +120
  smoke-sni: a horizontal Scroll wrote nothing within 1.5 s
  smoke-sni: the app printed 'ddc-tray: brightness 80 -> 75' after a vertical Scroll of -120
  smoke-sni: OK — PID 796142 registered its tray item, was alive 2 s later, stepped the brightness on a vertical Scroll only and never panicked
  ```

### Provas negativas (mutações NÃO commitadas, binários em scratchpad, árvore restaurada e release reconstruído limpo depois)
| Mutação | Comando | Resultado |
|---|---|---|
| `orientation == Orientation::Vertical` → `Horizontal` em `status_item.rs` | `smoke-sni.sh --fake --scroll` | `FAIL: no 'ddc-tray: brightness 75 -> 80' on stderr within 5 s of a vertical Scroll of +120`, exit 1 |
| roda sem checar a orientação (bônus) | `smoke-sni.sh --fake --scroll` | `FAIL: a horizontal Scroll wrote the brightness: 'ddc-tray: brightness 80 -> 85'`, exit 1 |
| popup que se esconde 0,8 s depois de mostrar (`show_popup` agenda um `hide`) | `smoke-sni.sh --activate` | `the app printed 'ddc-tray: popup shown' after Activate` → `FAIL: the popup was hidden within 1.5 s of 'ddc-tray: popup shown' ('ddc-tray: popup hidden')`, exit 1 |
| `<select>` criado em runtime (`document.createElement(['sel','ect'].join(''))` no fim do `app.js`), invisível ao grep estático | `npx playwright test` | grep estático do Verify: continua `OK` (não vê); Playwright: **52 failed**, 6 skipped, todos com `Error: native <select> elements on the page` / `locator('select')` |
| `script_path` volta a cair em `temp_dir()` | `cargo test -p ddc-tray --lib runtime_directory` | `without_a_runtime_directory_no_script_is_written_anywhere ... FAILED` |

### W-2 — prova ao vivo com a MINHA instância (monitor simulado, nunca a do usuário)
- Instância do usuário encerrada com `pkill -x ddc-tray` antes; em cada rodada só a minha estava viva (`pgrep -xa ddc-tray` → `…/target/release/ddc-tray`).
- `pkill -TERM -x ddc-tray` (SIGTERM), `kill -INT`, `kill -HUP`:
  | Sinal | Antes | Saída | Depois | stderr |
  |---|---|---|---|---|
  | SIGTERM (`pkill -TERM`) | `isScriptLoaded` `b true`, arquivo com `POPUP_PID = 775623` | exit 0 em 52 ms | `isScriptLoaded` **`b false`**, arquivo removido | `SIGTERM received, quitting` / `popup placement unloaded from KWin` |
  | SIGINT | `b true` (`POPUP_PID = 775766`) | exit 0 em 23 ms | **`b false`**, arquivo removido | `SIGINT received, quitting` / `popup placement unloaded from KWin` |
  | SIGHUP | `b true` (`POPUP_PID = 775904`) | exit 0 em 23 ms | **`b false`**, arquivo removido | `SIGHUP received, quitting` / `popup placement unloaded from KWin` |
- "Sair" pelo menu (`com.canonical.dbusmenu.Event` no id 9 = "Sair") continua igual: `b true` → `b false`, arquivo removido, `popup placement unloaded from KWin`.
- Timeout: mutante com `unloadScript` que nunca responde (`std::future::pending()` antes da chamada) + SIGTERM → exit 0 em **2026 ms**, `ddc-tray: could not unload the popup placement from KWin: no answer within 2s`, arquivo removido mesmo assim. O script órfão que o mutante deixou foi descarregado à mão (`unloadScript` → `b true`).
- 2º sinal: mutante com a saída travada 30 s + SIGTERM e outro SIGTERM 1 s depois → exit **1** em 1024 ms, `SIGTERM received, quitting` / `SIGTERM received while quitting, exiting at once`. Órfão descarregado e arquivo removido à mão.
- Efeito colateral bom: depois dos smokes do Verify final (que encerram com SIGTERM), `isScriptLoaded` = `b false` (na iter 3 ficava `b true` com o PID morto).
- Teste unitário `a_stop_signal_the_app_listens_to_is_heard_by_name`: registra SIGHUP e manda `kill -HUP` ao próprio processo de teste; 8 execuções seguidas da suíte do crate, 8 verdes.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, conferidos como substring exata, rodados com `bash` a partir da raiz)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` (agora 7 `pub fn …<M: MonitorControl + ?Sized>`) | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo + total, 0 fail/cancelled/skipped/todo, nenhum `skip:`/`todo:`/`only:` | `OK` (falhava no HEAD `9fde585` por causa da chave `only`) |
| C7 | i18n paridade/HTML | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` (PID próprio, popup mostrado e NÃO escondido em 1,5 s) | `OK` (`the popup was still shown 1.5 s later`) |
| C12 | teste de hardware `#[ignore]` gated, guard vivo | `OK` (não executado — regra) |
| C13 | Gate 7 (`reuseExistingServer: false`, 10 `critical_paths`, 16/16 dropdown, pulados = screenshots, `locator('select')` no helper) | `OK` (52 passed, 6 skipped) |
| C14 | nenhum `<select>` em `src/` | `OK` |
| C15 | `ksni` + ≥3 testes `scroll` + `smoke-sni.sh --fake --scroll` | `OK` (`test result: ok. 20 passed`; smoke `75 -> 80`, horizontal sem escrita, `80 -> 75`) |
| C16 | TODO/FIXME em todo arquivo versionado do produto | `OK` |
| C17 | screenshots regenerados sem nada pulado, byte a byte iguais | `OK` (6 passed; falhava no HEAD `9fde585` com `2 skipped`) |
| P1 | `cargo test --workspace --locked` | `OK` |
| P2 | cobertura ≥80% (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78% |
| P3 | TODO/FIXME em `*.rs` | `OK` |

Os smokes (C11, C15) rodaram com o protocolo do orquestrador: `pkill -x ddc-tray` (instância do usuário, PID 779685) → Verify → `setsid -f /home/slipalison/.local/bin/ddc-tray` (voltou como PID 796385, script do KWin recarregado com o PID dela).

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (iter 3: 371) |
| cross-check `ddc-core ddc-adapters ddc-cli` em `x86_64-unknown-linux-gnu` e `x86_64-pc-windows-msvc` | exit 0 / exit 0 (`crates/` não foi tocado nesta iteração) |
| `node --test` (todos) | 124 pass, 0 fail, 0 cancelled, 0 skipped, 0 todo |
| Playwright | 52 passed, 6 skipped (screenshots sem `SCREENSHOTS=1`); com `SCREENSHOTS=1`: 6 passed |
| cobertura (gate, `--fail-under-lines 80`, sem `main.rs`/`build.rs`) | exit 0, TOTAL lines **83.22%** (iter 3: 83.98%). Lógica: `scroll.rs` 100%, `popup.rs` 100%, `fixture.rs` 100%, `panel.rs` 99.31%. Cola: `stop_signals.rs` 62.07% (o `quit_on_stop_signals` só roda com o app), `tray.rs` 57.23%, `commands.rs` 49.32%, `kwin_placement.rs` 45.36%, `lib.rs` 37.75%, `status_item.rs` 23.03% |
| `Cargo.lock` nesta iteração | só a aresta `ddc-tray → tokio` (nenhum crate novo; as features `signal` e `time` já eram compiladas via `tauri`/`zbus`) |

### Desvios e observações
- **Arquivos fora do PLAN** (todos pedidos nesta iteração ou consequência direta): `src-tauri/src/fixture.rs`, `src-tauri/src/stop_signals.rs`, `docs/screenshots/tray-popup-{dialog,list}-dark.png`.
- **Contrato Rust interno:** `panel::set_brightness_percent` e `tray::brightness_shortcut` devolvem `BrightnessChange` (antes `ReadBackDto`/`PanelChangedDto`); entrou `panel::write_brightness`. Nada muda para a UI: o evento `panel-changed` e os goldens são os mesmos.
- **Além do pedido, pequeno:** o 2º sinal sai na hora (sem ele, depois que o app assume o SIGTERM, uma saída travada só morreria com SIGKILL); o `--scroll` tem um 3º passo (`-120` → `80 -> 75`); um load recusado pelo KWin apaga o arquivo; o helper de teste `dell_monitor` substituiu 3 cópias.
- **Dois Verify do CONTEXT novo falhavam no HEAD `9fde585` sem mudança de código de produto:** C6 (chave `only` num teste do i18n) e C17 (screenshots pulados no tema escuro). Corrigidos em `e49de89` e `83b4c59`.
- **Cobertura caiu 0,76 ponto** (83.98% → 83.22%). A cola nova de sinais e de D-Bus só roda com o app de pé (provada ao vivo acima); o que é lógica ficou em funções testadas (`script_path`, `remove_script`, `StopSignals`, `BrightnessChange`, `simulated_osd`).
- **Incidente de processo, sem efeito no resultado:** na prova negativa do W-3 usei `git checkout` no arquivo que ainda tinha a edição do W-3 não commitada; reapliquei a mesma edição antes do commit `d2fedba`. As mutações seguintes foram feitas só sobre arquivos já commitados.
- **Instância do usuário:** encerrada e reaberta 3 vezes (sessão das provas do smoke, sessão do W-2, Verify final), sempre com `pkill -x ddc-tray` e `setsid -f /home/slipalison/.local/bin/ddc-tray`; nada em `~/.local` foi tocado. Popups mostrados na tela: 2 `--activate` positivos (conferência e Verify final) e 1 do mutante que se esconde (~0,8 s). Os smokes da roda e as provas de sinais não abrem popup.
- **Monitor real:** nenhuma escrita. A roda e os atalhos só rodaram com `DDC_TRAY_FAKE=1`; o `--activate` com backend real só lê. O teste de hardware `rtk_qhd_hdr` não foi executado (fica com o orquestrador, como no DoD).

## Iteração 5 — lacunas do critic + W-2

Modo fix, cadeia autônoma do `/jdi-issue`, iter 5 do loop. A iter 4 foi aprovada pelo reviewer (APPROVED_PENDING_MANUAL, com W-1 herdado e W-2), mas o DoD critic demonstrou 2 lacunas de teste: debounce vs throttle e a configuração do axe sem trava. O orquestrador travou a D-2026-09-27-tray-app-6 e endureceu o CONTEXT (`0ec2d08`). Não editei o CONTEXT. As 8 tasks do PLAN continuam `completed`. W-1 (`cargo audit`) segue com a `ci-crossbuild`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `93dcd83` | test | Item 1: teste `a burst longer than the debounce window writes once, after it ends` em `tests/ui/debounce.test.mjs`, com `mock.timers`. São 20 pushes, um a cada 20 ms (380 ms de rajada, e o teste afirma que ela passa de 4× a janela). Depois de cada tick ele drena as promises (`setImmediate` real) e exige zero escritas. Continua em zero 79 ms depois do último push e, aos 80 ms, exatamente `[[0x10, 95]]` (o último valor). |
| `4280a59` | fix | Item 2 (D-6): `bridge.js` só cria o demo quando `__TAURI__` está ausente E `isLocalDevServer(location)` (`http:`/`https:` em `localhost`/`127.0.0.1`, qualquer porta). Em qualquer outro caso, um bridge `mode: 'unavailable'` rejeita todo comando com `{ kind: 'backend_unavailable', message: UNAVAILABLE_MESSAGE }`, e a UI mostra o estado de erro que já existia. Há 4 testes `node --test` novos e 1 spec Playwright novo. Os helpers de `bridge-demo.test.mjs` e `contract.test.mjs` passam a usar `new URL('http://localhost:1420/…')` como `location`. |
| `7482711` | test | Item 3 (W-2): nos 2 testes de `rtk_qhd_hdr.rs`, logo após `if !hardware_enabled() { return; }`, entra `assert!(std::env::var_os("DDC_TRAY_FAKE").is_none(), "{NOT_SIMULATED}")`. A mensagem: `DDC_TRAY_FAKE is set: the app would serve its simulated RTK, and this run would be no evidence of the real monitor; unset it`. |
| `c37491c` | docs | README (Browser demo, suíte Playwright, testes de hardware), CHANGELOG e `docs/hardware-validation.md` passo 7: o demo só roda em servidor local, e o teste de hardware recusa `DDC_TRAY_FAKE`. |

### 1 — debounce vs throttle
- Prova negativa: apliquei a mutação do critic (remover `stopTimer(state);` só do `push` de `src/debounce.js`, com o `flush` intacto), sem commitar. O arquivo foi restaurado por cópia (`git diff --quiet` limpo). Resultado:
  ```text
  not ok 4 - a burst longer than the debounce window writes once, after it ends
    error: |-
      no write 80 ms into the burst
  # pass 13
  # fail 1
  ```
  O teste novo é o único que falha: os 13 antigos passam com o mutante, que é justamente a lacuna que o critic apontou. O mutante escreve já aos 80 ms da rajada, ou seja, virou throttle.

### 2 — bridge só em dev local (D-2026-09-27-tray-app-6)
- **`createBridge(win)`:**
  - Com `__TAURI__.core.invoke`, devolve o bridge `tauri`, como antes.
  - Com `__TAURI__` ausente (`== null`) e servidor local, devolve o `demo`.
  - Em qualquer outro caso, devolve o `unavailable`. Isso inclui um `__TAURI__` presente sem `invoke`: a D-6 permite o demo só quando `__TAURI__` está ausente.
  - Exporta `isLocalDevServer(location)` e `UNAVAILABLE_MESSAGE`.
- **Bridge `unavailable`:**
  - Os 7 comandos (`list_monitors`, `select_monitor`, `load_panel`, `load_features`, `probe_features`, `set_feature`, `hide_popup`) rejeitam com `backend_unavailable`.
  - `onPopupShown`/`onPanelChanged` resolvem com um `unlisten` no-op. Evento não é comando, e rejeitar ali faria o `app.js` mostrar um toast repetindo o erro que o primeiro comando já mostra.
  - Não expõe `window.__ddcDemo`.
- **Testes `node --test` novos em `tests/ui/bridge-demo.test.mjs`:**
  - `outside a local dev server the bridge never falls back to the demo` (título exato do Verify): `tauri://localhost/index.html`, `http://tauri.localhost/index.html`, `https://tauri.localhost/index.html`, `tauri://localhost/?demo=rtk` e `http://tauri.localhost/?demo=two-monitors`, todos sem `__TAURI__`. Exige `mode: 'unavailable'`, nenhum `__ddcDemo` e os 8 chamados (7 comandos, `set_feature` com e sem `confirmed`) rejeitados com exatamente `{ kind: 'backend_unavailable', message: UNAVAILABLE_MESSAGE }`.
  - `the demo needs http or https on localhost or 127.0.0.1`:
    - aceitos: `http://localhost:1420/`, `http://localhost/?demo=error`, `https://localhost:8443/`, `http://127.0.0.1:1420/?demo=empty`;
    - recusados: `tauri://localhost/`, `http(s)://tauri.localhost/`, `file:///…`, `http://192.168.0.10:1420/`, `http://localhost.example.com/`, `http://[::1]:1420/`, `ftp://localhost/`;
    - `createBridge({})`/`createBridge(undefined)` dão `unavailable`.
  - `a __TAURI__ without invoke on a dev server is not a demo either`.
  - `the unavailable bridge accepts listeners that never hear anything`.
- **Prova no navegador (`tests/e2e/unavailable.spec.mjs`, 2 caminhos × 2 temas = 4 testes):**
  - O Playwright roteia `http://tauri.localhost/**` (a origem do app no Windows) para os mesmos arquivos de `src/` do servidor da suíte, sob a CSP do app.
  - Em `/` e `/?demo=rtk`, exige:
    - o estado `error`, com o heading `error.backend_unavailable` e o detalhe `UNAVAILABLE_MESSAGE`;
    - "Tentar novamente" visível, painel e botão de energia ocultos;
    - `typeof window.__ddcDemo === 'undefined'`;
    - console limpo, nenhum `<select>` e axe sem critical/serious (o `expectAccessible` travado).
  - Os `critical_paths` continuam no demo em `http://localhost:1420`.
- **Provas negativas** (mutações sobre o arquivo commitado, restaurado por cópia e `git diff --quiet` limpo):
  | Mutação em `src/bridge.js` | `node --test bridge-demo` | Playwright `unavailable.spec.mjs` |
  |---|---|---|
  | M1: `return demoBridge(…)` sem a condição (o comportamento antigo) | `# fail 3`, incluindo `not ok 1 - outside a local dev server the bridge never falls back to the demo` | **4 failed** (`Expected: "error"` / `Received: "ready"`) |
  | M2: sem checar o protocolo (`tauri://localhost` vira dev) | `# fail 2`, incluindo o `not ok 1` | — |
  | M3: sem checar o hostname (`http://tauri.localhost` vira dev) | `# fail 2`, incluindo o `not ok 1` | — |

### 3 — W-2: o teste de hardware recusa o monitor simulado
- **Com a guarda:** rodei `DDC_HW_TESTS=1 DDC_TRAY_FAKE=1 cargo test -p ddc-tray --locked --test rtk_qhd_hdr -- --ignored --test-threads=1`. Os 2 testes FALHAM em `rtk_qhd_hdr.rs:126` e `:174`, com a mensagem `DDC_TRAY_FAKE is set: …`, antes do `compose_osd()`.
  - A rodada é segura por construção. A asserção dispara antes de qualquer backend. Mesmo sem ela, com `DDC_TRAY_FAKE=1` o `compose_osd()` retorna o `simulated_osd()` em memória antes de `DdcHiMonitorBackend::new()` (`lib.rs:72-75`).
  - Sem `DDC_HW_TESTS`, o resultado segue `0 passed; 0 failed; 2 ignored`.
- **Prova negativa:** tirei as 2 asserções do arquivo commitado e rodei com as mesmas variáveis. Em seguida restaurei o arquivo.
  - `rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored` **passa oco**, com `brightness 75 -> 85 (max 100)` e `restored brightness: … current: 75` em memória: `1 passed; 1 failed`. É exatamente o W-2.
  - O teste do monitor mudo falha, porque o fixture não tem monitor mudo.

### 4 — axe travado (conferido, sem mudança)
- `tests/e2e/support.mjs:127` tem literalmente `new AxeBuilder({ page }).withTags(AXE_TAGS).analyze()`, e `:32` tem `const BLOCKING_IMPACTS = new Set(['critical', 'serious']);`.
- `grep -nE 'disableRules|\.exclude\(|\.include\(|\.options\(|\.disableFrameRules|setLegacyMode' tests/e2e/*.mjs` dá 0 linhas.
- O `AxeBuilder` só é construído em `support.mjs`, inclusive no spec novo, que usa `expectAccessible`.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, conferidos como substring exata, rodados com `bash` a partir da raiz)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test`: título da rajada presente, por módulo e total, 0 fail/cancelled/skipped/todo | `OK` |
| C7 | i18n paridade/HTML | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 895841, item `org.kde.StatusNotifierItem-895841-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated, guard vivo, `var_os("DDC_TRAY_FAKE").is_none()` | `OK` (não executado contra hardware; regra) |
| C13 | Gate 7 (axe travado, 10 `critical_paths`, dropdown completo, pulados = screenshots) | `OK` |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri == true` | `OK` |
| C15 | nenhum `<select>` em `src/` | `OK` |
| C16 | `ksni` + ≥3 testes `scroll` + `smoke-sni.sh --fake --scroll` | `OK` (`75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75`) |
| C17 | TODO/FIXME em todo arquivo versionado do produto | `OK` |
| C18 | screenshots regenerados sem nada pulado, byte a byte iguais | `OK` (SHA-1 dos 6 PNGs iguais antes e depois) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥80% (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78% |
| P3 | TODO/FIXME em `*.rs` | `OK` |

Os smokes (C11, C16) seguiram o protocolo do orquestrador. A instância do usuário (PID 836348) foi encerrada com `pkill -x ddc-tray`, os 2 Verify rodaram, e `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu logo depois (PID 896681). A porta 1420 estava livre antes e depois do C13/C18.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 (só o aviso future-incompat de `nom v3.2.1`, pré-existente) |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (igual à iter 4: nenhum teste Rust novo que rode sem hardware) |
| cross-check `ddc-core ddc-adapters ddc-cli` em `x86_64-pc-windows-msvc` e `x86_64-unknown-linux-gnu` | exit 0 / exit 0 (`crates/` não foi tocado nesta iteração) |
| `node --test` (todos) | **129 pass** (iter 4: 124; +1 debounce, +4 bridge), 0 fail, 0 cancelled, 0 skipped, 0 todo |
| Playwright | **56 passed** (iter 4: 52; +4 do `unavailable.spec.mjs`), 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate, `--fail-under-lines 80`, sem `main.rs`/`build.rs`) | exit 0, TOTAL lines **83.22%** (igual à iter 4; só mudaram JS, docs e um teste `#[ignore]`) |
| `Cargo.lock` / `package-lock.json` | intocados |

### Desvios e observações
- **Arquivos além dos 4 itens:** todos são consequência direta da D-6.
  - `tests/e2e/unavailable.spec.mjs`: prova no navegador de que a UI mostra o estado de erro.
  - `tests/ui/contract.test.mjs`: o helper precisava de uma `location` de servidor local.
  - `playwright.config.mjs`: só o comentário.
  - README, CHANGELOG e `docs/hardware-validation.md`: a doc precisa refletir o comportamento, que é item Manual do DoD.
- **Interpretações:**
  - `__TAURI__` presente sem `invoke` → `unavailable`, não demo (leitura literal de "demo só quando `__TAURI__` ausente").
  - `[::1]` não conta como local, porque a D-6 lista só `localhost`/`127.0.0.1`.
  - O teste de hardware recusa `DDC_TRAY_FAKE` com QUALQUER valor, inclusive vazio. É mais estrito que o `switch_on` do app, que só liga com `1`, e segue o texto da D-6 ("recusa rodar com `DDC_TRAY_FAKE` definido").
- **Mudança de contrato interno:** `createBridge(undefined)`/`createBridge({})` agora devolvem `unavailable`; antes, devolviam o demo. Nenhum chamador de produção faz isso: o `app.js` passa `window`.
- **Item 4:** nenhuma mudança necessária. As duas linhas literais já estavam lá, e não havia forma proibida.
- **Instância do usuário:** encerrada e reaberta 1× (só para os smokes C11/C16), com `pkill -x ddc-tray` e `setsid -f`. Nada em `~/.local` foi tocado, e o binário instalado continua sendo o build da iter 3, como a review já notou. Um único popup apareceu na tela (C11).
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` rodou 2× e só com `DDC_TRAY_FAKE=1`: a guarda, que dispara antes de qualquer backend, e o mutante, que usou o monitor em memória. A execução real com `DDC_HW_TESTS=1` segue com o orquestrador (Deferred), agora protegida pela guarda.

## Iteração 6 (rodada 2) — arraste do slider + texto literal

Modo fix, cadeia autônoma do `/jdi-issue`, iter 6 (rodada 2, depois do AUTO-RESET 1). A iter 5 foi aprovada pelo reviewer (APPROVED_PENDING_MANUAL, só o W-1 herdado), mas o DoD critic demonstrou 2 lacunas: um slider que escreve a cada `input` passava em tudo, e `element(tag, classe, 'texto')` no `app.js` escapava dos testes de i18n. O orquestrador endureceu o CONTEXT (`05ef2ae`). Não editei o CONTEXT. As 8 tasks do PLAN continuam `completed`. O W-1 (`cargo audit`) segue com a `ci-crossbuild`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `31150bd` | test | Item 1: `tests/e2e/slider.spec.mjs` com o teste `dragging the brightness slider writes once, after the drag` (2 temas → 2 ✓) e um segundo teste de valor lido de volta. |
| `7479761` | test | Item 2: `app.js passes no literal text to element() or setText()` em `tests/ui/i18n-html.test.mjs`, mais um autoteste do scanner. |
| `f9e263d` | docs | README (testes do tray) e CHANGELOG descrevem os dois testes novos. |

### 1 — arraste real do slider de brilho (D-2026-09-26-tray-app-5)
- **O arraste:** em `?demo=rtk`, `page.mouse.move` até o centro do polegar em 75 %, `page.mouse.down()`, 20 `page.mouse.move` ao longo da trilha até 20 % e `page.mouse.up()`.
  - A posição do polegar considera a largura dele (18 px em `styles.css`).
  - Cada passo do laço deixa passar 20 ms no relógio da página e 20 ms no relógio de parede.
- **O que a página registra** (listeners de observação, instalados antes do arraste):
  - cada `input`: valor, `performance.now()` e quantas escritas o demo já tinha tomado;
  - o `pointerup`, na fase de captura da `window`, antes dos handlers do app;
  - o instante em que cada escrita chega em `__ddcDemo.writes`.
- **O que o teste exige:**
  - ≥ 15 eventos `input` (dão 20) e ≥ 300 ms do primeiro ao último (dão 380 ms de página);
  - nenhuma pausa ≥ `DEBOUNCE_MS` durante o arraste;
  - zero escritas em todo `input` e no `pointerup`, e o último valor exatamente 20;
  - depois de soltar, exatamente `[{ monitorId: RTK, code: 0x10, value: 20, confirmed: false }]`, e ainda só ela depois de mais `2 × DEBOUNCE_MS + DEMO_LATENCY_MS`;
  - a escrita chega exatamente `DEMO_LATENCY_MS` (60 ms) depois do release, ou seja, foi enviada no release;
  - o slider mostra `20` e `aria-valuetext` `20%`.
- **Teste extra:** `a dragged slider settles on the brightness the monitor read back`. O `bridge.js` de teste mantém a leitura do 0x10, como o `dropdown.spec.mjs` faz com o preset. O arraste termina em 20, a escrita leva 20, e o slider volta para `75`/`75%`, com o anúncio `announce.readBack`. O título não contém a substring do Verify, então o `dr` continua 2.
- **Relógio da página (desvio de método):**
  - A primeira versão usava só tempo real. Sob carga (suíte ×4 com 24 workers), uma pausa entre dois `move` chegou a 80,5 ms e o teste falhou: 1 em 264 execuções.
  - Com a janela de 80 ms, essa escrita no meio do arraste seria correta, e a falha, espúria.
  - Agora o relógio da página é o do Playwright (`page.clock.install()` + `pauseAt`) e fica parado durante o arraste. Cada passo avança exatamente 20 ms com `page.clock.runFor`, o que dispara qualquer timer vencido.
  - Os eventos de mouse continuam reais (CDP). Um debounce mais curto que 20 ms, um throttle ou um flush imediato continuam disparando dentro do arraste.
  - Estabilidade depois da mudança: 2 × 264 (suíte ×4, 24 workers) e 160 execuções só do spec, 0 falhas.
- **Prova negativa (mutação do critic, não commitada):** `if (!entry.dto.dangerous) queueWrite(entry, value, { now: true });` na linha 508 de `src/app.js`. O arquivo foi restaurado por cópia, e `git diff --quiet` ficou limpo.
  - Suíte inteira: **4 failed** (os 2 testes do `slider.spec.mjs` × 2 temas), 56 passed, 6 skipped. Os 56 testes anteriores passam com o mutante, que é justamente a lacuna.
  - O teste do Verify falha em `writes taken at each input`:
    ```text
    ✘  1 [light] › tests/e2e/slider.spec.mjs:88:1 › dragging the brightness slider writes once, after the drag
        Error: writes taken at each input
        - Expected  - 17
        + Received  + 17
        (recebido: 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6)
    ```
    São 6 escritas durante um arraste de 400 ms, uma a cada 60 ms de latência do demo.
- **Prova negativa extra (throttle):** tirar o `stopTimer(state)` do `push` de `src/debounce.js` → os 4 testes do `slider.spec.mjs` falham (`writes taken at each input`). Restaurado.

### 2 — nenhum texto literal no DOM montado pelos scripts (D-2026-09-26-tray-app-6)
- **Escopo:** o teste `app.js passes no literal text to element() or setText()` lê todo script de `src/` (recursivo) que monta DOM, detectado por `document`/`createElement(NS)`/`textContent`/`setAttribute`. Hoje são `app.js`, `dropdown.js` e `icons.js`, e o teste exige que os 3 sejam encontrados.
- **Como lê:** um tokenizador pequeno, no próprio arquivo de teste e sem dependências. Ele descarta comentários de linha e de bloco, separa strings, templates (com as expressões `${}` tokenizadas) e regexes (distingue `/` de divisão) e casa parênteses.
- **O que ele reprova:** um literal com letra (`\p{L}`) fora de `t(…)`, em qualquer destes lugares:
  - nos helpers do próprio módulo com parâmetro `text`, na posição desse parâmetro: `element()` (3º argumento no `app.js`, 4º no `dropdown.js`), `setText()`, `announce()`, `showToast()`, `withCode()`;
  - em atribuição (`=`, `+=`, `??=`, `||=`, `&&=`) a uma propriedade de texto: `textContent`, `innerText`, `outerText`, `nodeValue`, `title`, `alt`, `placeholder`, `ariaLabel`, `ariaDescription`, `ariaRoleDescription`, `ariaValueText`, `ariaPlaceholder`, `ariaKeyShortcuts`;
  - em `setAttribute` de atributo legível (`READABLE_ATTRIBUTES`, que ganhou `aria-braillelabel`, `aria-brailleroledescription` e `aria-keyshortcuts`) ou de nome dinâmico;
  - em `setAttribute` de outro `aria-*` cujo valor não seja token ARIA (`true`, `false`, `polite`, `listbox`…). Os atributos de id (`aria-controls`, `aria-labelledby`…) ficam de fora;
  - numa propriedade de texto passada a `Object.assign`;
  - como argumento string de `append`/`prepend`/`replaceChildren`/`before`/`after`/`replaceWith`, fora de chamadas, e em `createTextNode`/`insertAdjacentText`.
- **Contra teste oco:** o teste exige ter lido os lugares que guarda: no `app.js`, ≥ 8 `element()` (há 26), ≥ 4 `setText()` (4), ≥ 10 `.textContent =` (16) e ≥ 10 `setAttribute()` (12); no `dropdown.js`, ≥ 1 `element()` (8).
- **Autoteste:** `the literal-text scan flags each way a text reaches the DOM, and nothing else`. Um script sintético de 22 linhas fixa por igualdade as 12 marcações esperadas e as formas permitidas: `t()`, dados, `aria-hidden 'true'`, `aria-controls` com template, comentários de linha e de bloco, regex com aspas e divisão.
- **Ocorrências reais:** nenhuma em `src/`. Nada foi corrigido no produto.
- **Provas negativas (mutações não commitadas, restauradas por cópia, `git diff --quiet` limpo):**
  | Mutação | Resultado |
  |---|---|
  | M1 (pedida): `app.js:832` `element('span', 'tag', 'Undeclared')` | `not ok 8 - app.js passes no literal text to element() or setText()`, `+ 'app.js:832 element() "Undeclared"'`, `# pass 14`, `# fail 1` |
  | M2: `app.js:379` `setText(ui.messageTip, … ? 'Check the DDC/CI setting' : null)` | `not ok 8`, `+ 'app.js:379 setText() "Check the DDC/CI setting"'`, `# fail 1` |
  | M3: `dropdown.js:412` `element(doc, 'span', 'dropdown-label', 'Pick one')` | `not ok 8`, `+ 'dropdown.js:412 element() "Pick one"'`, `# fail 1` |

  Nos 3 casos, os outros 14 testes do arquivo passam com o mutante, que é justamente a lacuna.

### 3 — conferências sem mudança
- **Coletor de console em `tests/e2e/support.mjs`:** as linhas literais que o Verify do Gate 7 exige estão lá.
  - `:54` `if (message.type() === 'error') pageErrors.push(` com `` `console.error: ${message.text()}` ``;
  - `:56` `page.on('pageerror', (error) => pageErrors.push(`;
  - `:60` `expect(pageErrors, 'console errors and uncaught page errors').toEqual([]);`.
  - Não há filtro (`Failed to load`, `filter`, `startsWith` ou `return` no handler).
- **`rtk_qhd_hdr.rs`:** 2 `#[test]`, 2 `#[ignore]`, 2 `if !hardware_enabled() {`, e 2 `assert!(std::env::var_os("DDC_TRAY_FAKE").is_none(), …)` (`:127`, `:175`), um por teste. O Verify C12 confere `= $t`.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, conferidos como substring exata, rodados com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo e total, 0 fail/cancelled/skipped/todo | `OK` (131 pass) |
| C7 | i18n: título `app.js passes no literal text to element() or setText()` presente + paridade/HTML | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 1109074, item `org.kde.StatusNotifierItem-1109074-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated, guard vivo, `DDC_TRAY_FAKE` recusado em cada teste | `OK` (não executado contra hardware) |
| C13 | Gate 7 (coletor e axe travados, 10 `critical_paths`, dropdown completo, `dr = 2`, pulados = screenshots) | `OK` (60 passed, 6 skipped) |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri == true` | `OK` |
| C15 | nenhum `<select>` em `src/` | `OK` |
| C16 | `ksni` + ≥ 3 testes `scroll` + `smoke-sni.sh --fake --scroll` | `OK` (`75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75`) |
| C17 | TODO/FIXME em todo arquivo versionado do produto | `OK` |
| C18 | screenshots regenerados sem nada pulado, byte a byte iguais | `OK` (SHA-1 dos 6 PNGs iguais antes e depois) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥ 80 % (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78 % |
| P3 | TODO/FIXME em `*.rs` | `OK` |

- **Protocolo dos smokes (C11, C16):** um de cada vez. Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray`; logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu. PIDs: 930850 → 1109309 → 1110145, que é a instância viva no fim.
- **Porta 1420:** livre antes e depois do C13/C18.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (igual à iter 5: nenhum `.rs` mudou) |
| cross-check `ddc-core ddc-adapters ddc-cli` em `x86_64-unknown-linux-gnu`; `ddc-cli ddc-adapters` em `x86_64-pc-windows-msvc` | exit 0 / exit 0 |
| `node --test` (todos) | **131 pass** (iter 5: 129; +2 em `i18n-html`), 0 fail, 0 cancelled, 0 skipped, 0 todo |
| Playwright | **60 passed** (iter 5: 56; +4 do `slider.spec.mjs`), 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate, `--fail-under-lines 80`, sem `main.rs`/`build.rs`) | exit 0, TOTAL lines **83.22 %** (igual à iter 5) |
| `Cargo.lock` / `package-lock.json` | intocados |

### Desvios e observações
- **Relógio controlado no arraste:** os ≥ 300 ms do CONTEXT valem no relógio da página (380 ms do primeiro ao último `input`, exigido pelo teste) e também no de parede (20 × 20 ms de `waitForTimeout`). Os eventos de mouse são reais. O motivo, a medição e a estabilidade estão no item 1.
- **Testes além dos 2 títulos pedidos:** `a dragged slider settles on the brightness the monitor read back`, que prova que o slider mostra o valor lido e não o arrastado, e o autoteste do scanner. Nenhum dos dois títulos contém as substrings dos Verify.
- **`READABLE_ATTRIBUTES`** ganhou 3 atributos ARIA legíveis, o que também deixa um pouco mais estrito o teste do HTML (`the page has no literal readable attribute`). O `index.html` não usa nenhum deles.
- **Docs:** README e CHANGELOG ganharam uma frase cada sobre os testes novos (`f9e263d`), porque são itens Manual do DoD.
- **Suspeitas `objective:false` do critic:**
  - «15» (guard de `DDC_TRAY_FAKE` uma só vez) já está coberto pelo Verify, que conta `= $t`.
  - «3/20» (`Todo` em caixa mista) não foi tratado: não fazia parte do pedido, e o CONTEXT não foi editado.
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` não foi executado. Os smokes usaram o backend real só com leituras (C11) e o monitor simulado (C16).
- **Instância do usuário:** encerrada e reaberta 2× (só para os smokes), com `pkill -x ddc-tray` e `setsid -f`. Nada em `~/.local` foi tocado. O binário instalado continua anterior à iter 5, como a review já notou.

## Iteração 7 (rodada 2) — pseudo-locale

Modo fix, cadeia autônoma do `/jdi-issue`, iter 7 (rodada 2). A iter 6 foi aprovada pelo reviewer (APPROVED_PENDING_MANUAL, só o W-1 herdado), mas o DoD critic mostrou 2 lacunas:
- um literal guardado numa `const` ou devolvido por helper passava pelo scanner estático;
- o harness podia ser enfraquecido.

O orquestrador travou a D-2026-09-27-tray-app-7 e acrescentou as linhas ao CONTEXT (`7758aef`). Não editei o CONTEXT. As 8 tasks do PLAN continuam `completed`. O W-1 (`cargo audit`) segue com a `ci-crossbuild`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `d32319f` | feat | `?pseudo=1` só com o bridge demo local: `t()` envolve cada texto traduzido em `⟦…⟧`, depois de preencher os placeholders. Chave inexistente continua voltando sem marca, então o `translateOr` segue caindo no nome do core. Testes `node` do pseudo e de `pseudoRequested`. |
| `3ca5bba` | feat | Dados do monitor e do core sob `translate="no"`. A dica i2c teve o caminho do guia tirado de dentro da frase. |
| `0263cc0` | test | `tests/e2e/pseudo-locale.spec.mjs`: 13 estados × 2 temas com o título `every visible text is translated or marked as data: …`, mais a prova de que `?pseudo=1` é ignorado na origem do app. |
| `2e69e82` | docs | README (demo com `pseudo=1` e o que o check lê) e CHANGELOG. |

Depois do `0263cc0`, `playwright.config.mjs`, `tests/e2e/*.mjs` e `tests/ui/*.mjs` não mudaram mais.

### 1 — pseudo-localização em runtime (D-2026-09-27-tray-app-7)
- **Onde liga:**
  - `app.js` monta o bridge antes do tradutor;
  - `pseudo = bridge.mode === 'demo' && pseudoRequested(location.search)`;
  - `translator(locale, LOCALES, { pseudo })`.
- **Fora do demo, nunca liga:** na origem do app (`tauri://localhost`, `http://tauri.localhost`) o bridge é `unavailable` e o parâmetro é ignorado.
- **O que é dado (`translate="no"`):**
  - o nome do monitor no cabeçalho (`#monitor-name`, só quando mostra o monitor; o título do app continua traduzido);
  - o valor e as opções do seletor de monitores, exceto a opção "(sem DDC/CI)", que vem de `t()`;
  - nomes de valor NC sem chave: entradas (`HDMI-1`…) e presets (`sRGB`, `5000 K`…), nos chips, nos dropdowns (valor mostrado e opções) e nos rádios do diálogo;
  - o nome MCCS do core quando o alias não tem chave (`labelVerbatim`);
  - os códigos VCP (`.code`, `0x0C`…);
  - a mensagem técnica do backend (`#message-detail`, fixo no HTML);
  - o caminho `docs/linux-ddc-setup.md` da dica.
- **Como a marcação é decidida:** o view-model diz o que é dado, com `verbatim` nas opções, `currentVerbatim` e `labelVerbatim`. O dropdown recebe `verbatim` por opção. O app nunca decide pela presença do marcador.
- **Números e percentuais** formatados por `t()` (`format.percent`, `format.fraction`, `format.unnamed` "Valor 0x1B", `more.probeSilent`) continuam traduções: o padrão vem do arquivo de locale e sai com o marcador.
- **Texto real que o spec encontrou** (rodado antes da marcação, em 12 dos 13 estados, 32 pares texto/elemento distintos):
  - nomes de monitor (`RTK QHD HDR`, `DELL U2723QE`) no cabeçalho e no seletor;
  - as 7 entradas do RTK;
  - `sRGB`, `5000 K`, `6500 K`, `7500 K` e `9300 K`;
  - 9 códigos `0x..`;
  - as duas mensagens do backend (`no DDC/CI backend could be started (demo)` e o `transport error: …` da TV);
  - o caminho do guia;
  - o fragmento `.⟧` da dica i2c.

  Todos eram dados, não literais. Nenhuma tradução estava marcada como dado.
- **Dica i2c (correção de texto):** antes era "… /dev/i2c-*. Veja {doc}." e o app partia a frase em volta do caminho. Sob pseudo, sobrava um nó `.⟧` sem `⟦`. Agora `hint.i2c` termina em "Guia de configuração:" / "Setup guide:" e o caminho vem depois, em `<code translate="no">`, com `margin-inline-start: 0.3em`. Conferi visualmente em screenshots de rascunho (pt-BR, en e pseudo) no scratchpad.
- **O spec** (`every visible text is translated or marked as data: …`):
  - **Estados (13 × 2 temas = 26 ✓, o Verify exige ≥ 16):**
    - os 5 `critical_paths`: `/ ready`, `rtk ready`, `two monitors ready`, `empty`, `error`;
    - os diálogos de entrada e de energia abertos;
    - "Todos os ajustes" aberto e sondado no RTK;
    - "Todos os ajustes" aberto e sondado no DELL, onde nada responde e aparece `more.probeEmpty`, exatamente o caminho da mutação do critic;
    - a lista de presets aberta e a lista de monitores aberta;
    - a TV muda escolhida (estado de erro `transport`);
    - `loading`, com o relógio da página parado.
  - **O que cada teste lê:** todo nó de texto não vazio após trim, cujo pai passa em `checkVisibility({ visibilityProperty: true })`, fora de `script`/`style`/`template`/`noscript`.
  - **O que cada teste exige:**
    - cada nó contém `⟦` ou tem o `[translate]` mais próximo igual a `no` (semântica do HTML);
    - nenhum nó sob `translate="no"` contém `⟦`, então marcar tudo como dado não passa;
    - a mesma regra para `aria-label`, `aria-description`, `aria-roledescription`, `aria-valuetext`, `aria-placeholder`, `title`, `alt`, `placeholder` e `label`, em todos os elementos do documento, visíveis ou não;
    - nenhum texto gerado por `::before`/`::after`;
    - `document.title` = `⟦DDC Control⟧`;
    - ≥ 3 nós lidos e ≥ 3 traduções;
    - axe em todos os estados, menos o `loading`.
  - **Contagem lida por estado** (nós visíveis / traduzidos, incluindo atributos / dados):
    - `loading`: 3 / 10 / 0;
    - `empty`: 8 / 14 / 1;
    - `rtk ready`: 20 / 21 / 9;
    - lista de presets aberta: 27 / 23 / 14;
    - "Todos os ajustes" sondado no RTK: 53 / 51 / 18.
  - **Os `reach`** acham elementos e esperam por eles, mas nunca afirmam texto. Quem pega o literal é o check genérico.
  - **Os marcadores** estão escritos no spec, não importados do produto: um pseudo que deixasse de marcar não redefine o que o teste procura.
- **`translate="no"` não afeta o visual:** `SCREENSHOTS=1` regenerou os 6 PNGs com SHA-1 idênticos, e `git diff --quiet -- docs/screenshots` ficou limpo.
- **`translate="no"` não afeta o axe:** os 10 `critical_paths`, os diálogos, o "Todos os ajustes" e os 24 testes pseudo com axe passam.

### Provas negativas (mutações não commitadas, revertidas com `git checkout -- src`, `git status` limpo depois de cada uma)
"Suíte antiga" = os 66 testes Playwright anteriores (`--grep-invert` dos novos). "`node`" = os 135 testes, inclusive o scanner estático do `i18n-html`.

| Mutação | `node` | Suíte antiga | Spec pseudo |
|---|---|---|---|
| M1 (a do critic): `src/app.js` `const note = answering.length === 0 ? 'No other setting answered' : t('more.probeSilent', …)` | 135 pass | **60 passed**, 6 skipped | **2 failed** (DELL sondado × 2 temas): `text "No other setting answered" in <p#probe-note.probe-note> is neither translated nor under translate="no"` |
| M2 (literal devolvido por helper): `metaText()` devolve `` `${current.manufacturer} · DDC/CI` `` | 135 pass | **60 passed**, 6 skipped (o texto é igual ao pt-BR) | **18 failed** (os 9 estados `ready` × 2): `text "RTK · DDC/CI" in <p#monitor-meta…>` e `"DEL · DDC/CI"` |
| M3 (helper do view-model): `unnamed()` devolve `` `Valor ${hex(value)}` `` | 135 pass | **60 passed**, 6 skipped | **2 failed**: `text "Valor 0x1B" in <span.chip-label>` |
| M4 (trapaça): `<body translate="no">` | 135 pass | — | **26 failed**: `… is a translation under translate="no"` (ex.: `⟦DDC Control⟧` no `h1`, `[aria-label] "⟦Monitor⟧"`) |
| M5 (pseudo desligado): `const pseudo = false && …` | 135 pass | — | **26 failed**: `Expected: "⟦DDC Control⟧"`, `Received: "DDC Control"` |
| M6 (marcação de dado tirada): sem `label.translate = !option.verbatim` no `dropdown.js` | 135 pass | — | **4 failed** (as 2 listas abertas × 2): `text "sRGB" in <span.dropdown-label>`, `"RTK QHD HDR"`… |

Estabilidade: o spec rodou `--repeat-each=5`, 140/140 passed.

### 2 — harness
- O coletor de console/pageerror e as chamadas do axe em `tests/e2e/support.mjs` não mudaram. O spec novo só acrescenta chamadas a `expectAccessible` e usa o fixture `page` travado.
- Em `tests/ui/` só entraram testes: 3 em `i18n-parity` e 1 em `view-model`. Os `deepEqual` do view-model ganharam os campos `verbatim`/`labelVerbatim`/`currentVerbatim`, e o texto novo da dica. Nenhuma asserção saiu.
- **Hash atual do harness:** `9787e931bc453d5221fd1163d8b238ed913fd04b54933e2e8274436f3ef33288`. São 19 arquivos: `playwright.config.mjs`, 10 em `tests/e2e/` e 8 em `tests/ui/`. É igual ao `HEAD` `0263cc0`. Foi calculado com `cd apps/ddc-tray && sha256sum playwright.config.mjs tests/e2e/*.mjs tests/ui/*.mjs | sha256sum | cut -c1-64`.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, rodados com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo e total | `OK` (135 pass, 0 fail/cancelled/skipped/todo) |
| C7 | i18n paridade/HTML + scanner | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 1266125, `org.kde.StatusNotifierItem-1266125-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated | `OK` (listado, não executado) |
| C13 | Gate 7 | `OK`: 88 passed, 6 skipped (= screenshots), 0 failed/flaky; 10/10 `critical_paths`, dropdown 16/16, `dr = 2`, `ps = 26` |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | `OK` |
| C15 | hash do harness | **falha esperada**: o CONTEXT ainda tem `HARNESS_SHA256_PENDING`; o hash atual está acima |
| C16 | nenhum `<select>` em `src/` | `OK` |
| C17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | `OK` (PID 1267334, `75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75`) |
| C18 | TODO/FIXME em todo arquivo versionado do produto | `OK` |
| C19 | screenshots regenerados, nada pulado, byte a byte iguais | `OK` (SHA-1 dos 6 PNGs iguais antes e depois) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥ 80 % (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78 % (gate com exclusões e `--fail-under-lines 80`: 83.22 %) |
| P3 | TODO/FIXME em `*.rs` | `OK` |

- **Protocolo dos smokes (C11, C17):** um de cada vez. Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray`. Logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu. PIDs: 1131592 → 1266367 → 1267586, que é a instância viva no fim. Nada em `~/.local` foi tocado.
- **Porta 1420:** livre antes e depois do C13/C19.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 / exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (nenhum `.rs` mudou) |
| `node --test` (todos) | **135 pass** (iter 6: 131; +3 `i18n-parity`, +1 `view-model`), 0 fail/cancelled/skipped/todo |
| Playwright | **88 passed** (iter 6: 60; +28 do `pseudo-locale.spec.mjs`), 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate) | TOTAL lines **83.22 %** |
| `Cargo.lock` / `package-lock.json` | intocados |

### Desvios e observações
- **Estado indisponível sob pseudo: não viável por construção.** A D-7 só liga o pseudo no demo. O estado é coberto de duas formas:
  - `/?demo=error` renderiza o mesmo estado `error`/`backend_unavailable`: mesmos nós, só muda a mensagem técnica, que é dado;
  - o teste `?pseudo=1 is ignored at the app origin: no text is wrapped` (2 temas, fora da contagem `ps`) prova que, na origem do app, nenhum texto sai marcado.
- **`loading` sem axe:** o axe espera timers, e o relógio da página, parado para segurar o estado, não os dispara (o axe estourou os 30 s). O check de texto roda normalmente. Esse estado vai além dos 8 pedidos.
- **Texto da dica i2c mudou** (en e pt-BR), pelo motivo do item 1. Só aparece nos estados vazio/erro, que não estão nos screenshots versionados.
- **Rigor além do pedido:**
  - check inverso (tradução sob `translate="no"` reprova);
  - `::before`/`::after`;
  - atributos de todos os elementos, e 4 atributos legíveis a mais;
  - título do documento;
  - mínimos de leitura;
  - regra do `[translate]` mais próximo, em vez de "qualquer ancestral `no`" (equivalente aqui, e mais estrita se houver um `translate="yes"` aninhado).
- **Comportamento de borda do produto:**
  - chave inexistente agora volta sem interpolação (nenhuma chave tem `{}`);
  - um nome do core vazio (`''`) cai em `format.code` em vez de mostrar vazio.
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` não foi executado. Os smokes usaram o backend real só com leituras (C11) e o monitor simulado (C17).

## Iteração 8 (rodada 2) — warnings W-2/W-3

Modo fix, cadeia autônoma do `/jdi-issue`, iter 8 (rodada 2, rodada de warnings). A iter 7 foi aprovada (APPROVED_PENDING_MANUAL). Esta iteração trata o W-2 e o W-3 do REVIEW; o W-1 (`cargo audit`) segue com a `ci-crossbuild`. Não editei o CONTEXT: o C15 ainda tem o hash da iter 7, e quem refixa é o orquestrador (D-2026-09-27-tray-app-7). As 8 tasks do PLAN continuam `completed`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `06ea05b` | test | W-2: o check do pseudo-locale fica estrito. Fora de `translate="no"`, tira todos os `⟦…⟧` do texto e reprova se sobrar letra. |
| `1693cec` | fix | W-3: um nó de texto `' '` real entre a frase da dica i2c e o `<code>` do caminho. Sai o `margin-inline-start: 0.3em`. O cenário vazio/erro passa a conferir o texto copiado. |
| `8907c8e` | docs | README (l. 366) e CHANGELOG descrevem a regra estrita. |

Depois do `1693cec`, `playwright.config.mjs`, `tests/e2e/*.mjs` e `tests/ui/*.mjs` não mudaram mais.

### W-2 — checagem estrita (`tests/e2e/pseudo-locale.spec.mjs`)
- **Regra nova do `judge`**, igual para nós de texto visíveis e atributos legíveis:
  - sob `translate="no"` (o `[translate]` mais próximo): dado. Se contém `⟦`, reprova com `is a translation under translate="no"` (regra mantida);
  - fora dele, sem `⟦`: reprova com `is neither translated nor under translate="no"` (regra mantida);
  - fora dele, com `⟦`: remove os segmentos `⟦…⟧` balanceados, do mais interno para fora e em laço até estabilizar (cobre os aninhados). Se sobra qualquer `\p{L}`, reprova com `has text outside the marks: "<o que sobrou>"`.
- Uma marca sem par não "lava" o texto: em `⟦No Linux, o` nada é removido e as letras sobram.
- Os marcadores continuam escritos no spec. `textsOf` recebe `{ open, close, attributes }` por uma constante `READING`.
- `::before`/`::after`, o título do documento, os mínimos de leitura, o axe e o teste da origem do app não mudaram.
- **Produto atual:** 28/28 no spec e 88 passed + 6 skipped na suíte inteira. A regra estrita não achou texto real no produto, então o produto não precisou mudar por causa dela.
- **Estabilidade:** `pseudo-locale.spec.mjs` + `scenarios.spec.mjs` com `--repeat-each=5`: 220/220 passed.

### Provas negativas (mutações não commitadas)
- **Onde rodaram:** numa cópia descartável de `apps/ddc-tray` no scratchpad. `src/` e `tests/` foram copiados; `node_modules` e `src-tauri` entraram por symlink, só para leitura.
- Na cópia rodei também o spec do `HEAD` da iter 7 (`git show 5919fb0:…/pseudo-locale.spec.mjs`) para comparar.
- `src/` foi recopiado do repo antes de cada mutação, e a cópia foi apagada no fim. O repo não foi tocado (`git status` limpo).
- "`node`" = `i18n-html` + `view-model`, 41 testes, com o scanner estático. O `i18n-parity` lê arquivos de fora de `apps/ddc-tray` e não roda na cópia; no repo, os 135 passam.

| Mutação (`src/app.js`) | `node` | Spec antigo (iter 7) | Spec estrito |
|---|---|---|---|
| **M1, a pedida:** `t('more.probeEmpty') + ' (nothing else to try)'` em `showProbe` | 41 pass | **28 passed** | **2 failed** (DELL sondado × 2 temas): `text "⟦Nenhum ajuste oculto respondeu.⟧ (nothing else to try)" in <p#probe-note.probe-note> has text outside the marks: " (nothing else to try)"` |
| M1b, a do reviewer: o mesmo sufixo via `const PROBE_SUFFIX` | 41 pass | 28 passed | 2 failed, mesma mensagem |
| M2, atributo: `aria-valuetext` = `` `${view.valueText} level` `` | 1 fail (o scanner pega o template no `setAttribute`) | 28 passed | **20 failed**: `[aria-valuetext] "⟦30%⟧ level" in <input#control-98.range> has text outside the marks: " level"` |
| M3, prefixo: `metaText()` devolve `` `DDC · ${t('header.meta', …)}` `` | 41 pass | 28 passed | **18 failed** (os 9 estados `ready` × 2): `text "DDC · ⟦RTK · DDC/CI⟧" … has text outside the marks` |
| M4, tradução partida: a dica i2c cortada em `slice(0, 12)` + `<span>` + `slice(12)` | 41 pass | 6 failed (só a metade `…⟧`, como `neither translated`) | 6 failed, e agora também `text "⟦No Linux, o" … has text outside the marks` |

- O M1 é o mutante do W-2: o spec antigo o aceitava e o estrito o reprova.

### W-3 — espaço real antes do caminho do guia
- **Teste primeiro:**
  - Em `tests/e2e/scenarios.spec.mjs`, o `showsI2cHintOnLinux` (estados `empty` e `error` dos `critical_paths`) seleciona o `#message-hint` inteiro com a Selection API e lê `selection.toString()`, que é o que uma cópia dá. Ele exige `` `${t('hint.i2c')} docs/linux-ddc-setup.md` ``.
  - Antes da correção: **4 failed**, com `Received: "…Guia de configuração:docs/linux-ddc-setup.md"`.
- **Correção:**
  - em `app.js`, `replaceChildren(status.hint.text, ' ', verbatim('code', '', status.hint.doc))`, com uma linha de WHY;
  - em `styles.css`, sai o `margin-inline-start` do `.message-hint code`;
  - os locales e o view-model não mudaram.
- **Depois da correção:** 88 passed + 6 skipped, e `node` com 135 pass. O texto copiado fica `…Guia de configuração: docs/linux-ddc-setup.md`.
- **Pseudo-locale:** o nó `' '` fica fora do `translate="no"`, mas não tem letra, e o spec ignora texto vazio após `trim`. O spec estrito segue 28/28.
- **Visual:**
  - Screenshots de rascunho no scratchpad, antes e depois: pt-BR `empty`, pseudo `empty` e `error`, nos 2 temas. Mesmas dimensões (310 × 74/75), e a olho não há diferença: o espaço da fonte substitui os 0.3em.
  - Os PNGs versionados não mostram a dica. O C19 regenerou os 6 com SHA-1 idênticos (`git diff --quiet -- docs/screenshots`), então não houve screenshot para commitar.

### Harness
- Nesta iteração mudaram `tests/e2e/pseudo-locale.spec.mjs` (W-2) e `tests/e2e/scenarios.spec.mjs` (teste do W-3). Nada foi enfraquecido:
  - todas as regras anteriores continuam;
  - o `showsI2cHintOnLinux` só ganhou asserções.
- O coletor de console/pageerror, o axe e o `playwright.config.mjs` não mudaram.
- **Hash novo do harness:** `0566f41628d9be1004bf1fceac666ee721135502929c005185fae3d3dd04518a`. São 19 arquivos, com a última mudança em `1693cec`. Foi calculado com `cd apps/ddc-tray && sha256sum playwright.config.mjs tests/e2e/*.mjs tests/ui/*.mjs | sha256sum | cut -c1-64`.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, rodados com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo e total | `OK` (135 pass, 0 fail/cancelled/skipped/todo) |
| C7 | i18n paridade/HTML + scanner | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 1410485, `org.kde.StatusNotifierItem-1410485-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated | `OK` (listado, não executado) |
| C13 | Gate 7 | `OK`: 88 passed, 6 skipped (= screenshots), 0 failed/flaky; n = 10, dd = 16/16, dr = 2, ps = 26 |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | `OK` |
| C15 | hash do harness | **não rodado, como pedido.** O CONTEXT tem o hash da iter 7 (`9787e931…3288`), e o novo está acima |
| C16 | nenhum `<select>` em `src/` | `OK` |
| C17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | `OK` (PID 1411677, `75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75`) |
| C18 | TODO/FIXME em todo arquivo versionado do produto | `OK` |
| C19 | screenshots regenerados, nada pulado, byte a byte iguais | `OK` (6 PNGs, SHA-1 inalterados) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥ 80 % (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.84 % (gate com exclusões e `--fail-under-lines 80`: 83.22 %, exit 0) |
| P3 | TODO/FIXME em `*.rs` | `OK` |

- **Protocolo dos smokes (C11, C17):** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray` (nunca `-f`).
  - Logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - PIDs do usuário: 1315274 → 1410705 → 1411926, que é a instância viva no fim. Nada em `~/.local` foi tocado.
- **Porta 1420:** livre antes e depois do C13/C19.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 / exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (nenhum `.rs` mudou) |
| `node --test` (todos) | **135 pass**, 0 fail/cancelled/skipped/todo |
| Playwright | **88 passed**, 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate) | TOTAL lines **83.22 %** |
| `Cargo.lock` / `package-lock.json` | intocados |

### Desvios e observações
- **README/CHANGELOG** (`8907c8e`) não estavam na lista da iteração. Mudei 1 frase em cada porque descreviam a regra frouxa, e o DoD manual #24 pede o README fiel ao comportamento atual.
- **`scenarios.spec.mjs` também mudou**, além do spec pseudo: é o teste de regressão do W-3 (o bugfix começa por um teste que falha). Por isso ele entra no hash novo.
- **Limite de desenho que segue (registrado pelo reviewer):** um literal colado a um *dado* sob `translate="no"` continua sem check. A D-7 aceita nós de dado sem ler o conteúdo, e a regra estrita vale só fora de `translate="no"`.
- **Cobertura literal:** 82.84 % contra os 82.78 % da iter 7, sem nenhum `.rs` alterado. É variação de execução do `llvm-cov` (testes com threads e timers). O gate ficou igual (83.22 %).
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` não foi executado. O C11 usou o backend real só com leituras, e o C17 o monitor simulado.

## Iteração 9 (rodada 2) — estados de falha no pseudo-locale

Modo fix, cadeia autônoma do `/jdi-issue`, iter 9 (rodada 2). O DoD critic da iter 8 (BLOCKED) mostrou que um literal colado a um texto de FALHA (`showToast(errorText(error, t) + NOT_APPLIED)`) passava em tudo: o demo nunca falhava, então nenhum estado do pseudo-locale mostrava o toast, o status de erro/vazio de "Todos os ajustes" nem o rótulo de sondagem. A outra linha do critic (TODO minúsculo / `todo!`) foi resolvida pelo orquestrador no CONTEXT/PROJECT (D-2026-09-27-tray-app-8), sem mudança de código: C18 e P3 dão `OK`. Não editei o CONTEXT, e o C15 ainda tem o hash da iter 8. As 8 tasks do PLAN continuam `completed`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `4423525` | feat | `&fail=` no bridge demo (só em servidor local, D-6) + testes `node --test` |
| `cdc0a90` | test | 7 estados novos no `pseudo-locale.spec.mjs` (falhas e esperas), cada um também com axe |
| `2a8b59d` | test | 4 caminhos de falha no `scenarios.spec.mjs` (locale real): texto exato, comportamento, axe e console |
| `a6498ee` | docs | README (demo + notas de teste) e CHANGELOG descrevem o `fail=` e os 20 estados |

Depois do `2a8b59d`, `playwright.config.mjs`, `tests/e2e/*.mjs`, `tests/ui/*.mjs` e `scripts/smoke-sni.sh` não mudaram mais.

### `&fail=` no demo (`src/demo-data.js`, `src/bridge.js`)
- `FAILURES` = `{ write: 'set_feature', features: 'load_features', probe: 'probe_features' }`. `failingCommands(search)` lê `fail=` (lista com vírgulas ou repetida) e ignora palavra desconhecida, como `scenarioName` ignora cenário desconhecido.
- O comando marcado acha o monitor (`not_found` e o erro do TV mudo continuam valendo) e, na escrita, passa antes pelas checagens do core (`needs_confirmation`, `invalid_value`). Só então rejeita com o timeout do core, `{ kind: 'timeout', message: 'monitor did not respond in time' }`, sem mudar nada. É o `DdcError::Timeout`, e o mesmo erro que o demo já dava ao escrever num código sondado sem resposta, agora numa constante só.
- **Nada inventado:** o `bridge-demo.test.mjs` lê a mensagem do `#[error("…")]` de `Timeout` em `crates/ddc-core/src/domain/error.rs`, como o `i18n-parity` já lê o `mccs_catalog.rs`, e compara o erro do demo com ela.
- **Testes novos (6):** parsing do `fail=`; escrita que expira depois das checagens, sem gravar nada; `features`/`probe` expiram, e o resto do demo segue respondendo; o monitor é procurado antes da falha; o código sondado sem resposta expira como no core. O teste "outside a local dev server…" ganhou 2 URLs com `fail=`, e fora do servidor local continua tudo `backend_unavailable`.
- `hide_popup` fica fora: o comando real nunca rejeita (`src-tauri/src/commands.rs:233`, só reporta no stderr).

### Estados novos do pseudo-locale (`tests/e2e/pseudo-locale.spec.mjs`)
| Estado | Como chega |
|---|---|
| toast depois de uma escrita de brilho que expirou | `fail=write`, `ArrowRight` no slider |
| toast depois de trocar o preset de cor e expirar | `fail=write`, escolhe "Nativo" no dropdown |
| "Todos os ajustes" ainda carregando (`more.loading`) | variante de teste do `bridge.js` que nunca responde `load_features` |
| "Todos os ajustes" com falha (texto do erro + "Tentar de novo") | `fail=features` |
| "Todos os ajustes" sem nada além dos rápidos (`more.empty`) | variante de teste do `bridge.js` com `load_features` → `[]` |
| sondagem em andamento (`more.probing`) | variante de teste do `bridge.js` que nunca responde `probe_features` |
| toast depois de uma sondagem que expirou | `fail=probe` |

- **Por que variante do `bridge.js`, e não o relógio pausado:** o axe espera timers (é por isso que o estado `loading` antigo não tem axe). Com o comando segurado e o relógio andando, os 3 estados de espera também passam pelo axe. É o mesmo mecanismo de `slider`/`dropdown`/`confirm` (`serve(route, { patch })`), e o spec exige que a linha remendada ainda exista no `bridge.js` (`the demo line this state patches`).
- O estado vazio só existe por variante: nenhum monitor do demo deixa de declarar ajustes, e criar um mudaria cenários presos aos goldens.
- `reach` só espera elementos (o toast visível, `data-kind` do status, `aria-disabled` da sondagem). Julgar o texto continua sendo só do check.
- **Produto atual:** 19 estados em `STATES` + `loading` = 20 estados × 2 temas = 40 (`ps = 40`), mais a origem do app × 2 = 42/42. (Conta corrigida na iter 10: aqui dizia "21 estados × 2 temas + `loading` × 2 + origem × 2", que somaria 46, como o reviewer notou.) Os estados novos não acharam texto real fora das marcas, então `app.js` não mudou. Estabilidade: os estados novos com `--repeat-each=5` deram 70/70.

### Caminhos de falha no Gate 7 (`tests/e2e/scenarios.spec.mjs`, locale pt-BR real)
- **Escrita de brilho e troca de preset** (`fail=write`): toast e announcer = `t('error.timeout')`, com "Tentar de novo" no toast. O slider volta a `75`/`75%`, e o preset a `0x01`/`sRGB`.
- **"Todos os ajustes"** (`fail=features`): `data-kind="error"`, o texto `t('error.timeout')` e "Tentar de novo" no status, nenhum item, nenhum toast.
- **Sondagem** (`fail=probe`): o toast com o texto, o botão de novo disponível e nenhum resultado.
- **Todos** terminam com `writes` vazio, axe e o coletor de console. São 8 testes (4 × 2 temas), e o `--repeat-each=5` deu 40/40.
- Os títulos (`/?demo=rtk&fail=… after …`) não casam com `› <path> settles`, então o `n = 10` do C13 não muda.

### Provas negativas (mutações não commitadas)
- **Onde rodaram:** numa cópia descartável de `apps/ddc-tray` no scratchpad. `src/` e `tests/` foram copiados; `node_modules` e `src-tauri` entraram por symlink.
- Junto rodaram os specs do `HEAD` da iter 8 (`git show 3f8de04:…`) como `old-*`.
- `src/` foi recopiado antes de cada mutação, e a cópia foi apagada no fim. O repo não foi tocado.
- "`node`" = `i18n-html` + `view-model` (41 testes, com o scanner estático).

| Mutação (`src/app.js`, via `const` no topo do módulo) | `node` | pseudo antigo | pseudo novo | cenários antigos | cenários novos |
|---|---|---|---|---|---|
| **M1, a do critic:** `showToast(errorText(error, t) + NOT_APPLIED)` em `writeFailed`, `NOT_APPLIED = ' The monitor kept its previous value.'` | 41 pass | **28 passed** | **4 failed** (escrita de brilho e preset × 2 temas): `text "⟦O monitor não respondeu a tempo.⟧ The monitor kept its previous value." in <span#toast-text.toast-text> has text outside the marks`, idem em `<p#announcer>` | 16 passed | **4 failed** (texto exato) |
| M2: `errorText(…) + ' (see the log)'` no status de "Todos os ajustes" | 41 pass | 28 passed | **2 failed**: `<span#more-status-text> has text outside the marks: " (see the log)"` | 16 passed | **2 failed** |
| M3: `t('more.loading') + ' please wait'` | 41 pass | 28 passed | **2 failed** (ainda carregando) | 16 passed | 24 passed |
| M4: `t('more.empty') + ' Nothing to show.'` | 41 pass | 28 passed | **2 failed** (nada além dos rápidos) | 16 passed | 24 passed |
| M5: `t('more.probing') + ' (reading codes)'` | 41 pass | 28 passed | **2 failed**: `<span#probe-label> has text outside the marks: " (reading codes)"` | 16 passed | 24 passed |
| M6: `errorText(…) + ' Probe stopped.'` no toast da sondagem | 41 pass | 28 passed | **2 failed** (toast e announcer) | 16 passed | **2 failed** |
| M7, comportamento: `writeFailed` sem devolver o controle ao valor do monitor | 41 pass | 28 passed | 42 passed (texto intacto) | 16 passed | **4 failed**: `Expected "75" / Received "76"`, preset `"1"`/`"2"` |

- Os seis literais M1 a M6 passavam no scanner estático e nos specs da iter 8. Agora o pseudo-locale reprova todos, e os cenários de texto exato reprovam M1, M2 e M6 também.

### Harness
- Mudaram nesta iteração `tests/ui/bridge-demo.test.mjs`, `tests/e2e/pseudo-locale.spec.mjs` e `tests/e2e/scenarios.spec.mjs`. Nada foi enfraquecido:
  - só entram testes, estados e asserções;
  - `textsOf`/`judge`, os mínimos de leitura, o teste `loading` e o da origem do app ficaram como estavam;
  - `support.mjs` (coletor de console/pageerror, axe, `expectNoNativeSelect`), `playwright.config.mjs` e `scripts/smoke-sni.sh` não mudaram.
- **Hash novo do harness:** `b911f1d74c2d5178d82e382eefa47363469b8d079dcbe09759c9b88a5d8a6e87`. São 20 arquivos (o manifesto agora inclui `scripts/smoke-sni.sh`, D-8), com a última mudança em `2a8b59d`. Foi calculado com `cd apps/ddc-tray && sha256sum playwright.config.mjs tests/e2e/*.mjs tests/ui/*.mjs scripts/smoke-sni.sh | sha256sum | cut -c1-64`.

### Verify do CONTEXT.md e do PROJECT.md (extraídos por script do `.md`, rodados com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`)
| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo e total | `OK` (141 pass, 0 fail/cancelled/skipped/todo) |
| C7 | i18n paridade/HTML + scanner | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 1567990, `org.kde.StatusNotifierItem-1567990-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated | `OK` (listado, não executado) |
| C13 | Gate 7 | `OK`: 110 passed, 6 skipped (= screenshots), 0 failed/flaky; n = 10, dd = 16/16, dr = 2, ps = 40 (eram 26); caminhos `fail=` 8/8 |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | `OK` |
| C15 | hash do harness | **não rodado, como pedido.** O CONTEXT tem o hash da iter 8 (`0566f416…518a`), e o novo está acima |
| C16 | nenhum `<select>` em `src/` | `OK` |
| C17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | `OK` (PID 1569170, `75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75`) |
| C18 | TODO/FIXME/`todo!` em todo arquivo versionado do produto (regra da D-8) | `OK` |
| C19 | screenshots regenerados, nada pulado, byte a byte iguais | `OK` (6 PNGs, SHA-1 inalterados) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥ 80 % (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78 % (gate com exclusões e `--fail-under-lines 80`: 83.22 %, exit 0) |
| P3 | TODO/FIXME/`todo!` em `*.rs` (regra da D-8) | `OK` |

- **Protocolo dos smokes (C11, C17):** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray` (nunca `-f`).
  - Logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - PIDs do usuário: 1453315 → 1568221 → 1569412, que é a instância viva no fim. Nada em `~/.local` foi tocado.
- **Porta 1420:** livre antes e depois do C13/C19.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 / exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (nenhum `.rs` mudou) |
| `node --test` (todos) | **141 pass** (eram 135), 0 fail/cancelled/skipped/todo |
| Playwright | **110 passed** (eram 88), 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate) | TOTAL lines **83.22 %** |
| `Cargo.lock` / `package-lock.json` | intocados |

### Desvios e observações
- **README/CHANGELOG** (`a6498ee`) não estavam na lista. Descrevem o `fail=` e passam de 13 para 20 estados, porque o DoD manual #24 pede o README fiel ao comportamento atual.
- **Estados além dos pedidos:** "Todos os ajustes" carregando (`more.loading`) também era um texto transitório fora de todo estado, então entrou junto com a sondagem em andamento.
- **Mecanismo de espera:** os estados de espera usam variante de teste do `bridge.js`, não o relógio, para rodarem com axe (ver acima).
- **Nenhum texto real** do produto apareceu fora das marcas nos estados novos, então não houve commit `fix`.
- **Limites de desenho que seguem (registrados nas iters 7/8):**
  - um literal colado a um *dado* sob `translate="no"` não é checado;
  - um literal passado como parâmetro de `t()` sai dentro das marcas, e é coberto pelas asserções exatas dos cenários. Os textos de falha não têm parâmetros.
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` não foi executado. O C11 usou o backend real só com leituras, e o C17 o monitor simulado.

## Iteração 10 (rodada 2) — todos os toasts cobertos

Modo fix, cadeia autônoma do `/jdi-issue`, iter 10 (rodada 2, rodada de warnings). A iter 9 saiu APPROVED_PENDING_MANUAL com o W-2: os toasts de `app.js:200` (falha do `listen()`) e `app.js:207` (falha do `hidePopup()` no Esc) não apareciam em nenhum estado do pseudo-locale, e o mutante do reviewer (literal colado nos dois) passava em todos os gates. O W-1 (`cargo audit`) fica com a `ci-crossbuild`. Não editei o CONTEXT: o C15 ainda tem o hash da iter 9, e o novo está abaixo. As 8 tasks do PLAN continuam `completed`.

### Commits
| Commit | Tipo | O quê |
|---|---|---|
| `2e2c76d` | feat | `fail=events` e `fail=hide` no demo, e o `listen` do bridge Tauri passa a devolver `{ kind, message }`; testes `node --test` |
| `d3e54a6` | fix | o toast da falha do `listen()` não é mais escondido pelo carregamento seguinte (bug achado pelos testes novos, ver abaixo) |
| `2d19645` | test | 2 estados novos no pseudo-locale (com axe) + `TOAST_STATES` |
| `b278478` | test | 2 caminhos novos no Gate 7 (`scenarios.spec.mjs`, locale real): texto exato, console limpo, axe |
| `1988f50` | test | trava estrutural `tests/ui/toast-states.test.mjs` |
| `0244b91` | docs | README e CHANGELOG: `fail=events,hide`, o toast que persiste e os 22 estados |

Depois do `1988f50`, `playwright.config.mjs`, `tests/e2e/*.mjs`, `tests/ui/*.mjs` e `scripts/smoke-sni.sh` não mudaram mais.

### `fail=events` e `fail=hide` no demo (`src/demo-data.js`, `src/bridge.js`)
- `FAILURES` ganhou `events: 'plugin:event|listen'` e `hide: 'hide_popup'`. O `listen` do demo e o `hide_popup` do demo rejeitam quando marcados. O resto do demo continua respondendo, e com `hide` o `__ddcDemo.hides` fica em 0.
- **`{kind, message}` reais.** Esses dois comandos não passam pelo core, e o `hide_popup` em Rust nunca devolve erro (`commands.rs:233`). O único jeito de rejeitarem no app é o próprio Tauri recusar a chamada IPC. Em build release, o Tauri 2.12 rejeita com a string `Command {cmd} not allowed by ACL` (`tauri-2.12.0/src/webview/mod.rs:2112`). O `listen` do JS é o comando `plugin:event|listen` (`scripts/bundle.global.js`). O `normalizeError` do bridge transforma essa string em `{ kind: 'unknown', message }`, e é isso que o demo devolve:
  - `{ kind: 'unknown', message: 'Command hide_popup not allowed by ACL' }`;
  - `{ kind: 'unknown', message: 'Command plugin:event|listen not allowed by ACL' }`.
  - O popup mostra `t('error.unknown')`, porque o `ErrorKind` do Rust não tem (nem precisa ter) um kind para uma recusa que nunca chega ao comando.
- **Bridge Tauri:** o `listen` repassava a rejeição crua (uma string). Agora passa pelo `normalizeError`, como todo `invoke` já passava. O texto na tela não muda, porque uma string também caía em `error.unknown`. A diferença é que o objeto que o popup recebe segue o contrato `{ kind, message }`, e o demo fica idêntico ao app.
- **Testes novos (4):**
  - `fail=events`: as duas inscrições são recusadas, nenhum evento chega e os comandos seguem respondendo;
  - `fail=hide`: o Esc é recusado, nada é escondido e o resto funciona;
  - "the demo's refusals are what the app's bridge makes of Tauri's": compara o demo com um bridge Tauri cujo `invoke`/`listen` rejeitam com as strings do Tauri;
  - o `listen` do Tauri recusado vira `{ kind, message }`.
  - O teste de parsing do `fail=` ganhou as duas palavras. `hide` saiu da lista de palavras desconhecidas; `hidden`, `listen` e `hide_popup` continuam desconhecidas.
  - "outside a local dev server…" ganhou `tauri://localhost/?fail=events,hide` (continua tudo `backend_unavailable`), e o bridge indisponível segue aceitando listeners com `fail=events`.
  - Com o `src/` da iter 9 (via `git stash` só do `src/`, restaurado em seguida), os 5 testes afetados reprovam.

### Bug achado: a falha do `listen()` sumia antes de alguém vê-la (`d3e54a6`)
- **O que a sonda mostrou** (script descartável no scratchpad, servidor na porta 1499): com `fail=events`, o toast aparecia no estado `loading` e ficava `hidden` assim que o painel carregava. O `showLoaded` chama `hideToast()`.
- **No app real é pior:** o popup começa oculto (D-5). O `listen()` falha no start, e o primeiro carregamento, também no start, esconde o toast antes de o usuário abrir o popup. Sem `popup-shown` e `panel-changed`, nada mais recarrega o painel, que passa a mostrar os valores lidos no start sem aviso nenhum.
- **Correção mínima em `app.js`:** `model.toastLasts`. O `listen()` marca o toast dele como duradouro. O `hideToast()` (chamado por carregamentos e escritas bem-sucedidos) não esconde um toast duradouro. O `showToast()` limpa a marca, então outra falha substitui o aviso, e depois dela tudo volta ao normal. O "Tentar de novo" continua lendo o monitor de novo (`refresh`). Nenhuma chamada `showToast(` nova foi criada.
- **Prova de que o teste reproduz o bug:** com os specs novos e o `app.js` sem a correção (via `git stash` só do `app.js`, antes do commit), os 4 testes de `events` reprovaram (pseudo × 2, cenário × 2), com `expect(locator).toBeVisible()`, `Expected: visible`, `Received: hidden`. Os 4 de `hide` passaram. Com a correção, os 8 passam.

### Pseudo-locale (`tests/e2e/pseudo-locale.spec.mjs`)
| Estado | Como chega |
|---|---|
| `rtk after listening to the tray was refused` | `fail=events`; o toast já está visível no `ready` |
| `rtk after hiding the popup was refused` | `fail=hide`, Esc |

- **`TOAST_STATES`:** um item por menção a `showToast` em `src/`, fora a definição. Cada item tem a forma `{ site: 'arquivo › função', state }`, e os sítios de hoje são `app.js › listen`, `app.js › hideOnEscape`, `app.js › writeFailed` e `app.js › probe`. Cada estado listado passa por `toastShows` no loop, antes do check. Um teste do próprio spec (`every toast state is a state checked here, and each has its own`) exige que cada estado exista em `STATES` e que não haja estado repetido.
- `textsOf`/`judge`, os mínimos de leitura, o teste `loading`, o da origem do app e o `support.mjs` não mudaram.
- **Contagem:** 21 estados em `STATES` + `loading` = 22 × 2 temas = 44 (`ps = 44`). Somando a origem do app × 2 e o teste de `TOAST_STATES` × 2, o spec tem 48/48.

### Gate 7 (`tests/e2e/scenarios.spec.mjs`, locale pt-BR real)
- **`fail=events`:** depois de um carregamento que funciona (clique em "Atualizar", esperando o fim com `loadEnds`), o toast e o announcer ainda dizem exatamente `t('error.unknown')`, com "Tentar de novo". O painel está `ready`, com brilho `75`.
- **`fail=hide`:** depois do Esc, o popup continua `ready`, com o mesmo texto exato no toast e no announcer, e `hides` fica em 0.
- Os dois terminam com `writes` vazio, axe e o coletor de console, nos 2 temas (4 testes). Os títulos (`/?demo=rtk&fail=… after …`) não casam com `› <path> settles`, então o `n = 10` do C13 não muda.

### Trava estrutural (`tests/ui/toast-states.test.mjs`)
- **`every toast call site has a pseudo-locale state`:**
  - lê todo script de `src/` (recursivo) e acha cada menção `\bshowToast\b` que não seja a própria declaração;
  - atribui cada menção à última `function` declarada antes dela, ou a `(module)`;
  - exige que a lista ordenada dessas menções seja igual aos `site` de `TOAST_STATES`, lido do spec como texto (importar o spec rodaria o Playwright). Toda linha do bloco precisa ser uma entrada literal `{ site: '…', state: '…' }`, senão reprova;
  - exige que cada `state` seja um `name:` de `STATES` e que não haja estado repetido.
  - Como a comparação é de multiconjunto, qualquer menção a mais reprova: chamada nova, segunda chamada numa função já listada, alias ou chamada em outro script.
- **`only showToast writes the toast's text`:** `toastText` só aparece no mapa `ui` e dentro de `showToast`, e `'toast-text'` só uma vez, em `app.js`. Assim um toast escrito direto no DOM também não escapa.

### Provas negativas (mutações não commitadas)
- **Onde rodaram:** as mutações da trava e o mutante do reviewer rodaram numa cópia descartável de `apps/ddc-tray` no scratchpad (`src/`, `tests/` e a config copiados; `node_modules` e `src-tauri` por symlink). `src/` foi recopiado antes de cada mutação, e a cópia foi apagada no fim.
- **Specs da iter 9:** entraram como `old-*`, via `git show 19f86b3:…`.
- **Via `git stash` no repo:** as duas provas "reprova sem a mudança" (a do bridge demo e a do bug) rodaram com `git stash` no repo, restaurado logo em seguida, antes dos commits.
- "`node`" = `i18n-html` + `view-model` (41 testes, com o scanner estático).

| Mutação | Resultado |
|---|---|
| **M-a (pedida):** `showToast(t('x'));` num sítio novo (`refresh()`, antes de `showState('empty')`) | trava **reprova**: `+ 'app.js › refresh'` |
| M-b: segunda chamada `else showToast(t('x'))` em `probe()` | trava **reprova**: `+ 'app.js › probe'` a mais |
| M-c: alias `const tell = showToast;` no módulo | trava **reprova** (menção a mais) |
| M-d: `export function oops(t) { showToast(t('x')); }` em `dropdown.js` | trava **reprova**: `+ 'dropdown.js › oops'` |
| M-e: `ui.toastText.textContent = t('x')` em `end()` | 2º teste **reprova**: `+ 'app.js › end'` |
| M-f: `document.getElementById('toast-text').textContent = t('x')` | 2º teste **reprova** (2 ocorrências de `toast-text`) |
| S-1 a S-4, no spec: estado inexistente em `TOAST_STATES`, entrada calculada (`state: PROBE_STATE`), dois sítios num estado só, sítio removido da lista | trava **reprova** nos 4 |
| **Mutante do reviewer, nos dois sítios:** `const EVENTS_OFF = ' Changes made from the tray will not show here.'`, com `showToast(errorText(error, t) + EVENTS_OFF)` em `listen` e no Esc | `node` 41 pass. Pseudo novo: **4 failed** / 44 passed (`events` e `hide` × 2 temas), `has text outside the marks: " Changes made from the tray will not show here."` no `#toast-text` e no `#announcer`. Cenários novos: **4 failed** / 24 passed (`Expected: "Algo deu errado."`, `Received: "Algo deu errado. Changes made from the tray will not show here."`). Pseudo antigo (19f86b3): **42 passed**. Cenários antigos: **24 passed** |
| Mutante do reviewer só em `listen` | pseudo novo **2 failed**, cenários novos **2 failed** |
| Mutante do reviewer só no Esc | pseudo novo **2 failed**, cenários novos **2 failed** |
| Correção revertida (`app.js` sem `toastLasts`) | **4 failed** (`events`: pseudo × 2 e cenário × 2), `Received: hidden` |

- Resumo: o mutante que passava por tudo na iter 9 agora reprova no pseudo-locale e nos cenários de texto exato, em cada um dos dois sítios. E um `showToast(` novo sem estado reprova no `node --test` (C6), antes de qualquer navegador.

### Harness
- **Mudaram nesta iteração:**
  - `tests/ui/bridge-demo.test.mjs`;
  - `tests/e2e/pseudo-locale.spec.mjs`;
  - `tests/e2e/scenarios.spec.mjs`;
  - `tests/ui/toast-states.test.mjs`, que é novo.
- **Nada foi enfraquecido:** só entram testes, estados e asserções. `support.mjs`, `playwright.config.mjs` e `scripts/smoke-sni.sh` não mudaram.
- **Hash novo do harness:** `c3976dbfdd2a37fb40ccbd2fb4e01c68049658d9d2dee5ed6fb49582aac037e2`, sobre 21 arquivos (eram 20; entrou o `toast-states.test.mjs`). A última mudança no harness é `1988f50`. Calculado com `cd apps/ddc-tray && sha256sum playwright.config.mjs tests/e2e/*.mjs tests/ui/*.mjs scripts/smoke-sni.sh | sha256sum | cut -c1-64`.

### Verify do CONTEXT.md e do PROJECT.md
Extraídos por script do `.md` e rodados com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`.

| # | Critério | Resultado |
|---|---|---|
| C1 | fmt + clippy `-D warnings` | `OK` |
| C2 | build release `ddc-tray` | `OK` |
| C3 | só `lib.rs` constrói `DdcHiMonitorBackend` (1×) | `OK` |
| C4 | `panel.rs` puro sobre `MonitorControl` | `OK` |
| C5 | `#![forbid(unsafe_code)]`, nenhum `unsafe` | `OK` |
| C6 | `node --test` por módulo e total | `OK` (147 pass, 0 fail/cancelled/skipped/todo) |
| C7 | i18n paridade/HTML + scanner | `OK` |
| C8 | CSP | `OK` |
| C9 | capabilities | `OK` |
| C10 | single-instance 1º no builder | `OK` |
| C11 | smoke `--activate` | `OK` (PID 1658579, `org.kde.StatusNotifierItem-1658579-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras) |
| C12 | teste de hardware `#[ignore]` gated | `OK` (listado, não executado) |
| C13 | Gate 7 | `OK`: 120 passed, 6 skipped (= screenshots), 0 failed/flaky; `n = 10`, `dd = dl = 16`, `sk = sl = 6`, `dr = 2`, `ps = 44` (eram 40); caminhos `fail=` 12/12; pseudo-locale 48/48 |
| C14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | `OK` |
| C15 | hash do harness | **não rodado, como pedido.** O CONTEXT tem o hash da iter 9 (`b911f1d7…6e87`); o novo está acima |
| C16 | nenhum `<select>` em `src/` | `OK` |
| C17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | `OK` (PID 1666329, `75 -> 80` na vertical, nada na horizontal em 1,5 s, `80 -> 75`) |
| C18 | TODO/FIXME/`todo!` em todo arquivo versionado do produto | `OK` |
| C19 | screenshots regenerados, nada pulado, byte a byte iguais | `OK` (SHA-1 dos 6 PNGs inalterados) |
| P1 | `cargo test --workspace --locked` | `OK` (383 passed, 0 failed, 9 ignored) |
| P2 | cobertura ≥ 80 % (literal `cargo llvm-cov --workspace --summary-only`) | TOTAL lines 82.78 % (gate com exclusões e `--fail-under-lines 80`: 83.22 %, exit 0) |
| P3 | TODO/FIXME/`todo!` em `*.rs` | `OK` |

- **Protocolo dos smokes (C11, C17):** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário. Ela foi encerrada com `pkill -x ddc-tray` (nunca `-f`), esperando o processo sair.
  - Logo depois de cada smoke, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - PIDs do usuário: 1596918 → 1658807 → 1666583, que é a única instância viva no fim. Nada em `~/.local` foi tocado.
- **Porta 1420:** livre antes e depois do C13/C19. A sonda usou a 1499, encerrada em seguida.

### Gates (números finais)
| Gate | Resultado |
|---|---|
| `cargo fmt --check` / `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 / exit 0 |
| `cargo test --workspace --locked` | **383 passed**, 0 failed, 9 ignored (nenhum `.rs` mudou) |
| `node --test` (todos) | **147 pass** (eram 141), 0 fail/cancelled/skipped/todo |
| Playwright | **120 passed** (eram 110), 6 skipped (screenshots sem `SCREENSHOTS=1`) |
| cobertura (gate) | TOTAL lines **83.22 %** |
| `Cargo.lock` / `package.json` / `package-lock.json` | intocados |

### Desvios e observações
- **Correção de produto não pedida (`d3e54a6`):** o `fail=events` mostrou um bug real, descrito acima, e a regra do doer é que bugfix começa por teste que reprova. A mudança é mínima (`toastLasts`), só muda o comportamento quando o `listen()` falha, e não cria nenhum `showToast(` novo.
- **`listen` do bridge Tauri normalizado:** não estava na lista. Entrou porque sem ele o `{kind, message}` do demo não seria o que o app real entrega ao popup (seria uma string crua).
- **Kind `unknown`, e não um kind do `ErrorKind` do Rust:** a recusa acontece na camada IPC do Tauri, antes de qualquer comando do app. `unknown` é o kind que o próprio bridge define para "um comando que as capabilities recusam" (doc do `normalizeError`).
- **Sítios em `TOAST_STATES`:** são `arquivo › função` (`listen`, `hideOnEscape`, `writeFailed`, `probe`), e não os apelidos `listen`/`hide`/`write`/`probe` da sugestão. A trava deriva o sítio do código, então o nome é o da função.
- **Travas além da pedida:** `only showToast writes the toast's text` (node) e `every toast state is a state checked here, and each has its own` (Playwright, 2 testes, fora do `ps`).
- **README/CHANGELOG (`0244b91`)** não estavam na lista. Entraram porque o DoD manual #24 pede o README fiel ao comportamento atual (o `fail=` novo, o toast que persiste, 22 estados).
- **Texto do toast:** a falha do `listen()` e a do Esc mostram o genérico "Algo deu errado." (`error.unknown`). Um texto próprio ("as mudanças feitas pela bandeja não vão aparecer aqui") exigiria chaves i18n novas e ficou fora do escopo. Fica como sugestão para uma issue.
- **Contagem da iter 9 corrigida** no próprio texto da iter 9 (20 estados × 2 = 40, mais a origem × 2 = 42, e não "21 × 2 + …").
- **Monitor real:** nenhuma escrita. O `rtk_qhd_hdr` não foi executado. O C11 usou o backend real só com leituras, e o C17 o monitor simulado.
