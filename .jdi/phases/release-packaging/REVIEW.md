# Phase 7: Review  (slug: release-packaging)

**Verdict:** BLOCKED

> Loop iter 3. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
>
> **Prova da herança (Gates 1–7 e revisão do template herdam da iteração 2)**
> - `git diff --quiet 45c8433 HEAD -- . ':!.jdi'` sai 0.
> - O head do gw#14 é `d6d340a`, e os checks de lá estão em success.
> - O `release-evidence.env` está igual ao da iteração 2 (sha256 `dfe01e30…`).
>
> **Fato novo durante a revisão:** o humano mergeou o #11 às 16:49:19Z (squash → `8156ae1`). O push na `main` rodou o run 36453793875, todo em success. O `lancar` desse run (egresso `block`) apagou o rascunho 398419349 às 17:04:55.68Z e publicou a release REAL **398454265** às 17:05:00Z: `v0.1.0` → `8156ae1`, 8 anexos. A árvore do `8156ae1` é igual à do `HEAD_SHA` fora de `.jdi/`.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Herdado (prova acima). |
| Tests | PASS | Re-executado: 386 passed, 0 failed, 9 ignored. |
| Coverage | PASS | Re-executado: 83.36% com exclusão; 82.93% literal. |
| Lint | PASS | Herdado. |
| Hexagonal/Safety/Hygiene | PASS | Herdado. 5.10 re-executado: `cargo audit` sem vulnerabilidade. |
| Consistency | PASS | Herdado. Os commits da iteração 3 só tocam `.jdi/`. |
| Segurança do template (gw#14) | PASS | Herdado. Nenhum `run:` do `lancar.yml@d6d340a` contém `${{`. |
| UI Validation | PASS | Herdado. |
| DoD | BLOCK | Os 9 do CONTEXT deram `OK` às 17:04–17:06Z. No re-run das 17:11Z, as linhas 6 e 7 saíram com exit 1 por causa da publicação real. |

## Blockers
- **B-1 — linha 7 ("Nenhuma tag `v*`").**
  - Agora existe `refs/tags/v0.1.0 → 8156ae1`, criada pelo push do merge. Não é defeito de código.
  - **Proibido "corrigir" apagando a tag ou a release.**
- **B-2 — linha 6 (rascunho `DRAFT_RELEASE_ID`).** O rascunho 398419349 dá 404, porque foi apagado pelo `lancar` da `main`, como a D-2 manda. A mesma checagem contra a release real 398454265 / run 36453793875 / `8156ae1` dá `OK`.

## Warnings
- **W-7 residual (texto):** agora está na release PÚBLICA 398454265. O corpo (20.159 caracteres) mistura o CHANGELOG em inglês com os títulos em pt-BR das notas do `versao`. Revisar o corpo já é possível, e editar não mexe em tag nem em anexo.
- **W-8 (herdado, fora do diff):** continua aberto.
- **W-9 (baixo):** ferramentas fixadas por hash em tags que o dono reescreve. Falha fechado.
- **W-10 (cosmético):** comentário de 133 caracteres em `qualidade.yml:912`.
- **W-11 (novo, operacional, prioridade alta): ordem do merge invertida.**
  - A `main` do ddc-control roda com `uses: …@d6d340a…`, que é head de PR não mergeado no gw.
  - Ordem: mergear o gw#13; rebasear e mergear o gw#14; logo depois, o PR que troca `@d6d340a…` por `@main`.
  - Esse PR publica a `v0.1.1`. Juntá-lo ao PR de ship do `.jdi/` economiza uma release.
- **W-12 (novo, processo):** os commits `.jdi/` da iteração 3 não estão na `main`. Um rebase da branch sobre a `main` quebraria a linha 1, que usava ancestralidade.

## Notas
- **Linha 8 reescrita — não oca, discrimina.**
  - O modo do runner é equivalente; 15 valores foram classificados igual nos dois modos.
  - 17 mutantes reprovam: `${{` no `run`, `::error::` removido, regex frouxa, caminho absoluto aceito, `if`/`continue-on-error` no passo e no job, passo depois do `gh release create`, env trocado ou ausente, passo duplicado etc.
- **Linha 3 reescrita — não oca, discrimina.**
  - O `0.1.0` dos pacotes só pode vir do `Carimbar versao`: a única fonte de versão é o workspace em `0.0.0`, e nenhum crate fixa versão própria.
  - 4 mutantes e o `07323e5` (com `0.1.0`) reprovam.
- **Linha do tempo (UTC).** Às 16:49:19, o merge. Às 17:04:17, a linha 7 dá OK. Às 17:04:55, o rascunho é apagado. Às 17:05:00, a release é publicada. Às 17:11, as linhas 6 e 7 dão exit 1.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run real de `pull_request` no `HEAD_SHA` (success, `rust-linux` em 22.04, `salto=inicial versao=0.1.0`, `lancar` skipped) | CONTEXT | Auto | PASS | `OK` (17:04:55Z). Ver W-12 |
| 2 | Template que rodou = PR #14 (`WORKFLOWS_SHA`) | CONTEXT | Auto | PASS | `OK` |
| 3 | `pacotes-rust-linux` inspecionados; carimbo provado (0.0.0, `tauri.conf.json` sem `version`, `version.workspace = true`) | CONTEXT | Auto | PASS | `OK`; as checagens novas discriminam |
| 4 | `pacotes-rust-windows` e os 5 passos | CONTEXT | Auto | PASS | `OK` |
| 5 | Ensaio `ENSAIO_RUN_ID` | CONTEXT | Auto | PASS | `OK` |
| 6 | Rascunho `DRAFT_RELEASE_ID` | CONTEXT | Auto | FAIL | `OK` às 17:04:17–17:04:54Z; exit 1 às 17:11:16Z (404, apagado pelo `lancar` da `main`) |
| 7 | Nenhuma tag `v*` | CONTEXT | Auto | FAIL | `OK` às 17:04:17Z; exit 1 às 17:11:15Z (`v0.1.0 → 8156ae1`, criada pelo merge) |
| 8 | Validação de entrada do `lancar.yml` (YAML parseado, sem `${{`, recusa pelo `::error::` do script) | CONTEXT | Auto | PASS | `OK`; 17/17 mutantes reprovam |
| 9 | Nenhum `TODO`/`FIXME` sem issue | CONTEXT | Auto | PASS | `OK` |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386/0/9 |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 82.93% literal |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só `## [Unreleased]`; a `v0.1.0` já foi publicada, então o heading `## [0.1.0]` já é devido |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: Install com `SHA256SUMS` e o aviso de instalador sem assinatura; "What publishes" e "Do not rehearse near a merge" |

**Totals:** 14 items | Auto: 12 (10 PASS, 2 FAIL) | Manual: 2 pending

## Recommendation
Código e template aprovados. O BLOCKED vem só das linhas 6 e 7, que o merge humano do #11 tornou impossíveis de passar, do jeito previsto. O doer não tem o que corrigir.

Emenda de DoD pós-merge para o orquestrador:
- `MAIN_SHA`, `MAIN_RUN_ID` e `RELEASE_ID` no `release-evidence.env`;
- a linha 6 aponta para a release real e liga o rascunho validado pelo log "Rascunho 398419349 de v0.1.0 apagado.";
- a linha 7 passa a exigir que a única tag `v*` seja `v0.1.0 → MAIN_SHA`;
- a linha 1 compara árvores em vez de ancestralidade.

Não fazer:
- apagar a tag ou a release;
- recriar um rascunho;
- empurrar `ensaio-release/*`.

> **Emenda aplicada pelo orquestrador (verify reset iter 3 → 4):** é exatamente a recomendada acima. A linha 1 passou a usar `MAIN_SHA` ancestral de `origin/main` e árvore do `MAIN_SHA` igual à do `HEAD_SHA` fora de `.jdi/`. As linhas 6 e 7 agora tratam da release real. Os 9 `Verify:` foram rodados literalmente em `bash`: todos `OK`.
