# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 6 (rodada 2, depois do AUTO-RESET 1), commits `31150bd..b28784d`. Nesta iteração não mudou nenhum `.rs`, `Cargo.*` ou `package*.json`: entraram 2 arquivos de teste (`tests/e2e/slider.spec.mjs`, `tests/ui/i18n-html.test.mjs`), README, CHANGELOG e SUMMARY. Todos os gates rodaram do zero em 2026-09-27, em Linux (Fedora 44, KDE Wayland). Cada `Verify:` foi extraído por script do `.md` e rodado literalmente com `bash`, a partir da raiz.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli` em `x86_64-unknown-linux-gnu` exit 0 e em `x86_64-pc-windows-msvc` exit 0. O tray fica fora do cross-check Windows (D-2026-09-26-tray-app-9). |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware: 7 `real_monitor` + 2 `rtk_qhd_hdr`). Igual à iter 5. |
| Coverage | PASS | TOTAL Lines **83.22%** (gate: `--fail-under-lines 80`, `main.rs`/`build.rs` excluídos), exit 0. O literal do PROJECT (`cargo llvm-cov --workspace --summary-only`, sem exclusão) dá 82.78%. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 limpos (detalhes abaixo). 5.10: W-1 (`cargo audit`), conhecido e adiado para `ci-crossbuild`. |
| Consistency | PASS | Commits `test`/`docs(tray-app)`, sem misturar código e `.jdi/`. Os arquivos novos estão nos globs do PLAN (`tests/e2e/*.spec.mjs`, `tests/ui/*.test.mjs`). D-1, D-2 e D-tray-app-2/-5/-6/-8 conferidos, sem violação. |
| UI Validation | PASS | Suíte Playwright do app (D-2026-09-26-tray-app-8): **60 passed**, 6 skipped (só screenshots sem `SCREENSHOTS=1`), 0 failed/flaky. Os 10 `critical_paths`×tema com ✓, o dropdown completo e `dragging the brightness slider…` ✓ nos 2 temas. |
| DoD | PASS_PENDING_MANUAL | 21/21 auto PASS, 2 manuais pendentes |

### Gate 5 em detalhe
- **5.1 (deps do core):** sem saída. `ddc-core` só depende de `thiserror`.
- **5.2 (I/O e cfg de plataforma no core):** sem saída.
- **5.3:** `impl MonitorBackend` no core, sem saída. Nenhum `pub trait` em adapters/CLI/apps.
- **5.4:** sem saída nos dois greps. O adapter real só é construído em `apps/ddc-tray/src-tauri/src/lib.rs:77` (composition root).
- **5.5:** o único hit é `lib.rs:146`, e é doc comment (`/// … takes \`unsafe\` (std::env::set_var)`), não código. Nenhum `unsafe {` em `ddc-adapters`. Todas as raízes de crate têm `#![forbid(unsafe_code)]`.
- **5.6:** os hits de `unwrap`/`expect` estão todos em `#[cfg(test)] mod tests`: `kwin_placement.rs` depois de `:195`, `lib.rs` depois de `:239`, `stop_signals.rs` depois de `:82`. `worker.rs:141` é doc comment.
- **5.7 (monitor-write safety):**
  - A classificação de risco existe (`feature.rs:33`, `:156` — código desconhecido cai em `Dangerous`).
  - Nenhum `Confirm::Yes` fora de `ddc-cli`/`src-tauri/src/commands*`.
  - Os 7 testes de `real_monitor.rs` são `#[ignore]` e gated por `DDC_HW_TESTS`. Na suíte do tray, `DdcHiMonitorBackend` só aparece em `lib.rs`.
