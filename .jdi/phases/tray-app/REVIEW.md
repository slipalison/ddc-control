# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 8 (rodada 2, rodada de warnings). Re-verificação completa dos gates 1-8 sobre o `HEAD` `8382d9c`, cobrindo os commits `06ea05b..0f89eb0` e o `8382d9c`, que refixa o hash do harness revisado.

A revisão foi read-only: no repo, só este REVIEW.md foi escrito. As mutações rodaram numa cópia descartável de `apps/ddc-tray` no scratchpad, já apagada. `git status` terminou limpo, exceto por este arquivo e pelo `.idea/`, que foi ignorado.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli`: exit 0 em `x86_64-unknown-linux-gnu` e em `x86_64-pc-windows-msvc`. O tray fica fora do cross-check Windows (D-2026-09-26-tray-app-9). |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware), igual à iter 7, porque nenhum `.rs` mudou. `node --test`: 135 pass, 0 fail/cancelled/skipped/todo. |
| Coverage | PASS | 83.22% lines na linha TOTAL, com `main.rs`/`build.rs` excluídos e `--fail-under-lines 80` exit 0. Literal do PROJECT, sem exclusões: 82.78%. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. 5.10 = W-1, conhecido, fica com a `ci-crossbuild`. |
| Consistency | PASS | 5 commits `(tray-app)`, com tipos coerentes (test/fix/docs). Nenhum `.rs`, `Cargo.lock` ou `package*.json` foi tocado. D-1, D-2, D-2026-09-26-tray-app-6 e D-2026-09-27-tray-app-7 estão conformes. |
| UI Validation | PASS | Playwright: 88 passed, 6 skipped (= screenshots sem `SCREENSHOTS=1`), 0 failed/flaky. Sub-contagens: `n = 10`, dropdown 16/16, `dr = 2`, `ps = 26`. O teste do texto copiado da dica i2c (W-3) passou nos 4 estados `empty`/`error` × tema. |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings

- **W-1 (5.10, herdado, fica com a `ci-crossbuild`):** `cargo audit` achou 1 vulnerabilidade e 3 avisos, os mesmos da iter 7.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5).
  - Avisos: RUSTSEC-2024-0370 (`proc-macro-error`, sem manutenção), RUSTSEC-2024-0320 (`yaml-rust`, sem manutenção), RUSTSEC-2024-0429 (`glib` 0.18.5, unsound).
  - Nenhuma dependência nova entrou na iter 8: `Cargo.lock`, `package.json` e `package-lock.json` estão sem diff desde `5919fb0`.

## Warnings da iter 7 conferidos

- **W-2 (check do pseudo-locale aceitava literal colado a uma tradução): FECHADO.**
  - **O que mudou:** `apps/ddc-tray/tests/e2e/pseudo-locale.spec.mjs:180-210`. Fora de `translate="no"`, o spec remove os segmentos `⟦…⟧` balanceados, do mais interno para fora, em laço. Se sobra `\p{L}`, reprova.
  - **Regras anteriores mantidas:** texto sem marca e fora de dado reprova; tradução sob `translate="no"` reprova.
  - **Mutante do W-2 refeito de forma independente** (cópia descartável): `const PROBE_SUFFIX = ' (nothing else to try)'` e `t('more.probeEmpty') + PROBE_SUFFIX` em `showProbe`.
    - `i18n-html.test.mjs`: 15/15 pass. O scanner estático continua sem seguir a `const`, como esperado.
    - `pseudo-locale.spec.mjs`: **2 failed**, DELL sondado × 2 temas, com `text "⟦Nenhum ajuste oculto respondeu.⟧ (nothing else to try)" in <p#probe-note.probe-note> has text outside the marks: " (nothing else to try)"`.
  - **Produto atual:** 28/28 no spec.
- **W-3 (espaço antes do caminho do guia só existia como margem): FECHADO.**
  - **Correção:** `apps/ddc-tray/src/app.js:390` põe um nó `' '` real entre a frase e o `<code translate="no">`. O `margin-inline-start` saiu de `styles.css`.
  - **Teste:** `tests/e2e/scenarios.spec.mjs:57-78` copia a dica com a Selection API e exige `` `${t('hint.i2c')} docs/linux-ddc-setup.md` ``. Passou nos 4 estados `empty`/`error` × tema.
  - **Pseudo-locale:** o nó só tem espaço e o spec o descarta no `trim`.

## Observações (não bloqueiam, não são warnings)

