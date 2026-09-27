# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 9 (rodada 2). Re-verificação completa dos gates 1-8 sobre o `HEAD` `19f86b3`. Cobre os commits `4423525..f98c10a` e o `19f86b3`, que refixa o hash do harness revisado; o manifesto agora inclui `scripts/smoke-sni.sh` (D-2026-09-27-tray-app-8).

A revisão foi read-only: no repo, só este REVIEW.md foi escrito. As mutações rodaram numa cópia descartável de `apps/ddc-tray` no scratchpad, já apagada. `git status` terminou limpo, exceto por este arquivo e pelo `.idea/`, que foi ignorado.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli` em `x86_64-pc-windows-msvc`: exit 0. O tray fica fora do cross-check Windows (D-2026-09-26-tray-app-9). |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware). É igual à iter 8, porque nenhum `.rs` mudou. `node --test`: 141 pass (eram 135), 0 fail/cancelled/skipped/todo. |
| Coverage | PASS | 83.22% de lines na linha TOTAL (3271 linhas, 549 sem cobertura), com `main.rs`/`build.rs` excluídos e `--fail-under-lines 80` exit 0. Literal do PROJECT, sem exclusões: 82.78%. |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Nenhum `#[allow(...)]` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. 5.10 = W-1, conhecido, fica com a `ci-crossbuild`. |
| Consistency | PASS | 6 commits `(tray-app)` com tipos coerentes (feat/test/docs) e D-XX citada no corpo. Nenhum commit mistura código e `.jdi/`. `Cargo.lock`, `package.json` e `package-lock.json` estão sem diff. D-1, D-2, D-2026-09-26-tray-app-2/-3/-6 e D-2026-09-27-tray-app-6/-7/-8 estão conformes. |
| UI Validation | PASS (com W-2) | Playwright: 110 passed, 6 skipped (= screenshots sem `SCREENSHOTS=1`), 0 failed/flaky. Sub-contagens: `n = 10`, dropdown 16/16, `dr = 2`, `ps = 40`. Caminhos `fail=`: 8/8. Pseudo-locale: 42/42. |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings

- **W-1 (5.10, herdado, fica com a `ci-crossbuild`):** `cargo audit` acha 1 vulnerabilidade e 3 avisos, os mesmos das iters 7 e 8.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5).
  - Avisos: RUSTSEC-2024-0370 (`proc-macro-error`, sem manutenção), RUSTSEC-2024-0320 (`yaml-rust`, sem manutenção) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound).
  - Nenhuma dependência nova entrou na iter 9.

- **W-2 (Gate 7 / D-2026-09-27-tray-app-7, novo): dois toasts de falha continuam fora de todo estado do pseudo-locale.** É a mesma classe de lacuna que o DoD critic da iter 8 apontou. A iter 9 fechou 3 dos 5 pontos em que `app.js` mostra um erro num toast (`writeFailed`, "Todos os ajustes" e a sondagem). Ficaram de fora:
  - `apps/ddc-tray/src/app.js:200`, no `catch` de `listen()`, quando registrar `popup-shown`/`panel-changed` falha;
  - `apps/ddc-tray/src/app.js:207`, no `.catch` de `bridge.hidePopup()` (Esc).
  - **Por que a suíte não vê esses dois:** o demo nunca os faz falhar. O `listen` do demo (`bridge.js:152`) e o do bridge indisponível (`bridge.js:105`) nunca rejeitam, e `FAILURES` (`demo-data.js`) só tem `write`/`features`/`probe`. No app real, os dois rejeitam se o IPC do Tauri falhar: o `invoke` de `hide_popup` e o `event.listen` (`bridge.js:95`) rejeitam por permissão ou IPC, mesmo que o comando Rust `hide_popup` (`commands.rs:233`) devolva `()`. O SUMMARY exclui o `hide_popup` com o argumento "o comando real nunca rejeita" e não menciona o `listen`.
  - **Prova (mutante independente, cópia descartável):** `const EVENTS_OFF = ' Changes made from the tray will not show here.'`, com `showToast(errorText(error, t) + EVENTS_OFF)` nas duas linhas. Resultado: suíte Playwright inteira com **110 passed, 0 failed**, e `i18n-html` + `view-model` com **41 pass**. Ou seja, um literal hardcoded (D-2026-09-26-tray-app-6) passaria por todos os gates.
  - **Sugestão:** estender o `fail=` do demo com, por exemplo, `events` (o `listen` do demo rejeita) e `hide` (o `hide_popup` do demo rejeita). Depois, acrescentar os 2 estados ao `pseudo-locale.spec.mjs` e 2 cenários de texto exato ao `scenarios.spec.mjs`, como a iter 9 fez para os outros três. Outra saída é registrar numa D-XX por que esses dois ficam de fora.
  - Não é blocker: nenhum `Verify:` falha, e a D-7 enumera os estados cobertos sem prometer todos os toasts. Mas é a lacuna que o critic procuraria a seguir.

## Blocker da iter 8 (DoD critic) conferido