- **5.8 (paths de dispositivo no core):** sem saída.
- **5.9 (comandos Tauri bloqueantes):** sem saída. Todo `#[tauri::command]` é `async fn`.
- **5.10:** ver W-1.
- **5.11 (segredos e TODO/FIXME):** sem saída nos dois greps.

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, adiado para `ci-crossbuild` pelo orquestrador):** `cargo audit` acusa 1 vulnerabilidade e 3 avisos permitidos. `Cargo.lock` não mudou nesta iteração.
  - **RUSTSEC-2018-0005:** `serde_yaml 0.7.5`, puxado por `mccs-db 0.1.3` ← `ddc-hi 0.4.1`. Vem da phase `ddc-backends`.
  - **RUSTSEC-2024-0429 (unsound):** `glib 0.18.5`, puxado por `webkit2gtk`/Tauri. Entrou nesta phase.
  - **Unmaintained:** `proc-macro-error` (RUSTSEC-2024-0370) e `yaml-rust` (RUSTSEC-2024-0320).

### Observações (não bloqueiam, sem ação exigida)
- **Relógio parado no teste de arraste (avaliação pedida pelo orquestrador): o teste NÃO ficou oco.**
  - **Por que o relógio falso vale para o produto:** `debounce.js:8-11` e `bridge.js:21` resolvem `globalThis.setTimeout` na hora da chamada, então o `page.clock.install()` feito depois do load controla de fato o debounce e a latência do demo.
  - **Mutações reproduzidas pelo reviewer** numa cópia isolada de `apps/ddc-tray` no scratchpad (o repositório não foi tocado). Controle: 4/4 ✓.
    - **M1**, a mutação do critic: `queueWrite(entry, value, { now: true })` em `app.js:508`. Resultado: **4 ✘**. `dragging…` falha em `writes taken at each input`.
    - **M2**, throttle: sem o `stopTimer(state)` do `push` em `debounce.js:82`. Resultado: **4 ✘**, com a mesma asserção.
    - **M3**, extra: sem `writes.flush` no `change` (`app.js:513`), ou seja, a escrita só sai quando o debounce vence, não no release. Resultado: `dragging…` **2 ✘** (o `expect.poll` da escrita única).
  - **O que o relógio parado mudou nas asserções:**
    - "≥ 300 ms" e "pausa < `DEBOUNCE_MS`" agora são medidas no relógio falso, que o próprio teste avança em 20 ms por passo. Viraram verificações de sanidade da instrumentação, deterministas por construção.
    - Quem prova o critério são `writes taken at each input` = 0, `writesAtRelease` = 0, a escrita única com o último valor e `landedAt − releasedAt === DEMO_LATENCY_MS`.
    - Com relógio falso, o M3 cai já no `poll`. Com relógio real, só cairia na asserção de latência. A troca deixou o teste mais estrito, não menos.
- **Scanner de texto literal (`i18n-html.test.mjs`, teste 8):** o reviewer aplicou 3 mutações novas na cópia isolada, e cada uma derrubou exatamente o teste 8 (`# fail 1`).
  - 3º argumento literal em `element()`;
  - `pill.title = \`Level of ${…}\``;
  - `setAttribute('aria-label', 'Value')`.
