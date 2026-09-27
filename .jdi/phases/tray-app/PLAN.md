# Phase 5: Tray app — Plan  (slug: tray-app)

## Goal
App Tauri 2 `apps/ddc-tray`: ícone na bandeja (Linux + Windows), popup com seletor de monitor, sliders de brilho/contraste/volume com debounce, troca de entrada, preset de cor e energia. Os comandos Tauri chamam `ddc-core` via `MonitorControl`, e a UI não tem lógica DDC.

## Locked decisions (from CONTEXT.md)
- D-1/D-2 (hexagonal; `ddc-core` só com `thiserror`) valem. tray-app-2: crate `ddc-tray` em `apps/ddc-tray/src-tauri` e UI estática em `apps/ddc-tray/src` (sem bundler, sem npm de runtime); tauri 2.12, tauri-build 2.7, single-instance 2.5, positioner 2.4.
- tray-app-3: apresentação em Rust puro sobre `M: MonitorControl` (fake nos testes); comandos finos com `spawn_blocking`; `run()` monta `SoftwareOsd<CachingMonitorBackend<DdcHiMonitorBackend>>` com `default_cache_dir()`; backend que falha não derruba o app; nomes NC só do core.
- tray-app-4: contrato de comandos e erros `{kind,message}`. tray-app-5: UX do popup, menus por plataforma, debounce de 80 ms e exibição do valor lido de volta. tray-app-6: i18n en/pt-BR por chave (menu da bandeja incluso).
- tray-app-7: CSP estrita, capabilities mínimas, single-instance, `#![forbid(unsafe_code)]`, `Confirm::Yes` só em `src-tauri/src/commands*`. tray-app-8: `has_frontend: true` + Gate 7 = suíte Playwright do app. tray-app-9: validação (fake, `node --test`, smoke SNI, hardware `#[ignore]`); Windows fica para a `ci-crossbuild`.

## Assumptions (cadeia autônoma, sem AskUserQuestion)
- **A-1 Contrato UI↔Rust** (serde camelCase; o JS passa `monitorId`, o Rust recebe `monitor_id`), travado pelo golden `apps/ddc-tray/tests/fixtures/contract-rtk.json`: a T-2 o escreve, e o teste Rust e o `node --test` da demo o leem.
  - Comandos: `list_monitors() → [{id,label,manufacturer,model}]`; `select_monitor(monitorId)`; `load_panel(monitorId) → {monitorId, controls:[Control]}`; `load_features(monitorId)` e `probe_features(monitorId) → [Feature]`; `set_feature(monitorId,code,value,confirmed) → {current,max}` (sempre o valor lido de volta); `hide_popup()`.
  - `Control = {code, key, dangerous, value}`, com `key` = alias do catálogo (`brightness|contrast|volume|input|preset|power`). `value` = `{kind:"continuous",current,max}` ou `{kind:"nonContinuous",current,options:[{value,name}]}`: `current` NC é o byte SL, `options` vem do caps (senão do catálogo) e `name` do catálogo (ou `null`).
  - `Feature = {code, alias, name, dangerous, origin:"caps"|"probe", status:"ok"|"unsupported"|"unresponsive", value|null}`. Erro = `{kind,message}`, com os kinds da D-4. Eventos: `popup-shown` (a UI revalida) e `panel-changed {monitorId}` (depois de um atalho da bandeja).
