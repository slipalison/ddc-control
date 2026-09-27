# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 7 (rodada 2), re-verificação completa dos gates 1-8 sobre `HEAD` `5919fb0` (commits `d32319f..e5281a8` + `5919fb0`). Revisão read-only: nenhum arquivo do repo foi alterado além deste REVIEW.md. Os experimentos de mutação rodaram numa cópia descartável de `apps/ddc-tray` no scratchpad, já apagada.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli` em `x86_64-unknown-linux-gnu` exit 0 e em `x86_64-pc-windows-msvc` exit 0. O tray fica fora do cross-check Windows (D-2026-09-26-tray-app-9). |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware), igual à iter 6: nenhum `.rs` mudou na iter 7. `node --test`: 135 pass, 0 fail/cancelled/skipped/todo. |
| Coverage | PASS | 83.22% lines (linha TOTAL, `main.rs`/`build.rs` excluídos, `--fail-under-lines 80` exit 0). Literal do PROJECT (sem exclusões): 82.78%. |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; nenhum `#[allow(...)]` fora de testes. |
| Hexagonal/Safety/Hygiene | WARN | 5.1-5.9 e 5.11 limpos. 5.10 = W-1 (conhecido, fica com a `ci-crossbuild`). |
| Consistency | PASS | 6 commits `(tray-app)` com tipos coerentes (feat/test/docs). Nenhum `.rs`, `Cargo.lock` ou `package*.json` tocado. D-1, D-2 e D-2026-09-26-tray-app-2/-6/-7 e D-2026-09-27-tray-app-6/-7 conformes no produto. |
| UI Validation | PASS | Playwright: 88 passed, 6 skipped (= screenshots sem `SCREENSHOTS=1`), 0 failed/flaky. 10/10 `critical_paths` × tema, dropdown 16/16, `dr = 2`, `ps = 26` (13 estados × 2 temas). |
| DoD | PASS_PENDING_MANUAL | 22/22 auto, 2 manual pending |

## Blockers
Nenhum.

## Warnings

- **W-1 (5.10, herdado, fica com a `ci-crossbuild`):** `cargo audit` lista 1 vulnerabilidade e 3 avisos.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5).
  - Avisos: RUSTSEC-2024-0370 (`proc-macro-error`, sem manutenção), RUSTSEC-2024-0320 (`yaml-rust`, sem manutenção), RUSTSEC-2024-0429 (`glib` 0.18.5, unsound).
  - Nenhuma dependência nova entrou na iter 7.