- **Textos de falha fora dos estados do pseudo-locale: FECHADO para escrita, "Todos os ajustes" e sondagem.** O resto está no W-2.
  - **Demo:** `apps/ddc-tray/src/bridge.js:109-210` e `demo-data.js:21-43`. O `fail=` só existe dentro de `demoBridge`, que só roda em servidor local (D-2026-09-27-tray-app-6). `tests/ui/bridge-demo.test.mjs` confirma que `tauri://localhost/?demo=rtk&fail=write` e `http://tauri.localhost/?fail=features,probe` continuam `backend_unavailable`.
  - **Mensagem de timeout:** vem do core. O teste lê o `#[error("…")] Timeout` de `crates/ddc-core/src/domain/error.rs:35` ("monitor did not respond in time") e compara com a do demo.
  - **Ordem:** a escrita ainda passa por `needs_confirmation`/`invalid_value` antes de expirar, e o monitor ainda é procurado antes (`not_found`, e `transport` para a TV muda).
  - **Pseudo-locale** (`tests/e2e/pseudo-locale.spec.mjs:56-84, 207-272`): 7 estados novos.
    - Os estados de espera usam uma variante do `bridge.js` servida por `page.route`, com a mesma trava de `slider`/`dropdown`/`confirm`: o spec exige que a linha remendada exista no `bridge.js` (`the demo line this state patches`).
    - `textsOf`/`judge`, os mínimos de leitura, o teste `loading` e o da origem do app não mudaram.
  - **Cenários** (`tests/e2e/scenarios.spec.mjs:169-252`): 4 caminhos × 2 temas, com texto exato `t('error.timeout')` no toast e no announcer. O controle volta ao valor lido (`75`/`75%`, preset `0x01`/`sRGB`), `writes` fica vazio, e passam o axe e o coletor de console.
  - **Mutante do critic refeito de forma independente:** `const NOT_APPLIED = ' The monitor kept its previous value.'` e `showToast(errorText(error, t) + NOT_APPLIED)` em `writeFailed` (`app.js:681`). Resultado: **8 failed**.
    - Pseudo-locale, 4 falhas: escrita de brilho e troca de preset × 2 temas, com `has text outside the marks: " The monitor kept its previous value."`.
    - Cenários `fail=write`, 4 falhas: texto exato.
- **TODO minúsculo / `todo!(`:** resolvido pelo orquestrador no `Verify:` (D-2026-09-27-tray-app-8). C18 e P3 passam com a regra nova.

## Observações (não bloqueiam, não são warnings)

- **Harness congelado (D-2026-09-27-tray-app-7/-8):**
  - O hash recalculado é `b911f1d74c2d5178d82e382eefa47363469b8d079dcbe09759c9b88a5d8a6e87`, sobre 20 arquivos (`smoke-sni.sh` incluído), e bate com o CONTEXT.
  - A última mudança no harness é `2a8b59d`, anterior ao refixo em `19f86b3`.
  - Li os diffs dos 3 arquivos alterados: só entram testes, estados e asserções. `support.mjs` (coletor de console/pageerror, axe, `expectNoNativeSelect`), `playwright.config.mjs` e `smoke-sni.sh` não mudaram.
- **Contagem no SUMMARY:** a iter 9 diz "21 estados × 2 temas + `loading` × 2 + origem do app × 2 = 42/42". A soma não fecha (daria 46). A lista real tem 19 estados em `STATES` + `loading` = 20 estados × 2 = 40 (`ps = 40`), mais a origem × 2 = 42. O README ("20 states") está certo.
- **Limites de desenho já registrados (iters 7/8):** um literal colado a um *dado* sob `translate="no"` não é checado. Um literal passado como parâmetro de `t()` sai dentro das marcas, e quem o pega são as asserções exatas.
- **Para o `/jdi-ship` (fora dos gates):** `origin/main` está em `1bca909`, o squash do PR #7. `phase/tray-app` tem 108 commits à frente, e 25 deles são os commits originais da `full-osd-control`. Como o usuário mergeia por squash, é preciso rebasear sobre o `origin/main` antes de abrir o PR.
- **`nom` 3.2.1:** o aviso de future-incompat de sempre, via `ddc-hi` 0.4.1. Não é desta phase.
- **Smokes e instância do usuário:** os smokes rodaram um de cada vez. Antes de cada um, `pkill -x ddc-tray` + `pidwait`; logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray`. PIDs do usuário: 1569412 → 1596371 → 1596918, que é a única instância viva no fim. A porta 1420 ficou livre antes e depois.

## Gate 5 (detalhe)
| Check | Resultado |
|---|---|
| 5.1 deps do core | só `thiserror` (sem saída) |
| 5.2 I/O/cfg de plataforma no core | sem saída |
| 5.3 portas | nenhum `impl MonitorBackend` no core; nenhum `pub trait` fora do core |
| 5.4 composition roots | sem saída nas duas buscas |
| 5.5 `unsafe` | único hit em `apps/ddc-tray/src-tauri/src/lib.rs:146`, que é doc comment (por que o `exec` evita `set_var`); `#![forbid(unsafe_code)]` em `ddc-core`/`ddc-cli`/`ddc-tray` (a busca por raiz sem o atributo não teve saída) |
| 5.6 panics | todos os hits estão em `#[cfg(test)]`: `kwin_placement.rs` (mod na linha 195), `lib.rs` (`switch_tests`, linha 239) e `stop_signals.rs` (linha 82). O `worker.rs:141` é doc comment |
| 5.7 escrita no monitor | `Dangerous` classificado no core (`feature.rs:33`). Nenhum `Confirm::Yes` fora de `ddc-cli`/`src-tauri/src/commands*`/testes. `DdcHiMonitorBackend` em testes só aparece em `crates/ddc-adapters/tests/real_monitor.rs`, com 7/7 `#[ignore]` + `DDC_HW_TESTS=1`. `apps/.../lib.rs:28,77` é o composition root |
| 5.8 paths de device no core | sem saída |
| 5.9 comandos Tauri | todos `async fn` |
| 5.10 supply chain | W-1 |
| 5.11 segredos/TODO | sem saída |

