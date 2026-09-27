# Phase 5: Review  (slug: tray-app)

**Verdict:** BLOCKED

Iteração 1. Revisor: `jdi-reviewer-ddc-control`, 2026-09-26, branch `phase/tray-app` (HEAD `2d13758`), host Linux (Fedora 44, KDE Plasma Wayland). Baseline do lock/diff: `6f4341b` (último commit antes da phase).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0 (inclui `ddc-tray`); cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` exit 0; `--target x86_64-unknown-linux-gnu` exit 0. `ddc-tray` fora do msvc de propósito (D-2026-09-26-tray-app-9). Só o aviso future-incompat de `nom v3.2.1` (via `ddc-hi`, anterior à phase). |
| Tests | PASS | 339 passed, 0 failed, 8 ignored (hardware). Início da phase: 247 (T-1), ou seja, +92, sem queda. |
| Coverage | PASS | 88.70% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded) |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; nenhum `#[allow(...)]` fora de testes |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 limpos; 5.10: `cargo audit` sai ≠0 (1 vulnerabilidade anterior à phase + 2 avisos novos, via Tauri/GTK). Ver W-1. |
| Consistency | BLOCK | B-1: um monitor com DDC/CI mudo carrega como painel "pronto" e vazio (D-2026-09-26-tray-app-5, -4). Os desvios de plano estão documentados (W-3). |
| UI Validation | PASS | apps/ddc-tray Playwright suite: 36 passed, 4 skipped (screenshots sem `SCREENSHOTS=1`); console errors 0; axe critical/serious 0 (moderate/minor 0) |
| DoD | PASS_PENDING_MANUAL | 17/17 auto, 2 manual pending |

## Blockers (if any)

- **B-1 — Gate 6 / D-2026-09-26-tray-app-5 (+ D-2026-09-26-tray-app-4): `apps/ddc-tray/src-tauri/src/panel.rs:49-60`.** `load_panel` devolve `Ok(PanelDto { controls: [] })` quando TODAS as leituras dos 6 controles rápidos falham com qualquer erro diferente de `MonitorNotFound` (`Err(_) => {}`, linha 54). O próprio doer provou isso na T-8 (`load_panel(mute TV) = Ok(... controls: [])`), e a leitura do código confirma.
  - **O que acontece no app real** (lido em `app.js`/`view-model.js`, com a LG TV listada em 1º, como na saída de hardware da T-3):
    - `firstAnswering` (`view-model.js:77-88`) só pula um monitor quando a promessa rejeita. Com o `Ok` vazio, a TV "responde".
    - `showLoaded` (`app.js:261-274`) chama `select_monitor(TV)`, e os atalhos de brilho do menu passam a mirar a TV. No Linux/SNI esses atalhos são a interação principal, e aqui falham em silêncio, só com uma linha no stderr.
    - `rememberMonitor` grava a TV no `localStorage`, então toda abertura seguinte tenta a TV primeiro e para nela.
    - `showState('ready')` pinta um painel sem nenhum controle. O cabeçalho mostra "LG TV SSCR2" / "GSM · DDC/CI", ou seja, afirma DDC/CI. Sobra só o "All settings" recolhido, sem mensagem, sem dica e sem "Try again".
  - **Por que contradiz as decisões travadas:**
    - A D-5 trava o estado de erro com "Tentar de novo" e as dicas de DDC/CI e de `/dev/i2c`. `statusView` mostra a dica de i2c para `timeout|transport`, e as chaves `header.noAnswer`, `header.silent` e `hint.ddc` existem justamente para esse caso. Com o backend real, nenhuma delas é alcançável para um monitor listado e mudo.
    - O caso mais comum é ainda pior: um único monitor com DDC/CI desligado no OSD. A enumeração só lê EDID (D-2026-09-25-ddc-backends-4), então ele é sempre listado e abre em branco, sem a dica "ative o DDC/CI".
    - A D-4 exige erros em `{kind,message}`, mas uma falha total não chega à UI como erro.
    - Critério "intuitivo" do card: na máquina do usuário, a experiência padrão é um painel vazio na TV e atalhos de brilho que não fazem nada.
  - **O Gate 7 não pega o defeito:** o `fallback.spec.mjs` fica verde porque o bridge demo (`bridge.js:136-140`) rejeita o monitor silencioso com `timeout`, coisa que o `load_panel` em Rust nunca faz (ele só rejeita com `not_found`). O golden `contract-rtk.json` cobre só o RTK, então a divergência demo↔Rust passa sem detecção. O fallback anunciado no CHANGELOG ("else the first whose panel loads") não funciona na máquina para a qual foi construído.
  - **Correção (pequena e no escopo da phase):**
    1. Em `load_panel`, guardar o 1º erro que não seja `MonitorNotFound`. Se nenhum controle carregar, devolver `Err(ui_error(primeiro_erro))`. A falha parcial continua como está, conforme o texto da D-4.
    2. Teste por igualdade em `panel/tests.rs`: os 6 códigos rápidos em `Transport` → `Err(UiError { kind: Transport, message: … })`. O teste de falha parcial atual (contraste em `Timeout` → 5 controles) continua.
    3. Recomendado: alinhar o kind do monitor silencioso da demo ao que o Rust devolve (a TV real dá `Transport`) e fixar o caso mudo no contrato, com teste Rust e `contract.test.mjs`, para a demo não voltar a divergir.
    4. Retirar a limitação "A monitor whose DDC/CI is mute opens as an empty panel" (`README.md:352`), a ressalva de `README.md:310` e a nota de `docs/hardware-validation.md:182`.
    
    Um monitor que responda `unsupported` para os 6 códigos também passaria a ser erro (`unsupported`). Isso é aceitável, porque não haveria nada a mostrar no painel rápido.

