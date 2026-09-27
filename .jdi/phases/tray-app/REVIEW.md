# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iter 11 (rodada 3). Revisão completa feita do zero em `0878450` (commits `efac15c..13f9c41` + `0878450`), no Linux (Fedora 44, KDE Plasma Wayland), em 2026-09-27. Cada `Verify:` do CONTEXT.md e do PROJECT.md foi extraído do `.md` por script e rodado literalmente com `bash` a partir da raiz (`grep` = `/usr/bin/grep`, sem alias; `DDC_HW_TESTS` e `DDC_TRAY_FAKE` removidos do ambiente). Nenhum teste `#[ignore]` rodou e nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` exit 0 (tray fora, D-2026-09-26-tray-app-9). |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware). Igual à iter 10 (383), nenhum `.rs` mudou. `node --test` com 151 pass e 0 fail/cancelled/skipped/todo. |
| Coverage | PASS | 83.22% lines, threshold 80% (linha TOTAL, `main.rs`/`build.rs` excluídos, `--fail-under-lines 80` exit 0). O comando literal do PROJECT, sem exclusões, dá 82.78%. |
| Lint | PASS | `cargo fmt --all --check` exit 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1 a 5.9 e 5.11 sem achados BLOCK. 5.10 = W-1 (conhecido). |
| Consistency | PASS | 9 commits com scope `tray-app` e tipos coerentes. Código e `.jdi/` nunca no mesmo commit, e a D-XX é citada no corpo. Nenhuma violação de D-1, D-2 ou das D-tray-app. |
| UI Validation | PASS (1 warn) | Playwright: 134 passed, 6 skipped (= screenshots), 0 failed/flaky. Nenhum achado axe moderate/minor no log. W-2 vem do julgamento da trava estática. |
| DoD | PASS_PENDING_MANUAL | 22/22 auto e 2 manual pendentes. |

### Detalhes por check (Gate 5)
- **5.1** (deps do core): nenhuma além de `thiserror`.
- **5.2** (I/O ou `cfg` de plataforma no core) e **5.3a** (`impl MonitorBackend` no core): nenhum hit.
- **5.3b** (`pub trait` fora do core): nenhum hit.
- **5.4** (adapter construído fora do composition root, ou adapter que importa adapter): nenhum hit.
- **5.5**:
  - `unsafe` fora de `ddc-adapters`: o único hit é `apps/ddc-tray/src-tauri/src/lib.rs:146`, dentro de um doc comment que explica por que o código não usa `set_var` (D-2026-09-26-tray-app-10). Não é código.
  - Nenhum `unsafe {` em `ddc-adapters`.
  - O `forbid(unsafe_code)` está presente nos três crate roots.
- **5.6** (panics fora de teste): todo `unwrap` encontrado em `kwin_placement.rs`, `lib.rs` e `stop_signals.rs` fica dentro de `#[cfg(test)] mod …`. O `panic!` de `worker.rs:141` é doc comment.
- **5.7**:
  - `Risk::Dangerous` presente no domínio (`feature.rs:33`, `:156`, `:163`).
  - Nenhum `Confirm::Yes` fora de `ddc-cli/src` e `src-tauri/src/commands*`.
  - `DdcHiMonitorBackend` em `crates/ddc-adapters/tests/real_monitor.rs`: 7/7 testes `#[ignore]` e gated por `DDC_HW_TESTS`. Em `apps/`, ele só aparece no composition root (`lib.rs:28`, `:77`).
- **5.8** (caminhos de device ou nomes de plataforma no core): nenhum hit.
- **5.9** (comando Tauri não-async): nenhum.
- **5.11** (segredos, TODO sem issue): nenhum hit.

### Mudanças de produto desta iteração (avaliadas)
- **`bdba5a2`: detalhe vazio no estado indisponível.**
  - O `unavailableBridge` rejeita com `{ kind: 'backend_unavailable', message: '' }`.
  - No `view-model.js:211`, `error?.message || null` vira `null`, e o `setText` (`app.js:905`) deixa `#message-detail` oculto e vazio. O popup mostra só o título traduzido e o "Tentar de novo".
  - Esse caminho só existe com o app quebrado (`withGlobalTauri` desligado), e o C14 trava `withGlobalTauri == true`.
  - O caminho Rust de backend indisponível (`AppState.osd = Err`) não mudou: a mensagem do core continua chegando como dado.
  - Correto, e `unavailable.spec` e `bridge-demo` passaram a exigir exatamente isso.
