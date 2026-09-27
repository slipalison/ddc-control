# Phase 5: Tray app — Context  (slug: tray-app)

## Goal
App Tauri 2 `apps/ddc-tray`: ícone na bandeja (Linux + Windows), popup com seletor de monitor, sliders brilho/contraste/volume com debounce, troca de entrada, preset de cor, energia; comandos Tauri chamam `ddc-core` via `MonitorControl` — nada de lógica DDC na UI.

## Locked decisions
- D-2026-09-26-tray-app-1: roadmap reordenado — `tray-app` passa a vir antes de `ci-crossbuild`/`release-packaging` (motivo: eles empacotam o tray, que ainda não existe).
- D-2026-09-26-tray-app-2: `apps/ddc-tray/src-tauri` = crate `ddc-tray` no workspace; `apps/ddc-tray/src` = UI web estática (HTML/CSS/ES modules), SEM bundler e SEM npm de runtime (`frontendDist: "../src"`, sem `devUrl`); deps tauri 2.12 (`tray-icon`,`image-png`), tauri-build 2.7, tauri-plugin-single-instance 2.5, tauri-plugin-positioner 2.4 (`tray-icon`); sem global-shortcut/autostart (YAGNI, pertence a `profiles-hotkeys`).
- D-2026-09-26-tray-app-3: hexagonal no tray — apresentação (montar painel, nomes NC, mapeamento de erro) em módulos Rust puros genéricos sobre `M: MonitorControl`, testados com `InMemoryMonitorBackend`; `#[tauri::command]` finos em `commands.rs` (`async fn` + `spawn_blocking`); composition root monta `SoftwareOsd` sobre `CachingMonitorBackend<DdcHiMonitorBackend>` com `default_cache_dir()` (mesmo cache do CLI); nomes de valor NC sempre do core (sem tabela MCCS duplicada em JS).
- D-2026-09-26-tray-app-4: contrato de comandos — `list_monitors`, `load_panel(monitor_id)` (brilho/contraste/volume/entrada/preset/energia, C→`{current,max}`, NC→`{current,options}`), `set_feature(id,code,value,confirmed)` devolve valor lido de volta, `confirmed` só após diálogo in-app para `Dangerous`, `load_features`/sondagem sob demanda; erros `{kind,message}` com kinds estáveis.
- D-2026-09-26-tray-app-5: UX do popup (~360px, sem decoração, oculto no start, esconde no blur/Esc); Windows = clique esquerdo alterna popup ancorado (positioner) / clique direito = menu; Linux/SNI = sem clique, menu com atalhos de brilho 0/25/50/75/100%; layout, tema claro/escuro, teclado/AA, debounce 80ms coalescendo, ícone SVG versionado.
- D-2026-09-26-tray-app-6: i18n en+pt-BR via chave (`data-i18n`/`t()`), zero string hardcoded, teste de paridade de chaves e de "todo texto vem de chave"; menu da bandeja também bilíngue.
- D-2026-09-26-tray-app-7: CSP estrita sem `unsafe-inline`, capabilities mínimas (sem shell/fs/http/opener), single-instance ativo, `#![forbid(unsafe_code)]` em `src-tauri`, `Confirm::Yes` só em `src-tauri/src/commands*`.
- D-2026-09-26-tray-app-8: flip `frontend.has_frontend: true` em `.jdi/PROJECT.md` (bridge JS fake em modo demo quando `window.__TAURI__` ausente) — Gate 7 deixa de ser SKIPPED; `frontend_url: http://localhost:1420`, `dev_command: python3 -m http.server 1420 --directory apps/ddc-tray/src`; edição do bloco `frontend:` é task do plano.
- D-2026-09-26-tray-app-9: validação — testes Rust com fake, `node --test` para módulos JS puros, smoke Linux via StatusNotifierWatcher, teste `#[ignore]` de hardware no RTK (painel + 1 escrita Safe restaurada); Windows só em `ci-crossbuild` (probe local: `cargo check --target x86_64-pc-windows-msvc` do tray falha em `tauri-winres`/`llvm-rc` ausente, sem sudo pra corrigir).

## Canonical refs
- Card: pedido do usuário via `/jdi-issue`, 2026-09-26 — "aplicativo no tray... Windows e Linux... bonito e intuitivo" + "crie um fluxo de CI e Release automático" (CI/Release fora desta phase). Objetivo global: "controlar tudo do monitor".
- `.jdi/roadmap/tray-app.md`; `.jdi/decisions/D-2026-09-26-tray-app-{1..9}.md` (fatos de hardware/pacotes/probe).
- `crates/ddc-core/src/ports/monitor_control.rs`; `crates/ddc-cli/src/main.rs` (composition root de referência); `crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt`.
- `.jdi/agents/jdi-doer-ddc-control.md` §§ `apps/ddc-tray`, Monitor-write safety; `.jdi/agents/jdi-reviewer-ddc-control.md` §§ Gate 5.8/5.9, Gate 7 (flip previsto), cross-check Windows (tray excluído de propósito).
- `.jdi/PROJECT.md` §Frontend (flag a flipar por esta phase, D-2026-09-26-tray-app-8).