- **A-2 Fake RTK** (teste Rust e demo): id `RTK-RTK-QHD-HDR-01010101`, caps da fixture do core, brilho 75/100, contraste 50/100, volume 30/100 (fora do caps, responde), entrada 0x0F, preset 0x01, energia 0x01. Os demais valores são fixados uma vez no teste Rust e espelhados na demo.
- **A-3 Energia:** nenhum byte fica em JS. O botão abre o diálogo com as `options` diferentes da atual (nomes do core) e só grava depois da confirmação.
- **A-4 "Todos os ajustes":** `load_features` = códigos RW declarados no caps, menos os 6 rápidos e menos NC sem lista (0x02); no RTK: `0x0C 0x16 0x18 0x1A 0x87 0xCA 0xCC`. `probe_features` = `probe_undeclared_features` filtrado pela mesma regra. Rótulo: a chave `feature.<alias>` quando existe; senão, o nome MCCS do core.
- **A-5 Dep extra:** `sys-locale = "0.3.2"` (D-6: locale do SO no menu, sem `unsafe` no Windows). É o único acréscimo à lista da D-2.
- **A-6 Hardware:** o teste `#[ignore]` segue o Gate 5.7 e fica inerte sem `DDC_HW_TESTS=1`. Como o Verify da CONTEXT não exporta a variável, ele passa sem tocar no monitor. A evidência real é a saída com `DDC_HW_TESTS=1`, registrada no SUMMARY. O orquestrador pode emendar o Verify.
- **A-7 Screenshots:** o spec só roda com `SCREENSHOTS=1` (sem ela, `skip`), para o Gate 7 não reescrever os PNGs versionados.
- **A-8 Windows:** ramos por `cfg!(windows)` (o código é checado no Linux) ou por `#[cfg(windows)]` mínimo. Nada é compilado para msvc nesta phase (D-9).

