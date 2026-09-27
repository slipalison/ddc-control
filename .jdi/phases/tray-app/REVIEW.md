# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Loop reiniciado, iteração 3. Esta é uma re-verificação completa (gates 1-8), escrita do zero sobre o HEAD `538c1e0`.

Desde a iter 1 (`2642382`), só `.jdi/` mudou: `git diff --stat 2642382 HEAD -- . ':!.jdi'` sai vazio. O `538c1e0` emenda a D-2026-09-27-tray-app-14 e o `Verify:` do C18: os comentários do `pt-BR.js` passam a ser checados, e só as palavras portuguesas dentro das strings seguem isentas. Fora de `.jdi/`, a única diferença desde a revisão de código da iter 15 (`c04133d`) é 1 linha no README e 1 no CHANGELOG.

Tudo rodou em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Os `Verify:` foram extraídos por script do CONTEXT.md e do PROJECT.md, sem transcrição à mão, e rodados literalmente, cada um num processo `bash`. Nenhum teste `#[ignore]` de hardware rodou: o C12 só faz `--ignored --list`. Nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` dá exit 0 (o tray fica fora por D-2026-09-26-tray-app-9). O mesmo `check` com `--target x86_64-unknown-linux-gnu` também dá exit 0 |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 linhas `test result`. A phase anterior (`full-osd-control`) tinha 247 passed e 7 ignored, então não houve queda |
| Coverage | PASS | 83.36% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0. `apps/ddc-tray/src-tauri/src/main.rs` só chama `ddc_tray::run()` e mapeia o erro, sem lógica desviada |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Não há nenhum `#[allow(` nem `#[expect(` fora de testes. O único `cfg_attr` é o `windows_subsystem` de `main.rs:6`, com comentário. Os 4 crates herdam `[workspace.lints]` (`unsafe_code = "deny"`; `unwrap_used`/`expect_used`/`panic = "warn"`) |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.4, 5.8, 5.9 e 5.11 limpos. Li os hits dos outros checks, e todos são benignos: o do 5.5 é doc comment (`lib.rs:154`), os do 5.6 estão em `#[cfg(test)]` (`kwin_placement.rs:195+`, `stop_signals.rs:82+`, `lib.rs` `switch_tests`/`build_tests`) e os do 5.7c estão em 7 testes `#[ignore]` gated por `DDC_HW_TESTS=1` (`real_monitor.rs`) ou no composition root (`lib.rs:77`). `Confirm::Yes` só em `commands.rs`/`ddc-cli`. 5.10: W-1 (`cargo audit`, conhecido) |
| Consistency | WARN | Os 132 commits de `c2fac93..HEAD` têm escopo `tray-app`, exceto o `chore(jdi)` do reorder do roadmap. Os 55 caminhos de `files_modified` do PLAN aparecem no log, inclusive `icons/*` e `tray.png` (em `e91a453`). As 8 tasks `completed` têm teste. D-1, D-2, D-2026-09-26-tray-app-2/-3/-7 e D-2026-09-27-tray-app-2/-3/-5/-10/-11/-12/-13/-14 estão conformes. Ver W-3 (branch empilhada) |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky. Erros de console: 0; axe critical/serious: 0 (asserção em cada teste via `support.mjs`, travada pelo C13/C15). Os 10 `critical_paths` × tema com ✓. Sliders nativos `input[type=range]` (`app.js:493`), `prefers-color-scheme` e `prefers-reduced-motion` em `styles.css`. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para `ci-crossbuild`).** `cargo audit` termina com `error: 1 vulnerability found!`:
  - RUSTSEC-2018-0005: `serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, presente desde a phase `ddc-backends`.
  - Há também 3 avisos permitidos: RUSTSEC-2024-0370 (`proc-macro-error` 1.0.4, unmaintained), RUSTSEC-2024-0320 (`yaml-rust` 0.4.5, unmaintained) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound, via tauri → gtk).
  - O `Cargo.lock` não mudou desde `c2ff8aa`.
- **W-3 (Gate 6, afeta só o ship; o código está correto).** `phase/tray-app` saiu de `phase/full-osd-control` (`c2fac93`), que entrou no `origin/main` por squash (`1bca909`, #7).
  - A árvore de `origin/main` é idêntica à de `c2fac93` fora de `.jdi/`.
  - Mesmo assim, `origin/main..HEAD` tem 157 commits, 25 deles da phase anterior.
  - O `main` local está defasado (`a318794`).
  - Sugestão para o `/jdi-ship`: `git fetch && git rebase --onto origin/main c2fac93 phase/tray-app`.
- **W-5 (Gate 8, robustez do `Verify:` do C18 emendado; o código está correto).**
  - A checagem dos comentários do `pt-BR.js` só reconhece linhas com `//`, `/*` ou `*` no início.
  - Por isso, um `todo` minúsculo numa linha de continuação de bloco que não começa com `*` passa. Isso acontece porque o `pt-BR.js` fica fora do grep sem distinção de caixa, e o grep com caixa só pega `TODO`/`FIXME`. Exemplo:
    ```
    /*
       todo: re-check the plural
    */
    ```
  - O comando segue a letra da emenda da D-14, que lista `//`, `/*` e `*`. O arquivo tem hoje só 2 linhas de comentário (`//`, linhas 1-2) e nenhuma palavra `todo`/`fixme` nelas, então o resultado atual está certo.
  - Sugestão ao orquestrador: também reprovar, no `pt-BR.js`, qualquer linha com a palavra que não esteja dentro de uma string entre aspas. Ou então proibir comentários de bloco nesse arquivo.
  - Observação menor: a emenda da D-14 está só em `.jdi/decisions/D-2026-09-27-tray-app-14.md`. A linha do índice (`.jdi/DECISIONS.md:222`) ainda traz o texto original.