## Warnings (if any)

- **W-1 — 5.10 supply chain (`cargo audit` sai ≠0).**
  - **Novos nesta phase**, via tauri 2.12 → gtk-rs 0.18, só no Linux:
    - RUSTSEC-2024-0429: `glib 0.18.5` unsound (`VariantStrIter`);
    - RUSTSEC-2024-0370: `proc-macro-error 1.0.4` sem manutenção (build-time, via `glib-macros`).
  - **Anteriores à phase**, via `ddc-hi 0.4.1` → `mccs-db 0.1.3`:
    - RUSTSEC-2018-0005: `serde_yaml 0.7.5` (abort por recursão; só lê o DB embutido do `mccs-db`, não entrada do usuário);
    - RUSTSEC-2024-0320: `yaml-rust 0.4.5` sem manutenção.
  - Não há correção possível do lado do tray, porque depende do upstream do Tauri e do `ddc-hi`. Na `ci-crossbuild`, um passo de audit vai falhar: registre um `audit.toml` com cada ignore justificado.
- **W-2 — Gate 8, item C9 (capabilities): o `Verify` é oco.**
  - `jq -r '.. .identifier? // empty'` nunca lê as permissões, que no arquivo são strings. Uma capability mutante com `"shell:allow-execute"`, `"fs:default"` e `"opener:default"` passou no Verify (JSON de teste montado no scratchpad).
  - A capability real está limpa: conferida à mão, tem só `core:event:default` e os 7 `allow-*`. Por isso o item fica PASS.
  - Sugestão de Verify: `! jq -r '.permissions[] | if type=="string" then . else .identifier end' apps/ddc-tray/src-tauri/capabilities/*.json | grep -qE '^(shell|fs|http|opener):' && echo OK`.
- **W-3 — Gate 6, consistência do plano:** os desvios estão todos documentados no SUMMARY, então não há ação a tomar.
  - Arquivos fora do `files_modified`: `tests/e2e/support.mjs`, `tests/e2e/fallback.spec.mjs`, `src-tauri/.gitignore`, `docs/screenshots/tray-popup-dialog-light.png` e `demo-data.js` alterado na T-6.
  - `Position::TrayCenter` no lugar de `TrayBottomCenter`, justificado pela D-5 ("acima do ícone").
  - O re-exec sem DMA-BUF, que virou a D-2026-09-26-tray-app-10.
  - Todos os commits usam o escopo `tray-app` com tipo coerente, e todo arquivo do plano aparece no log.
