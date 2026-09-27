# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 14 (rodada 3), re-verificação completa (gates 1-8) sobre o HEAD `3c1bb2d` (commits `b77a644..337761a` + `3c1bb2d`). Tudo rodado em `bash` a partir da raiz, sem `DDC_HW_TESTS`, `DDC_TRAY_FAKE` nem `DDC_TRAY_DEBUG`. Nenhum teste `#[ignore]` de hardware rodou, e nada foi escrito no monitor real.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` exit 0 (tray fora por D-2026-09-26-tray-app-9) |
| Tests | PASS | 385 passed, 0 failed, 9 ignored (hardware). Iter 13 tinha 384: +1, o `the_effective_csp_is_the_strict_policy` |
| Coverage | PASS | 83.31% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0 |
| Lint | PASS | `cargo fmt --all --check` exit 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. 5.10: W-1 (`cargo audit`, conhecido) |
| Consistency | PASS | commits com escopo `tray-app` e tipos coerentes, `Cargo.lock` intocado. D-1, D-2 e D-2026-09-26-tray-app-2/-6/-7, D-2026-09-27-tray-app-6/-10/-11 conformes |
| UI Validation | PASS | `npx playwright test`: 138 passed, 6 skipped (= screenshots), 0 failed/flaky. `text-guard.spec.mjs` ✓ nos 2 temas. Porta 1420 livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido, fica para `ci-crossbuild`).** `cargo audit`: `error: 1 vulnerability found!`, RUSTSEC-2018-0005 (`serde_yaml` 0.7.5, via `ddc-hi` 0.4.1 → `mccs-db` 0.1.3, desde a phase `ddc-backends`). Há ainda 3 avisos permitidos: RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained), RUSTSEC-2024-0320 (`yaml-rust`, unmaintained) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound, via tauri/gtk). O `Cargo.lock` não mudou nesta iteração.
- **W-2 (Gate 8 / C8, robustez do Verify, sem defeito no HEAD).** A trava nova do corpo do teste de CSP é um `grep -qF` sobre `lib.rs` inteiro, sem tirar comentários:
  - `grep -qF 'assert_eq!(csp.as_deref(), Some(STRICT_CSP));' $c/src/lib.rs` casa também com a linha `// assert_eq!(csp.as_deref(), Some(STRICT_CSP));`;
  - o filtro `--lib the_effective_csp_is_the_strict_policy` dá `1 passed` com o corpo vazio;
  - `super::context()` não é travado, então um corpo com `let csp = Some(STRICT_CSP.to_string());` também passa.
  - **Consequência:** um `tauri.linux.conf.json` com `"csp": null` (a M-a do critic, que o loop `jq` do C8 aceita num arquivo de plataforma) somado a esse corpo esvaziado voltaria a passar no C8. Isso exige duas edições simultâneas e não ocorre no HEAD: li `apps/ddc-tray/src-tauri/src/lib.rs:331-339`, e o corpo lê `super::context()` e afirma a CSP e o `dev_csp == None`. É a mesma classe da M2 da iter 13, mas lá o `grep` enganado era coberto pelo teste. Aqui nada cobre.
  - **Sugestão** (edição só no CONTEXT, pelo orquestrador): ler as asserções só em linhas que não são comentário, por exemplo `grep -vE '^\s*//' $c/src/lib.rs | grep -qF ...`. Travar também `let context = super::context();` e `assert_eq!(security.dev_csp, None);`, e manter o `grep` da `const STRICT_CSP` sobre o arquivo cru, porque a política tem `http://`. Conclusão por leitura, não executada: a review é read-only.

## Avaliação dirigida da iteração 14

### Guarda de tradução (`apps/ddc-tray/src/i18n/guard.js`, `MutationObserver`)
- **Só no demo local.**
  - `app.js:53-56`: `demo = bridge.mode === 'demo'`, `textGuard({ enabled: demo })` e `t = guard.track(...)`.
  - `bridge.js:42-46`: o modo `demo` só existe sem `__TAURI__` e com `isLocalDevServer`, isto é, `http:`/`https:` em `localhost`/`127.0.0.1` (D-2026-09-27-tray-app-6). O mesmo `demo` já decidia o `pseudo`.