- **Harness congelado (D-2026-09-27-tray-app-7):**
  - o hash atual é `0566f41628d9be1004bf1fceac666ee721135502929c005185fae3d3dd04518a` (19 arquivos) e bate com o CONTEXT;
  - a última mudança no harness é `1693cec`, anterior ao refixo do orquestrador em `8382d9c`;
  - li o diff dos dois specs alterados: só apertam regras ou acrescentam asserções. O coletor de console/pageerror, o axe e o `playwright.config.mjs` não mudaram.
- **Limite de desenho do pseudo-locale, registrado para o critic.** O `translator()` interpola os parâmetros DENTRO das marcas (`apps/ddc-tray/src/i18n/index.js:59`, `` `${before}${interpolate(text, params)}${after}` ``). Por isso, um literal passado como parâmetro de `t()` sai dentro de `⟦…⟧` e passa pelo spec estrito e pelo scanner estático.
  - **Reprodução** (cópia descartável): `t('header.meta', { manufacturer: current.manufacturer + META_SUFFIX })` com `const META_SUFFIX = ' monitor'`. A página mostra `⟦RTK monitor · DDC/CI⟧`. Resultado: `pseudo-locale.spec.mjs` 28/28 pass e `i18n-html` 15/15 pass.
  - **Pego pela defesa em profundidade:** 4 failed em `scenarios.spec.mjs:29` (`toHaveText(t('header.meta', { manufacturer: 'RTK' }))`). As outras rotas por parâmetro (`more.probeSilent`, `announce.readBack`, `header.silent`, `format.*`) também têm asserção exata em algum spec ou no `view-model.test.mjs`.
  - **Classificação:** não achei mutante realista que passe na suíte inteira, então não é warning. Mas a promessa da D-7 ("qualquer literal… reprova") depende dessas asserções exatas para essa rota.
- **Limite de desenho que segue (já registrado na iter 7 e no SUMMARY):** um literal colado a um *dado* sob `translate="no"` não é checado.
- **Para o `/jdi-ship` (fora dos gates):**
  - O PR #7 (`full-osd-control`) foi mergeado por squash, e `origin/main` já avançou para além de `2628f8c` (`git fetch --dry-run`: `2628f8c..1bca909`).
  - Esta branch ainda carrega os 25 commits originais da `full-osd-control` e o `6f4341b chore(jdi)`, que não estão no `origin/main`.
  - Como o usuário mergeia por squash, é preciso rebasear `phase/tray-app` sobre o `origin/main` atualizado antes de abrir o PR. Senão o PR carrega a phase anterior de novo.
- **`nom` 3.2.1:** em todo build aparece um aviso de future-incompat. Vem de `edid`/`mccs-caps` via `ddc-hi` 0.4.1, desde a phase `ddc-backends`, e não é desta phase.

## Gate 5 (detalhe)
| Check | Resultado |
|---|---|
| 5.1 deps do core | só `thiserror` (sem saída) |
| 5.2 I/O/cfg de plataforma no core | sem saída |
| 5.3 portas | nenhum `impl MonitorBackend` no core; nenhum `pub trait` fora do core |
| 5.4 composition roots | sem saída nas duas buscas |
| 5.5 `unsafe` | só `apps/ddc-tray/src-tauri/src/lib.rs:146`, que é doc comment explicando por que o `exec` evita `set_var`. `unsafe_code = "deny"` no workspace e `#![forbid(unsafe_code)]` em `ddc-core`/`ddc-cli`/`ddc-tray` |
| 5.6 panics | todos os hits estão em módulos `#[cfg(test)]`: `kwin_placement.rs` (mod em 195), `lib.rs` (`switch_tests`, 239) e `stop_signals.rs` (82) |
| 5.7 escrita no monitor | `Dangerous` classificado no core. Nenhum `Confirm::Yes` fora de `ddc-cli`/`src-tauri/src/commands*`/testes. `DdcHiMonitorBackend` em testes só em `crates/ddc-adapters/tests/real_monitor.rs`, onde os 7 testes são `#[ignore]` e gated por `DDC_HW_TESTS=1`. Os hits de `apps/.../lib.rs:28,77` são o composition root |
| 5.8 paths de device no core | sem saída |
| 5.9 comandos Tauri | todos `async fn` |
| 5.10 supply chain | W-1 |
| 5.11 segredos/TODO | sem saída |

## DoD Checklist (gate 8)

