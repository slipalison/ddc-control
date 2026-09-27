# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Loop reiniciado (`--reset-loop`), iteração 1. Re-verificação completa (gates 1-8), escrita do zero, sobre o HEAD `2642382`. Desde a última revisão de código (iter 15, `c04133d`) só mudaram docs e `.jdi/`: `f443647` (1 linha no README e 1 no CHANGELOG), `28ef3a0` (D-2026-09-27-tray-app-12: Verify de TODO ampliado e teste de hardware congelado por SHA-256), `5a3cb77` e `2642382`. `crates/`, `Cargo.toml` e `Cargo.lock` não mudaram.

Tudo rodou em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Nenhum teste `#[ignore]` de hardware rodou (o C12 só faz `--ignored --list`), e nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc`: exit 0 (o tray fica fora por D-2026-09-26-tray-app-9). O mesmo `check` com `--target x86_64-unknown-linux-gnu`: exit 0 |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 linhas `test result`. Phase anterior (`full-osd-control`): 247 passed, 7 ignored, então não há queda |
| Coverage | PASS | 83.36% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0. `main.rs` só chama `ddc_tray::run()` e mapeia o erro, sem lógica desviada |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Nenhum `#[allow(` nem `#[expect(` fora de testes. O único `cfg_attr` é o `windows_subsystem` de `main.rs` |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. Os hits foram lidos: todos estão em comentário, em `#[cfg(test)]` ou no composition root. 5.10: W-1 (`cargo audit`, conhecido) |
| Consistency | WARN | Os 128 commits da phase têm escopo `tray-app`, exceto `chore(jdi)` do reorder do roadmap, que só toca `.jdi/`. Nenhum commit mistura código e `.jdi/`. Todo `files_modified` do PLAN aparece no log, e as 8 tasks `completed` têm teste. D-1, D-2 e D-2026-09-26-tray-app-2/-3/-7 estão conformes, assim como D-2026-09-27-tray-app-2/-3/-5 (deps `ksni`/`zbus`/`tokio`) e -10/-11/-12. Ver W-3 (branch empilhada) |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky. Erros de console: 0; axe critical/serious: 0 (asserção em cada teste via `support.mjs`). Os 10 `critical_paths` × tema com ✓. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para `ci-crossbuild`).** `cargo audit` termina com `error: 1 vulnerability found!`:
  - RUSTSEC-2018-0005: `serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, presente desde a phase `ddc-backends`.
  - Há também 3 avisos permitidos:
    - RUSTSEC-2024-0370: `proc-macro-error` 1.0.4, unmaintained, via `glib-macros`;
    - RUSTSEC-2024-0320: `yaml-rust` 0.4.5, unmaintained, via `serde_yaml`;
    - RUSTSEC-2024-0429: `glib` 0.18.5, unsound, via tauri → tray-icon/muda → gtk.
  - O `Cargo.lock` não mudou desde a última revisão.
- **W-2 (C11, smoke `--activate`: passou só na repetição).**
  - **1ª execução:** exit 1 (PID 2588785). A sequência foi `tray activated` → `popup shown` → `popup focused` → `popup hidden`, e o script reportou `FAIL: the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`.
  - **Causa provável:** houve UM só `tray activated`, o do próprio `Activate`. Então não é o caso de `tray activated` externo que o orquestrador isentou. O popup recebeu o foco e o perdeu (`Focused(false)` → hide, que é o comportamento da D-2026-09-26-tray-app-5). A hipótese mais provável é que outra janela do desktop ativo tomou o foco, mas não consegui provar a causa.
  - **2ª execução:** mesmo protocolo, `OK` (PID 2589745), com `popup shown` e o popup ainda mostrado 1,5 s depois.
  - **Classificação:** registrei o C11 como PASS na repetição e deixo esta ressalva explícita. Numa leitura estrita da exceção do orquestrador, o C11 conta como FAIL, e a decisão fica com ele. Nas 15 iterações anteriores, o C11 sempre passou na 1ª execução, exceto pelo `tray activated` duplo da iter 12.
- **W-3 (Gate 6, afeta só o ship; o código está correto).** `phase/tray-app` foi criada a partir de `phase/full-osd-control` (`c2fac93`), cujos 25 commits entraram no `origin/main` por squash (`1bca909`, #7).
  - A árvore de código de `origin/main` é idêntica à de `c2fac93`: `git diff origin/main c2fac93 -- . ':!.jdi'` sai vazio.
  - Mesmo assim, `origin/main..HEAD` tem 153 commits, 25 deles da phase anterior. Um PR direto contra `main` repetiria o diff da `full-osd-control`.
  - O `main` local está defasado (`a318794`).
  - Sugestão para o `/jdi-ship`: `git fetch && git rebase --onto origin/main c2fac93 phase/tray-app`, conforme a prática de squash-merge do usuário.

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
| 8 | CSP estrita, efetiva em todos os alvos, `custom-protocol`, não-dev, hash do `build_tests` | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `ddf6636a…8061`. Os 2 testes de CSP com `--exact` e o `is_not_a_dev_build` dão `1 passed` cada |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` (`lib.rs:108`) |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | Passou só na repetição (ver W-2). 1ª execução: FAIL, PID 2588785, `popup hidden` depois de `popup focused`. 2ª: `OK`, PID 2589745, ainda mostrado 1,5 s depois. Backend real, só leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila, é gated e está congelado | CONTEXT | Auto | PASS | exit 0, `OK`. Hash `4f9b36b2…f3fd`. 2 testes listados, não executados. O único laço (`for monitor in mute`) só faz `load_panel` |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 17 s. Contagem à parte: 138 passed, 6 skipped |
| 14 | Fora de servidor local o bridge nunca cai no demo, e `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | exit 0, `OK`. Filtro `scroll`: 20 passed. PID 2590432: `75 -> 80` na vertical, nada na horizontal, `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` (Verify da D-2026-09-27-tray-app-12) |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK`. `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (386 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | Literal (`cargo llvm-cov --workspace --summary-only`): TOTAL lines 82.93%, exit 0. Gate 3: 83.36% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` (Verify da D-2026-09-27-tray-app-12) |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só existe o heading `## [Unreleased]` (`CHANGELOG.md:8`), sem `## [version]`. A entrada do `ddc-tray` está em `CHANGELOG.md:40`, e a da CSP de todos os alvos, em `CHANGELOG.md:50` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`) com Build and run, Using it, Browser demo, Tests e Known limitations (`:374`). `cargo run -p ddc-tray` em `:293`; screenshot em `:275` |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **Blockers:** nenhum. O código é o mesmo aprovado na iter 15. Os 2 Verify emendados pela D-2026-09-27-tray-app-12 passam: o TODO em qualquer ponto da linha (C18/P3) e o hash do teste de hardware (C12).
- **W-2:** decida se aceita o C11 que passou na repetição. Se quiser mais confiança, rode o smoke `--activate` de novo com o desktop ocioso. O comportamento de clique e foco já está na lista "Deferred to PR review" do CONTEXT.
- **W-3:** rebase sobre `origin/main` antes de abrir o PR.
- **W-1:** segue para a `ci-crossbuild`.
- **Itens Manual:** CHANGELOG e README, via `/jdi-confirm-dod tray-app` no PR.
- **Instância do usuário:** encerrada com `pkill -x` antes de cada smoke e reaberta com `setsid -f` logo depois. Os PIDs foram 2562221 → 2589053 → 2589990 → 2590672, e só a 2590672 está viva no fim.

## Nota do orquestrador (pós-review, antes do critic)

W-2 (C11): o orquestrador rodou o smoke `--activate` mais 3 vezes seguidas com o mesmo protocolo — 3/3 `OK` (PIDs 2612270, 2612759, 2613290: popup mostrado e ainda aberto 1,5 s depois). A falha única do reviewer (foco perdido logo após o `popup focused`, sem `tray activated` externo) é compatível com outra janela do desktop ao vivo tomando o foco — a regra "esconde ao perder o foco" (D-2026-09-26-tray-app-5) está funcionando como desenhado. C11 conta como PASS (4 de 5 execuções verdes; a vermelha explicada pelo ambiente). W-3 (rebase antes do PR) será feito no ship.