## Tasks
Specialist único: `jdi-doer-ddc-control` (glob `**/*`). Scope `tray-app`, com a D-XX citada no corpo do commit. Nunca código e `.jdi/` no mesmo commit; nunca `--no-verify`, `JDI_ALLOW_MIXED` ou `JDI_GATE_DISABLE`. Todo commit mantém verdes:
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` e `cargo test --workspace --locked`;
- `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` (o tray fica fora, D-9);
- a partir da T-4: `node --test 'apps/ddc-tray/tests/ui/**/*.test.mjs'`.

Lições das phases anteriores: teste fixa valor por igualdade (o critic da phase 4 derrubou um teste oco); mutações pedidas são provadas no SUMMARY, sem commit; em hardware, só escritas Safe reversíveis e sem `sudo`, nunca entrada, energia ou qualquer `dangerous`.

### Wave 1

#### T-1: Scaffold do crate `ddc-tray`: Cargo, config Tauri, capabilities e ícones
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `Cargo.toml`, `Cargo.lock`, `apps/ddc-tray/src-tauri/Cargo.toml`, `apps/ddc-tray/src-tauri/build.rs`, `apps/ddc-tray/src-tauri/src/main.rs`, `apps/ddc-tray/src-tauri/src/lib.rs`, `apps/ddc-tray/src-tauri/tauri.conf.json`, `apps/ddc-tray/src-tauri/capabilities/default.json`, `apps/ddc-tray/src-tauri/icons/icon.svg`, `apps/ddc-tray/src-tauri/icons/*` (gerados + `tray.png`), `apps/ddc-tray/src/index.html` (placeholder sem texto)
- **Acceptance:**
  - **Crate e deps:** entra em `members` como package `ddc-tray` (lib `ddc_tray` + `[[bin]] ddc-tray`), com `*.workspace = true` e `[lints] workspace = true`. Deps: as da D-2 com as features dela (tauri `tray-icon`+`image-png`, positioner `tray-icon`), mais serde (derive), serde_json, `sys-locale` (A-5) e `ddc-core`/`ddc-adapters` via workspace. O fake é sempre compilado, e só testes o usam. `crates/` fica intocado.
  - **`main.rs`:** `windows_subsystem` + `ddc_tray::run()`; erro → `eprintln!` + `ExitCode::FAILURE`, sem `expect`. **`lib.rs`:** `#![forbid(unsafe_code)]` compila com `generate_context!`; se um macro do Tauri colidir com o `forbid`, a task fica `blocked` (não troca por `deny`).
  - **`tauri.conf.json`** (conforme as notas do orquestrador): sem `version` (herda do Cargo), `frontendDist: "../src"`, sem `devUrl`, `withGlobalTauri`, janela `popup` 360×560 oculta e sem decoração, CSP exata. `bundle.active: false`, mas com `bundle.icon` completo (o Windows exige `icon.ico`). **`capabilities/default.json`:** janela `popup`, só `core:event:default`.
  - **Ícones:** `icon.svg` próprio (monitor colorido, legível em barra clara e escura), passado por `npx -y @tauri-apps/cli@2.12.0 icon`; o conjunto gerado é commitado, junto com o `tray.png` da bandeja.
- **Dependencies:** none
- **Test:** Verify da CONTEXT de CSP, capabilities, `forbid` e `cargo build -p ddc-tray --release --locked`
- **Status:** completed

### Wave 2

#### T-2: Apresentação pura (`panel.rs`) + DTOs (`dto.rs`) + golden do contrato
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/lib.rs`, `apps/ddc-tray/src-tauri/src/panel.rs`, `apps/ddc-tray/src-tauri/src/panel/tests.rs`, `apps/ddc-tray/src-tauri/src/dto.rs`, `apps/ddc-tray/src-tauri/src/dto/tests.rs`, `apps/ddc-tray/tests/fixtures/contract-rtk.json`
- **Acceptance:**
  - **`panel.rs`**, genérico em `M: MonitorControl + ?Sized` e sem `tauri` nem em comentário (DoD grep): `monitors`; `load_panel` (0x10, 0x12, 0x62, 0x60, 0x14 e 0xD6, nessa ordem; um controle cuja leitura falha some, e `MonitorNotFound` vira erro); `load_features`/`probe_features` (A-4); `ui_error(DdcError) → UiError` (tabela da D-4); `brightness_for_percent(max, pct)` (arredonda e limita); `set_brightness_percent` (lê o max e grava com `Confirm::No`). Alias, nome, risco e opções vêm só do core (`mccs_catalog`, `Feature`).
  - **`dto.rs`:** os tipos serde do A-1 (união com `#[serde(tag = "kind")]`), mapeados a partir de tipos do core.
  - **Testes com o fake** (`InMemoryMonitorBackend::builder()` + caps da fixture do core, A-2), todos por igualdade: o painel RTK inteiro; volume presente; 0x60 e 0x14 com as 7 opções do caps e seus nomes, e 0xD6 com 3; leitura com `Timeout` → o controle some e o resto fica; `MonitorNotFound` → `not_found`; lista A-4 exata; percentuais 0/25/50/75/100 com max 100 e com max 80; todo `DdcError` → o kind esperado.
  - **Golden:** `{monitors, panel, features}` do fake RTK em `serde_json` == `contract-rtk.json` (diff no erro; regenerar só de propósito).
- **Dependencies:** T-1
- **Test:** Verify da CONTEXT de `panel.rs` + `cargo test -p ddc-tray --locked`
- **Status:** completed

### Wave 3 (parallel-eligible)

#### T-3: Comandos Tauri + composition root + ciclo do popup + teste de hardware RTK
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/lib.rs`, `apps/ddc-tray/src-tauri/src/commands.rs`, `apps/ddc-tray/src-tauri/src/commands/tests.rs`, `apps/ddc-tray/src-tauri/src/popup.rs`, `apps/ddc-tray/src-tauri/build.rs`, `apps/ddc-tray/src-tauri/capabilities/default.json`, `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs`
- **Acceptance:**
  - **`commands.rs`:** helpers síncronos, genéricos e testados, mais os 7 comandos do A-1 como `#[tauri::command] async fn` finos, com `spawn_blocking`. `confirmed` → `Confirm` só aqui (o único `Confirm::Yes` fora de testes). `AppState { osd: Result<Arc<dyn MonitorControl + Send + Sync>, UiError>, selected: Mutex<Option<MonitorId>> }`: sem backend, todo comando responde `backend_unavailable` e o app segue de pé.
  - **`lib.rs`:**
    - `pub fn compose_osd()` tem o único `DdcHiMonitorBackend::new` (que não aparece em nenhum comentário) e monta o `SoftwareOsd` sobre `CachingMonitorBackend` com `default_cache_dir()`; sem diretório, fica sem cache, como o CLI;
    - `run() -> Result<(), tauri::Error>`, com `tauri_plugin_single_instance` como 1º plugin (a 2ª execução mostra o popup);
    - o popup se esconde em `Focused(false)` e no `CloseRequested`, e mostrá-lo emite `popup-shown`.
  - **`popup.rs`** (puro, testado): gate da corrida blur↔clique; um clique até 300 ms depois de um hide por blur não reabre o popup. **Permissões:** `build.rs` com `AppManifest::commands`, e a capability ganha o `allow-*` dos 7 comandos (allow-list explícita; o DoD de capabilities continua OK).
  - **Testes com o fake:** 0x60 sem `confirmed` → `needs_confirmation`, sem `WriteVcp`; com `confirmed` → `WriteVcp` + read-back; com `ignoring_writes_to`, o valor devolvido é o lido, não o pedido; valor fora da lista → `invalid_value`; estado sem backend → `backend_unavailable`; alvo do atalho = o monitor selecionado, senão o 1º da lista. Mutação no SUMMARY: `Confirm::Yes` fixo derruba o teste de `needs_confirmation`.
  - **Hardware:** `rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored`, `#[ignore]` e com `DDC_HW_TESTS=1` (A-6).
    - Passa por `compose_osd()` sem nomear o adapter e carrega o painel RTK com 6 controles (7 entradas, 7 presets).
    - Faz UMA escrita de brilho (original ±10, dentro do max, `confirmed=false`); um guard `Drop` restaura e confere o read-back = original. Nunca `confirmed=true`.
    - O doer roda com `DDC_HW_TESTS=1 … --nocapture` e registra a saída no SUMMARY.
- **Dependencies:** T-2
- **Test:** Verify da CONTEXT de `DdcHiMonitorBackend::new`, single-instance e hardware + `cargo test -p ddc-tray --locked`
- **Status:** pending

#### T-4: Módulos JS puros (bridge demo, i18n, debounce, view-model) + `node --test`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src/bridge.js`, `apps/ddc-tray/src/demo-data.js`, `apps/ddc-tray/src/debounce.js`, `apps/ddc-tray/src/view-model.js`, `apps/ddc-tray/src/i18n/index.js`, `apps/ddc-tray/src/i18n/en.js`, `apps/ddc-tray/src/i18n/pt-BR.js`, `apps/ddc-tray/tests/ui/debounce.test.mjs`, `apps/ddc-tray/tests/ui/view-model.test.mjs`, `apps/ddc-tray/tests/ui/bridge-demo.test.mjs`, `apps/ddc-tray/tests/ui/contract.test.mjs`, `apps/ddc-tray/tests/ui/i18n-parity.test.mjs`
- **Acceptance:**
  - **Módulos:** ES modules sem deps, com `window`/`navigator`/timers injetados (testáveis em Node); os testes resolvem caminhos por `import.meta.url`.
  - **`bridge.js`:** usa o Tauri quando `__TAURI__` existe; senão, entra em demo por `?demo=rtk|two-monitors|empty|error` (padrão `rtk`) com o contrato A-1. Na demo: `dangerous` sem `confirmed` → `needs_confirmation`; valor fora das `options` → `invalid_value`; a resposta é o read-back; as escritas ficam em `window.__ddcDemo.writes` (só no modo demo); `error` faz `list_monitors` rejeitar com `backend_unavailable`.
  - **`debounce.js`:** 80 ms, coalescência no último valor, no máximo 1 escrita em voo por `code`, o pendente sai quando ela termina, `flush(code)` no `change`, e um erro não trava a fila.
  - **`view-model.js`:** DTO → modelo de render. Sliders com `percent`/`valueText`, entrada segmentada, preset, energia (A-3) e "Todos os ajustes". Estados `loading|ready|empty|error` com `retry`; a dica de `/dev/i2c` → `docs/linux-ddc-setup.md` aparece só em Linux. Rótulo de valor pela chave `value.<slug do nome do core>`, com fallback para o nome cru.
  - **`i18n/`:** `en` e `pt-BR` com as mesmas chaves (UI, `value.*`, `feature.*`, `error.<kind>`). `resolveLocale` (`pt*` → pt-BR, senão en). `t()` cai para en e, depois, para a própria chave.
  - **Testes** (nenhum em `src/`), com pelo menos 20 `test()` verdes: coalescência (5 inputs em menos de 80 ms → 1 escrita com o último; a 2ª espera a 1ª; `flush` manda o valor final; com `mock.timers`); view-model e demo por cenário; `contract.test.mjs`, em que o RTK da demo `deepEqual` o golden; paridade de chaves, sem valor vazio. Mutação no SUMMARY: sem coalescência, o teste de 1 escrita falha.
- **Dependencies:** T-2
- **Test:** Verify da CONTEXT de `node --test` e i18n
- **Status:** pending

### Wave 4 (parallel-eligible)

#### T-5: Bandeja: ícone, menu bilíngue, eventos, positioner + smoke SNI
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/lib.rs`, `apps/ddc-tray/src-tauri/src/tray.rs`, `apps/ddc-tray/src-tauri/src/menu.rs`, `apps/ddc-tray/src-tauri/src/i18n.rs`, `apps/ddc-tray/scripts/smoke-sni.sh`
- **Acceptance:**
  - **Módulos puros e testados:** `i18n.rs` com `Locale::from_tag` (`pt*` → pt-BR, senão en) e os rótulos en/pt-BR num struct (paridade por construção). `menu.rs` com `menu_entries(locale, platform)`: no Windows, Abrir painel e Sair; no Linux, Abrir painel, Brilho 0/25/50/75/100% e Sair. Os testes cobrem as 2 plataformas e a ida e volta `MenuAction` ↔ id.
  - **`tray.rs`:**
    - `sys_locale` só aqui; ícone `tray.png`, tooltip e o menu de `menu_entries`;
    - clique esquerdo `Up` (Windows) alterna o popup em `TrayBottomCenter`, com o gate de `popup.rs` e o `on_tray_event` do positioner; no Linux, o popup sai sem ancoragem (Wayland), ignorando o erro de posição;
    - atalho de brilho: `spawn_blocking` → alvo → `panel::set_brightness_percent` (Safe) → emite `panel-changed`, com erro só no log; "Sair" → `app.exit(0)`.
  - **`smoke-sni.sh <bin>`** (CONTEXT Notes), com `set -euo pipefail`:
    - sem watcher, falha com mensagem clara;
    - aceita tanto entrada `:1.N/…` quanto nome bem conhecido, e confere o PID via `GetConnectionUnixProcessID`;
    - exige o processo vivo 2 s depois do registro, e encerra com trap SIGTERM + `wait`;
    - falha se houver `panicked` ou se o app sair cedo (outra instância), com mensagem explícita.
  - `cargo llvm-cov` TOTAL ≥ 80% com o glue do Tauri, com o número colado no SUMMARY.
- **Dependencies:** T-3
- **Test:** Verify da CONTEXT do smoke SNI + `cargo test -p ddc-tray --locked`
- **Status:** pending

#### T-6: UI do popup: HTML, CSS Fluent-like, `app.js` e diálogo de confirmação
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src/index.html`, `apps/ddc-tray/src/styles.css`, `apps/ddc-tray/src/app.js`, `apps/ddc-tray/src/icons.js`, `apps/ddc-tray/src/i18n/en.js`, `apps/ddc-tray/src/i18n/pt-BR.js`, `apps/ddc-tray/tests/ui/i18n-html.test.mjs`
- **Acceptance:**
  - **`index.html`:** sem nenhum texto literal; texto e atributos acessíveis entram por `data-i18n`/`data-i18n-attr`. Só `<script type="module" src="app.js">`: por causa da CSP, nenhum script ou `style` inline e nenhum `setAttribute('style')` (a trilha usa `style.setProperty('--fill')`).
  - **Layout (D-5):**
    - cabeçalho com o monitor, que vira `select` quando há mais de um;
    - sliders `input[type=range]` com SVG inline próprio, valor e `aria-valuetext`; entrada como `radiogroup`; preset como `select`; botão de energia;
    - `<details>` "Todos os ajustes", carregado sob demanda, com "Sondar" + progresso; rodapé com estado ou erro e "Tentar de novo".
    - Esc → `hide_popup`. O skeleton aparece só na 1ª carga; depois, a UI revalida em `popup-shown`/`panel-changed` e mostra o read-back.
  - **Diálogo de confirmação:** `<dialog>` modal com foco no Cancelar; Esc cancela; o texto descreve o efeito. Vale para todo `dangerous` (entrada, energia e "Todos os ajustes"), e nada chega ao bridge antes de confirmar.
  - **`styles.css`:** tokens em `:root`, claro/escuro por `prefers-color-scheme` e `prefers-reduced-motion`; cartões com raio de 12px, trilha preenchida, foco visível, contraste AA e fonte `"Segoe UI Variable", system-ui`.
  - **`i18n-html.test.mjs`:** o `index.html` não tem texto nem atributo acessível literal, e toda chave de `data-i18n*` e de `t('…')` em `app.js` existe nos 2 locales.
- **Dependencies:** T-4
- **Test:** Verify da CONTEXT de `node --test` e i18n + os 5 `critical_paths` abertos à mão via `python3 -m http.server`, sem erro de console
- **Status:** pending

### Wave 5

#### T-7: Suíte Playwright (Gate 7) + screenshots claro/escuro
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/package.json`, `apps/ddc-tray/package-lock.json`, `apps/ddc-tray/.gitignore`, `apps/ddc-tray/playwright.config.mjs`, `apps/ddc-tray/tests/e2e/scenarios.spec.mjs`, `apps/ddc-tray/tests/e2e/keyboard.spec.mjs`, `apps/ddc-tray/tests/e2e/confirm.spec.mjs`, `apps/ddc-tray/tests/e2e/screenshots.spec.mjs`, `docs/screenshots/tray-popup-light.png`, `docs/screenshots/tray-popup-dark.png`
- **Acceptance:**
  - **Pacote:** `package.json` `private`, `"type": "module"`, só `devDependencies` exatas (`@playwright/test` 1.63.0, `@axe-core/playwright` 4.13.0), com o lock gerado por `npm install --ignore-scripts`. `.gitignore` com `node_modules/`, `test-results/` e `playwright-report/`. `src/` intocado.
  - **Config:** `webServer` `python3 -m http.server 1420 --directory src` (`reuseExistingServer`), `baseURL` `:1420`, viewport 360×560, Chromium, projetos `light` e `dark` (`colorScheme`).
  - **`scenarios`:** em `/`, `?demo=rtk`, `two-monitors`, `empty` e `error`, e de novo com o diálogo aberto: nenhum console error nem `pageerror`; o estado esperado aparece; axe com as tags `wcag2a`/`wcag2aa`/`wcag21aa` e `expect(critical+serious).toEqual([])` (um assert, não um log).
  - **`keyboard`:** o Tab chega ao slider de brilho; `ArrowRight` muda o valor e o `aria-valuetext`; depois do debounce, `__ddcDemo.writes` tem exatamente 1 escrita em 0x10.
  - **`confirm`:** outra entrada abre o diálogo com 0 escritas; Cancelar → 0 escritas e nada muda; Confirmar → 1 escrita `{code:0x60, confirmed:true}`, e a UI mostra o read-back. Mutação no SUMMARY: sem o diálogo, o spec falha.
  - **`screenshots`** (A-7): `?demo=rtk`, animações desligadas → `docs/screenshots/tray-popup-{light,dark}.png`, commitados.
- **Dependencies:** T-6
- **Test:** Verify da CONTEXT do Gate 7 e dos screenshots
- **Status:** pending

### Wave 6

#### T-8: Docs: bloco `frontend:` do PROJECT (D-8), README, CHANGELOG, roteiro de hardware
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.jdi/PROJECT.md`, `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`
- **Acceptance:** o commit não leva nenhum arquivo de código (o hook aceita `.jdi/` + docs).
  - **`.jdi/PROJECT.md`:** muda só o bloco `frontend:`: `has_frontend: true`, `frontend_url: http://localhost:1420`, `dev_command: python3 -m http.server 1420 --directory apps/ddc-tray/src`, `critical_paths` = os 5 caminhos da T-7, e um comentário citando a D-2026-09-26-tray-app-8.
  - **README (en):**
    - status na fase 5; seção "Tray app" com os screenshots, `cargo run -p ddc-tray` e o build release;
    - deps de build no Linux (webkit2gtk4.1, libappindicator-gtk3 e librsvg2, com os `-devel` do Fedora e os equivalentes do Debian); `/dev/i2c` → `docs/linux-ddc-setup.md`;
    - como usar (menu da bandeja no Linux; cliques esquerdo e direito no Windows; confirmação do que é `dangerous`; en/pt-BR), a demo no navegador (`?demo=`) e os testes (`node --test`, Playwright); Layout com `apps/ddc-tray`;
    - Known limitations: Wayland sem posicionamento, SNI sem clique, GNOME precisa da extensão AppIndicator, Windows não compilado nem testado em hardware, sem autostart/hotkeys (`profiles-hotkeys`), sem instalador.
  - **CHANGELOG** `[Unreleased]` → Added: o tray app. **`docs/hardware-validation.md`:** seção do tray com o smoke SNI, o teste `DDC_HW_TESTS=1 … rtk_qhd_hdr` e o que nunca fazer pelo popup (entrada, energia, `dangerous`).
- **Dependencies:** T-5, T-7
- **Test:** `grep -q 'has_frontend: true' .jdi/PROJECT.md && grep -q 'cargo run -p ddc-tray' README.md && grep -q 'tray-popup-light.png' README.md && grep -q '^## \[Unreleased\]' CHANGELOG.md`
- **Status:** pending

## Execution
- 8 tasks em 6 waves; W3 (T-3 ‖ T-4) e W4 (T-5 ‖ T-6) são paralelizáveis; speedup ≈ 1.3x. Tipos: T-1 `build`; T-2 a T-6 `feat`; T-7 `test`; T-8 `docs`.
- DoD da CONTEXT por task: fmt/clippy → todas; release build, `forbid`, CSP e capabilities → T-1; `panel.rs` → T-2; `DdcHiMonitorBackend::new`, single-instance e hardware → T-3; `node --test` → T-4 + T-6; i18n → T-4 (paridade) e T-6 (HTML); smoke SNI → T-5; Gate 7 e screenshots → T-7; CHANGELOG/README (manual) → T-8.
- Adiado para o PR (CONTEXT): o visual, o Windows real, o GNOME e A-6 (o Verify de hardware sem `DDC_HW_TESTS=1`).

## Riscos
- **R-1 KDE Wayland/SNI:** sem clique no ícone, sem posicionamento, e o foco/blur pode esconder o popup logo que ele abre. O smoke só prova o registro no watcher, não a UX, então o doer valida à mão no KDE e descreve no SUMMARY.
- **R-2 Windows nunca compilado nesta phase** (`tauri-winres`/`llvm-rc`): o código Windows (clique, positioner, `icon.ico`) só é provado na `ci-crossbuild`. Mitigação: A-8.
- **R-3 Cobertura ≥ 80% com o glue do Tauri:** `lib.rs`, `tray.rs` e os wrappers ficam sem cobertura. Mitigação: lógica nos módulos puros e medição na T-5.

## Files modified (all tasks)
- `Cargo.toml`, `Cargo.lock`; `apps/ddc-tray/src-tauri/{Cargo.toml,build.rs,tauri.conf.json}`, `capabilities/default.json`, `icons/*`, `tests/rtk_qhd_hdr.rs`
- `apps/ddc-tray/src-tauri/src/{main,lib,panel,dto,commands,popup,tray,menu,i18n}.rs`, `src/{panel,dto,commands}/tests.rs`
- `apps/ddc-tray/src/{index.html,styles.css,app.js,icons.js,bridge.js,demo-data.js,debounce.js,view-model.js}`, `src/i18n/{index,en,pt-BR}.js`
- `apps/ddc-tray/tests/ui/*.test.mjs`, `tests/e2e/*.spec.mjs`, `tests/fixtures/contract-rtk.json`; `apps/ddc-tray/{package.json,package-lock.json,.gitignore,playwright.config.mjs}`, `scripts/smoke-sni.sh`
- `docs/screenshots/tray-popup-{light,dark}.png`, `docs/hardware-validation.md`, `README.md`, `CHANGELOG.md`, `.jdi/PROJECT.md`

## Test requirements
- Rust: `cargo test --workspace --locked`; os 14 Verify auto da CONTEXT imprimem OK. Hardware, só pelo doer: `DDC_HW_TESTS=1 cargo test -p ddc-tray --locked -- --ignored rtk_qhd_hdr --test-threads=1 --nocapture`.
- JS: `node --test 'apps/ddc-tray/tests/ui/**/*.test.mjs'` (≥ 20, 0 falhas); `cd apps/ddc-tray && npm ci --ignore-scripts && npx playwright test`.
- Cobertura ≥ 80% de linhas: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`.
