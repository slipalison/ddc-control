# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Loop reiniciado, iteração 2. Re-verificação completa (gates 1-8), escrita do zero, sobre o HEAD `f1e40bd`. Desde a iter 1 (`2642382`), só mudaram `.jdi/`: o `f1e40bd` troca o `Verify:` de TODO do PROJECT (P3) e do CONTEXT (C18) pela regra de palavra da D-2026-09-27-tray-app-13. Não mudaram `crates/`, `apps/`, `Cargo.toml` nem `Cargo.lock`. Fora de `.jdi/`, a única diferença desde a revisão de código da iter 15 (`c04133d`) é 1 linha no README e 1 no CHANGELOG.

Tudo rodou em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Nenhum teste `#[ignore]` de hardware rodou (o C12 só faz `--ignored --list`), e nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc`: exit 0 (o tray fica fora por D-2026-09-26-tray-app-9). O mesmo `check` com `--target x86_64-unknown-linux-gnu`: exit 0 |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 linhas `test result`. Phase anterior (`full-osd-control`): 247 passed, 7 ignored, então não há queda |
| Coverage | PASS | 83.36% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0. `apps/ddc-tray/src-tauri/src/main.rs` só chama `ddc_tray::run()` e mapeia o erro, sem lógica desviada |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Nenhum `#[allow(` nem `#[expect(` fora de testes. O único `cfg_attr` é o `windows_subsystem` de `main.rs:6`, com comentário. Os 4 crates herdam `[workspace.lints]` (`unsafe_code = "deny"`, `unwrap_used`/`expect_used`/`panic = "warn"`) |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. Os hits foram lidos, e todos são benignos: o 5.5 é doc comment (`lib.rs:154`); os do 5.6 estão em `#[cfg(test)]` (`kwin_placement.rs:195-289`, `stop_signals.rs:82-114`, `lib.rs` `switch_tests`/`build_tests`); o 5.7c está em testes `#[ignore]` gated por `DDC_HW_TESTS=1` ou no composition root (`lib.rs:77`). `Confirm::Yes` só aparece em `commands.rs:112` e `ddc-cli/src/run.rs:166`. 5.10: W-1 (`cargo audit`, conhecido) |
| Consistency | WARN | Os 130 commits de `c2fac93..HEAD` têm escopo `tray-app`, exceto o `chore(jdi)` do reorder do roadmap. O único commit que toca `.jdi/` e outro arquivo (`c22aef0`) é o do flip do `has_frontend` com README/CHANGELOG, previsto na T-8, e não tem código. Todo `files_modified` do PLAN aparece no log, e as 8 tasks `completed` têm teste. D-1, D-2, D-2026-09-26-tray-app-2/-3/-7 e D-2026-09-27-tray-app-2/-3/-5/-10/-11/-12/-13 estão conformes. Ver W-3 (branch empilhada) |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky. Erros de console: 0; axe critical/serious: 0 (asserção em cada teste via `support.mjs`). Os 10 `critical_paths` × tema com ✓. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para `ci-crossbuild`).** `cargo audit` termina com `error: 1 vulnerability found!`:
  - RUSTSEC-2018-0005: `serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, presente desde a phase `ddc-backends`.
  - Há também 3 avisos permitidos:
    - RUSTSEC-2024-0370: `proc-macro-error` 1.0.4, unmaintained;
    - RUSTSEC-2024-0320: `yaml-rust` 0.4.5, unmaintained;
    - RUSTSEC-2024-0429: `glib` 0.18.5, unsound, via tauri → gtk.
  - O `Cargo.lock` não mudou desde a última revisão.
- **W-3 (Gate 6, afeta só o ship; o código está correto).** `phase/tray-app` saiu de `phase/full-osd-control` (`c2fac93`), que entrou no `origin/main` por squash (`1bca909`, #7).
  - A árvore de `origin/main` é idêntica à de `c2fac93` fora de `.jdi/`.
  - Mesmo assim, `origin/main..HEAD` tem 155 commits, 25 deles da phase anterior.
  - O `main` local está defasado (`a318794`).
  - Sugestão para o `/jdi-ship`: `git fetch && git rebase --onto origin/main c2fac93 phase/tray-app`.
- **W-4 (Gate 8, robustez do `Verify:` novo da D-13; o código está correto).**
  - **(a) O C18 depende do locale.** Com o locale desta máquina (`pt_BR.UTF-8`), ou com `C.UTF-8`/`en_US.UTF-8`, ele sai `OK`. Com `LC_ALL=C`, sai 1: o `\b` passa a tratar os bytes do `é` como não-palavra, e o `\b(todo|fixme)\b` casa "Mé**todo**" em `docs/hardware-validation.md:185` ("Método inexistente"). É falso positivo, não TODO. Não afeta o veredito, mas um CI que rode o C18 com `LC_ALL=C` quebraria. Sugestão ao orquestrador: fixar `LC_ALL=C.UTF-8` no comando.
  - **(b) O P3 reprova "Todos".** O segundo grep do P3 (PROJECT) usa `\b(todo|fixme)s?\b` com `-i`, então reprova "Todos" num `*.rs`. Isso contradiz a nota da D-13 de que "Todos os ajustes" não casa, que só vale para o C18. Hoje não há nenhum hit (o P3 dá `OK`), mas uma string pt-BR futura no Rust (menu de `i18n.rs`) daria falso positivo.
- **W-2 da iter 1 (C11 passou só na repetição): resolvido nesta rodada.** O C11 passou na 1ª execução.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | exit 0, `OK` |
| 3 | Só o composition root constrói `DdcHiMonitorBackend` (1×, em `lib.rs`) | CONTEXT | Auto | PASS | exit 0, `OK` (`lib.rs:77`) |
| 4 | `panel.rs` Rust puro sobre `MonitorControl` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 5 | `#![forbid(unsafe_code)]` ativo, nenhum `unsafe` em src/tests/build.rs | CONTEXT | Auto | PASS | exit 0, `OK` |
| 6 | `node --test` por módulo e total, 0 fail/cancelled/skipped/todo | CONTEXT | Auto | PASS | exit 0, `OK`. Total: 157 pass, 0 fail/cancelled/skipped/todo |
| 7 | Paridade i18n, sem texto hardcoded, trava de frases, título da guarda | CONTEXT | Auto | PASS | exit 0, `OK` |
| 8 | CSP estrita, efetiva em todos os alvos, `custom-protocol`, não-dev, hash do `build_tests` | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `ddf6636ab1b4…`. Os 2 testes de CSP com `--exact` e o `the_tray_is_not_a_dev_build` dão `1 passed` cada |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` (`lib.rs:108`) |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | exit 0, `OK` na 1ª execução (PID 2661367): `popup shown` e ainda mostrado 1,5 s depois. Backend real, só leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila, é gated e está congelado | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `4f9b36b29375…`. 2 testes listados (`--list`), não executados |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 18 s |
| 14 | Fora de servidor local o bridge nunca cai no demo, e `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | exit 0, `OK`. Filtro `scroll`: 20 passed. PID 2660900: `75 -> 80` na vertical, nada na horizontal, `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` (Verify da D-13, locale `pt_BR.UTF-8`). O `git grep` leu 136 arquivos. As sondas do critic (`todo re-list`, `Fixme once`, `todo` em continuação de bloco, `To do:`) são pegas; "Todos os ajustes" não. Ver W-4a (`LC_ALL=C`) |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK`. `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (386 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | Literal (`cargo llvm-cov --workspace --summary-only`): TOTAL lines 82.93%, exit 0. Gate 3: 83.36% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` (Verify da D-13; 78 `*.rs` lidos, também com `LC_ALL=C`). Ver W-4b |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só existe o heading `## [Unreleased]` (`CHANGELOG.md:8`), sem `## [version]`. A entrada do `ddc-tray` está em `CHANGELOG.md:40` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`) com Build and run (`:290`), Using it (`:308`), Browser demo (`:327`), Tests (`:352`) e Known limitations (`:374`). Screenshots em `:275-276` |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **Blockers:** nenhum. O código é o mesmo aprovado na iter 15 e na iter 1 do loop reiniciado. Os `Verify:` da D-13 (C18/P3) passam e pegam as formas que o critic demonstrou.
- **W-4:** decisão do orquestrador. Fixar `LC_ALL=C.UTF-8` no C18 e alinhar o `s?` do P3 com a nota da D-13. Nenhum dos dois muda o resultado hoje.
- **W-3:** rebase sobre `origin/main` antes de abrir o PR.
- **W-1:** segue para a `ci-crossbuild`.
- **Itens Manual:** CHANGELOG e README, via `/jdi-confirm-dod tray-app` no PR.
- **Instância do usuário:** encerrada com `pkill -x` antes de cada smoke e reaberta com `setsid -f` logo depois. Os PIDs foram 2613525 → 2661149 → 2661604, e só a 2661604 está viva no fim. Nenhuma instância de smoke ficou para trás.

## Nota do orquestrador (pós-review, antes do critic)

W-4 era dos `Verify:` de TODO (orquestrador): D-2026-09-27-tray-app-14 fixa `LC_ALL=C.UTF-8` e alinha o plural na linha da phase (única exclusão por padrão exato: a linha `.jdi/todos.md` do `.gitignore`). OK em três locales; mutantes reprovam. W-3 (rebase) fica para o ship.