Todos os `Verify:` foram extraídos por script do `.md` e rodados literalmente com `bash`, a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 3 | só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (`lib.rs:77`, 1×) |
| 4 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 5 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` |
| 6 | `node --test` (debounce, view-model, bridge demo…) | CONTEXT | Auto | PASS | `OK`: 135 pass, 0 fail/cancelled/skipped/todo |
| 7 | paridade i18n + nenhum texto hardcoded | CONTEXT | Auto | PASS | `OK` |
| 8 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 9 | capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | `OK` |
| 10 | single-instance 1º no builder | CONTEXT | Auto | PASS | `OK` |
| 11 | smoke Linux/KDE `--activate` | CONTEXT | Auto | PASS | `OK`: PID 1452578, `org.kde.StatusNotifierItem-1452578-1`, `popup shown` e ainda mostrado 1,5 s depois; backend real só com leituras |
| 12 | teste `#[ignore]` de hardware gated por `DDC_HW_TESTS=1` | CONTEXT | Auto | PASS | `OK`: 2 testes listados com `--ignored --list`, nenhum executado |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious… | CONTEXT | Auto | PASS | `OK`: 88 passed, 6 skipped (= screenshots); `n = 10`, `dd = 16/16`, `dr = 2`, `ps = 26` |
| 14 | bridge nunca cai no demo fora de dev local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 15 | harness = o revisado (hash) | CONTEXT | Auto | PASS | `OK`: `0566f416…518a` |
| 16 | nenhum `<select>` nativo | CONTEXT | Auto | PASS | `OK` |
| 17 | SNI `ksni` + roda testada em Rust + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`: PID 1453083; vertical `75 -> 80`, horizontal sem escrita em 1,5 s, `80 -> 75` |
| 18 | nenhum TODO/FIXME sem issue em arquivo versionado | CONTEXT | Auto | PASS | `OK` |
| 19 | screenshots claro/escuro regenerados e byte a byte iguais | CONTEXT | Auto | PASS | `OK`: `git diff --quiet -- docs/screenshots` limpo depois da regeneração |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`: 383 passed, 0 failed, 9 ignored |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL Lines 82.78% (gate com exclusões: 83.22%) |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | `OK` |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` em `CHANGELOG.md:8`, com a linha do tray atualizada em `8907c8e` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` em `README.md:265`; a l. 366 descreve a regra estrita (`8907c8e`) |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

Os 2 itens `Manual` do CONTEXT são cópias da baseline do PROJECT e foram contados uma vez só.

**Protocolo dos smokes (C11, C17):**
- um de cada vez;
- antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray` (nunca `-f`);
- logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu;
- PIDs do usuário: 1411926 → 1452797 → 1453315. O 1453315 é a única instância viva no fim;
- a porta 1420 estava livre antes e depois do Gate 7/C13/C19.

**Monitor real:** nenhuma escrita. Nenhum teste `#[ignore]` foi executado. O C11 usou o backend real só com leituras, e o C17 o monitor simulado.

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
A iter 8 fecha o W-2 e o W-3 sem enfraquecer o harness, e o hash refixado bate. Todos os gates auto passam, e o único warning é o W-1, herdado. Os próximos passos:
1. confirmar os 2 itens manuais (`/jdi-confirm-dod tray-app`, que no fluxo autônomo fica para o PR);
2. antes do PR, rebasear `phase/tray-app` sobre o `origin/main` atualizado, porque o PR #7 foi mergeado por squash;
3. anexar ao corpo do PR a saída do `rtk_qhd_hdr` com `DDC_HW_TESTS=1`, rodado pelo orquestrador.

Se o DoD critic rodar, o ponto a olhar é o limite registrado acima, o de literal passado como parâmetro de `t()`. Hoje ele é coberto só pelas asserções exatas dos specs de cenário.

## DoD Critic (enhanced)

- DoD row «7 (i18n)»: literal colado ao toast de falha de escrita (`showToast(errorText(error, t) + NOT_APPLIED)`) chega à tela — nenhum estado do pseudo-locale mostra o toast (o demo nunca falha uma escrita); idem textos de status vazio/erro de "Todos os ajustes" e o rótulo transitório de sondagem.
- DoD row «18/22 (TODO)»: `todo!(…)` num módulo só de Windows e `// todo:` minúsculo passam (grep sensível a caixa, sem `todo!`).
- Suspeitas (objective:false): «11/17» o `smoke-sni.sh` (prova de painel aberto e roda só vertical) não está no manifesto do harness congelado.

**Verdict:** BLOCKED