- **W-4 — Gate 3 (baixa prioridade):** `lib.rs` tem 0% (74 linhas), `commands.rs` 52.14% e `tray.rs` 62.65%. É a cola do Tauri, prevista no R-3, e o TOTAL fica folgado. A única decisão sem teste ali é a checagem da variável em `restart_without_dmabuf_renderer` (`lib.rs:95-110`). Se o arquivo for mexido de novo, vale extraí-la para uma função pura.

Observações sem severidade:
- **Hexagonal:**
  - `ddc-core` continua só com `thiserror`;
  - `panel.rs` é puro sobre `MonitorControl`;
  - `DdcHiMonitorBackend::new` só aparece em `compose_osd` (`lib.rs:43`);
  - `Confirm::Yes` só aparece em `commands.rs:90`;
  - os 7 `#[tauri::command]` são `async fn`, e os que tocam DDC passam por `spawn_blocking`;
  - não há `unsafe`: a única ocorrência é um comentário em `lib.rs:91`;
  - `#![forbid(unsafe_code)]` está em `lib.rs` e em `main.rs`.
- **Teste de hardware (5.7):** `rtk_qhd_hdr` é `#[ignore]` e fica inerte sem `DDC_HW_TESTS=1`. O revisor não o rodou, conforme a regra.
- **`demo-data.js`:** traz nomes MCCS literais como fixture da demo, não como tabela de consulta. Só o `bridge.js` o importa em modo demo, e o `contract.test.mjs` o prende ao golden. Isso é compatível com a D-3 e com o A-2.
- **Gate 7 (julgamento manual):**
  - sliders são `input[type=range]` nativos com `aria-valuetext`;
  - `prefers-color-scheme` e `prefers-reduced-motion` estão presentes em `styles.css`;
  - há foco visível e nenhuma string fora dos locales (testes i18n verdes);
  - o CSP do Tauri é injetado pela suíte, e o axe dá `toEqual([])` nos 5 `critical_paths`, nos 2 diálogos e em "All settings" sondado.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` — 339 passed, 0 failed, 8 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL lines 88.70% (saída do Gate 3, reaproveitada) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | exit 0, `OK` |
| 4 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Composition root é o único ponto que constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1 ocorrência, `lib.rs:43`) |
| 7 | `panel.rs` é Rust puro, sem runtime do Tauri | CONTEXT | Auto | PASS | `OK` |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` (`lib.rs:8`) |
| 9 | `node --test` (debounce, view-model, bridge demo), 0 falhas, testes fora de `src/` | CONTEXT | Auto | PASS | `OK` — `# tests 83`, `# pass 83`, `# fail 0`; nenhum `*.test.*` em `src/` |
| 10 | Paridade i18n en/pt-BR e HTML sem texto literal | CONTEXT | Auto | PASS | `OK` — `# pass 24`, `# fail 0` |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK`; Verify oco (W-2), critério conferido à mão |
| 13 | `tauri-plugin-single-instance` no composition root | CONTEXT | Auto | PASS | `OK` |
| 14 | Smoke Linux/KDE via StatusNotifierWatcher (PID conferido, vivo, sem `panicked`) | CONTEXT | Auto | PASS | `OK` — PID 394307 dono de `:1.3103/org/ayatana/NotificationItem/tray_icon_tray_app_ddc_control`; `pgrep -x ddc-tray` vazio depois |
| 15 | Teste `#[ignore]` de hardware RTK existe, compila, gated por `DDC_HW_TESTS=1` | CONTEXT | Auto | PASS | `OK` (não executado pelo revisor; o orquestrador roda com `DDC_HW_TESTS=1` e cola a saída no PR) |
| 16 | Gate 7: zero erros de console e zero axe critical/serious | CONTEXT | Auto | PASS | `OK` — 36 passed, 4 skipped |
| 17 | Screenshots claro/escuro versionados | CONTEXT | Auto | PASS | `OK` — 2 PNGs de 720×1120 |
| 18 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` → Added com o item `ddc-tray` (`CHANGELOG.md:40-48`); ainda não há heading `## [version]` (nenhum release) |
| 19 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Tray app` (`README.md:265`) e `### Known limitations of the tray app` (`README.md:347`); `README.md:352` descreve o defeito B-1 e deve sair junto com a correção |