- **W-4 da iter 2: resolvido pela D-2026-09-27-tray-app-14.** O C18 e o P3 fixam `LC_ALL=C.UTF-8`, e o plural fica alinhado.

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
| 8 | CSP estrita, efetiva em todos os alvos, `custom-protocol`, não-dev, hash do `build_tests` | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `ddf6636ab1b4…` confere. Os 2 testes de CSP (`--exact`) e o `is_not_a_dev_build` dão `1 passed` cada |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` (`lib.rs:108`) |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | exit 0, `OK` na 1ª execução. No PID 2735444, o item pertencia ao próprio PID, `popup shown` apareceu e o popup seguia mostrado 1,5 s depois. Backend real, só leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila, é gated e está congelado | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `4f9b36b29375…` confere. 2 testes listados (`--list`), não executados |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 18 s |
| 14 | Fora de servidor local o bridge nunca cai no demo, e `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | exit 0, `OK`. Filtro `scroll`: 20 passed. No PID 2736005, a vertical deu `75 -> 80`, a horizontal não escreveu nada e a volta deu `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` (Verify emendado da D-14). Sondas em cópia descartável: `// todo:` (a do critic), `// Todo`, `/* @todo */`, ` * To do:`, `// todos` ao fim de uma linha e `'fixme'` em string reprovam; `'Todo o brilho'` em string e `// TODO(#12)` passam. Ver W-5 (continuação de bloco sem `*`) |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK`. `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0, `OK` (386 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | Literal (`cargo llvm-cov --workspace --summary-only`): TOTAL lines 82.93%, exit 0. Gate 3: 83.36% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` (Verify da D-14, `LC_ALL=C.UTF-8`) |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só existe o heading `## [Unreleased]` (`CHANGELOG.md:8`), sem `## [version]`. A entrada do `ddc-tray` está em `CHANGELOG.md:40` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`) com Build and run (`:290`), Using it (`:308`), Browser demo (`:327`), Tests (`:352`) e Known limitations (`:374`). Screenshots em `:275-284` |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **Blockers:** nenhum. O código é o mesmo aprovado na iter 15 e nas iters 1-2 do loop reiniciado. O C18 emendado passa e pega a sonda do critic (`// todo:` no `pt-BR.js`).
- **W-5:** decisão do orquestrador. É uma lacuna residual do comando, não do código; hoje não há nenhuma ocorrência.
- **W-3:** rebase sobre `origin/main` antes de abrir o PR.
- **W-1:** segue para a `ci-crossbuild`.
- **Itens Manual:** CHANGELOG e README, via `/jdi-confirm-dod tray-app` no PR.
- **Instância do usuário:** encerrada com `pkill -x` antes de cada smoke e reaberta com `setsid -f` logo depois. Os PIDs foram 2661604 → 2735699 → 2736286, e só a 2736286 está viva no fim. Nenhuma instância de smoke nem servidor na porta 1420 ficou para trás.

## Nota do orquestrador (pós-review, antes do critic)

W-5 era do `Verify:` do C18 (orquestrador): emenda 2 da D-2026-09-27-tray-app-14 — no `pt-BR.js` as strings são removidas e a palavra reprova em qualquer ponto do resto. OK em `LC_ALL=C` e `pt_BR.UTF-8`; reprova continuação de bloco `/* … */` sem `*` e `// todo:`; aceita strings portuguesas, inclusive com `//` dentro. O índice `.jdi/DECISIONS.md` é view gerada (`npx jdi-cli render`).
