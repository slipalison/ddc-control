# Phase 7: Review  (slug: release-packaging)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 4. Escrito pelo orquestrador a partir do resultado integral do reviewer: o harness nega escrita de `.md` ao subagente.
>
> **Prova da herança (Gates 1–7 e revisão do template herdam da iteração 3, que herdou da 2)**
> - `git diff --quiet f2c54ad HEAD -- . ':!.jdi'` sai 0. `f6063d7 (HEAD_SHA)..HEAD` fora de `.jdi/` também sai 0.
> - Único commit novo: `b8f0265` (só `CONTEXT.md`, `REVIEW.md` e o `release-evidence.env`, que ganhou `MAIN_SHA`, `MAIN_RUN_ID` e `RELEASE_ID`).
> - O head do gw#14 continua `d6d340a`, com os checks em pass. O `origin/main` é `8156ae1` = `MAIN_SHA`.
> - Gate 8: os 9 `Verify:`, extraídos por programa, rodados literalmente com `bash --noprofile --norc` entre 17:20 e 17:22:36Z, e em seguida a baseline do PROJECT.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Herdado. |
| Tests | PASS | Re-executado: 386 passed, 0 failed, 9 ignored. |
| Coverage | PASS | Re-executado: 83.36% (Gate 3), 82.93% (literal). Piso de 80%. |
| Lint | PASS | Herdado. |
| Hexagonal/Safety/Hygiene | PASS | Herdado. 5.10 re-executado: `cargo audit` sem vulnerabilidade. |
| Consistency | PASS | `b8f0265`: escopo `release-packaging`, tipo `docs`, só `.jdi/`. D-8 respeitada: a tag nasceu do merge. |
| Segurança do template (gw#14) | PASS | Herdado; head `d6d340a` inalterado. |
| UI Validation | PASS | Herdado. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto PASS (9 CONTEXT + 3 PROJECT); 2 manuais pendentes |

## Blockers
- nenhum. Os B-1 e B-2 da iteração 3 foram resolvidos pela emenda pós-merge, e as linhas discriminam (ver Notas).

## Warnings
- **W-7 residual (texto):** o corpo da release pública 398454265 mistura o CHANGELOG em inglês com os títulos em pt-BR das notas do `versao`. Editar o corpo não mexe em tag nem em anexo. Mas a linha 6 exige os 42 itens do `[Unreleased]` no corpo, então uma edição que tire algum deles derruba a linha.
- **W-8 (herdado, fora do diff):** continua aberto.
- **W-9 (baixo):** ferramentas fixadas por hash em tags que o dono reescreve. Falha fechado.
- **W-10 (cosmético):** comentário de 133 caracteres em `qualidade.yml:912`.
- **W-11 (operacional, prioridade alta):** ordem do merge invertida.
  - A `main` roda com `uses: …@d6d340a…` (`ci.yml:42/54/130`), e os gw#13 e #14 continuam abertos.
  - Ordem: gw#13; depois rebasear e mergear o gw#14; depois o PR que troca por `@main`.
  - O próximo merge na `main` publica a `v0.1.1`, e a linha 7 ("única tag `v*` é `v0.1.0`") só vale até lá. Fazer o ship antes.
- **W-12 (processo, MITIGADO):** a linha 1 compara árvores e não quebra com rebase. Se o `HEAD_SHA` sumir localmente, `git fetch origin f6063d7…` o traz (o `refs/pull/11/head` o mantém).
- **W-13 (baixo, lacuna da emenda):** a linha 1 prova `MAIN_SHA ≡ HEAD_SHA` fora de `.jdi/`, mas não compara `HEAD_SHA` com a HEAD da branch. Hoje o fato vale (`git diff --quiet f6063d7 HEAD -- . ':!.jdi'` sai 0). Não bloqueia.

## Notas
- **Linha 1 emendada: não é oca, discrimina.** Base `OK`, e 5 mutantes reprovam:
  - `MAIN_SHA=07323e5`, que está na `main` mas tem outra árvore;
  - `MAIN_SHA=f6063d7`, fora da `main` por causa do squash;
  - `MAIN_SHA=b8f0265`;
  - `MAIN_SHA=45c8433`;
  - `HEAD_SHA=59d16ef`, com código diferente.
- **Linha 6 emendada: não é oca, discrimina.** Base `OK`, e 7 mutantes reprovam:
  - `MAIN_RUN_ID` = o run de PR;
  - `DRAFT_RELEASE_ID=398335754`, apagado pelo ensaio e não pela `main`;
  - `RELEASE_ID` = o rascunho apagado;
  - `MAIN_SHA=07323e5`;
  - `VERSAO=0.1.1`;
  - artefatos do ensaio no lugar dos da `main`, que falha no `cmp`;
  - um item falso no `Unreleased`.
  - O grep "Rascunho 398419349 de v0.1.0 apagado." casa com a saída real (log, linha 485), não com o eco do script.
- **Linha 7 emendada: não é oca, discrimina.** Base `OK`, e 3 mutantes reprovam. O `git ls-remote --tags origin` mostra UMA tag no repositório (`v0.1.0 → 8156ae1`, leve). A única release é a 398454265, de `github-actions[bot]`, com 8 anexos. Não sobrou rascunho.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run real de `pull_request` no `HEAD_SHA` (success, `rust-linux` em 22.04, `salto=inicial versao=0.1.0`, `lancar` skipped); `MAIN_SHA` ancestral de `origin/main` e com árvore igual à do `HEAD_SHA` fora de `.jdi/` | CONTEXT | Auto | PASS | `OK`; 5/5 mutantes reprovam |
| 2 | Template que rodou = PR #14 (`WORKFLOWS_SHA`) | CONTEXT | Auto | PASS | `OK`; head `d6d340a` |
| 3 | `pacotes-rust-linux` inspecionados; carimbo provado | CONTEXT | Auto | PASS | `OK` |
| 4 | `pacotes-rust-windows` e os 5 passos | CONTEXT | Auto | PASS | `OK` |
| 5 | Ensaio `ENSAIO_RUN_ID` | CONTEXT | Auto | PASS | `OK` |
| 6 | Release PUBLICADA 398454265 / run 36453793875 / `8156ae1`; o log liga o rascunho 398419349; assets = artefatos da `main` + `SHA256SUMS`, byte a byte; corpo com o `Unreleased` | CONTEXT | Auto | PASS | `OK`; 7/7 mutantes reprovam |
| 7 | Única tag `v*` = `v0.1.0 → MAIN_SHA` | CONTEXT | Auto | PASS | `OK`; 3/3 mutantes; `ls-remote` com 1 tag. Vale até o próximo merge na `main` (W-11) |
| 8 | Validação de entrada do `lancar.yml` | CONTEXT | Auto | PASS | `OK` |
| 9 | Nenhum `TODO`/`FIXME` sem issue | CONTEXT | Auto | PASS | `OK` |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386/0/9 |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 82.93% literal |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `CHANGELOG.md:8` só tem `## [Unreleased]`; a `v0.1.0` foi publicada, então o heading `## [0.1.0]` é devido e entra no PR de ship |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Install` (`README.md:7`) com `sha256sum -c --ignore-missing SHA256SUMS` (`:21`) e o aviso de instalador sem assinatura (`:22`); "What publishes" (`:498`) e "Do not rehearse near a merge" (`:499`) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod release-packaging` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Código, template e DoD aprovados. As 3 linhas emendadas discriminam (15/15 mutantes reprovam). Falta a confirmação dos 2 manuais.

Antes do próximo merge na `main`, registrar o verify/ship desta phase (W-11/linha 7). Também antes:
- mergear o gw#13;
- rebasear e mergear o gw#14;
- abrir o PR que troca por `@main`, junto com o ship do `.jdi/` e o heading `## [0.1.0]` do CHANGELOG, para gastar uma release só (`v0.1.1`).

Não fazer:
- apagar ou mover a tag `v0.1.0` ou a release;
- recriar rascunho;
- empurrar `ensaio-release/*`;
- tirar do corpo da release os itens do `Unreleased`.

## DoD Critic (enhanced)

- DoD row «6»: o corpo não era provado inteiro. O `grep -E '^- '` só conferia os 42 itens de nível 0 do `[Unreleased]` e ignorava os 25 subitens aninhados e os títulos `###`.
  - Mutante realista demonstrado: `Montar as notas` com `dentro && /^- /`, que gera um corpo de 11.668 caracteres sem subitens.
  - Com esse corpo servido por um shim de `gh`, a linha imprimia OK. O controle, sem 1 item de nível 0, reprovava, o que mostra que o shim estava em uso.
- Suspeita (não objetiva): depois da emenda, nenhuma linha ligava o `DRAFT_RELEASE_ID` ao `ENSAIO_RUN_ID`.
- As outras 11 linhas Auto foram julgadas não ocas, e as emendadas 1 e 7 discriminam.

**Verdict:** BLOCKED

> Correção do orquestrador (verify reset iter 4 → 5, CONTEXT linha 6):
> - o corpo tem de COMEÇAR pela seção `## [Unreleased]` inteira do CHANGELOG do `MAIN_SHA`, byte a byte, como o `awk` do template a extrai. Isso cobre títulos, itens e subitens, e a seção precisa ter pelo menos 10 linhas;
> - o log do `lancar / Tag e release` do `ENSAIO_RUN_ID` precisa ter `ID: $DRAFT_RELEASE_ID`.
>
> Resultado: a linha dá `OK` contra a release real, e o mesmo corpo sem os subitens aninhados reprova (rc=1).