**Totals:** 19 items | Auto: 17 (17 PASS, 0 FAIL) | Manual: 2 pending

Os 2 itens Manual aparecem nas duas DoD (PROJECT e CONTEXT, `Source: PROJECT`) e foram contados uma vez.

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation

Uma rodada curta de correção resolve o bloqueio.
- **B-1:**
  - `load_panel` passa a devolver o 1º erro quando nenhum controle rápido é lido;
  - 1 teste Rust por igualdade;
  - de preferência, a demo alinhada ao kind real e o caso mudo fixado no contrato;
  - retirar a limitação do README e do roteiro de hardware.
  
  Depois da correção, validar à mão, nesta máquina, que o popup abre no RTK com a TV listada em 1º, marcada "(no DDC/CI)", e que os atalhos de brilho miram o RTK. Isso é só leitura mais o fluxo já coberto. Nenhuma escrita perigosa.
- **Opcionais para a mesma rodada:**
  - W-2: consertar o Verify de capabilities na CONTEXT;
  - W-1: registrar os advisories para a `ci-crossbuild`.
- Os demais gates estão verdes e não há regressão em `crates/`.
- Antes do `/jdi-ship`:
  - o orquestrador roda o teste de hardware com `DDC_HW_TESTS=1` e cola a saída no PR;
  - os 2 itens Manual vão para `/jdi-confirm-dod tray-app`.

## DoD Critic (enhanced)

- DoD row «15 (teste de hardware)»: `grep -qi 'rtk_qhd_hdr'` casa com a linha `Running tests/rtk_qhd_hdr.rs` do próprio cargo (stderr juntado); sem `#[ignore]`, sem o gate `if !hardware_enabled()` ou até com erro de compilação o Verify imprime OK (3 mutações demonstradas).
- DoD row «12 (capabilities)»: `.. .identifier?` não lê permissões em forma de string — `shell:allow-execute`/`fs:default`/`http:default`/`opener:default` passaram (confirma W-2).
- DoD row «11 (CSP)»: `script-src 'self' 'unsafe-eval' https://cdn.jsdelivr.net *` passa; o critério é "restringe a 'self'".
- DoD row «16 (Gate 7)»: a leitura da saída ignora exit code e `skipped` — `test.skip(true)` no loop de critical paths dá OK sem axe nos 5 caminhos.
- DoD row «3 (TODO/FIXME)»: `--include='*.rs'` não lê os 34 arquivos JS/HTML/CSS/MJS/SH da phase — `// TODO` sem issue em `src/app.js` passou.
- DoD row «8 (forbid unsafe)»: casa com o atributo comentado (`// #![forbid(unsafe_code)]`) + `#[allow(unsafe_code)] unsafe {…}` compila e passa.
- DoD row «13 (single-instance)»: `.plugin(tauri_plugin_single_instance::init(..))` comentado ainda passa.
- DoD row «7 (panel.rs puro)»: não cobre "sobre `MonitorControl`" — `pub type RealOsd = SoftwareOsd<CachingMonitorBackend<DdcHiMonitorBackend>>` em `panel.rs` passa.
- DoD row «17 (screenshots)»: dark = cópia do light, ou PNG preto 1×1, passam; não confere versionamento.
- DoD row «9 (node --test)»: só conta ≥20 pass — apagar `debounce.test.mjs` e `bridge-demo.test.mjs` ainda dá 52 pass e OK.

O código real atende os 17 critérios (conferido linha a linha pelo critic); a oquidão é dos comandos `Verify:` — correção a cargo do orquestrador no CONTEXT.md antes da iter 2.

**Verdict:** BLOCKED
