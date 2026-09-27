# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Loop reiniciado, iteração 4. Re-verificação completa (gates 1-8), escrita do zero sobre o HEAD `c8578ab`.

Desde a iter 1 (`2642382`), só `.jdi/` mudou: `git diff --stat 2642382 HEAD -- . ':!.jdi'` sai vazio. O `c8578ab` troca os `Verify:` de TODO/FIXME do PROJECT.md (P3) e do CONTEXT.md (C18) pela versão da D-2026-09-27-tray-app-15: tokenizador JS no `pt-BR.js`, referência de issue por ocorrência, `to-do(s):`.

Tudo rodou em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Os `Verify:` foram extraídos por script do CONTEXT.md (19 auto) e do PROJECT.md (3 auto), sem transcrição à mão, e cada um rodou literalmente num processo `bash`. Nenhum teste `#[ignore]` de hardware rodou: o C12 só faz `--ignored --list`. Nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc`: exit 0 (o tray fica fora por D-2026-09-26-tray-app-9). O mesmo `check` com `--target x86_64-unknown-linux-gnu`: exit 0 |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 linhas `test result`. A phase anterior (`full-osd-control`) tinha 247 passed e 7 ignored: sem queda |
| Coverage | PASS | 83.36% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0. O `main.rs` do tray só chama `ddc_tray::run()` e mapeia o erro para `ExitCode`, sem lógica desviada |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Nenhum `#[allow(`/`#[expect(` fora de testes. O único `cfg_attr` é o `windows_subsystem` de `main.rs:6`, comentado. Os 4 crates herdam `[workspace.lints]` (`unsafe_code = "deny"`; `unwrap_used`/`expect_used`/`panic = "warn"`) |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.4, 5.7b, 5.8, 5.9 e 5.11 limpos. Li os hits dos demais, todos benignos: 5.5 é doc comment (`apps/ddc-tray/src-tauri/src/lib.rs:154`); 5.6 está em `#[cfg(test)]` (`tray/kwin_placement.rs:195+`, `stop_signals.rs:82+`, `lib.rs` `switch_tests`:247/`build_tests`:312) ou em doc comment (`crates/ddc-adapters/src/ddc_hi_backend/worker.rs:141`); 5.7a acha `Risk::Dangerous` em `feature.rs`/`mccs_catalog.rs`; 5.7c: 7 testes `#[ignore]` com `hardware_enabled()` (`DDC_HW_TESTS=1`) em `real_monitor.rs`, e o composition root (`lib.rs:28/77`). 5.10: W-1 (conhecido) |
| Consistency | WARN | 134 commits em `c2fac93..HEAD`, todos com escopo `tray-app`, exceto o `chore(jdi)` do reorder do roadmap (`6f4341b`). Os 54 caminhos explícitos de `files_modified` do PLAN aparecem no log, e também os ícones gerados (`icons/*`), e as 8 tasks `completed` têm teste. D-1, D-2, D-2026-09-26-tray-app-2/-3/-7 e D-2026-09-27-tray-app-2/-3/-5/-10/-11/-12/-13/-14/-15 estão conformes (o código é o aprovado na iter 15). Ver W-3 (branch empilhada) |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky, em 16 s. Erros de console: 0; axe critical/serious: 0 (asserção em cada teste via `support.mjs`, travada pelo C13/C15). Os 10 `critical_paths` × tema com ✓. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para a `ci-crossbuild`).** `cargo audit` termina com `error: 1 vulnerability found!`:
  - RUSTSEC-2018-0005: `serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, presente desde a phase `ddc-backends`.
  - 3 avisos permitidos: RUSTSEC-2024-0370 (`proc-macro-error`), RUSTSEC-2024-0320 (`yaml-rust`) e RUSTSEC-2024-0429 (`glib` 0.18.5, via tauri → gtk).
  - O `Cargo.lock` não mudou desde `c2ff8aa`.
- **W-3 (Gate 6, afeta só o ship; o código está correto).** `phase/tray-app` saiu de `phase/full-osd-control` (`c2fac93`), que entrou no `origin/main` por squash (`1bca909`, #7).
  - A árvore de `origin/main` é idêntica à de `c2fac93` fora de `.jdi/`, mas `origin/main..HEAD` tem 159 commits, 25 deles da phase anterior.
  - Sugestão para o `/jdi-ship`: `git fetch && git rebase --onto origin/main c2fac93 phase/tray-app`.
- **W-6 (Gate 8, robustez do tokenizador do C18; o código está correto).** O tokenizador JS da D-15 não conhece regex literal nem expressão de template. Numa cópia descartável (`git clone --shared` do HEAD), cada mutação no `apps/ddc-tray/src/i18n/pt-BR.js` imprimiu `OK` no C18:
  - **G2:** a 1ª linha é `const quote = /["]/; // todo: handle quotes`. O `"` do regex abre uma "string" que o tokenizador não fecha (o arquivo não tem outro `"`), e tudo que vem depois some, inclusive o `// todo`.
  - **H1:** `` export const z = `a ${/* todo */ 1} b`; ``. O template inteiro, com o comentário dentro do `${…}`, conta como string.
  - **H2:** `` export const w = `a ${`inner`} // todo: nested` ; ``. O template aninhado inverte a paridade e isenta o `// todo`.
  - O G1 (`/'/` + `// todo`) só reprovou por acaso: com a paridade invertida, a string `'Todos os ajustes'` virou código.
  - **Hoje não há ocorrência.** Fora de comentários e strings, o esqueleto do `pt-BR.js` é só `export default Object.freeze({ k: v, … });`, sem `/` nem crase (a única crase está no comentário da linha 1). Por isso o resultado atual está certo.
  - **Sugestão ao orquestrador (fail-closed, mais simples que ensinar regex/template ao tokenizador):** depois do tokenizador, exigir que o esqueleto sem espaços case `^exportdefaultObject\.freeze\(\{(:,)*\}\);$`. Assim, qualquer regex, template ou código extra no arquivo de locale reprova o item.
- **W-5 da iter 3: resolvido.** A continuação de bloco `/* … todo: … */` sem `*` no `pt-BR.js` agora reprova (sonda B abaixo).

### Sondas do C18/P3 (D-15), cópia descartável, cada mutação sozinha
| Sonda | Onde | C18 | P3 (`*.rs`) |
|---|---|---|---|
| nenhuma (base) | — | OK | OK |
| A: `// Don't shorten it: todo check … it's the longest text.` (a do critic, iter 3) | `pt-BR.js` | reprova | OK (não é `.rs`) |
| B: `/*` ↵ `   todo: re-check the plural` ↵ `*/` (W-5) | `pt-BR.js` | reprova | OK |
| C: `// To-dos: foo` | `popup.rs` / `app.js` | reprova / reprova | reprova / OK |
| D: `// TODO(#12): foo FIXME: bar` | `popup.rs` | reprova | reprova |
| E: `// TODO(#12): foo` | `popup.rs` / `pt-BR.js` | OK / OK | OK / OK |
| F: `'Todo o brilho'` e `"todos os // ajustes"` em string | `pt-BR.js` | OK | OK |
| I: `'x'; // todo: check` | `pt-BR.js` | reprova | OK |
| `# todo later` / `<!-- Todo: fix -->` / `/* fixme */` | `smoke-sni.sh` / `index.html` / `styles.css` | reprova (3) | OK |
| `To do: something` / `What to do.` | `docs/hardware-validation.md` | reprova / OK | OK |
| `// @todo`, `todo!()`, `unimplemented!()`, `// todo(#1) and todo` | `popup.rs` | reprova (4) | reprova (4) |
| `// To-dos: rever`, `/* FIXME */`, `// It's fine; todo later`, `'it\'s'; // todo` | `pt-BR.js` | reprova (4) | OK |
| `// Método de teste e métodos` | `pt-BR.js` | OK | OK |
| G2 / H1 / H2 (W-6) | `pt-BR.js` | **OK (lacuna)** | OK |

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | exit 0, `OK` |
| 3 | Só o composition root constrói `DdcHiMonitorBackend` (1×, em `lib.rs`) | CONTEXT | Auto | PASS | exit 0, `OK` (`lib.rs:77`) |
| 4 | `panel.rs` Rust puro sobre `MonitorControl` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 5 | `#![forbid(unsafe_code)]` ativo, nenhum `unsafe` em src/tests/build.rs | CONTEXT | Auto | PASS | exit 0, `OK` |
| 6 | `node --test` por módulo e total, 0 fail/cancelled/skipped/todo | CONTEXT | Auto | PASS | exit 0, `OK` |
| 7 | Paridade i18n, sem texto hardcoded, trava de frases, título da guarda | CONTEXT | Auto | PASS | exit 0, `OK` |
| 8 | CSP estrita, efetiva em todos os alvos, `custom-protocol`, não-dev, hash do `build_tests` | CONTEXT | Auto | PASS | exit 0, `OK` (hash `ddf6636ab1b4…` confere; os 2 testes de CSP `--exact` e o `is_not_a_dev_build` dão `1 passed` cada) |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | exit 0, `OK` na 1ª execução, em 4 s. O item do PID 2831154 era desse PID, `popup shown` apareceu e o popup seguia mostrado 1,5 s depois. Backend real, só leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila, é gated e está congelado | CONTEXT | Auto | PASS | exit 0, `OK` (hash `4f9b36b29375…` confere; listado com `--list`, não executado) |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 18 s |
| 14 | Fora de servidor local o bridge nunca cai no demo, e `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | exit 0, `OK`. Filtro `scroll`: 20 passed. No PID 2831701, a vertical deu `75 -> 80`, a horizontal não escreveu nada e a volta deu `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` (Verify da D-15). Sanidade: o `git grep` lê 136 dos 310 arquivos versionados (os demais estão excluídos ou são binários). Sondas na tabela acima. Ver W-6 |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK` em 4 s; `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0, `OK` (386 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | Literal (`cargo llvm-cov --workspace --summary-only`): TOTAL lines 82.93%, exit 0. Gate 3: 83.36% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` (Verify da D-15, `LC_ALL=C.UTF-8`). As sondas em `.rs` (C, D, `@todo`, `todo!`, `unimplemented!`, `todo(#1) and todo`) reprovam, e `TODO(#12)` sozinho passa |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só existe o heading `## [Unreleased]` (`CHANGELOG.md:8`), sem `## [version]`. A entrada do `ddc-tray` está em `CHANGELOG.md:40` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`), com Build and run (`:290`), Using it (`:308`), Browser demo (`:327`), Tests (`:352`) e Known limitations of the tray app (`:374`). Screenshots em `:275-276` |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **Blockers:** nenhum. O código é o mesmo aprovado na iter 15 e nas iters 1-3 do loop reiniciado. O C18/P3 da D-15 passa no HEAD e reprova os mutantes do critic (A, `To-dos:`, `TODO(#12) … FIXME:`) e os anteriores (B, I e `// todo:`).
- **W-6:** decisão do orquestrador. A lacuna é do comando, não do código: hoje não há nenhuma ocorrência. A correção fail-closed sugerida fecha a classe inteira (regex, template, código extra) sem complicar o tokenizador.
- **W-3:** rebase sobre `origin/main` antes de abrir o PR.
- **W-1:** segue para a `ci-crossbuild`.
- **Itens Manual:** CHANGELOG e README, via `/jdi-confirm-dod tray-app` no PR.
- **Instância do usuário:** encerrada com `pkill -x` antes de cada smoke e reaberta com `setsid -f` logo depois. PIDs: 2786470 → 2831404 → 2831949, e só a 2831949 está viva no fim. Nenhum `tray activated` externo, então não houve repetição. Nenhuma instância de smoke nem servidor na porta 1420 ficou para trás. A cópia descartável das sondas foi apagada.

## Nota do orquestrador (pós-review, antes do critic)

W-6 era do `Verify:` do C18 (orquestrador): D-2026-09-27-tray-app-16 exige que o `pt-BR.js` seja dado puro (esqueleto `export default Object.freeze({ k: v, … });`, sem crase) antes do tokenizador. OK em `LC_ALL=C`/`pt_BR.UTF-8`; mutantes (regex com aspa, `${/* todo */}`, comentário com apóstrofos) reprovam; strings portuguesas aceitas.