## Out of scope
- CI, release, bundles/instaladores (MSI/NSIS, deb/rpm/AppImage), regra udev empacotada -> phases `ci-crossbuild`/`release-packaging`.
- Hotkeys globais, perfis, autostart -> phase `profiles-hotkeys`.
- Notificações do sistema, auto-update do app.
- Build/execução real no Windows (só `#[cfg(windows)]` mínimo aqui; prova em `ci-crossbuild`/PR).
- GNOME sem extensão AppIndicator (limitação conhecida, não corrigida nesta phase).

## Definition of Done

### Auto-verifiable
- [ ] `cargo fmt --check` e `cargo clippy --workspace --all-targets -- -D warnings` limpos incluindo o crate `ddc-tray`.
      **Verify:** `cargo fmt --check && cargo clippy --workspace --all-targets --locked -- -D warnings && echo OK`
      **Source:** CONTEXT
- [ ] `ddc-tray` compila em release no Linux.
      **Verify:** `cargo build -p ddc-tray --release --locked && echo OK`
      **Source:** CONTEXT
- [ ] Composition root é o único ponto que constrói `DdcHiMonitorBackend` (adapter real nunca instanciado em `commands.rs`/`panel.rs`).
      **Verify:** `n=$(grep -RIn 'DdcHiMonitorBackend::new' apps/ddc-tray/src-tauri/src | wc -l); [ "$n" = "1" ] && grep -q 'DdcHiMonitorBackend::new' apps/ddc-tray/src-tauri/src/lib.rs && echo OK`
      **Source:** CONTEXT
- [ ] Módulo de apresentação (`panel.rs`) é Rust puro sobre `MonitorControl`, sem importar o runtime do Tauri.
      **Verify:** `test -f apps/ddc-tray/src-tauri/src/panel.rs && ! grep -nE '\btauri(::|_)' apps/ddc-tray/src-tauri/src/panel.rs | grep -q . && echo OK`
      **Source:** CONTEXT
- [ ] `#![forbid(unsafe_code)]` presente em `ddc-tray/src-tauri`.
      **Verify:** `grep -RIlq '#!\[forbid(unsafe_code)\]' apps/ddc-tray/src-tauri/src/lib.rs && echo OK`
      **Source:** CONTEXT
- [ ] `node --test` cobre debounce/coalescência do slider, o view-model de apresentação e o bridge demo, com zero falhas (testes FORA de `src/`, que é o `frontendDist` embutido no binário).
      **Verify:** `o=$(node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/**/*.test.mjs' 2>&1); echo "$o" | grep -qE '^# pass ([2-9][0-9]|[1-9][0-9]{2,})$' && echo "$o" | grep -qE '^# fail 0$' && ! find apps/ddc-tray/src -name '*.test.*' | grep -q . && echo OK`
      **Source:** CONTEXT
- [ ] Paridade de chaves i18n entre `en` e `pt-BR` e ausência de texto hardcoded no HTML fora das chaves.
      **Verify:** `o=$(node --test --test-reporter=tap 'apps/ddc-tray/tests/ui/i18n*.test.mjs' 2>&1); echo "$o" | grep -qE '^# pass [1-9]' && echo "$o" | grep -qE '^# fail 0$' && echo OK`
      **Source:** CONTEXT
- [ ] CSP do tray não permite `unsafe-inline` em script e restringe `script-src` a `'self'`.
      **Verify:** csp=$(jq -r '.app.security.csp' apps/ddc-tray/src-tauri/tauri.conf.json); echo "$csp" | grep -qE "script-src[^;]*'self'" && ! echo "$csp" | grep -q 'unsafe-inline' && echo OK
      **Source:** CONTEXT
