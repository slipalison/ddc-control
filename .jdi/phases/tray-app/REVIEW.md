# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 2. Revisor: `jdi-reviewer-ddc-control`, 2026-09-27, branch `phase/tray-app` (HEAD `12c7ddb`), host Linux (Fedora 44, KDE Plasma Wayland). Re-verificação completa (gates 1-8) depois das correções `b110026..12c7ddb` e do endurecimento dos `Verify:` em `9bc74c0`. O REVIEW da iter 1 (BLOCKED) está no histórico (`git show 86f93c9:.jdi/phases/tray-app/REVIEW.md`).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0 (inclui `ddc-tray`). Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked` exit 0 em `--target x86_64-unknown-linux-gnu` e em `x86_64-pc-windows-msvc`. `ddc-tray` fica fora do msvc de propósito (D-2026-09-26-tray-app-9). Único aviso: o future-incompat de `nom v3.2.1` (via `ddc-hi`), anterior à phase. |
| Tests | PASS | 344 passed, 0 failed, 9 ignored (hardware). Na iter 1 eram 339 + 8: entraram +5 (2 do B-1, 1 do golden mudo, 2 do W-4) e +1 ignored de hardware. Nenhuma queda. |
| Coverage | PASS | 88.79% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded). `main.rs` do tray inalterado, só a cola `run()` → `ExitCode`. |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; nenhum `#[allow(...)]` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 limpos (ver abaixo). 5.10: `cargo audit` sai ≠0 com os mesmos 4 advisories da iter 1 (W-1). O `Cargo.lock` não mudou na iter 2. |
| Consistency | PASS | O B-1 da iter 1 está resolvido e conforme a D-2026-09-26-tray-app-4/-5. Nenhuma violação de D-1, D-2, D-2026-09-26-tray-app-3/-7/-10. Todos os `files_modified` do PLAN aparecem no log, e os 26 commits usam o escopo `tray-app` com tipo coerente. |
| UI Validation | PASS | Suíte Playwright de `apps/ddc-tray`: 36 passed, 4 skipped (screenshots sem `SCREENSHOTS=1`). Console errors 0, axe critical/serious 0, moderate/minor 0 (nenhuma linha `axe moderate/minor` impressa). A porta 1420 ficou livre depois. |
| DoD | PASS_PENDING_MANUAL | 18/18 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1–5.4 (hexagonal):**
  - `ddc-core` continua só com `thiserror`, sem I/O e sem `cfg` de plataforma;
  - nenhum `impl MonitorBackend` no core e nenhum `pub trait` fora dele;
  - nenhum adapter é construído fora dos composition roots. `DdcHiMonitorBackend::new` aparece uma única vez, em `apps/ddc-tray/src-tauri/src/lib.rs:47` (`compose_osd`).
  - `crates/` não foi tocado na iter 2 (`git diff 86f93c9..HEAD -- crates/` vazio).
- **5.5 (`unsafe`):** a única ocorrência é o comentário `lib.rs:95`. `#![forbid(unsafe_code)]` está em `lib.rs:8` e em `main.rs`.
- **5.6 (pânicos):** o único hit fora de testes é o doc comment `crates/ddc-adapters/src/ddc_hi_backend/worker.rs:141`, anterior à phase. Os `unwrap`/`expect` de `tests/rtk_qhd_hdr.rs` estão em arquivo de teste.
- **5.7 (escrita segura):**
  - a classificação `Dangerous` existe no domínio;
  - `Confirm::Yes` só aparece em `commands.rs:90`;
  - os 7 testes de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs` são `#[ignore]` e gated por `hardware_enabled()` / `DDC_HW_TESTS`;
  - o teste novo `rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads` só lê. O revisor não rodou nenhum deles.
- **5.8 / 5.9 / 5.11:**
  - nenhum caminho de dispositivo no core;
  - nenhum `#[tauri::command]` sem `async fn`;
  - nenhum segredo e nenhum TODO/FIXME sem issue, nem em `*.rs` nem nos arquivos não-Rust do tray.