- **`THUMB_PX = 18` em `slider.spec.mjs:26`** repete `width: 18px` de `styles.css:544`. São só 2 lugares, com comentário apontando a origem, então não chega a ser achado de DRY (a regra pede 3+).
- **Para o `/jdi-ship`:** `origin/main` está em `2628f8c` (`ddc-backends`). `phase/tray-app` está empilhada sobre `full-osd-control`, ainda não mergeada, com 88 commits à frente de `origin/main`. Como o usuário faz squash-merge, a ordem ou o rebase das PRs precisa ser tratada no ship.
- **Instância do usuário:** encerrada com `pkill -x ddc-tray` antes de cada smoke (C11, C16) e reaberta com `setsid -f /home/slipalison/.local/bin/ddc-tray` logo depois. PIDs: 1110145 → 1131071 → 1131592, que é a única instância viva no fim. A porta 1420 estava livre antes e depois de C13/C18. Nenhum teste `#[ignore]` rodou, e nada foi escrito no monitor real (C11 usa o backend real só com leituras; C16 usa o monitor simulado).

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` (exit 0) |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 3 | só o composition root constrói `DdcHiMonitorBackend` (1×) | CONTEXT | Auto | PASS | `OK` (`lib.rs:77`) |
| 4 | `panel.rs` Rust puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 5 | `#![forbid(unsafe_code)]` em `src-tauri`, nenhum `unsafe` | CONTEXT | Auto | PASS | `OK` |
| 6 | `node --test` (debounce com rajada > janela, view-model, bridge demo…) | CONTEXT | Auto | PASS | `OK`: 131 pass, 0 fail/cancelled/skipped/todo, nenhum teste em `src/` |
| 7 | paridade i18n + nenhum texto hardcoded (inclui `app.js passes no literal text to element() or setText()`) | CONTEXT | Auto | PASS | `OK`: `i18n-html` 15/15, `ok 8` presente |
| 8 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 9 | capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 10 | single-instance é o 1º plugin do builder | CONTEXT | Auto | PASS | `OK` |
| 11 | smoke SNI `--activate` (PID próprio, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | `OK`: PID 1130834, `org.kde.StatusNotifierItem-1130834-1`, `popup shown` e ainda mostrado 1,5 s depois |
| 12 | teste `#[ignore]` de hardware gated, guard vivo, `DDC_TRAY_FAKE` recusado por teste | CONTEXT | Auto | PASS | `OK` (listado, não executado — regra do reviewer) |
| 13 | Gate 7: console/axe travados, 10 `critical_paths`, dropdown completo, `dr = 2` | CONTEXT | Auto | PASS | `OK`: 60 passed, 6 skipped (= screenshots), 0 failed/flaky |
| 14 | bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 15 | nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | `OK` |
| 16 | `ksni` + ≥3 testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`: PID 1131344, `75 -> 80` vertical, horizontal sem escrita em 1,5 s, `80 -> 75` |
| 17 | nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | `OK` |
| 18 | screenshots claro/escuro regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | `OK`: SHA-1 dos 6 PNGs iguais antes e depois, `git status` limpo |
| 19 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | `OK` (383 passed, 0 failed, 9 ignored) |
| 20 | cobertura ≥ 80% de linhas | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL Lines 82.78% (gate com exclusões: 83.22%) |
| 21 | nenhum TODO/FIXME sem issue em `*.rs` | PROJECT | Auto | PASS | `OK` |
| 22 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (CHANGELOG.md:8) descreve o tray, inclusive o arraste do slider e o scanner de texto literal (`f9e263d`); ainda não há heading de versão |
| 23 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: README cita os testes novos (`f9e263d`); revisar o diff na PR |

**Totals:** 23 items | Auto: 21 (21 PASS, 0 FAIL) | Manual: 2 pending. Os 2 manuais aparecem nas duas DoDs, a do PROJECT e a do CONTEXT (`Source: PROJECT`), e contam uma vez só.

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Aprovar com os 2 itens manuais pendentes. O que a iteração 6 trouxe resolve as 2 lacunas do critic da iter 5:
- **Arraste real do slider:** a mutação `now: true`, o throttle e o "sem flush no release" derrubam o teste, reproduzido de forma independente pelo reviewer.
- **Texto literal no DOM:** o scanner reprova 3 formas novas de texto literal, também testadas pelo reviewer.

O relógio parado não deixa o teste oco. W-1 continua com a `ci-crossbuild`. Os itens Deferred do CONTEXT ficam para o corpo da PR:
- ancoragem e roda no KDE, na máquina do usuário;
- o `rtk_qhd_hdr` com `DDC_HW_TESTS=1`, rodado pelo orquestrador;
- Windows real;
- GNOME sem AppIndicator.

No ship, tratar o empilhamento sobre `full-osd-control`, porque o usuário faz squash-merge.

## DoD Critic (enhanced)

- DoD row «7 (i18n)»: literal guardado numa `const` local (`const note = … ? 'No other setting answered' : t(…)`, padrão de `app.js:857`) ou devolvido por helper chega ao DOM em inglês no popup pt-BR e o scanner estático não vê (Verify OK).
- DoD row «13 (Gate 7)»: `if (message.text().startsWith('Failed to load resource')) return;` antes das linhas travadas do coletor esconde um 404 real; e `if (state !== 'empty') await expectAccessible(page);` no loop dos `critical_paths` pula o axe de `/?demo=empty` — Verify OK nos dois casos.

**Verdict:** BLOCKED
