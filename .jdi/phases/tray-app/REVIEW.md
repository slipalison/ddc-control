# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iter 12 (rodada 3). Revisão completa feita do zero em `19752b8`, no Linux (Fedora 44, KDE Plasma Wayland), em 2026-09-27. Os commits da iteração são `c14bb5e` (teste), `99f35c5` (SUMMARY) e `19752b8` (hash do harness refixado). Cada `Verify:` do CONTEXT.md e do PROJECT.md foi extraído do `.md` por script e rodado literalmente com `bash` a partir da raiz. Nesse `bash`, `grep` é `/usr/bin/grep`, sem alias, e `DDC_HW_TESTS` e `DDC_TRAY_FAKE` foram removidos do ambiente. Nenhum teste `#[ignore]` rodou e nada foi escrito no monitor real.

Desde a revisão da iter 11 (`0878450`), o único arquivo fora de `.jdi/` que mudou é `apps/ddc-tray/tests/ui/i18n-html.test.mjs` (+86/−10). Nenhum `.rs`, nada em `src/` do popup e nem `Cargo.lock`, `package.json` ou `package-lock.json`.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` exit 0 (tray fora, D-2026-09-26-tray-app-9) |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware). Igual à iter 11. `node --test` com 153 pass (eram 151, +2 do `i18n-html`) e 0 fail/cancelled/skipped/todo |
| Coverage | PASS | 83.22% lines, threshold 80% (linha TOTAL, `main.rs`/`build.rs` excluídos, `--fail-under-lines 80` exit 0). O comando literal do PROJECT, sem exclusões, dá 82.78% |
| Lint | PASS | `cargo fmt --all --check` exit 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes |
| Hexagonal/Safety/Hygiene | WARN | 5.1 a 5.9 e 5.11 sem achados. 5.10 = W-1 (conhecido) |
| Consistency | PASS | 3 commits com scope `tray-app` e tipos coerentes (`test`, `docs`, `docs`). Código e `.jdi/` nunca no mesmo commit. Nenhuma violação de D-1, D-2 ou das D-tray-app |
| UI Validation | PASS | Playwright: 134 passed, 6 skipped (= screenshots), 0 failed/flaky; `n=10`, `dr=2`, `ps=54`, `dd=16`. Pseudo-locale com 58 ✓ e `confirm.spec` com 14 ✓. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto e 2 manual pendentes |

### Detalhes por check (Gate 5)
- **5.1** (deps do core): nenhuma além de `thiserror`.
- **5.2** (I/O ou `cfg` de plataforma no core), **5.3a** (`impl MonitorBackend` no core) e **5.3b** (`pub trait` fora do core): nenhum hit.
- **5.4** (adapter construído fora do composition root, ou adapter que importa adapter): nenhum hit.
- **5.5**:
  - `unsafe` fora de `ddc-adapters`: o único hit é `apps/ddc-tray/src-tauri/src/lib.rs:146`, dentro do doc comment que explica por que o código não usa `set_var`. Não é código.
  - Nenhum `unsafe {` em `ddc-adapters`.
  - O `forbid(unsafe_code)` está presente nos três crate roots.
- **5.6** (panics fora de teste):
  - os `unwrap` de `kwin_placement.rs:246-267`, `lib.rs:269` e `stop_signals.rs:105-108` ficam dentro de `#[cfg(test)] mod …` (`kwin_placement.rs:195`, `lib.rs:239`, `stop_signals.rs:82`);
  - o `panic!` de `worker.rs:141` é doc comment.
- **5.7**:
  - `Risk::Dangerous` está presente no domínio (`crates/ddc-core/src/domain/feature.rs:33`, `:156`, `:163`).
  - Nenhum `Confirm::Yes` fora de `ddc-cli/src` e `src-tauri/src/commands*`.
  - `DdcHiMonitorBackend` em `crates/ddc-adapters/tests/real_monitor.rs`: 7/7 testes `#[ignore]` e gated por `DDC_HW_TESTS`. Em `apps/`, ele só aparece no composition root (`lib.rs:28`, `:77`).
- **5.8** (caminhos de device ou nomes de plataforma no core) e **5.9** (comando Tauri não-async): nenhum hit.
- **5.11** (segredos, TODO sem issue): nenhum hit.

### Mudança desta iteração (avaliada): W-2 da iter 11 fechado
- **O que mudou em `c14bb5e`:**
  - a isenção da trava de frase passou a ser um `Set` exato com `i18n/en.js`, `i18n/pt-BR.js` e `demo-data.js` (`i18n-html.test.mjs:604-607`);
  - `i18n/index.js` entrou na leitura obrigatória;
  - a trava nova `only bridge.js imports demo-data.js` e o autoteste dela.
- **Nada foi enfraquecido:**
  - o `deepEqual` de `files left out` caiu de 4 para 3 arquivos, e a leitura obrigatória ganhou um;
  - nenhum teste ou asserção foi removido. `i18n-html` passou de 19 para 21 testes, todos ✓.
- **Hash do harness:**
  - recalculado, dá `fd6985f9…8560` sobre 21 arquivos, igual ao refixado em `19752b8`;
  - a última mudança do harness é `c14bb5e`.
- **Conferência independente (cópia descartável de `src/` e `tests/ui/` no scratchpad, já apagada; repo intocado):**

| Mutação | Resultado |
|---|---|
| **M1, a do W-2:** `demo-data.js` exporta `GENERIC_NOTE` com a frase, `app.js` importa de `./demo-data.js` e `app.js:703` a usa | **reprova**: `only bridge.js imports demo-data.js` com `app.js:15 "./demo-data.js"` (na iter 11: 19/19 passavam) |
| **M2:** `export const MISSING = "Missing translation"` em `i18n/index.js` | **reprova**: `i18n/index.js:131 "Missing translation"` |
| **M3:** arquivo novo `src/i18n/notes.js` com uma frase | **reprova**: `i18n/notes.js:1 "Undo it yourself"`. O arquivo agora é lido, não isento |
| **R1 (limite residual declarado no SUMMARY):** `bridge.js` reexporta `GENERIC_NOTE` de `./demo-data.js` e `app.js` a importa de `./bridge.js` | passa na camada estática (21/21). Ver Observações |

- **Conformidade com a D-2026-09-27-tray-app-9:** a D-9 isenta "`i18n/` e os dados de demo". Reduzir a isenção a dois arquivos de `i18n/` deixa a trava mais estrita, então a mudança conforma.

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido e adiado para `ci-crossbuild`):** `cargo audit` 0.22.2 acusa 1 vulnerabilidade e 3 warnings permitidos.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5, transitivo do `ddc-hi`).
  - Warnings: RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained), RUSTSEC-2024-0320 (`yaml-rust`, unmaintained) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound, transitivo do Tauri/GTK).
  - `Cargo.lock` intocado nesta iteração.
  - Informativo: `nom` v3.2.1 emite aviso de future-incompat no build (transitivo, anterior a esta phase).