## Blockers (if any)
- Nenhum.

## Warnings (if any)
- **W-1 — 5.10 supply chain (`cargo audit` 0.22.2 sai ≠0).** Continua como na iter 1. O endereçamento é conhecido e fica com a phase `ci-crossbuild` (`audit.toml`, com cada ignore justificado).
  - **Novos nesta phase**, via tauri 2.12 → gtk-rs 0.18, só no Linux:
    - RUSTSEC-2024-0429: `glib 0.18.5` unsound;
    - RUSTSEC-2024-0370: `proc-macro-error 1.0.4` sem manutenção.
  - **Anteriores à phase**, via `ddc-hi` → `mccs-db`:
    - RUSTSEC-2018-0005: `serde_yaml 0.7.5`;
    - RUSTSEC-2024-0320: `yaml-rust 0.4.5`.

Observações sem severidade:
- **B-1 (iter 1) resolvido.** `load_panel` (`apps/ddc-tray/src-tauri/src/panel.rs:48-70`) guarda o 1º erro que não seja `MonitorNotFound` (linha 59). Quando nenhum controle rápido é lido, devolve `Err(ui_error(..))` (linha 64); a falha parcial continua `Ok`.
  - Os testes são por igualdade:
    - `a_mute_monitor_fails_its_panel_instead_of_loading_it_empty` (`panel/tests.rs:355`);
    - `a_panel_with_no_control_fails_with_its_first_reading_error` (`:371`), em que brilho `Transport`, contraste `Timeout` e resto `unsupported` provam que é o 1º erro, e não o último;
    - `mute_contract_matches_the_golden` (`:840`);
    - o teste de falha parcial (`:343`) continua verde.
  - A demo (`demo-data.js`/`bridge.js`) agora rejeita a TV com o mesmo `{kind,message}` do Rust. O `contract.test.mjs` a prende ao `contract-mute.json`, e o `fallback.spec.mjs` fica verde pelo caminho certo (`error.transport` + `hint.ddc` + `header.noAnswer`).
  - O README não traz mais a limitação "mute opens as an empty panel".
- **W-4 (iter 1) resolvido.**
  - A decisão do re-exec é a função pura `dmabuf_switch_to_set` (`lib.rs:120`), conforme a D-2026-09-26-tray-app-10: só `None` → `"1"`, e qualquer valor do usuário, inclusive vazio ou não-UTF-8, é mantido.
  - Ela tem 2 testes sem `set_var` (`lib.rs:173`).
  - `lib.rs` foi de 0% para 17.78%. O resto é cola do Tauri, como `commands.rs` (52.14%) e `tray.rs` (62.65%), previsto no R-3.
- **W-2 (iter 1) resolvido** pelo `9bc74c0`: o Verify de capabilities agora lê as permissões em forma de string. As 8 permissões reais são `core:event:default` e os 7 `allow-*`.
- **DRY (menor, só em teste).** A mensagem real da TV (`transport error: DDC/CI I2C error: … (gave up after attempt 3 of 3)`) aparece literal em 4 lugares:
  - `panel/tests.rs:33`, que é a fonte;
  - `contract-mute.json`, que é gerado;
  - `demo-data.js:162`, uma cópia inevitável, porque `src/` não lê `tests/fixtures`, e presa pelo `contract.test.mjs`;
  - `tests/ui/bridge-demo.test.mjs:85`.
  
  A última poderia ler o `contract-mute.json`, como o `contract.test.mjs` já faz. Não é bloqueio: uma divergência derruba os testes.
