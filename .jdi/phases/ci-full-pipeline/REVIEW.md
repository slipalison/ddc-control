# Phase 8: Review  (slug: ci-full-pipeline)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 1, gravada pelo orquestrador a partir do relatório integral do reviewer (o harness nega ao subagente a escrita de `.md`).

A revisão foi feita em 2026-09-28 sobre o HEAD local `66e5a5a`. Os commits de `cccd78f..66e5a5a` mexem só em `.jdi/` (o SUMMARY e o `pipeline-evidence.env`), então o código revisado é o mesmo do run de evidência. O template revisado é o do gw#15 (`ad8a96e`), provado pela branch descartável `teste/esteira-app-desktop` (`a13c154`).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` saiu 0. `cargo check` de `ddc-core`, `ddc-adapters` e `ddc-cli` no alvo linux-gnu também saiu 0. O lado Windows é compilado só no CI: `esteira / qualidade / rust-windows` deu `success` no run 36496391097. |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware). É o mesmo número da `release-packaging`, e nenhum `.rs` mudou. |
| Coverage | PASS | 83,36% de linhas (3.294 linhas, 548 não cobertas), sobre um piso de 80%. |
| Lint | PASS | fmt e clippy com `-D warnings` saíram 0. actionlint 1.7.12 com shellcheck 0.10.0 limpo no `ci.yml`, no `pipeline.yml` e no `sonar.yml` de `ad8a96e`, e no exemplo; shellcheck `-S style` limpo no `sonar-coverage.sh`. |
| Hexagonal/Safety/Hygiene | PASS (com WARN) | Nada novo de 5.1 a 5.9 (nenhum `.rs` mudou). `cargo audit` com o banco atualizado saiu 0 (524 crates), sem dependência nova. A revisão de segurança do CI ficou com os avisos W4 e W8. |
| Consistency | WARN | D-1 a D-6 conformes. Avisos W2, W6 e W9. |
| UI Validation | PASS | `npm ci --ignore-scripts` + `npm test`: `node --test` com 161/161 (88,15% de linhas); Playwright com 138 passed e 6 skipped, saída 0. |
| DoD | PASS_PENDING_MANUAL | 11/11 auto (8 do CONTEXT e 3 do PROJECT), 2 manuais pendentes. |

## Blockers
Nenhum.

## Warnings
- **W1 — O TruffleHog não varre o histórico inteiro em PR nem em push.**
  - Quatro lugares dizem que varre: D-2, `ci.yml:11`, `README.md:498` e `CHANGELOG.md:12`.
  - A action `trufflesecurity/trufflehog@363923b`, com `base: ""`, pega o intervalo do evento: em PR, `base.sha..head.sha`; em push, `event.before`. O run 36496391097 varreu 50 chunks (76.004 bytes). Quem varre o histórico inteiro é só o Gitleaks (339 commits).
  - O comentário do `seguranca.yml:357` no gw vem de antes do #15 e fica como todo lá.
- **W2 — O `lancar` do `pipeline.yml` roda em qualquer evento que não seja PR, `workflow_dispatch` incluído, e a doc diz "push only".**
  - Onde: `README.md:502` e `ci.yml:15-16`.
  - Risco baixo: disparar o workflow exige escrita, e os portões continuam valendo. Registrar na D-1, ou tirar o gatilho.
- **W3 — O caminho de release pelo `pipeline.yml` nunca rodou.**
  - Não foram exercitados: `artefatos_release→artefatos`, `rascunho`, o `startsWith` do `ramo_de_ensaio` e a expressão do `tag_movel_major`. Nenhuma linha do DoD cobre isso.
  - `lancar.yml`, `qualidade.yml` e `versao.yml` são idênticos aos de `d6d340a`, que publicou a v0.1.0. Um ensaio em `ensaio-release/*` provaria a fiação.
- **W4 — `SONAR_TOKEN` sem expiração, no `env` do JOB do Sonar.**
  - O `comando_testes` (build scripts e proc-macros de 524 crates) roda nesse mesmo job e poderia lê-lo, por exemplo por um crate comprometido.
  - Recomendação ao usuário: expiração e rotação. No gw, fica como todo expor o token só no passo `Analisar`.
- **W5 — O pin está numa branch descartável, e a evidência não acompanha o repin.**
  - As branches `teste/esteira-app-desktop*` não podem ser apagadas antes do `@main`.
  - Depois do repin, o run do PR precisa sair verde antes do merge.