- **W-2 (Gate 7 / DoD #13, D-2026-09-27-tray-app-7; lacuna só do harness, o produto está limpo):** o check do pseudo-locale aceita um literal colado a uma tradução no mesmo nó de texto ou atributo.
  - **Onde:** `apps/ddc-tray/tests/e2e/pseudo-locale.spec.mjs:174` (`const marked = value.includes(mark);`) e `:178`. A regra só exige que o texto *contenha* `⟦`. Não exige que todo texto fique entre marcadores.
  - **Reprodução** (cópia descartável, repo intocado):
    - Mutação: `const PROBE_SUFFIX = ' (nothing else to try)';` e `t('more.probeEmpty') + PROBE_SUFFIX` em `showProbe` (`src/app.js:857`).
    - A página mostra `⟦Nenhum ajuste oculto respondeu.⟧ (nothing else to try)`: um literal em inglês no popup pt-BR.
    - Resultado: `i18n-html.test.mjs` 15/15 pass (o scanner estático não segue a `const`), `pseudo-locale.spec.mjs` 28/28 pass e suíte inteira 88 passed, 6 skipped. Ou seja, as condições do Verify do C13 continuam satisfeitas.
  - **Isso contradiz o rationale da D-7:** "qualquer literal hardcoded, venha de `const`, helper ou template, aparece sem marcador e reprova".
  - **Uma variante estrita é viável sem mexer no produto.** Ela remove os segmentos `⟦…⟧` balanceados (inclusive os aninhados) e reprova se sobrar `\p{L}` fora de dado. Resultados:
    - no produto atual: 28/28 pass;
    - no mutante: 2 failed, com `has text outside the marks: " (nothing else to try)"`.
  - **Limite de desenho, só registrado:** um literal colado a um DADO sob `translate="no"` (ex.: nome do monitor + sufixo via `const`) não é pego por nenhum check. A D-7 aceita nós de dado sem verificar o conteúdo.
  - **Por que é WARN e não BLOCK:** nenhum Verify falha e o produto não viola a D-6. Mas a linha #13 do DoD está parcialmente oca no subcritério "vem de tradução".
  - **Correção:** exige mudar o spec e refixar o hash do harness (C15). Pela D-7, isso é ato do orquestrador.

- **W-3 (menor, texto da dica i2c):** o espaço entre "Setup guide:"/"Guia de configuração:" e o caminho existe só no visual (`margin-inline-start: 0.3em` em `apps/ddc-tray/src/styles.css:1118`).
  - `app.js:389` monta `replaceChildren(texto, <code>)` sem nó de espaço. O `innerText` e o texto copiado ficam `…Guia de configuração:docs/linux-ddc-setup.md`.
  - A árvore de acessibilidade separa `text` e `code`, então leitor de tela não é afetado.
  - Sugestão: um espaço real, seja no fim do texto do locale, seja num nó `' '`. O scanner e o spec pseudo ignoram texto sem letras.

## Avaliações pedidas pelo orquestrador

**Mudança de texto da dica i2c (en/pt-BR): correta.**
- `hint.i2c` perdeu o placeholder `{doc}` nos dois locales, o que mantém a paridade de placeholders (C7 OK).
- Agora termina em "Setup guide:" / "Guia de configuração:", e o caminho vem depois, em `<code translate="no">`.
- Isso resolve o `.⟧` órfão que a frase partida gerava sob pseudo.
- As duas traduções são paralelas e naturais. `i2c-dev` e `/dev/i2c-*` continuam dentro da tradução, como identificadores técnicos, o que é aceitável.
- A dica só aparece nos estados vazio e erro no Linux, fora dos screenshots versionados. Os 6 PNGs têm SHA-1 idêntico antes e depois do C19.
- Único reparo: W-3.

**`translate="no"` só em dados, nunca em traduções: confirmado.**

Lendo o diff:
- `markVerbatim(..., true)` / `verbatim(...)` só são aplicados a:
  - nome do monitor no cabeçalho, só quando há monitor (o título do app segue traduzido);
  - opções do seletor de monitores, exceto a `header.silent`, que é `t()`;
  - nomes de valor NC sem chave `value.*` (`optionLabel` → `verbatim: true` só quando `translateOr(..., null)` devolve `null`);
  - nome MCCS do core sem alias traduzido (`labelVerbatim`);
  - códigos VCP (`view.hex`);
  - `#message-detail`, a mensagem técnica do backend, dado do contrato `{kind,message}`;
  - o caminho do guia.
- `format.unnamed`, `format.code`, `format.percent`, `header.silent` e `confirm.*` continuam `translate` normais, com marcador.
- O único dado que não vem do core/contrato é o caminho `docs/linux-ddc-setup.md` (`I2C_DOC`). É nome de arquivo, que por definição não se traduz, e a spec HTML prevê `translate="no"` para esse caso.

Em runtime:
- O spec reprova tradução sob `translate="no"` (check inverso). Ele passa em 26/26.
- A mutação M4 do SUMMARY (`<body translate="no">`) reprova 26.
- O pseudo só liga com `bridge.mode === 'demo'` (`app.js:48`). O teste da origem do app (`tauri.localhost`) prova 0 textos marcados.

## DoD Checklist (gate 8)

Cada `Verify:` foi extraído do `.md` por script, conferido byte a byte contra o CONTEXT (diff vazio nos 19 Auto) e rodado literalmente com `bash` a partir da raiz, sem `DDC_HW_TESTS` nem `DDC_TRAY_FAKE`. Os 2 itens Manual do CONTEXT repetem os do PROJECT (`Source: PROJECT`) e aparecem uma vez só.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` limpos com `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 3 | Só a composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (`lib.rs:77`, 1×) |
| 4 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 5 | `#![forbid(unsafe_code)]` em `src-tauri`, nenhum `unsafe` | CONTEXT | Auto | PASS | `OK` (o único hit do 5.5 é o doc comment `lib.rs:146`) |
| 6 | `node --test` (debounce, view-model, bridge demo…) sem falhas | CONTEXT | Auto | PASS | `OK`, 135 pass, 0 fail/cancelled/skipped/todo |
| 7 | Paridade i18n + nenhum texto hardcoded no HTML | CONTEXT | Auto | PASS | `OK` |
| 8 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 9 | Capabilities sem shell/fs/http/opener | CONTEXT | Auto | PASS | `OK` |
| 10 | `tauri-plugin-single-instance` 1º no builder | CONTEXT | Auto | PASS | `OK` |
| 11 | Smoke SNI `--activate` | CONTEXT | Auto | PASS | `OK`: PID 1314512, item `org.kde.StatusNotifierItem-1314512-1` do próprio PID, `popup shown` e ainda mostrado 1,5 s depois, sem panic. Backend real, só leituras. |
| 12 | Teste `#[ignore]` de hardware RTK gated por `DDC_HW_TESTS=1` | CONTEXT | Auto | PASS | `OK` (listado, NÃO executado) |
| 13 | Gate 7: console/axe limpos nos `critical_paths`, arraste, pseudo-locale | CONTEXT | Auto | PASS | `OK`: 88 passed, 6 skipped = screenshots, n=10, dd=16/16, dr=2, ps=26. Ver W-2 (literal + tradução no mesmo nó passa). |
| 14 | Bridge nunca cai no demo fora de servidor local + `withGlobalTauri` | CONTEXT | Auto | PASS | `OK` |
| 15 | Harness de UI = o revisado (hash D-7) | CONTEXT | Auto | PASS | `OK`: `9787e931…3288`, 19 arquivos, última mudança em `0263cc0` |
| 16 | Nenhum `<select>` nativo em `src/` | CONTEXT | Auto | PASS | `OK` |
| 17 | SNI `ksni`: roda → percentual testado + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`: PID 1315032, vertical `75 -> 80`, horizontal sem escrita em 1,5 s, `80 -> 75` |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | `OK` |
| 19 | Screenshots claro/escuro regenerados e versionados | CONTEXT | Auto | PASS | `OK`: nada pulado, 6 PNGs com SHA-1 idêntico, `git diff --quiet -- docs/screenshots` |
| 20 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | `OK` (383 passed, 0 failed, 9 ignored) |
| 21 | Cobertura >= 80% de linhas | PROJECT | Auto | PASS | literal `cargo llvm-cov --workspace --summary-only`: TOTAL Lines 82.78% |
| 22 | Nenhum TODO/FIXME sem issue (`*.rs`) | PROJECT | Auto | PASS | `OK` |
| 23 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` com `### Added`, que já descreve o pseudo-locale (iter 7) |
| 24 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: README ganhou o parágrafo `pseudo=1` (l. 345) e o bullet do check pseudo (l. 366) |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Notas de execução
- **Smokes:** um de cada vez.
  - Antes de cada um, `pgrep -xa ddc-tray` mostrava a instância do usuário, encerrada com `pkill -x ddc-tray` (nunca `-f`). Logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu.
  - PIDs do usuário: 1267586 → 1314750 → 1315274. Só esta última está viva no fim.
- **Porta 1420:** livre antes e depois dos testes com Playwright.
- **Monitor real:** nenhuma escrita. `rtk_qhd_hdr` não foi executado. O C11 usou o backend real só com leituras, e o C17 usou o monitor simulado.
- **Gate 5, detalhes:**
  - 5.6: os hits estão dentro de `#[cfg(test)]` (`kwin_placement.rs:195+`, `lib.rs:239+`, `stop_signals.rs:82+`) ou num doc comment (`worker.rs:141`).
  - 5.7c: `real_monitor.rs` tem só testes `#[ignore]` com `DDC_HW_TESTS`. O `lib.rs:77` é a composition root.
  - 5.5c: `[workspace.lints.rust] unsafe_code = "deny"` mais o `forbid` nos crate roots do tray.

## Recommendation
- Os gates 1-7 estão verdes e os 22 itens Auto do DoD passam, rodados literalmente. Faltam os 2 itens Manual (CHANGELOG/README) via `/jdi-confirm-dod tray-app`, ou no PR, conforme a cadeia autônoma.
- **W-2 é o ponto que um DoD critic tende a apontar:** o spec pseudo aceita `⟦tradução⟧ + literal` no mesmo nó.
  - O ajuste cabe só no harness: exigir que nada com letra sobre fora dos segmentos `⟦…⟧` em nós que não são dado.
  - Já foi provado viável: o produto atual passa, o mutante reprova.
  - Custa um novo hash no C15, que é ato do orquestrador (D-7).
- **W-3** é cosmético e pode ir junto.
- **W-1** segue com a `ci-crossbuild`.
- **Para a validação humana do PR:** o binário instalado em `~/.local/bin/ddc-tray` (03:10) é anterior às iter 5-7. O popup que o usuário vê hoje não tem as mudanças de UI dessas iterações.
