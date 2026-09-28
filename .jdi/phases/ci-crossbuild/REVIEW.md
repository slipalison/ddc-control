# Phase 6: Review  (slug: ci-crossbuild)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 3. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente. O HEAD é `14185de`. A iteração 3 não mudou código: os Gates 1–7 herdam da iteração 2, pela prova abaixo. O Gate 8 rodou completo agora, em `bash`.

## Prova de herança dos Gates 1–7
- `git diff --quiet cb8eea2 HEAD -- . ':!.jdi'` sai 0.
  - Entre `cb8eea2` e `14185de` só mudaram 4 arquivos, todos em `.jdi/phases/ci-crossbuild/`: `CONTEXT.md` (só o `Verify:` e o texto da linha 5 do DoD), `PLAN.md`, `REVIEW.md` e `SUMMARY.md`.
- `gh pr view 13 --repo slipalison/github-workflows` dá headRefOid `9e91d1ac9604aa581ac17438c54d75a2fcc213fa`, OPEN, fora do rascunho, MERGEABLE. É o mesmo `WORKFLOWS_SHA` julgado na iteração 2.
- O código local também é igual ao de `origin/phase/ci-crossbuild` (`0aaaf1c`). Os 4 commits locais ainda não enviados só mexem em `.jdi/`.
- Reconfirmação feita pela baseline do Gate 8, com os mesmos números da iteração 2:
  - `cargo test --workspace --locked`: 386 passed / 0 failed / 9 ignored, em 15 binários;
  - cobertura no mesmo profile: 83.36% com a exclusão do Gate 3 e 82.93% sem ela;
  - `cargo audit`: sai 0 (1271 advisories, 524 crates).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Herdado da iteração 2 (prova acima). Build nativo Linux OK. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli` para `x86_64-unknown-linux-gnu` e `x86_64-pc-windows-msvc` OK. O `ddc-tray` no Windows só é provado pelo CI: no run 36345684775, clippy, testes e `Build ddc-tray (release)` terminaram em success. Não há `llvm-rc` local (D-2026-09-26-tray-app-9). O workspace foi recompilado agora pelo `cargo test` e pelo `cargo llvm-cov` do Gate 8. |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 binários. Herdado e reconfirmado agora pela linha 10 do DoD. Iguais ao `rust-linux` do CI. |
| Coverage | PASS | 83.36% de linhas na linha TOTAL (`main.rs`/`build.rs` excluídos), piso de 80%. Herdado e reconfirmado agora com `cargo llvm-cov report` sobre o profile da linha 11. Sem exclusão, como no Verify literal do PROJECT e no CI: 82.93%. |
| Lint | PASS | Herdado. `cargo fmt --all --check` OK. `cargo clippy --workspace --all-targets --locked -- -D warnings` OK, e também com `--all-features`, como o CI roda. Nenhum `#[allow(` fora de testes. |
| Hexagonal/Safety/Hygiene | PASS | Herdado: 5.1–5.9 sem achado novo. Na phase, `crates/` e `apps/` só mudam em 7 arquivos (`build.rs`, manifesto, `package.json`, 2 `.mjs`, teste `.mjs`, `worker/tests.rs`). Os hits de 5.5a, 5.6 e 5.7c já existiam. 5.10 refeito agora: `cargo audit` 0.22.2 sai 0 (1271 advisories). 5.11 limpo (linhas 9 e 12 do DoD). |
| Consistency | PASS | Herdado: D-1, D-2, D-2026-09-27-ci-crossbuild-1..8 e D-2026-09-26-full-osd-control-6 conformes (ver Notas). Na iteração 3, o PLAN ganhou a seção "Iteração 3" (`completed`), sem `files_modified` de código. Os commits `35ea5d3`, `57f1939` e `14185de` são todos `docs(ci-crossbuild)`. |
| UI Validation | PASS | Herdado (`has_frontend: true` em `.jdi/PROJECT.md:80`). `npm ci` + Playwright: 138 passed, 6 skipped (screenshots opt-in), 0 failed/flaky. `node --test`: 161/161, `# fail 0`. Nada em `apps/ddc-tray/src`, `tests/e2e` ou `playwright.config.mjs` mudou nesta phase. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto (9 CONTEXT + 3 PROJECT), rodados agora em `bash`; 2 manuais pendentes |

