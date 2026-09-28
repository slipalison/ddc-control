# Phase 7: Review  (slug: release-packaging)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 5. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
>
> **Prova da herança (Gates 1–7 vêm da iteração 4, que herdou da 3 e da 2)**
> - `git diff --quiet b8f0265 HEAD -- . ':!.jdi'` sai 0. `git diff --quiet f6063d7 (HEAD_SHA) HEAD -- . ':!.jdi'` também sai 0.
> - O único commit novo é o `301211d`, que só mexe em `CONTEXT.md` (linha 6 do DoD) e no `REVIEW.md`.
> - O head do gw#14 continua `d6d340a`, com os checks em pass. O `origin/main` é `8156ae1` = `MAIN_SHA`.
> - Gate 8: os 9 `Verify:` foram extraídos por programa e rodados literalmente com `bash --noprofile --norc` das 17:47:24Z às 17:49:29Z; depois, a baseline do PROJECT.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Herdado. |
| Tests | PASS | Re-executado: 386 passed, 0 failed, 9 ignored. |
| Coverage | PASS | Re-executado: 82.93% literal. 83.36% no Gate 3 (herdado). Piso de 80%. |
| Lint | PASS | Herdado. |
| Hexagonal/Safety/Hygiene | PASS | Herdado. |
| Consistency | PASS | `301211d`: escopo `release-packaging`, tipo `docs`, só `.jdi/`. D-8 respeitada: nada foi criado nem apagado nesta iteração. |
| Segurança do template (gw#14) | PASS | Herdado; head `d6d340a` inalterado. |
| UI Validation | PASS | Herdado. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto PASS (9 CONTEXT + 3 PROJECT); 2 manuais pendentes |

## Blockers
- nenhum. O bloqueio do crítico da iteração 4 na linha 6 foi resolvido pela reescrita, que discrimina (ver Notas).

## Warnings
- **W-7 residual (texto):** o corpo da release pública 398454265 mistura o CHANGELOG em inglês com os títulos em pt-BR das notas do `versao`.
  - A linha 6 exige que o corpo COMECE pelos 18.838 bytes da seção `[Unreleased]`, byte a byte, então qualquer edição nesse trecho derruba a linha.
  - Só as notas do `versao`, que vêm depois da seção, podem ser editadas.
- **W-8 (herdado, fora do diff):** continua aberto.
- **W-9 (baixo):** ferramentas fixadas por hash em tags que o dono reescreve. Falha fechado.
- **W-10 (cosmético):** comentário de 133 caracteres em `qualidade.yml:912`.
- **W-11 (operacional, prioridade alta):** a ordem do merge está invertida.
  - A `main` roda com `uses: …@d6d340a…` (`ci.yml:42/54/130`), e os gw#13 e #14 continuam abertos.
  - Ordem: gw#13; depois rebasear e mergear o gw#14; depois o PR que troca por `@main`.
  - O próximo merge que peça versão publica a `v0.1.1`. A linha 7 só vale até lá, então o ship vem antes.
- **W-12 (processo, MITIGADO):** a linha 1 compara árvores e não quebra com rebase.
- **W-13 (baixo):** a linha 1 prova `MAIN_SHA ≡ HEAD_SHA`, mas não compara o `HEAD_SHA` com a HEAD da branch. Hoje o fato vale (re-provado às 17:46:55Z).

## Notas
- **Linha 6 reescrita: não é oca, discrimina.**
  - O `awk` do Verify e a função `secao` do template (`lancar.yml:361-367`) extraem a MESMA seção: 78 linhas, 18.838 bytes, 4 títulos `###`, 42 itens e 25 subitens.
  - Controles OK: o corpo real, o corpo com CRLF (tolerância intencional) e o log do ensaio sem mudança.
  - 9 mutantes reprovam (rc=1):
    - B1: o mutante do crítico, `Montar as notas` com `dentro && /^- /`;
    - B2: sem os `###`;
    - B3: sem a última linha;
    - B4: um subitem com recuo diferente;
    - B5: as notas do `versao` antes da seção;
    - B6: um byte trocado;
    - L1: `ID` errado no log do ensaio;
    - E1: `ENSAIO_RUN_ID` do ensaio anterior, cujo log diz `ID: 398335754`;
    - E2: o ensaio que falhou.
  - Divergência mínima, que falha fechado: o `awk` do Verify não pula linhas só com espaços no começo da seção. Isso dá falso negativo, nunca falso positivo.
- **Ligação ensaio → rascunho → publicação, provada pela própria linha.** O `  ID: 398419349` do log do ensaio vem do `env:` do passo `Resumo` (`steps.release.outputs.id`), resolvido pelo runner, e não de eco. Esse id é o do rascunho que ESTE ensaio criou: o ensaio apagou o 398335754 antes. Em seguida:
  - a `main` apagou o 398419349 ("Rascunho 398419349 de v0.1.0 apagado.");
  - a `main` publicou a 398454265 no `MAIN_SHA`, que tem a árvore do `HEAD_SHA`.
- **Linhas 1 e 7 inalteradas.** Os mutantes da iteração 4 continuam valendo. O estado externo segue igual: 1 tag (`v0.1.0 → 8156ae1`), 1 release (398454265), nenhum rascunho.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run real de `pull_request` no `HEAD_SHA` (success, `rust-linux` em 22.04, `salto=inicial versao=0.1.0`, `lancar` skipped); `MAIN_SHA` ancestral de `origin/main` e com árvore igual à do `HEAD_SHA` fora de `.jdi/` | CONTEXT | Auto | PASS | `OK` |
| 2 | Template que rodou = PR #14 (`WORKFLOWS_SHA`) | CONTEXT | Auto | PASS | `OK`; head `d6d340a` |
| 3 | `pacotes-rust-linux` inspecionados; carimbo provado | CONTEXT | Auto | PASS | `OK` |
| 4 | `pacotes-rust-windows` e os 5 passos | CONTEXT | Auto | PASS | `OK` |
| 5 | Ensaio `ENSAIO_RUN_ID` | CONTEXT | Auto | PASS | `OK` |
| 6 | Release PUBLICADA 398454265 / run 36453793875 / `8156ae1`; o log da `main` apaga o rascunho 398419349; assets = artefatos da `main` + `SHA256SUMS`, byte a byte; o corpo COMEÇA pela seção `[Unreleased]` inteira; o log do ensaio tem `ID: 398419349` | CONTEXT | Auto | PASS | `OK`; 9/9 mutantes reprovam, 3 controles passam |
| 7 | Única tag `v*` = `v0.1.0 → MAIN_SHA` | CONTEXT | Auto | PASS | `OK`; vale até o próximo merge que peça versão (W-11) |
| 8 | Validação de entrada do `lancar.yml` | CONTEXT | Auto | PASS | `OK` |
| 9 | Nenhum `TODO`/`FIXME` sem issue | CONTEXT | Auto | PASS | `OK` |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386/0/9 |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 82.93% |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `CHANGELOG.md:8` só tem `## [Unreleased]`; a `v0.1.0` foi publicada, então o heading `## [0.1.0]` é devido e entra no PR de ship |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Install` (`README.md:7`) com `sha256sum -c --ignore-missing SHA256SUMS` (`:21`) e "The installers are not signed" (`:22`); "What publishes" (`:498`) e "Do not rehearse near a merge" (`:499`) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod release-packaging` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Código, template e DoD aprovados. Falta confirmar os 2 itens manuais.

Antes do próximo merge na `main` que peça versão, fazer o ship. Depois:
- mergear o gw#13;
- rebasear e mergear o gw#14;
- abrir um PR só com a troca por `@main`, o ship do `.jdi/` e o heading `## [0.1.0]` (vira a `v0.1.1`).

Não fazer:
- apagar ou mover a tag ou a release;
- recriar rascunho;
- empurrar `ensaio-release/*`;
- editar os primeiros 18.838 bytes do corpo da release.

## DoD Critic (enhanced)

Iteração 5, forçada pelo `/jdi-issue`. O crítico reavaliou as 12 linhas Auto do zero e não achou nenhuma oca:
- a linha 6 reescrita recusa 8 mutantes, com o controle real OK;
- as linhas 1 e 7 emendadas discriminam;
- as linhas 8 e 3 continuam firmes.

**Verdict:** APPROVED