## DoD Checklist (gate 8)

Os `Verify:` foram extraídos por script do `.md` e rodados literalmente em `bash`, a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`. O P2 (formato "`cmd` → coluna Lines") foi rodado literalmente, e a coluna foi lida a olho.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos, incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 3 | Só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1× em `lib.rs:77`) |
| 4 | `panel.rs` é Rust puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 5 | `#![forbid(unsafe_code)]` em `src-tauri`, nenhum `unsafe` | CONTEXT | Auto | PASS | `OK` |
| 6 | `node --test` (debounce, view-model, bridge demo…), 0 falhas | CONTEXT | Auto | PASS | `OK`: 141 pass, 0 fail/cancelled/skipped/todo |
| 7 | Paridade i18n + nenhum texto hardcoded no HTML/scanner | CONTEXT | Auto | PASS | `OK` |
| 8 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | `OK` |
| 10 | single-instance é o 1º plugin do builder | CONTEXT | Auto | PASS | `OK` |
| 11 | Smoke SNI `--activate` (PID próprio, popup mostrado e mantido) | CONTEXT | Auto | PASS | `OK`: PID 1596135, `org.kde.StatusNotifierItem-1596135-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras |
| 12 | Teste de hardware `#[ignore]` gated por `DDC_HW_TESTS=1` | CONTEXT | Auto | PASS | `OK`: 2 testes listados, não executados |
| 13 | Gate 7: console/axe nos `critical_paths` + dropdown + arrasto + pseudo | CONTEXT | Auto | PASS | `OK`: 110 passed, 6 skipped; `n=10`, `dd=dl=16`, `sk=sl=6`, `dr=2`, `ps=40` (ver W-2) |
| 14 | Bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 15 | Harness igual ao revisado (hash do manifesto) | CONTEXT | Auto | PASS | `OK`: `b911f1d7…6e87`, 20 arquivos |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`: 20 testes `scroll`; PID 1596683 com `75 -> 80` na vertical, nada na horizontal em 1,5 s, `80 -> 75` |
| 18 | Nenhum TODO/FIXME/`todo!` sem issue nos arquivos versionados do produto | CONTEXT | Auto | PASS | `OK` (regra da D-2026-09-27-tray-app-8) |
| 19 | Screenshots claro/escuro regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | `OK`: 6 passed; 720×1120; SHA-1 dos 6 PNGs inalterados |
| 20 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | `OK`: 383 passed, 0 failed, 9 ignored |
| 21 | Cobertura >= 80% de lines | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL lines 82.78% (gate com exclusões: 83.22%) |
| 22 | Nenhum TODO/FIXME/`todo!` sem issue em `*.rs` | PROJECT | Auto | PASS | `OK` (regra da D-2026-09-27-tray-app-8) |
| 23 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` em `CHANGELOG.md:8`, com o tray em Added, que agora descreve o `fail=` e os 20 estados |
| 24 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`); `fail=` em `README.md:345`; rótulos "All settings"/"Probe hidden settings" batem com `i18n/en.js:16,19` |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rode `/jdi-confirm-dod tray-app` para confirmar cada item manual com evidência. Sem isso, o `/jdi-ship` recusa a phase.

## Recommendation
A iter 9 fecha o blocker do DoD critic da iter 8 para os textos de falha de escrita, de "Todos os ajustes" e da sondagem, sem enfraquecer o harness, e o hash refixado bate. Todos os gates auto passam. Os próximos passos:

1. **Decidir o W-2 antes de rodar o critic de novo.** Os toasts de `app.js:200` (`listen`) e `app.js:207` (`hide_popup`) aceitam um literal colado sem que nenhum gate perceba. Há duas saídas: uma rodada curta que estende o `fail=` (`events`/`hide`) e acrescenta os 2 estados, ou uma D-XX que os deixa de fora com justificativa.
2. **W-1** continua com a `ci-crossbuild`.
3. **Antes do PR:** rebasear `phase/tray-app` sobre o `origin/main` (`1bca909`), rodar `/jdi-confirm-dod tray-app` para os 2 itens manuais e anexar ao PR a saída do `rtk_qhd_hdr` com `DDC_HW_TESTS=1`, rodado pelo orquestrador.
