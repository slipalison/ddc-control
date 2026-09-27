# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 15 (rodada 3, a última do loop). Re-verificação completa (gates 1-8) sobre o HEAD `c04133d`, que inclui os commits `3b1c45b` (teste), `e04ff2e` (SUMMARY) e `c04133d` (CONTEXT: hash do `build_tests` refixado e filtros `--exact`). Tudo rodou em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Nenhum teste `#[ignore]` de hardware rodou, e nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` exit 0. O tray fica fora por D-2026-09-26-tray-app-9 |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 linhas `test result`. A iter 14 tinha 385: o +1 é o `the_effective_csp_is_the_strict_policy_on_every_target` |
| Coverage | PASS | 83.36% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0 |
| Lint | PASS | `cargo fmt --all --check` exit 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos (hits lidos, todos em comentário ou `#[cfg(test)]`). 5.10: W-1 (`cargo audit`, conhecido) |
| Consistency | PASS | commits da iter 15 com escopo `tray-app` e tipos coerentes. `Cargo.toml`/`Cargo.lock` intocados. D-1, D-2 e D-2026-09-26-tray-app-2/-7, D-2026-09-27-tray-app-10/-11 (com a emenda da iter 14) conformes |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky. Erros de console: 0; axe critical/serious: 0 (o coletor e o axe de `support.mjs` afirmam isso em cada teste). Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para `ci-crossbuild`).** `cargo audit` termina com `error: 1 vulnerability found!`:
  - RUSTSEC-2018-0005: `serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, presente desde a phase `ddc-backends`.
  - Há também 3 avisos permitidos:
    - RUSTSEC-2024-0370: `proc-macro-error`, unmaintained;
    - RUSTSEC-2024-0320: `yaml-rust`, unmaintained;
    - RUSTSEC-2024-0429: `glib` 0.18.5, unsound, via tauri/gtk.
  - O `Cargo.lock` não mudou nesta iteração.

## Avaliação dirigida da iteração 15

### O teste `the_effective_csp_is_the_strict_policy_on_every_target` (`apps/ddc-tray/src-tauri/src/lib.rs:350-362`)
- **Mesmo caminho do Tauri.** O teste chama `read_from(target, CARGO_MANIFEST_DIR)` para `Linux`, `Windows` e `MacOS` e desserializa o resultado em `tauri::Config`. Conferi no fonte do registry que o codegen faz a mesma coisa:
  - `tauri-codegen` 2.7.0, `lib.rs:83`: `serde_json::from_value(tauri_utils::config::parse::read_from(target, &parent)?.0)`;
  - `tauri-utils` 2.10.0, a versão do `Cargo.lock`, `config/parse.rs:180-204`: `read_from` mescla por `json_patch::merge` o `tauri.<alvo>.conf.json` sobre a base;
  - o mapa alvo → arquivo está em `parse.rs:51-70`, com `tauri.windows.conf.json`, `tauri.macos.conf.json` e `tauri.linux.conf.json`.
- **Os mesmos arquivos que o build lê.** Nenhuma cópia do `tauri-utils` na árvore tem `config-json5` ou `config-toml` ligada (conferido com `cargo tree -e features,normal,build -i tauri-utils`). Então o teste e o `generate_context!` leem o mesmo conjunto de `.json`. Os globs do C8 também reprovam qualquer `tauri*.json5` ou `Tauri*.toml`.
- **Imune ao `target/` incremental.** O arquivo de plataforma é lido em runtime a partir de `CARGO_MANIFEST_DIR`, sem depender de recompilação. Isso complementa o `rerun-if-changed=.` do `build.rs` (iter 14), que continua valendo para o teste do `context()`.
- **Asserções.** O teste exige a string exata `STRICT_CSP` e `dev_csp == None` por alvo, e a mensagem nomeia o alvo. `.unwrap()` só aparece dentro do `#[cfg(test)] mod build_tests` (`lib.rs:312-363`).
- **Travas do C8.**
  - O módulo `build_tests` (51 linhas) tem SHA-256 `ddf6636a…8061`, que bate com o CONTEXT.
  - Rodado cada um com `-- --exact`, `build_tests::the_effective_csp_is_the_strict_policy` e `…_on_every_target` dão `ok. 1 passed; … 138 filtered out`. O `is_not_a_dev_build` também dá `1 passed`.
  - `tauri feature "custom-protocol"` aparece 1× na árvore de `x86_64-unknown-linux-gnu` e 1× na de `x86_64-pc-windows-msvc`.
- **A CSP não muda por código.** Não há `on_web_resource_request`, `register_*uri_scheme_protocol`, `Content-Security-Policy` nem `dangerousDisableAssetCspModification` em `src-tauri/src` ou no `tauri.conf.json`. O único hit é o doc comment de `lib.rs:319`.
- **Sem dependência nova.** O `serde_json` já era dependência normal do crate (`Cargo.toml`). O `tauri::utils` é reexportado pelo próprio `tauri`.
- **Provas negativas.** Não repeti as mutações (a review é read-only). As 4 da SUMMARY (M1-M4) são coerentes com o código lido: uma `csp` nula, frouxa ou em forma de mapa num arquivo de plataforma de qualquer alvo muda a string comparada, e um `devCsp` quebra a 2ª asserção.
- **Limites, informativos e sem WARN:**
  - um `TAURI_CONFIG` definido no build de outro alvo só é visto pelo teste do `context()` quando ele roda naquele alvo, o que acontece a partir da `ci-crossbuild`. Não é estado do repositório;
  - `Target::Ios` e `Target::Android` ficam de fora, mas o app não é distribuído para eles, e o laço `jq` do C8 proíbe bloco `app.security` em qualquer `tauri.*.conf.json`.