- **W6 — O roadmap ficou com a premissa antiga do Clippy.**
- **W7 — Três linhas do DoD provam menos do que o texto diz:**
  - linha 7: o `Verify` não confere a "tabela no painel";
  - linha 5: a metade da anotação é fail-open se o `gh api` falhar;
  - linha 6: as ferramentas vêm do ref do PR, não do `RUN_ID`, e não cobrem o TruffleHog nem o SBOM.
- **W8 — `sonar-coverage.sh:67`:** o `grep | grep -qv` sob `pipefail` pode pular o guard em silêncio (SIGPIPE).
- **W9 — T11 pela metade:** o gw#15 e o PR #12 continuam em rascunho.

Conferido e OK:
- sha256 do Node 24.18.0 e do cargo-llvm-cov 0.9.1;
- `.trivyignore` ↔ `audit.toml`;
- `pacotes_sistema` e as entradas do `Portao` por `env`;
- as permissões da D-1;
- nenhuma composite tocada;
- ruleset intacto (D-6).

## DoD Checklist (gate 8)

Os `Verify:` foram extraídos por regex do CONTEXT.md e do PROJECT.md e rodados literalmente com `bash --noprofile --norc -eo pipefail` e `LC_ALL=C.UTF-8`. Os mutantes rodaram contra o run 36493710073 (Sonar sem token).

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `ci.yml` = um job `esteira`, `pipeline.yml` no `WORKFLOWS_TEST_SHA`/`@main`, Sonar sem dispensa, CodeQL 3 linguagens, pisos 80 | CONTEXT | Auto | PASS | `OK` (estrutural; comportamento nas linhas 2–7) |
| 2 | Run de PR `success`, CONJUNTO de jobs da esteira em `success`, `lancar` não roda | CONTEXT | Auto | PASS | `OK`; mutante reprova |
| 3 | Template = gw#15 (`referenced_workflows`, TEST_SHA filho só com o repin, head do #15) | CONTEXT | Auto | PASS | `OK`; head do gw#15 = `ad8a96e` |
| 4 | Sonar: os dois relatórios, sensores LCOV/Rust Enterprise, lcov-ui lido, QG PASSED; API: OK e cobertura ≥ 80 | CONTEXT | Auto | PASS | `OK coverage=86.3`; mutante reprova |
| 5 | `node-ui`: `Aprovado: n% >= piso de 80%`, sem anotação de relatório ausente | CONTEXT | Auto | PASS | `OK` (W7: metade fail-open) |
| 6 | Gitleaks > 0 commits, Semgrep > 0 regras, {CodeQL, Semgrep, Trivy} no code scanning do PR | CONTEXT | Auto | PASS | `OK gitleaks=339 semgrep=1074 tools=CodeQL,Semgrep,Trivy` |
| 7 | Check run `esteira / Portao` em `success` no `HEAD_SHA` | CONTEXT | Auto | PASS | `OK` (check run 109178859408) |
| 8 | Script fora do CI escreve os dois LCOV, `SF:` da UI relativos e com o conjunto exato | CONTEXT | Auto | PASS | `OK` |
| 9 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386/0/9 |
| 10 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 83,36% |
| 11 | No `TODO`/`FIXME` without linked issue | PROJECT | Auto | PASS | `OK` |
| 12 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `[Unreleased]` com Added e Changed desta phase, e `## [0.1.0] - 2026-09-28` datado. Ressalva W1. |
| 13 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: a seção `## CI` foi reescrita. Ressalvas W1 e W2. |

**Totals:** 13 items | Auto: 11 (11 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod ci-full-pipeline`. Sem isso, o `/jdi-ship` recusa a phase.

## Recommendation
Aprovado. Antes do merge do PR #12, por ordem de custo:
1. Corrigir W1, W2 e W6; emendar a D-2 e registrar o `workflow_dispatch` na D-1. Os itens manuais só se confirmam depois disso.
2. Corrigir o W8.
3. Pedir ao usuário expiração e rotação do token (W4), e abrir o todo no gw.
4. Fazer um ensaio em `ensaio-release/*` (só rascunho) para provar a release pelo pipeline (W3).
5. Depois do gw#14 e do gw#15: repin para `@main`, run verde do PR, e só então apagar as branches `teste/*` (W5); tirar os PRs de rascunho (W9).
6. `esteira / Portao` obrigatório no ruleset, se o usuário decidir (D-6).