- **Nada no app real.** Desligada, a guarda devolve o `OFF` congelado (`guard.js:46-51`): `track` devolve o próprio translator, `check` nunca reporta e `watch` não cria observer. O teste `node --test` confere `off.track(tr) === tr` e `FakeObserver.made == []`. No browser, `text-guard.spec.mjs` serve o popup em `http://tauri.localhost` e confere 0 reports e `__ddcDemo` indefinido. Os dois testes passaram nos 2 temas.
- **Sem falsos positivos.**
  - O `index.html` não tem texto estático com letra: `<title data-i18n>` vazio, `#message-detail translate="no"`, sem script ou style inline. Assim a varredura inicial do `watch`, que roda antes do `translatePage`, não acusa nada.
  - A checagem roda na microtask, depois da tarefa que escreveu. Isso cobre o padrão "texto primeiro, `translate="no"` logo depois" (`markVerbatim(element(...))`).
  - A suíte Playwright inteira (138 testes, todos os cenários, pseudo-locale, teclado, arrasto, diálogos e falhas) roda com a guarda ligada e termina com 0 `console.error`.
- **Travada contra regressão.** Desligar a guarda, tirar o `watch` ou tirar o `track` reprova o `text-guard.spec.mjs` ou a suíte inteira. O spec e os 4 testes `node --test` estão no manifesto do harness (C15, hash `ba730a00…c001`), e o título exato está no C7.
- **Desvio de forma aceito (D-2026-09-27-tray-app-11).** A D-11 pede que os sinks emitam o `console.error`, e a implementação usa um observer do documento, que é um superconjunto: pega também escritas diretas e sinks futuros.
  - Limites registrados pelo doer, nenhum bloqueante: um valor idêntico a uma tradução do locale ativo não é acusado; um texto trocado dentro da mesma tarefa não é checado; dado sob `translate="no"` é aceito como dado.
  - O scanner estático e o pseudo-locale continuam cobrindo o primeiro limite.
- **Exceção nova no `NOT_LANGUAGE`** (`i18n-html.test.mjs:620-624`):
  - é exata, casando arquivo `i18n/guard.js` e literal `'ddc-tray: untranslated text:'` por igualdade (`:731`);
  - reprova se o literal sumir (`:739-743`);
  - o motivo está registrado;
  - é mensagem de console do dev, só emitida no demo, e nunca aparece no popup. O template do report, lido como `${} ${} in ${}`, tem uma palavra só e não é frase.
  - Se alguém importar `UNTRANSLATED` para a UI, a própria guarda acusaria no demo. Aceita.

### CSP efetiva (`lib.rs`, `build.rs`)
- `generate_context!` aparece uma vez só (`lib.rs:147-149`) e é usado por `run()` (`:134`) e pelo teste. Por isso o teste lê a config que o binário embute: base mesclada com a plataforma.
- `build_tests` é `#[cfg(test)]` sem `target_os`, então roda em toda plataforma.
- `build.rs`: `cargo::rerun-if-changed=.` (o diretório do crate) fecha o furo do build incremental que não via um `tauri.<plataforma>.conf.json` novo. Não há `unsafe`.
- Conferi que não há rebuild perpétuo: dois `cargo build -p ddc-tray --locked` seguidos, 0,20 s e 0,17 s, sem `Compiling`. O `src-tauri/` não contém `target/`.

### Plano
- `i18n/guard.js` e `tests/e2e/text-guard.spec.mjs` não estão nos `Files modified` do PLAN. Vêm da D-2026-09-27-tray-app-11, como os demais arquivos das iterações de correção. É informativo, não é WARN. As 8 tasks continuam `completed`, cada uma com teste.