## Observações (sem efeito no veredito)
- **O repasse pelo `bridge.js` continua invisível à camada estática.**
  - Confirmado pela mutação R1 acima: se o único leitor permitido reexportar uma frase de `demo-data.js`, ela chega a `app.js` sem reprovar nenhuma das duas travas estáticas. Um especificador montado só com variáveis (`'./demo-' + suffix`) também passa.
  - O próprio SUMMARY (iter 12, "Limite residual conhecido") declara as duas coisas.
  - Não classifiquei como warning, por três motivos:
    - exige mudar de propósito o único leitor permitido;
    - hoje o `bridge.js` exporta só `DEMO_LATENCY_MS`, `createBridge`, `isLocalDevServer` e `normalizeError`, e nenhum deles repassa texto do demo além das mensagens de backend previstas pela D-9;
    - o pseudo-locale continua julgando os 27 estados que visita. R1 no diálogo genérico reprovaria lá (M1 da iter 11).
  - Endurecimento barato, se o orquestrador quiser: travar a lista de exports do `bridge.js` nesses 4 nomes. Isso muda o hash do harness.
- **O commit `19752b8` não cita a D-XX no corpo.** É o commit do orquestrador que refixa o hash (D-2026-09-27-tray-app-7). O `0878450`, da iter 11, também não citava, e o `abfafab` e o `19f86b3` citavam. É só convenção de commit em `.jdi/`, sem efeito no código.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`, 383 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL lines 82.78% (3304/569), exit 0. Gate com exclusões: 83.22% |
| 3 | No `TODO`/`FIXME` without linked issue reference (`*.rs`, D-2026-09-27-tray-app-9) | PROJECT | Auto | PASS | `OK` |
| 4 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (`lib.rs:77`, 1×) |
| 7 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` |
| 9 | `node --test` (debounce, view-model, bridge demo…) com zero falhas | CONTEXT | Auto | PASS | `OK`: 153 pass, 0 fail/cancelled/skipped/todo |
| 10 | Paridade i18n, nenhum texto hardcoded, trava de frase | CONTEXT | Auto | PASS | `OK`. `i18n-html` com 21/21 ✓, inclusive a isenção exata e `only bridge.js imports demo-data.js`. Limite residual em Observações |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 13 | `tauri-plugin-single-instance` 1º no builder | CONTEXT | Auto | PASS | `OK` |
| 14 | Smoke Linux/KDE SNI `--activate` | CONTEXT | Auto | PASS | `OK` na 1ª execução: PID 2026653, `org.kde.StatusNotifierItem-2026653-1` com dono = o próprio PID, `popup shown` após Activate e ainda mostrado 1,5 s depois, sem `panicked`. Backend real, só leituras |
| 15 | Teste `#[ignore]` de hardware RTK existe, compila e é gated | CONTEXT | Auto | PASS | `OK`: 2 testes listados com `--ignored --list` (`…_panel_loads_and_one_safe_brightness_write_is_restored`, `…_mute_monitor_fails_its_panel_and_the_rtk_loads`), **não executados** |
| 16 | Gate 7: 0 erro de console e 0 axe critical/serious nos `critical_paths` | CONTEXT | Auto | PASS | `OK`: 134 passed, 6 skipped (= screenshots), 0 failed/flaky; `n=10`, `dr=2`, `ps=54`, `dd=dl=16` |
| 17 | Bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 18 | Harness idêntico ao revisado (hash) | CONTEXT | Auto | PASS | `OK`: `fd6985f9460f639ae88ff94f229c1624cbd46706e978defb108b092e39d08560` (21 arquivos, última mudança `c14bb5e`) |
| 19 | Nenhum `<select>` nativo no popup | CONTEXT | Auto | PASS | `OK` |
| 20 | Ícone Linux = SNI `ksni`, roda → brilho testado | CONTEXT | Auto | PASS | `OK`: 20 testes `scroll` ✓; smoke `--fake --scroll` com PID 2027211, `brightness 75 -> 80` na vertical, nada na horizontal em 1,5 s, `80 -> 75` |
| 21 | Nenhum `TODO`/`FIXME` sem issue em arquivo versionado do produto (`git grep`) | CONTEXT | Auto | PASS | `OK` |
| 22 | Screenshots claro/escuro gerados e versionados, byte a byte iguais | CONTEXT | Auto | PASS | `OK`: a regeneração rodou sem nada pulado, o SHA-1 dos 6 PNGs ficou igual antes e depois e o `git status` ficou limpo |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (CHANGELOG.md:8), com o tray app em Added. A última atualização foi em `d3edb87` (trava de frase, 27 estados). Não há heading de versão porque o release não foi cortado |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Tray app` (README.md:265) e `### Known limitations of the tray app` (:371). "no phrase … outside the locale files and the demo's data" (README.md:362) continua fiel à isenção exata desta iteração |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Protocolo de execução
- **Smokes (itens 14 e 20, que são C11 e C17 do CONTEXT):** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava só a instância do usuário (1999215, depois 2026886). Ela foi encerrada com `pkill -x ddc-tray` (nunca `-f`), esperando o processo sair.
  - Logo depois de cada smoke, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - O C11 passou na 1ª execução, sem `tray activated` externo.
  - No fim só está viva a instância do usuário, PID 2027457.
- **Porta 1420:** livre antes e depois dos itens 16/22.
- **Probes:** as mutações da trava rodaram numa cópia descartável no scratchpad, já apagada. O repo ficou só com `REVIEW.md` recriado e `.idea/` (ignorado).

## Recommendation
Pode seguir para `/jdi-confirm-dod tray-app`, e depois `/jdi-ship tray-app`: nenhum blocker e todos os 22 Auto verdes.

O W-2 da iter 11 está fechado: a mutação que o demonstrava agora reprova. W-1 continua com a `ci-crossbuild`. O repasse residual pelo `bridge.js` (Observações) já está declarado no SUMMARY e não pede outra rodada. Se o orquestrador quiser endurecer, a correção é travar os exports do `bridge.js`, e isso exige refixar o hash do harness.

## DoD Critic (enhanced)

- DoD row «11 (CSP)»: sem `tauri/custom-protocol`, o `cargo build --release` gera um build "dev" do Tauri (`is_dev() == true`), que serve `app.security.devCsp` no lugar de `csp`; um `devCsp` com `'unsafe-inline' 'unsafe-eval'` passa em todos os Verify e fica ativo em runtime (eval executou no binário com `DDC_TRAY_FAKE=1`).

**Verdict:** BLOCKED