- [ ] Capabilities do tray não concedem `shell`/`fs`/`http`/`opener`.
      **Verify:** ! jq -r '.. .identifier? // empty' apps/ddc-tray/src-tauri/capabilities/*.json 2>/dev/null | grep -qE '^(shell|fs|http|opener):' && echo OK
      **Source:** CONTEXT
- [ ] `tauri-plugin-single-instance` registrado no composition root.
      **Verify:** `grep -q 'tauri_plugin_single_instance' apps/ddc-tray/src-tauri/src/lib.rs && echo OK`
      **Source:** CONTEXT
- [ ] Smoke Linux/KDE: o binário release sobe, registra no StatusNotifierWatcher um item cuja conexão D-Bus pertence AO PRÓPRIO processo (PID conferido via `org.freedesktop.DBus.GetConnectionUnixProcessID`), continua vivo depois do registro, e não imprime `panicked` no stderr; o script encerra o app ao final.
      **Verify:** `cargo build -p ddc-tray --release --locked -q && bash apps/ddc-tray/scripts/smoke-sni.sh target/release/ddc-tray && echo OK` (o script sai ≠0 em qualquer uma das condições violadas, com timeout de registro de 15 s)
      **Source:** CONTEXT
- [ ] Teste `#[ignore]` de hardware no monitor "RTK QHD HDR" existe, compila e é gated por `DDC_HW_TESTS=1` (painel carregado só com leituras + UMA escrita Safe de brilho, restaurada ao valor original; nada Dangerous é gravado). O reviewer NUNCA o roda (regra do reviewer); o orquestrador o roda nesta máquina com `DDC_HW_TESTS=1` e anexa a evidência no corpo do PR (mesma prática de `ddc-backends`/`full-osd-control`).
      **Verify:** `cargo test -p ddc-tray --locked -- --ignored --list 2>&1 | grep -qi 'rtk_qhd_hdr' && grep -RqE 'DDC_HW_TESTS' apps/ddc-tray/src-tauri/tests && echo OK`
      **Source:** CONTEXT
- [ ] Gate 7 (frontend-validator): zero erros de console e zero violações axe critical/serious nos `critical_paths` de demo.
      **Verify:** `cd apps/ddc-tray && npm ci --ignore-scripts --no-audit --no-fund --silent && o=$(npx playwright test --reporter=list 2>&1); echo "$o" | grep -qE '[1-9][0-9]* passed' && ! echo "$o" | grep -qE '[0-9]+ (failed|flaky)' && echo OK`
      **Source:** CONTEXT
- [ ] Screenshots do popup (tema claro e escuro, cenário RTK) gerados pela suíte Playwright e versionados para o revisor do PR.
      **Verify:** `for f in docs/screenshots/tray-popup-light.png docs/screenshots/tray-popup-dark.png; do test -s "$f" && file "$f" | grep -q 'PNG image' || exit 1; done && echo OK`
      **Source:** CONTEXT

### Manual
- [ ] CHANGELOG.md updated with entry per release
      **Verify:** human confirmation required
      **Evidence:** new `## [version]` heading in CHANGELOG.md for current release
      **Source:** PROJECT
- [ ] README accurately describes current behavior
      **Verify:** human confirmation required
      **Evidence:** README diff reviewed in PR
      **Source:** PROJECT

## Deferred to PR review
- Julgamento humano do visual "bonito e intuitivo" (screenshots claro/escuro commitadas em `docs/` para o revisor).
- Execução real do teste de hardware `rtk_qhd_hdr` com `DDC_HW_TESTS=1` (orquestrador roda e cola a saída no PR).
- Teste no Windows real: popup ancorado no ícone, tema claro/escuro, DDC via `ddc-winapi`/dxva2 nunca exercitado em hardware nesta phase.
- GNOME sem extensão AppIndicator (SNI não aparece — limitação conhecida, não corrigida aqui).
- Confirmação manual dos 2 itens `Manual` (CHANGELOG/README) da baseline do PROJECT.

## Notes
- Nome de arquivo mandatado por esta CONTEXT (não é liberdade do planner, ao contrário da convenção usual): módulo de apresentação puro em `apps/ddc-tray/src-tauri/src/panel.rs` — necessário para o DoD grep-based de pureza funcionar deterministicamente.
- Testes JS ficam FORA de `apps/ddc-tray/src/` (tudo em `src/` é embutido no binário como `frontendDist`): `apps/ddc-tray/tests/ui/*.test.mjs` (lógica de UI, `node --test`, sem dependências) e `apps/ddc-tray/tests/ui/i18n*.test.mjs` (paridade + nenhum texto hardcoded no HTML).
- Validação de UI (Gate 7) como suíte Playwright do próprio app: `apps/ddc-tray/package.json` `private: true` só com `devDependencies` fixadas em versão exata (`@playwright/test` 1.63.0, `@axe-core/playwright` 4.13.0) + `package-lock.json` commitado, instaladas com `npm ci --ignore-scripts`; `apps/ddc-tray/playwright.config.mjs` sobe `python3 -m http.server 1420 --directory src` como `webServer`; specs em `apps/ddc-tray/tests/e2e/`. Isso NÃO contradiz D-2026-09-26-tray-app-2 (zero npm de RUNTIME: nada disso entra em `src/`). Chromium do Playwright 1.63.0 já instalado user-level pelo orquestrador (`~/.cache/ms-playwright`, sem sudo). `node_modules/` e `test-results/`/`playwright-report/` no `.gitignore`.
- Smoke SNI: `apps/ddc-tray/scripts/smoke-sni.sh <binário>` (bash) — lista os itens do `org.kde.StatusNotifierWatcher` antes, sobe o app, espera até 15 s por um item novo cuja conexão (`:1.N`) tenha o PID do app, confere que o processo segue vivo, encerra com SIGTERM e falha se o stderr tiver `panicked`.
- Baseline de `.jdi/PROJECT.md` (`cargo test --workspace` verde, cobertura >=80% incluindo `apps/`, sem TODO/FIXME sem issue) herdada automaticamente pelo Gate 8 do reviewer — não duplicada aqui.