## Blockers
- nenhum

## Warnings
- **W-3 (github-workflows, já existia antes do PR, fora do diff, segurança) — continua aberto:** o `qualidade.yml` ainda interpola `matrix.c.projeto` direto em `run:` (dotnet restore/build/test, `npm run build -w`, `go vet`/`go test`). O `go run …gocover-cobertura@latest` continua sem versão fixa.
  - A iteração 2 registrou o problema na seção "Follow-up, fora deste PR" do corpo do #13, com o motivo e a correção.
  - Não há issue no gw nem entrada em `.jdi/todos.md`, então o registro some com o merge do PR.
  - Abrir uma issue no github-workflows ao fazer o merge.
- **W-6 (github-workflows, `qualidade.yml`, `runs-on` e `Conferir componente`; não é de segurança) — continua aberto:** a validação de `so` aceita qualquer sufixo depois de `ubuntu-`/`windows-`.
  - O `case "" | ubuntu-* | windows-*` e o `startsWith(...)` do `runs-on` deixam passar `ubuntu-lastest`, `windows-latest extra` e até um valor com quebra de linha. O script extraído do YAML sai 0 com `SO='ubuntu-latest extra'` e com `SO=$'windows-latest\nx'`.
  - O `runs-on` recebe esse rótulo, e o job fica na fila sem runner. É o que o comentário do `runs-on` e o corpo do #13 dizem evitar.
  - Já existia desde o `2335b52`. O head do #13 não mudou (`9e91d1a`), então o achado continua.
  - Sugestão, em follow-up: uma lista explícita de rótulos hospedados, no `case` e no `runs-on` (`contains(fromJSON('[...]'), matrix.c.so)`).
  - Não afeta o ddc-control, que usa `windows-latest` e omite `so` nos outros componentes.

### Pendências da iteração 2 (conferidas)
- **Segmento do DoD Critic iter 2 (linha 5) — resolvido.** A linha agora compara o conjunto `nome:estado` de todos os testes dos dois logs com uma lista congelada.
  - Conferi a lista contra os atributos `cfg` do código: bate 1:1, e todos os `cfg` são anteriores à phase.
  - Refiz a conta sobre os logs reais e passei 4 mutantes pela linha: todos foram recusados.
  - Detalhes em "Linha 5 nova", nas Notas.

### Pendências da iteração 1 (conferidas na iteração 2, continuam válidas)
- **W-1 — resolvida:** o `CONTEXT.md` tem exatamente um `## Definition of Done`, um `## Deferred to PR review` e um `### Manual`. Os 9 itens têm `Verify:` e `Source:`.
- **W-2 — resolvida (`33e00c7`):** o `Conferir componente` do `9e91d1a` foi rodado em 13 combinações.
  - Reprovam: `windows-*` com node, go, `Rust` (maiúscula) e com valor multilinha; `macos-latest`; apt com `;`; `"true"` com aspas; auditoria no Windows; `build_release` em node ou com `;`.
  - Passam: rust no Windows com `build_release`; rust no Linux com apt, auditoria e release; node sem `so`.
- **W-4 — resolvida no que cabe à phase:** o corpo do #13 tem a tabela de runs por head e a mensagem de squash sugerida. Usar essa mensagem é ato humano no merge (Deferred).
- **W-5 — resolvida (`9e91d1a`):** o passo aparece como `Build ddc-tray (release)` nos dois jobs rust do run 36345684775.
- **Segmento do DoD Critic iter 1 — resolvido:**
  - linha 2: composites `@main` em `eb9c69a` ≡ `9e91d1a` em `.github/actions` e `bin`;
  - linha 1: a árvore comparada inclui `clippy.toml`, `rustfmt.toml` e o toolchain;
  - linha 9: TODO em todo arquivo versionado.

## Notas de revisão