- **`edd7a7d`: `RangeError` do ícone.**
  - `createIcon` lança `new RangeError(name)`, e a mensagem é só o identificador.
  - Hoje nenhum nome de ícone vem de dado (o teste `every icon the page and the app ask for exists` trava isso). O teste novo `an unknown icon throws a RangeError whose message is its name alone` fixa o formato.
  - Correto.
- **`efac15c`: mensagens do demo em `demo-data.js`.**
  - Os textos não mudaram. Conferi um a um contra os `#[error(...)]` de `crates/ddc-core/src/domain/error.rs` (not found, not supported, exceeds its maximum, not an allowed value, dangerous and was not confirmed, did not respond in time): todos iguais.
  - Só `bridge.js` importa `demo-data.js`.
  - Está coerente com a D-2026-09-27-tray-app-9, que isenta os dados de demo da trava. Ver W-2 sobre o limite dessa isenção.

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido e adiado para `ci-crossbuild`):** `cargo audit` acusa 1 vulnerabilidade e 3 warnings permitidos.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5, transitivo do `ddc-hi`).
  - Warnings: RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained), RUSTSEC-2024-0320 (`yaml-rust`, unmaintained) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound, transitivo do Tauri/GTK).
  - `Cargo.lock` intocado nesta iteração.
  - Informativo: `nom` v3.2.1 emite aviso de future-incompat no build (transitivo, anterior a esta phase).