- **Teste de hardware do monitor mudo:** `rtk_qhd_hdr.rs:169-171` trata todo monitor que não seja o RTK como mudo e exige pelo menos um. Em outra máquina, com um 2º monitor que responda, ele falharia. É específico da máquina de dev por desenho e está documentado no passo 7 de `docs/hardware-validation.md`. O revisor não o rodou.
- **UX conhecida:** na 1ª abertura com a TV listada em 1º, o popup fica cerca de 5 s em "carregando" antes de cair no RTK. Está documentado no `README.md:310` e no passo 8 do roteiro, e a otimização fica para uma phase futura (SUMMARY, iter 2).

## DoD Checklist (gate 8)

Cada `Verify:` do CONTEXT.md foi extraído literalmente e executado com `bash` a partir da raiz do repo. Os itens 1 e 2 do PROJECT reaproveitam a saída dos Gates 2 e 3.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0 (Gate 2): 344 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL lines 88.79% (Gate 3, exit 0 com `--fail-under-lines 80`) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | exit 0 |
| 4 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Composition root é o único ponto que constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1 ocorrência, `lib.rs:47`) |
| 7 | `panel.rs` é Rust puro sobre `MonitorControl`, sem o runtime do Tauri | CONTEXT | Auto | PASS | `OK` (`use ddc_core::ports::MonitorControl;`, 5 `pub fn …<M: MonitorControl + ?Sized>`) |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` (`lib.rs:8` e `main.rs`, sem `allow(unsafe_code)` nem `unsafe {`) |
| 9 | `node --test` (debounce, view-model, bridge demo…), 0 falhas, testes fora de `src/` | CONTEXT | Auto | PASS | `OK`: os 6 módulos passam, e o total dá `# tests 84`, `# pass 84`, `# fail 0` |
| 10 | Paridade i18n en/pt-BR e HTML sem texto literal | CONTEXT | Auto | PASS | `OK`: `# pass 24`, `# fail 0` |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` (`core:event:default` + 7 `allow-*`) |
| 13 | `tauri-plugin-single-instance` registrado no composition root | CONTEXT | Auto | PASS | `OK` (1º `.plugin(` em `lib.rs:68`) |
| 14 | Smoke Linux/KDE via StatusNotifierWatcher (PID conferido, vivo, sem `panicked`) | CONTEXT | Auto | PASS | `OK`: o PID 488781 é dono de `:1.3413/org/ayatana/NotificationItem/tray_icon_tray_app_ddc_control`, estava vivo 2 s depois e nunca entrou em pânico; `pgrep -x ddc-tray` vazio antes e depois |
| 15 | Teste `#[ignore]` de hardware RTK existe, compila, gated por `DDC_HW_TESTS=1`, sem escrita Dangerous | CONTEXT | Auto | PASS | `OK`: `--ignored --list` mostra os 2 testes `rtk_qhd_hdr*`. Não foram executados pelo revisor; o orquestrador cola a saída no PR |
| 16 | Gate 7: zero erros de console e zero axe critical/serious nos `critical_paths` | CONTEXT | Auto | PASS | `OK`: exit 0, 10/10 `critical_paths` × tema com ✓, `36 passed`, `4 skipped` |
| 17 | Nenhum `TODO`/`FIXME` sem issue nos arquivos não-Rust do tray | CONTEXT | Auto | PASS | `OK` |
| 18 | Screenshots claro/escuro versionados, 720×1120, diferentes, claro claro e escuro escuro | CONTEXT | Auto | PASS | `OK` (média de cinza: light 0.938, dark 0.167) |
| 19 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (`CHANGELOG.md:8`) com o item `ddc-tray` (`:40`); ainda não há heading `## [version]` (nenhum release) |
| 20 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Tray app` (`README.md:265`), o fallback de monitor mudo em `:310`, os goldens em `:341` e `### Known limitations of the tray app` (`:347`) sem a limitação do B-1 |

**Totals:** 20 items | Auto: 18 (18 PASS, 0 FAIL) | Manual: 2 pending

Os 2 itens Manual aparecem nas duas DoD (PROJECT e CONTEXT, `Source: PROJECT`) e foram contados uma vez.

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation

A phase está pronta para o ship, pendente só das confirmações humanas.
- **O B-1 está corrigido com testes por igualdade** e contrato fixado dos 2 lados (Rust e demo). Os gates 1–4 e 6–7 estão verdes, sem regressão em `crates/`.
- **Antes do `/jdi-ship`:**
  1. `/jdi-confirm-dod tray-app` para os 2 itens Manual (CHANGELOG/README).
  2. O orquestrador roda `DDC_HW_TESTS=1 cargo test -p ddc-tray --locked --test rtk_qhd_hdr -- --ignored --test-threads=1 --nocapture` e cola a saída no PR. A saída da iter 2 já está no SUMMARY, e o PR pode reaproveitá-la se nada mudar.
  3. Validação manual sugerida para o PR (passo 8 de `docs/hardware-validation.md`): o popup abre no RTK com a TV marcada "(no DDC/CI)".
- **Opcional, em qualquer rodada futura:** fazer o `bridge-demo.test.mjs:85` ler a mensagem do `contract-mute.json` em vez de repeti-la.
- **W-1** segue para a `ci-crossbuild`.

## DoD Critic (enhanced)

- DoD row «3 (TODO/FIXME em *.rs — PROJECT)»: `grep -vE '#[0-9]+'` trata `{code:#04X}`/ids `…#2` como link de issue — `// TODO` numa linha que já tem `#04X` passa.
- DoD row «6 (composition root)»: `use ddc_adapters::DdcHiMonitorBackend as RealBackend;` + `RealBackend::new()` em `commands.rs` compila e passa (só conta a string `DdcHiMonitorBackend::new`).
- DoD row «7 (panel.rs puro)»: `use crate::{AppHandle, Runtime}` (reexport privado de `lib.rs`) ou submódulo `panel/runtime.rs` com `use tauri::…` passam.
- DoD row «8 (forbid)»: o atributo dentro de comentário de bloco `/* … */` passa; o scan de `unsafe` não lê `tests/`.
- DoD row «9 (node --test)»: ignora exit code e `# cancelled` — teste que estoura timeout dá `# fail 0` + `# cancelled 1` + exit 1 e o Verify imprime OK; `src/*.spec.mjs` escapa do `find`.
- DoD row «10 (i18n)»: mesma cegueira a `# cancelled`; o glob só exige 1 pass.
- DoD row «11 (CSP)»: `tauri.linux.conf.json` com CSP permissiva é mesclada pelo Tauri e embutida no binário; o Verify só lê `tauri.conf.json`.
- DoD row «12 (capabilities)»: capability em lista nomeada, em subdiretório de `capabilities/`, ou inline em `tauri.linux.conf.json` passam (o `jq` falha em silêncio).
- DoD row «13 (single-instance)»: registro sob `#[cfg(not(target_os = "linux"))]` passa.
- DoD row «15 (hardware)»: o gate só precisa aparecer uma vez no arquivo; um segundo teste sem gate escrevendo `RESTORE_FACTORY_DEFAULTS` pela porta driven passa.
- DoD row «16 (Gate 7)»: `reuseExistingServer: true` na porta fixa 1420 — um servidor antigo servindo outro `src/` faz a suíte validar a UI errada.
- DoD row «17 (TODO não-Rust)»: cor hex `#22d3ee` mascara TODO em CSS; `*.toml` e `.gitignore` fora do `--include`.
- DoD row «18 (screenshots)»: PNGs brancos/pretos 720×1120 passam — não prova que o conteúdo é o popup atual.

Todos demonstrados em cópias descartáveis; a árvore real atende os 18 critérios. Correção dos `Verify:` a cargo do orquestrador (+ D-XX para o Verify de TODO do PROJECT, que é LOCKED); ajustes de código pequenos (Playwright sem reuso de servidor) vão para a iter 3.

**Verdict:** BLOCKED