### Linha 5 nova: a lista congelada contra os `cfg` do código
- **De onde vêm as 20 diferenças.** Cada grupo corresponde a um `cfg` de plataforma. Os nomes de `#[test] fn` extraídos da fonte são idênticos aos da lista.

  | Grupo (lista) | Testes | `cfg` no código | Origem |
  |---|---|---|---|
  | `tray::kwin_placement::tests` | 7 | `apps/ddc-tray/src-tauri/src/tray.rs:18` `#[cfg(target_os = "linux")] mod kwin_placement;` | `c352118` (#8, tray-app) |
  | `tray::status_item::tests` | 3 | `tray.rs:22` `#[cfg(target_os = "linux")] mod status_item;` | `c352118` |
  | `stop_signals::tests` | 2 | `apps/ddc-tray/src-tauri/src/lib.rs:18` `#[cfg(target_os = "linux")] mod stop_signals;` | `c352118` |
  | `tests::` (dmabuf) | 2 | `lib.rs:365` `#[cfg(all(test, target_os = "linux"))] mod tests` | `c352118` |
  | `ddc_hi_backend::hardware::tests::a_linux_reply_carries_the_code_the_monitor_echoed` | 1 | `crates/ddc-adapters/src/ddc_hi_backend/hardware/tests.rs:55` `#[cfg(target_os = "linux")]` | `1bca909` (#7, full-osd-control) |
  | `invalidation_holds_when_the_stale_file_cannot_be_replaced` | 1 | `crates/ddc-adapters/tests/caching_backend.rs:350` `#[cfg(unix)]` | `2628f8c` (#6, ddc-backends) |
  | `tray::notification_area::tests` (só Windows) | 3 | `tray.rs:20` `#[cfg(not(target_os = "linux"))] mod notification_area;` | `c352118` |
  | `ddc_hi_backend::hardware::tests::a_reply_off_linux_carries_no_echo` (só Windows) | 1 | `hardware/tests.rs:74` `#[cfg(not(target_os = "linux"))]` | `1bca909` |

  - Soma: 7 + 3 + 2 + 2 + 1 + 1 = 16 só no Linux, e 3 + 1 = 4 só no Windows.
- **Completude.** `git grep` de todo `#[cfg`/`#![cfg`/`cfg_attr` com `windows|unix|linux|target_os|target_family|macos` em `crates/`, `apps/` e `src/` dá 20 hits. Os 8 da tabela são os únicos que envolvem testes. Os outros 12 são:
  - código de produção: `lib.rs:42,101,112,120,157,178`, `tray.rs:25,27` e `hardware.rs:92,97`;
  - o `windows_subsystem` de `main.rs:6`;
  - um comentário de doc (`build.rs:49`).
- **Outros jeitos de pular teste.**
  - Não há nenhum `cfg_attr(..., ignore)`.
  - O único `cfg!(windows)` em teste é `menu.rs:248` (`the_platform_is_the_build_target`). Ele roda nos dois SOs e só troca o valor esperado; está entre os 379 comuns.
  - Os 9 `#[ignore]` são todos de hardware (`real_monitor.rs` ×7, `rtk_qhd_hdr.rs` ×2), e o conjunto `ignored` é idêntico nos dois logs.
- **Justificativa.** Cada `cfg` segue a API da plataforma:
  - só no Linux: KWin/Plasma, StatusNotifierItem, renderer DMA-BUF do WebKitGTK com `std::os::unix::ffi::OsStrExt`, sinais POSIX, permissões `std::os::unix::fs::PermissionsExt`;
  - só no Windows: o ícone de bandeja do Tauri (`notification_area`);
  - o par do eco é a D-2026-09-26-full-osd-control-10.
- **Nenhum `cfg` é uma exclusão apressada desta phase.** `git diff origin/main...HEAD -- crates apps` não tem nenhuma linha nova de `cfg`, `#[ignore]` ou `#[test]`; a única linha com `cfg` é o comentário de `build.rs:49`. O único arquivo de teste Rust mexido na phase é o `worker/tests.rs`, que não tem `cfg`.
- **Skip em runtime.** O `invalidation_holds_when_the_stale_file_cannot_be_replaced` tem um caminho `if replaced { eprintln!("skipped: …"); return; }` para quem roda como root. No log do `rust-linux` do run, o teste sai `... ok`, e `skipped: running with permission` aparece 0 vezes: ele rodou de verdade.
- **Recontagem independente (bash, logs reais do run 36345684775):**
  - Windows: 383 nomes = 374 passed + 9 ignored;
  - Linux: 395 nomes = 386 + 9;
  - 11 = 11 binários e 379 testes comuns;
  - o `comm` dá exatamente a lista, sem nome duplicado e sem `\r` no log do Windows.
- **Mutação:** um shim de `gh` serviu o log do Windows editado, com os `test result` ajustados, e o log real do Linux. Tudo ficou no scratchpad; o repositório não foi tocado.
  - Controle (log real): `OK`.
  - M1, teste comum compilado fora só no Windows (`menu::tests::the_platform_is_the_build_target`): rc=1.
  - M2, o mesmo teste `ignored` só no Windows: rc=1.
  - M3, teste novo só no Windows, fora da lista: rc=1.
  - M4, teste comum com o caminho de módulo trocado: rc=1.
- **Resíduo aceito (não objetivo, como no crítico iter 2):** um `if cfg!(windows) { return; }` dentro de um teste continua invisível a qualquer checagem de log. Hoje nenhum teste faz isso: o único `cfg!` em teste é o `menu.rs:248`, que não retorna cedo. Um teste de plataforma novo exige atualizar a lista, e isso é intencional.
- **Observação, não é achado:** a linha depende de `bash`.
  - No zsh, o `echo "$LW"` interpreta o `\c` de `C:\agent\config.json` (linha 87 do log do Windows) e corta o log de 1779 para 86 linhas. A linha sai com rc=1.
  - É falso negativo, nunca um `OK` oco: as checagens positivas (contagem = total e o `comm`) falham juntas.
  - Se algum passo seguinte (`/jdi-confirm-dod`, `/jdi-ship`) rodar os `Verify:` no zsh do usuário, a linha 5 vai reprovar à toa.
  - Nas próximas reescritas, usar `printf '%s\n' "$LW"` no lugar de `echo`.

### `2871dc6` — orçamento `UNHURRIED` nos testes de panic (iteração 2; continua válida, o código não mudou)
- **O que mudou (só `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`):**
  - `const UNHURRIED = 10 s`, com comentário que cita o run 36345194940.
  - O `doomed` passa de 1 s para `UNHURRIED`.
  - No `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving`, 1 s vira `UNHURRIED`.
  - No `maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors`, o orçamento troca para `UNHURRIED` via `client.with_budgets(..)`, só depois de o `stuck` voltar e o gate abrir.
  - O `with_budgets` já existia (`worker.rs:397-399`) e mantém o MESMO worker.
- **Nenhuma asserção mudou:** são 61 blocos `assert*!` idênticos, 21 `#[test]` antes e depois, e nenhum `#[ignore]` novo.
- **O timeout real continua provado:**
  - `stuck` com 250 ms dá `Err(Timeout)`;
  - `caller_receives_timeout_when_worker_does_not_answer_within_budget` com 20 ms dá `Timeout` e `waited < 1 s`;
  - `expired_write_is_never_sent_to_the_monitor` com 20 ms dá `Timeout`, e nenhum `Write` é enviado.
- **Mutantes da iteração 2:**
  - panic 1,3 s mais lento: 21/21 passam no HEAD, e os 2 testes falham em `2871dc6^`, como no CI;
  - panic 11 s mais lento: os 2 falham no HEAD;
  - sem o `catch_unwind` em `isolated`: 3 falham.
- **O que se perdeu:** só um limite incidental de relógio de parede para a volta do worker depois de um panic. Nenhum nome de teste, asserção ou D-XX afirma esse limite.
  - As exigências da D-2026-09-26-full-osd-control-6 se mantêm: `Transport` com o texto do panic, a leitura seguinte `Ok` no mesmo backend, nenhum panic hook global.
  - "Sem retry nem checagem de presença depois de panic" continua provado em relógio virtual (`took == 0`).
- **Julgamento:** não enfraquece nenhum teste e não esconde regressão.

### Conformidade D-XX (iteração 2; continua válida, código e template inalterados)
- **D-1 / D-2 (projeto):** o `ddc-core` está intocado. A única mudança Rust da phase é teste de adapter.
- **ci-crossbuild-1:** conforme.
  - Gatilhos, `concurrency` com `cancel-in-progress` e `permissions: contents: read` inalterados.
  - Os 2 `uses:` estão em `@9e91d1ac9604aa581ac17438c54d75a2fcc213fa` = head do #13, conferido agora.
  - `ci-evidence.env` tem todas as chaves.
- **ci-crossbuild-2:** conforme. O `so: windows-*` restrito a rust é retrocompatível, porque o campo é novo neste mesmo PR. Composites e `bin/` intocados.
- **ci-crossbuild-3 / -4:** conforme. `Build ddc-tray (release)` em success nos dois SOs.
- **ci-crossbuild-5 / -6:** conforme. `build.rs`, o manifesto e `.cargo/audit.toml` não mudaram. O `audit.toml` do HEAD é igual ao de `9ef86cc^`.
- **ci-crossbuild-7:** conforme.
  - (a) O NEG `9ef86cc` só remove as linhas da exceção. O run 36345194940 dá `failure`, com o `cargo audit` do `rust-linux` em failure e `RUSTSEC-2018-0005` no log.
  - (b) O worktree da linha 8 foi refeito agora.
- **ci-crossbuild-8:** conforme. O `versao` só calcula.
- **D-2026-09-26-full-osd-control-6:** conforme (ver acima).

### Revisão de github-workflows#13 (iteração 2; continua válida, head `9e91d1a` inalterado)
- **Interpolação:** nenhum `${{ }}` novo dentro de `run:`. Nenhum `|| true` novo, nenhum download novo, permissões e secrets inalterados.
- **Pins:** `python3 bin/pinar_actions.py --verificar` aprova os 15 arquivos.
- **Commits:** `bin/versao.py conferir` aprova `33e00c7` e `9e91d1a`.
- **Sintaxe:** actionlint 1.7.12 limpo, local e no check `Sintaxe dos workflows` do gw (linha 2 do DoD, conferida agora).
- **PR #13:** OPEN, fora do rascunho, head `9e91d1a`, MERGEABLE (conferido agora).

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run de evidência em success no `HEAD_SHA`; commit ancestral do HEAD com a mesma árvore de código (`crates`, `apps`, `Cargo.*`, `.cargo`, `clippy.toml`, `rustfmt.toml`, toolchain); `uses:` do gw em `@WORKFLOWS_SHA`; `ci.yml` do HEAD igual, a menos da referência | CONTEXT | Auto | PASS | `OK` (bash, exit 0). Run 36345684775 = `success 2871dc6…`; depois dele só `.jdi/` mudou; 2 `uses:` em `@9e91d1a…` |
| 2 | `referenced_workflows` de `RUN_ID` e `NEG_RUN_ID` em `WORKFLOWS_SHA`; composites `@main` sem diferença de `WORKFLOWS_SHA` em `.github/actions`/`bin`; head do #13 = `WORKFLOWS_SHA`; `Scripts`, `Pins das actions` e `Sintaxe dos workflows` em success, sem check falho ou cancelado | CONTEXT | Auto | PASS | `OK` (bash, exit 0). headRefOid do #13 = `9e91d1a…`, conferido agora |
| 3 | Um `rust-linux` (`ubuntu-latest`) e um `rust-windows` (`windows-latest`), com os passos do contrato em success | CONTEXT | Auto | PASS | `OK` (bash, exit 0). Jobs: `qualidade / rust-linux` success ubuntu-latest, `qualidade / rust-windows` success windows-latest; Linux 6/6, Windows 5/5 |
| 4 | Log do `rust-linux`: teste da CSP ok, `test result: ok`, nenhum binário falho, painel ≥ 80 | CONTEXT | Auto | PASS | `OK` (bash, exit 0). `**Aprovado:** 82.93% >= piso de 80%`; 386 passed |
| 5 | Log do `rust-windows`: teste da CSP ok, sem falha, sem `STATUS_ENTRYPOINT_NOT_FOUND`; mesmo número de binários do Linux; conjunto `nome:estado` igual ao do Linux a menos da lista congelada (16 só no Linux, 4 só no Windows); nomes = passed + ignored em cada log | CONTEXT | Auto | PASS | `OK` (bash, exit 0). 11 = 11 binários; Windows 383 nomes = 374 + 9, Linux 395 = 386 + 9; 379 comuns; a lista bate 1:1 com os `cfg` do código (Notas); 4 mutantes recusados |
| 6 | `node-ui` e `npm test` em success; TAP `# pass` ≥ 150 e `# fail 0`; Playwright ≥ 130 passed, nenhum failed | CONTEXT | Auto | PASS | `OK` (bash, exit 0). `# pass 161`, `# fail 0`, `138 passed`, 6 skipped |
| 7 | Portão de auditoria morde no CI real: `NEG_SHA` só remove linhas de `audit.toml`; `ci.yml` do NEG = do `HEAD_SHA`; run falha em `cargo audit` com o advisory no log; `audit.toml` do HEAD = anterior ao NEG | CONTEXT | Auto | PASS | `OK` (bash, exit 0). NEG `9ef86cc`, run 36345194940 `failure`, `cargo audit` do `rust-linux` = failure com `RUSTSEC-2018-0005` |
| 8 | `audit.toml` com exatamente as 4 exceções e nada mais em `[advisories]`; `cargo audit` sai 0 no HEAD; no worktree sem 2018-0005 sai ≠ 0 citando o advisory; worktree removido | CONTEXT | Auto | PASS | `OK` (bash, exit 0). `git worktree list` só lista o principal depois do comando; `cargo audit` 0.22.2, 1271 advisories |
| 9 | Nenhum `TODO`/`FIXME` sem issue em nenhum arquivo versionado do produto (inclusive não-`.rs`) | CONTEXT | Auto | PASS | `OK` (bash, comando literal, exit 0) |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (bash, exit 0). 15 binários, 386 passed / 0 failed / 9 ignored |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | `cargo llvm-cov --workspace --summary-only` literal, exit 0: TOTAL Lines 82.93%. Com a exclusão do Gate 3, no mesmo profile: 83.36% |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` (bash, comando literal, exit 0) |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou Added (CI com `rust-linux`/`rust-windows`/`node-ui`/`versao`, `.cargo/audit.toml`, `npm test`) e Fixed (tauri#13419). Sem heading de versão nova: não houve release nesta phase (D-8) |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## CI` em README.md:468; README +31/-4 na phase |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod ci-crossbuild` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Nada bloqueia. A iteração 3 não mexeu em código, o que está provado por `git diff --quiet cb8eea2 HEAD -- . ':!.jdi'` = 0 e pelo head do #13 = `9e91d1a`. Os 12 `Verify:` automáticos deram `OK` literalmente agora, em `bash`, contra os runs reais 36345684775 (verde) e 36345194940 (NEG).

A nova linha 5 é sólida:
- a lista congelada de 16 + 4 corresponde exatamente aos `cfg(target_os = "linux")`, `cfg(unix)` e `cfg(not(target_os = "linux"))` do código;
- todos esses `cfg` são de phases anteriores já mescladas e justificados pela API da plataforma;
- os 4 mutantes (teste fora, `ignored`, teste novo e módulo trocado só no Windows) foram recusados.

Próximos passos:
1. Confirmar os 2 manuais (CHANGELOG/README) com `/jdi-confirm-dod ci-crossbuild`, ou deixá-los para o PR, conforme a cadeia autônoma.
2. Se algum passo seguinte rodar os `Verify:` de novo, usar `bash`: no zsh, a linha 5 dá falso negativo. Numa reescrita futura, `printf '%s\n'` no lugar de `echo` resolve.
3. Seguir a ordem do Deferred:
   - fazer o merge do github-workflows#13 com a mensagem de squash sugerida no corpo;
   - trocar os dois `@9e91d1a…` por `@main` no `.github/workflows/ci.yml` e esperar um run verde;
   - tirar o #10 do rascunho (hoje está `draft=true`, head remoto `0aaaf1c`) e atualizar o corpo.
4. Estado da branch:
   - está 1 commit atrás de `origin/main` (`e72790d`, só `.jdi/`) e 4 commits locais à frente de `origin/phase/ci-crossbuild` (só `.jdi/`, não enviados);
   - o #10 aparece como MERGEABLE.
5. Follow-ups no github-workflows, fora desta phase:
   - W-3: abrir uma issue no merge;
   - W-6: lista explícita de rótulos em `so`.
6. Qualquer mudança futura em `crates/`, `apps/`, `Cargo.*`, `.cargo/` ou no toolchain invalida o `HEAD_SHA` (linha 1 do DoD) e exige nova evidência.