- **W-2 (Gate 7, julgamento da trava de frase da D-2026-09-27-tray-app-9): a isenção da trava estática é mais larga que os arquivos de texto, e nada restringe quem importa `demo-data.js`.**
  - **Onde:** `apps/ddc-tray/tests/ui/i18n-html.test.mjs:604`, `const isLanguageFile = (path) => path.startsWith('i18n/') || path === 'demo-data.js';`. Não existe verificação de que só `bridge.js` importa `demo-data.js`.
  - **Prova (cópia descartável no scratchpad, já apagada; repo intocado):**
    - Foram aplicadas três mudanças:
      - `demo-data.js` ganhou `export const GENERIC_NOTE = "Some monitors only undo this from their own buttons.";`;
      - `app.js` passou a importar `GENERIC_NOTE` de `./demo-data.js`;
      - `app.js:703` passou a usar `: GENERIC_NOTE;`.
    - Resultado: `i18n-html` **19/19 pass**.
    - Controle, com o mesmo literal direto em `app.js:703`: **1 fail**.
    - A frase do critic da iter 10 volta ao popup pt-BR sem que a camada estática perceba.
  - **`i18n/index.js` é código, não texto.**
    - Rodei a trava sobre ele (isenção reduzida a `i18n/en.js`, `i18n/pt-BR.js` e `demo-data.js`), e ela continua 19/19. Ele não tem frase, então a isenção é desnecessária.
    - Uma frase ali, por exemplo no fallback do `t()`, sairia com `⟦…⟧` no pseudo-locale e passaria pelas duas camadas.
  - **Impacto hoje:** nenhum no produto.
    - `demo-data.js` só é importado por `bridge.js`, e `i18n/index.js` não tem frase.
    - O caso concreto acima ainda reprovaria no pseudo-locale, porque o estado do diálogo genérico existe desde esta iteração (mutante M1 do SUMMARY).
    - A brecha só importa para um caminho de texto que nenhum estado visita, que é justamente o que a D-9 diz que a trava estática cobre.
  - **Sugestão (barata):**
    - reduzir a isenção a `i18n/en.js`, `i18n/pt-BR.js` e `demo-data.js`;
    - afirmar que `bridge.js` é o único módulo de `src/` que importa `demo-data.js`.
    - Mexer no teste muda o hash do harness (C15), e o orquestrador precisaria refixá-lo.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`, 383 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL lines 82.78%, exit 0 (gate com exclusões: 83.22%) |
| 3 | No `TODO`/`FIXME` without linked issue reference (`*.rs`, D-2026-09-27-tray-app-9) | PROJECT | Auto | PASS | `OK`. Sanidade em amostras plantadas no scratchpad: reprova `// TODO:`, `//! todo:`, `// TODOs:`, `/// fixme`, `@todo`, ` * todo`, `todo!()`, `unimplemented!(…)`; aceita `TODO(#12)`, `TODO #12` e "Todos os ajustes" |
| 4 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (`lib.rs:77`, 1×) |
| 7 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` |
| 9 | `node --test` (debounce, view-model, bridge demo…) com zero falhas | CONTEXT | Auto | PASS | `OK`: 151 pass, 0 fail/cancelled/skipped/todo |
| 10 | Paridade i18n, nenhum texto hardcoded, trava de frase | CONTEXT | Auto | PASS | `OK`. Os títulos `app.js passes no literal text…` e `no natural-language literal outside the locale files` existem e passam. Limite da isenção em W-2 |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 13 | `tauri-plugin-single-instance` 1º no builder | CONTEXT | Auto | PASS | `OK` |
| 14 | Smoke Linux/KDE SNI `--activate` | CONTEXT | Auto | PASS | `OK`: PID 1919667, `org.kde.StatusNotifierItem-1919667-1` com dono = o próprio PID, `popup shown` após Activate e ainda mostrado 1,5 s depois, sem `panicked`. Backend real, só leituras |
| 15 | Teste `#[ignore]` de hardware RTK existe, compila e é gated | CONTEXT | Auto | PASS | `OK` (listado com `--ignored --list`, **não executado**) |
| 16 | Gate 7: 0 erro de console e 0 axe critical/serious nos `critical_paths` | CONTEXT | Auto | PASS | `OK`: 134 passed, 6 skipped (= screenshots), 0 failed/flaky; `n=10`, `dr=2`, `ps=54`, `dd=16`. Pseudo-locale com 58 ✓ e `confirm.spec` com 14 ✓ |
| 17 | Bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 18 | Harness idêntico ao revisado (hash) | CONTEXT | Auto | PASS | `OK`: `9816fd4c7b5e90a364c86de1a2154cde7c9ea750c2e02e901da43d935f9549cc` (21 arquivos) |
| 19 | Nenhum `<select>` nativo no popup | CONTEXT | Auto | PASS | `OK` |
| 20 | Ícone Linux = SNI `ksni`, roda → brilho testado | CONTEXT | Auto | PASS | `OK`: testes `scroll` verdes; smoke `--fake --scroll` com PID 1920172, `brightness 75 -> 80` na vertical, nada na horizontal em 1,5 s, `80 -> 75` |
| 21 | Nenhum `TODO`/`FIXME` sem issue em arquivo versionado do produto (`git grep`) | CONTEXT | Auto | PASS | `OK`. Sanidade num repo git descartável: reprova `# TODO`, `<!-- todo: -->`, `// todos`; aceita `TODO(#3)` e "Todos os ajustes" |
| 22 | Screenshots claro/escuro gerados e versionados, byte a byte iguais | CONTEXT | Auto | PASS | `OK`: regeneração rodou sem nada pulado, SHA-1 dos PNGs igual antes e depois, `git status` limpo |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (CHANGELOG.md:8), com o tray app em Added, atualizado em `d3edb87` (trava de frase, 27 estados). Nenhum heading de versão, porque o release não foi cortado |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Tray app` (README.md:265) e `### Known limitations of the tray app` (:371). `d3edb87` descreve o erro de indisponível só traduzido, a trava de frase e os 27 estados, e bate com o código revisado |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Protocolo de execução
- **Smokes (itens 14 e 20, que são C11 e C17 do CONTEXT):** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário (1865254, depois 1919916), encerrada com `pkill -x ddc-tray` (nunca `-f`).
  - Logo depois de cada smoke, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - No fim só está viva a instância do usuário, PID 1920424.
- **Porta 1420:** livre antes e depois dos itens 16/22.
- **Probes:** as sondas de W-2 e de TODO rodaram em cópias descartáveis no scratchpad, já apagadas. O repo ficou só com `REVIEW.md` recriado e `.idea/` (ignorado).

## Recommendation
Pode seguir para `/jdi-confirm-dod tray-app`, e depois `/jdi-ship tray-app`: nenhum blocker e todos os 22 Auto verdes.

W-1 continua com a `ci-crossbuild`. W-2 é endurecimento da trava estática, sem defeito no produto hoje: a correção é reduzir a isenção a `i18n/en.js`, `i18n/pt-BR.js` e `demo-data.js` e afirmar que só `bridge.js` importa `demo-data.js`. Pode entrar numa rodada curta de warnings ou virar issue, a critério do orquestrador; se entrar, é preciso refixar o hash do harness (item 18).