### Protocolo da instância do usuário
- Antes do C11 e do C17, `pgrep -xa ddc-tray` mostrava a instância. Ela foi encerrada com `pkill -x ddc-tray` (nunca `-f`), esperando o processo sair, e reaberta com `setsid -f /home/slipalison/.local/bin/ddc-tray` logo depois de cada smoke.
- PIDs: 2346029 → 2378511 → 2379032, a única viva no fim.
- O C11 passou na 1ª execução, sem `tray activated` externo.

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
| 8 | CSP estrita: sem devCsp/devUrl, `custom-protocol`, `is_not_a_dev_build`, CSP efetiva exata | CONTEXT | Auto | PASS | exit 0, `OK` (ver W-2: a trava do corpo do teste não tira comentários) |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | exit 0, `OK` |
| 10 | single-instance como 1º `.plugin` do builder | CONTEXT | Auto | PASS | exit 0, `OK` |
| 11 | Smoke SNI `--activate` (item do próprio PID, popup mostrado e mantido, sem panic) | CONTEXT | Auto | PASS | `OK` na 1ª execução: PID 2378276, `popup shown`, ainda mostrado 1,5 s depois; backend real só com leituras |
| 12 | Teste `#[ignore]` de hardware RTK existe, compila e é gated | CONTEXT | Auto | PASS | exit 0, `OK` (listado, não executado) |
| 13 | Gate 7: 0 erro de console, 0 axe critical/serious, dropdown, arrasto, pseudo | CONTEXT | Auto | PASS | exit 0, `OK` em 16 s; contagem à parte: 138 passed, 6 skipped |
| 14 | Fora de servidor local o bridge nunca cai no demo + `withGlobalTauri` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 15 | Harness = o revisado (hash `ba730a00…c001`) | CONTEXT | Auto | PASS | exit 0, `OK` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | exit 0, `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`: PID 2378787, `75 -> 80` na vertical, nada na horizontal, `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | exit 0, `OK` |
| 19 | Screenshots regenerados, nada pulado, byte a byte iguais | CONTEXT | Auto | PASS | exit 0, `OK`; `git status` sem mudança em `docs/screenshots` |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (385 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL lines 82.88%, exit 0. Gate 3: 83.31% |
| 22 | No TODO/FIXME without linked issue (`*.rs`) | PROJECT | Auto | PASS | exit 0, `OK` |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só o heading `## [Unreleased]` (CHANGELOG.md:8), sem `## [version]`; a entrada da iter 14 está em `### Added` |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: README cita `ddc-tray` 20×; a iter 14 acrescentou 2 bullets em Tests (CSP efetiva e guarda de tradução) |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- Nenhum blocker. O código da iteração 14 cumpre a D-2026-09-27-tray-app-11:
  - a CSP efetiva é provada por um teste que lê o contexto que o Tauri embute, e o build incremental agora vê um arquivo de plataforma novo;
  - a guarda de tradução fica só no demo local, tem custo zero no app, não deu falso positivo na suíte inteira, e sua exceção na trava de frases é exata e justificada.
- **W-2:** endurecer o C8 no CONTEXT, travando as asserções fora de comentários e também `super::context()` e o `dev_csp`, antes de rodar o DoD critic. Senão é o tipo de lacuna que ele acharia.
- **W-1:** segue com a `ci-crossbuild`.
- Os 2 itens Manual (CHANGELOG/README) ficam para o PR, via `/jdi-confirm-dod tray-app`.

## Nota do orquestrador (pós-review, antes do critic)

W-2 era do `Verify:` do C8 (escrito pelo orquestrador): o grep do corpo do teste casava com uma linha comentada. Agora o módulo `build_tests` inteiro de `lib.rs` é congelado por SHA-256 (`645f5f1c…2673`), o `run()` precisa usar `.build(context())` em linha não comentada e só pode haver um `generate_context!`. HEAD imprime OK; o mutante (asserção comentada + `tauri.linux.conf.json` com `"csp": null`) reprova numa cópia descartável. Nenhum arquivo do harness congelado mudou.