### Plano e escopo
- Desde a review da iter 14 (`e575ffb`), só `apps/ddc-tray/src-tauri/src/lib.rs` mudou em `apps/`, com +23/−0 dentro de `build_tests`. `src/` (UI), o harness congelado (C15 `OK`), README, CHANGELOG, `docs/` e os manifestos Cargo ficaram iguais.
- As 8 tasks do PLAN continuam `completed`, cada uma com teste.
- Como a UI não mudou, o julgamento estático de a11y/i18n do Gate 7 feito na iter 14 continua valendo.

### Protocolo da instância do usuário
- Antes do C11 e do C17, `pgrep -xa ddc-tray` mostrava a instância do usuário. Encerrei com `pkill -x ddc-tray` (nunca `-f`), esperei o processo sair e a reabri com `setsid -f /home/slipalison/.local/bin/ddc-tray` logo depois de cada smoke.
- PIDs: 2485798 → 2517759 → 2518323. O 2518323 é a única instância viva no fim.
- Os dois smokes passaram na 1ª execução, sem `tray activated` externo:
  - C11: smoke no PID 2517533, `popup shown` e popup ainda mostrado 1,5 s depois, backend real só com leituras;
  - C17: smoke no PID 2518063, monitor simulado, `75 -> 80` com rolagem vertical, nada com a horizontal, depois `80 -> 75`.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | exit 0, `OK` |
| 3 | Só o composition root constrói `DdcHiMonitorBackend` (1×, em `lib.rs`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 4 | `panel.rs` Rust puro sobre `MonitorControl` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 5 | `#![forbid(unsafe_code)]` ativo, nenhum `unsafe` em src/tests/build.rs | CONTEXT | Auto | PASS | exit 0, `OK` |
| 6 | `node --test` por módulo e total, 0 fail/cancelled/skipped/todo | CONTEXT | Auto | PASS | exit 0, `OK` |
| 7 | Paridade i18n, sem texto hardcoded, trava de frases, título da guarda | CONTEXT | Auto | PASS | exit 0, `OK` |
| 8 | CSP estrita: sem devCsp/devUrl, sem `app.security` em arquivo de plataforma, `custom-protocol` (linux+windows), `is_not_a_dev_build`, CSP efetiva no `context()` e em todos os alvos, hash do `build_tests` | CONTEXT | Auto | PASS | exit 0, `OK`: hash `ddf6636a…8061`; os 2 testes `--exact` dão `1 passed` cada |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | `OK` na 1ª execução, PID 2517533: `popup shown`, ainda mostrado 1,5 s depois; backend real só com leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila e é gated | CONTEXT | Auto | PASS | exit 0, `OK` (listado, não executado) |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 18 s; Gate 7 à parte: 138 passed, 6 skipped |
| 14 | Fora de servidor local o bridge nunca cai no demo + `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`, PID 2518063: `75 -> 80` com rolagem vertical, nada com a horizontal, `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK`; `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (386 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | `cargo llvm-cov --workspace --summary-only` literal: TOTAL lines 82.93%, exit 0. Gate 3: 83.36% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só existe o heading `## [Unreleased]` (CHANGELOG.md:8), sem `## [version]`. `CHANGELOG.md:49` diz "strict CSP, tested as Tauri embeds it, with any platform config merged", sem citar todos os alvos |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: README cita `ddc-tray` 20×. `README.md:362` descreve a CSP efetiva ("merged with the target's `tauri.<platform>.conf.json`"); a checagem por Linux, Windows e macOS não é mencionada |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- Nenhum blocker. A iteração 15 cumpre a emenda (iter 14) da D-2026-09-27-tray-app-11:
  - a CSP efetiva é conferida para Linux, Windows e macOS pelo mesmo `read_from` que o `generate_context!` usa;
  - a leitura é feita em runtime, então não depende do `target/` incremental;
  - o corpo do teste é travado pelo hash do módulo, e cada teste roda com `--exact` no C8.
- **W-1:** segue com a `ci-crossbuild`.
- **Itens Manual:** CHANGELOG e README ficam para o PR, via `/jdi-confirm-dod tray-app`. Na revisão, vale decidir se `README.md:362` e `CHANGELOG.md:49` devem dizer que a checagem cobre todos os alvos. Eles não estão errados, só incompletos.

## DoD Critic (enhanced)

- DoD row «18 (TODO repo-wide)»: `// Hotplug is not handled; todo: re-list monitors on udev events.` (marcador minúsculo/capitalizado NO MEIO do comentário), `// To do: …` e `(fixme: …)` passam — o 1º grep é sensível a caixa e o 2º só aceita o marcador colado à abertura do comentário. Correção proposta pelo critic: `git grep -InIEi '\b(to[ -]?do|fixme)s?\b[[:space:]]*[:(]'` com o mesmo filtro de `#N` (0 hits no HEAD; não pega "Todos os ajustes").
- DoD row «22 (TODO *.rs, PROJECT)»: mesma mutação em `scroll.rs` passa no Verify do PROJECT.
- Suspeita (objective:false): «12» a contagem textual de `write_feature(` não impede um laço de escritas Safe.

O código atende todos os critérios (nenhum TODO/FIXME na árvore); as lacunas são dos comandos `Verify:` escritos pelo orquestrador.

**Verdict:** BLOCKED
