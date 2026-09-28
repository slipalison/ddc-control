# Phase 6: Review  (slug: ci-crossbuild)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 1. Escrito pelo orquestrador a partir do resultado integral do reviewer: o harness nega escrita de `.md` ao subagente.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Build nativo Linux (`cargo build --workspace --locked`) OK. Cross-check de `ddc-core`/`ddc-adapters`/`ddc-cli` para `x86_64-pc-windows-msvc` e `x86_64-unknown-linux-gnu` OK. O `ddc-tray` no Windows só é provado pelo CI (run 36343108684: clippy, testes e `Build ddc-tray (release)` em success), porque não há `llvm-rc` local (D-2026-09-26-tray-app-9). |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), 15 binários. É o mesmo total do `rust-linux` do CI. Nenhum teste Rust mudou nesta phase. |
| Coverage | PASS | 83.36% de linhas (linha TOTAL, `main.rs`/`build.rs` excluídos), piso de 80%. Sem exclusão, como no Verify literal do PROJECT e no CI: 82.93%. |
| Lint | PASS | `cargo fmt --all --check` OK. `cargo clippy --workspace --all-targets --locked -- -D warnings` OK, também com `--all-features`, como o CI roda. Nenhum `#[allow(` novo. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.9 sem achado novo: nenhum `.rs` de `crates/` mudou, e o único `.rs` tocado, `apps/ddc-tray/src-tauri/build.rs`, não tem unwrap/expect/unsafe nem `#[cfg]` de host. Os 10 hits de 5.6 e os de 5.7c (`real_monitor.rs`, todos `#[ignore]` + `DDC_HW_TESTS`) já existiam e esta phase não os tocou. 5.10: `cargo audit` 0.22.2 sai 0, com as 4 exceções justificadas e conferidas. 5.11 limpo. |
| Consistency | PASS | D-1, D-2 e D-2026-09-27-ci-crossbuild-1..8 conformes (ver Notas). Todo arquivo do PLAN aparece nos commits; escopo `ci-crossbuild` e tipos coerentes; D-XX no corpo dos commits de código. |
| UI Validation | PASS | Suíte Playwright do `apps/ddc-tray`: 138 passed, 6 skipped (screenshots opt-in), 0 failed/flaky. Erros de console: 0. Violações axe critical/serious: 0 (asseridas em todo cenário). `node --test`: 161/161. |
| DoD | PASS_PENDING_MANUAL | 11/11 auto (8 CONTEXT + 3 PROJECT), 2 manuais pendentes |

## Blockers
- nenhum

## Warnings
- **W-1 (Gate 8, higiene do DoD) — CORRIGIDO pelo orquestrador após a revisão:** `.jdi/phases/ci-crossbuild/CONTEXT.md:57-64` tem um resíduo corrompido dentro de `## Definition of Done`.
  - O resíduo é um heading `## Deferred to PR review\`; assume os dois repositórios clonados lado a lado, como estão hoje)`, um `**Source:** CONTEXT` órfão e um `### Manual` duplicado. Veio assim desde o commit 3621de9.
  - Os 8 itens têm `Verify:` e foram parseados, então o gate não é INCONCLUSIVE.
  - Mesmo assim, um parser menos tolerante veria dois `## Deferred to PR review` e um item sem critério.
  - Limpar num commit só de `.jdi/` antes do ship.
- **W-2 (github-workflows, `qualidade.yml:31-32`, `82`, `121-124`):** `so: windows-*` é aceito para qualquer `linguagem`, mas só `rust` foi medido no Windows.
  - dotnet, python, node e go nunca rodaram lá com `shell: bash` + `preparar`. Por exemplo, o `go test -race` exige cgo/gcc no Windows.
  - Sugestão: no `Conferir componente`, reprovar `windows-*` fora de `rust`, ou documentar "medido só com rust".
  - Não afeta retrocompatibilidade nem segurança.
- **W-3 (github-workflows, já existia antes do PR, fora do diff, segurança):** o `qualidade.yml` do branch ainda interpola valor do chamador direto em `run:`.
  - Linhas afetadas: `matrix.c.projeto` em 174, 180, 185, 251-252, 264 e 270; `go run …gocover-cobertura@latest` sem versão fixa em 274.
  - Isso contradiz a própria tabela "Segurança" do README de lá ("Valores dinâmicos por `env`, nunca por interpolação dentro de `run:`").
  - O PR #13 não introduziu nem piorou nada disso, e o JSON é estático no workflow do chamador, o que limita a exposição.
  - Registrar como issue/todo no github-workflows. O `Conferir componente` é o lugar natural para validar `projeto` também.
- **W-4 (github-workflows, texto):** o corpo do commit `2335b52` descreve "um passo so do Windows poe no PATH um python3", que o `afc20cc` removeu.
  - No squash merge (hábito do mantenedor), editar a mensagem para tirar essa frase.
  - O corpo do PR #13 só cita o run 36341518707 (head `2335b52`). Vale acrescentar os runs 36342567853 e 36343108684, feitos no head final `afc20cc`.
- **W-5 (github-workflows, `qualidade.yml:391`, cosmético):** o passo `Build ${{ matrix.c.build_release }} (release)` aparece como `Build  (release)` (skipped) em todo componente sem o campo. Visto no `node-ui` do run 36343108684. Afeta só a tela do run.

## Notas de revisão

### Conformidade D-XX
- **D-1 / D-2 (projeto):** nada em `crates/` mudou; `ddc-core` intocado.
- **ci-crossbuild-1:** conforme.
  - Gatilhos `push` main, `pull_request` e `workflow_dispatch`.
  - `concurrency` `${{ github.workflow }}-${{ github.ref }}` com `cancel-in-progress`.
  - `permissions: contents: read` no topo e em cada job.
  - Jobs `versao` e `qualidade`, com os 2 `uses:` em `@afc20cc9f5457d045640df903e0ce9a7df1fb515` (40 caracteres).
  - PR #10 em rascunho.
  - `ci-evidence.env` com todas as chaves.
- **ci-crossbuild-2:** conforme.
  - Campos `so`, `pacotes_sistema` (regex idêntica à da decisão, passada por `env`, só Linux), `auditoria` (passo `cargo audit`, binário fixo 0.22.2, sem `cargo install`/`latest`/`|| true`/`--ignore`) e `build_release` (passo `Build <pacote> (release)`, `--locked`).
  - Só arquivo de workflow mudou (`qualidade.yml`, README, `exemplos/`); composites e `bin/` intocados.
  - Nota: a decisão cita `.zip` para o Windows. A implementação usa o `.tar.gz` que a v0.9.1 também publica, com sha256 conferido. É mais forte que "quando o projeto publica o hash", e o PLAN T-4 já registrava o ajuste. Não é violação.
- **ci-crossbuild-3:** conforme.
  - Nomes `rust-linux`/`rust-windows`/`node-ui`; `cobertura` 80/0; sem auditoria no Windows.
  - apt reduzido ao mínimo medido (`libwebkit2gtk-4.1-dev libudev-dev`), com o porquê em comentário. A decisão permite "ajustado ao que o run medir".
  - `npm test` = `node --test` TAP + Playwright. `--with-deps` só com `GITHUB_ACTIONS=true` e `RUNNER_ENVIRONMENT=github-hosted`, fixado por 4 testes.
- **ci-crossbuild-4:** conforme. `Build ddc-tray (release)` em success nos dois SOs.
- **ci-crossbuild-5:** conforme.
  - `build.rs:330-337` e `347-368` usam `new_without_app_manifest()`, `/MANIFEST:EMBED` e `/MANIFESTINPUT:`, decidindo por `CARGO_CFG_TARGET_OS`/`_ENV` lidos com `env::var`.
  - `windows-app-manifest.xml` é idêntico ao `tauri-build-2.7.0/src/windows-app-manifest.xml` (diff vazio).
  - No Linux nenhum `rustc-link-arg` é emitido: 0 linhas nos `output` de todos os build scripts do `ddc-tray` em `target/`.
- **ci-crossbuild-6:** conforme. Exatamente 4 `ignore`, só `[advisories].ignore`, e as justificativas foram reconferidas:
  - `cargo tree -i` bate com os caminhos citados.
  - O `ddc-hi` só chama `Database::from_version` (`ddc-hi-0.4.1/src/lib.rs:168,249`), que lê o YAML embutido por `include_bytes!` (`mccs-db-0.1.3/src/lib.rs:302-303`). Nenhum uso de `from_database` no repo.
  - O `glib` não aparece no grafo do alvo Windows.
- **ci-crossbuild-7:** conforme. (a) O run NEG 36342845774 falhou em `cargo audit` e `Cobertura`; a `Cobertura` falha porque os testes foram pulados, como esperado. (b) Worktree (DoD 8).
- **ci-crossbuild-8:** conforme. `versao` só roda "Quais commits…" e "Calcular", sem tag/release. As seções novas existem nos dois READMEs; a revisão do texto é humana (Deferred).

### Revisão de github-workflows#13 (head `afc20cc`, diff `origin/main...qualidade-rust-windows`)
- **pt-BR sem acento:** nenhuma letra acentuada nas linhas adicionadas de `.yml` nem nas mensagens de commit. O único não-ASCII é o travessão, estilo que já existia. O README mantém acentos, como o resto daquele README. `bin/versao.py conferir` aprova os 2 commits.
- **SHA pins:** nenhum `uses:` novo; `actions.lock.json` intocado. `python3 bin/pinar_actions.py --verificar` diz "Todos os `uses:` de 15 arquivo(s) estao fixados pelo lock."
- **`env` × interpolação:** todo `run:` novo lê só `env` (`SO`, `LINGUAGEM`, `PACOTES`, `AUDITORIA`, `RELEASE`, `PACOTE_RELEASE`, `VERSAO_*`, `SHA256_*`). O único `${{ }}` novo fora de `env`/`if` está num `name:` (linha 391), que nunca executa. As mensagens `::error::` não ecoam o valor do chamador, então não há injeção de workflow command.
- **Validação do JSON (`Conferir componente`, 102-150):**
  - `so` validado por `case`.
  - Lista apt validada por `[[ =~ ]]`, seguro contra quebra de linha; nomes sem `-` inicial, então não viram opção do apt.
  - `auditoria` lida por `toJSON`: `"true"` com aspas reprova.
  - `build_release` validado por regex e só aceito em rust.
  - `runs-on` cai no `ubuntu-latest` com `so` inválido, para o passo poder reprovar em vez de o job ficar na fila.
- **Retrocompatibilidade:** sem os campos novos, o job roda os mesmos passos e comandos no `ubuntu-latest`.
  - O `shell: bash` explícito só acrescenta `pipefail`. Conferi que nenhum `run:` com pipe dependia dele sem `set -euo pipefail`.
  - O `Conferir componente` passa com valores vazios.
  - O `cargo-llvm-cov` continua 0.9.1, no mesmo destino no ubuntu, agora com hash conferido.
- **Downloads:** os 3 sha256 fixados batem com o `digest` dos assets da release, consultado agora por `gh api`:
  - `cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz` = `b3f68e62…c41a`
  - `…-x86_64-pc-windows-msvc.tar.gz` = `15eefd5b…4948`
  - `cargo-audit-x86_64-unknown-linux-gnu-v0.22.2.tgz` = `ab28a1bd…0e39`
  - O `sha256sum -c` roda antes de extrair, e o log do CI mostra `: OK` nos dois SOs. `curl --proto '=https' --tlsv1.2 -fsSL`, com guarda de `RUNNER_ARCH`.
- **`|| true`:** nenhum novo. O único é o do `pip install` (linha 202), que já existia.
- **Permissões:** inalteradas (`contents: read` no workflow e no job); nenhum secret novo.
- **Sintaxe:** actionlint 1.7.12 limpo local em `.github/workflows/*.yml exemplos/*.yml` e no `ci.yml` do ddc-control (sem shellcheck local). O CI de lá passou no head (DoD 2).

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run de evidência em success no `HEAD_SHA`; commit ancestral do HEAD com a mesma árvore de código; `uses:` do gw em `@WORKFLOWS_SHA`; `ci.yml` do HEAD igual, a menos da referência | CONTEXT | Auto | PASS | `OK`. Run 36343108684 = `success 44fab10…`; depois dele só `.jdi/` mudou; 2 `uses:` em `@afc20cc…` |
| 2 | Head do PR gw#13 = `WORKFLOWS_SHA`; `Scripts`, `Pins das actions` e `Sintaxe dos workflows` em success, sem check falho ou cancelado | CONTEXT | Auto | PASS | `OK`. headRefOid `afc20cc9…`; 3/3 success; nenhum failure/cancelled/timed_out |
| 3 | Um `rust-linux` (`ubuntu-latest`) e um `rust-windows` (`windows-latest`), com os passos do contrato em success | CONTEXT | Auto | PASS | `OK`. Linux 6/6 (incluindo `cargo audit` e `Build ddc-tray (release)`); Windows 5/5 |
| 4 | Log do `rust-linux`: teste da CSP ok, `test result: ok`, nenhum binário falho, painel ≥ 80 | CONTEXT | Auto | PASS | `OK`. `**Aprovado:** 82.93% >= piso de 80%.` |
| 5 | Log do `rust-windows`: teste da CSP ok, sem falha, sem `STATUS_ENTRYPOINT_NOT_FOUND`, mesmo número de binários do Linux | CONTEXT | Auto | PASS | `OK`. 11 = 11 binários com `test result: ok.` |
| 6 | `node-ui` e `npm test` em success; TAP `# pass` ≥ 150 e `# fail 0`; Playwright ≥ 130 passed, nenhum failed | CONTEXT | Auto | PASS | `OK`. `# pass 161`, `# fail 0`, `138 passed`, 6 skipped |
| 7 | Portão de auditoria morde no CI real: `NEG_SHA` só remove linhas de `audit.toml`; run falha em `cargo audit` com o advisory no log | CONTEXT | Auto | PASS | `OK`. NEG `75c0b8e`, run 36342845774 `failure`, `cargo audit` = failure, RUSTSEC-2018-0005 no log; `audit.toml` do HEAD = estado anterior ao NEG |
| 8 | `audit.toml` com exatamente as 4 exceções; `cargo audit` sai 0 no HEAD; no worktree sem 2018-0005 sai ≠ 0 | CONTEXT | Auto | PASS | `OK`. Worktree criado e removido (`git worktree list` só lista o principal) |
| 9 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | Gate 2: exit 0, 386 passed / 0 failed |
| 10 | Coverage >= 80% of lines | PROJECT | Auto | PASS | Literal sem exclusão, reaproveitando o profile do Gate 3 via `cargo llvm-cov report --summary-only`: 82.93%. Gate 3: 83.36% |
| 11 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` (comando literal) |
| 12 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou 3 entradas em Added (CI, política de audit, `npm test`) e 1 em Fixed (tauri#13419). Sem heading de versão nova: não houve release nesta phase (D-8) |
| 13 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção nova `## CI` (README.md:468); Status, Known limitations ("Windows is built and unit-tested, not run") e Dev setup (`cargo install cargo-audit --locked`) atualizados |

**Totals:** 13 items | Auto: 11 (11 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod ci-crossbuild` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Nada bloqueia. A evidência é comportamental: os 8 `Verify:` consultam runs reais e deram `OK` literalmente agora. Os gates locais também passaram: build, 386 testes, 83.36% de cobertura, fmt/clippy, audit, Playwright 138 e node 161.

1. Confirmar os 2 manuais (CHANGELOG/README) com `/jdi-confirm-dod ci-crossbuild`, ou deixá-los para o PR, conforme a cadeia autônoma.
2. Antes do ship, remover o resíduo do `CONTEXT.md:57-64` (W-1) num commit só de `.jdi/`.
3. Seguir a ordem do Deferred:
   - fazer o merge do github-workflows#13, editando a mensagem do squash (W-4);
   - trocar os dois `@afc20cc…` por `@main` no `.github/workflows/ci.yml` e esperar um run verde;
   - tirar o #10 do rascunho e atualizar o corpo, que ainda diz "Draft: the CI is still converging".
4. A branch está 1 commit atrás de `origin/main` (`e72790d`, só `.jdi/`); o PR #10 aparece como MERGEABLE.
5. Follow-ups no github-workflows, fora desta phase: W-2 (restringir ou documentar `so: windows-*` além de rust), W-3 (issue de segurança sobre interpolação de `projeto` em `run:` e `gocover-cobertura@latest`) e W-5 (nome do passo pulado).

## DoD Critic (enhanced)

- DoD row «2»: o `Verify:` conferia só o head do PR #13 e os checks dele. O template no SHA fixado chama as composites por `@main` (`preparar`, `relatar-cobertura`, `versao`), e o log de cada job mostra `Download action repository 'slipalison/github-workflows@main' (SHA:eb9c69a…)`. Se o PR mudasse uma composite ou `bin/`, o run executaria a versão da `main` e a linha ainda daria OK.
- DoD row «11» (PROJECT, TODO): o `Verify:` do PROJECT só varre `*.rs`, e 8 dos 9 arquivos entregues pela phase não são `.rs`. Reproduzido: um `# TODO:` no `ci.yml` e outro no `playwright-browsers.mjs` passaram. A `tray-app` tinha a linha equivalente para todo arquivo do produto; esta phase não tinha.
- DoD row «5» (suspeita, não objetiva): a linha compara a QUANTIDADE de binários, e não os `ignored`. Um `#[cfg_attr(windows, ignore)]` apressado manteria 11 = 11.
- DoD row «1» (lacuna, não oca): a árvore comparada não inclui o `clippy.toml` da raiz, que o clippy do CI lê.

**Verdict:** BLOCKED

> Correção do orquestrador (verify reset iter 1 → 2, CONTEXT):
> - a linha 2 passa a exigir que `referenced_workflows` de `RUN_ID` e `NEG_RUN_ID` estejam todos em `WORKFLOWS_SHA`, e que o SHA das composites baixadas por cada job não difira de `WORKFLOWS_SHA` em `.github/actions/` e `bin/`;
> - a linha 5 passa a comparar o total de `ignored` (9 = 9 hoje);
> - a linha 1 inclui `clippy.toml`, `rustfmt.toml` e o toolchain;
> - entra a linha 9, a checagem de TODO em todo arquivo versionado, a mesma com que a `tray-app` foi aprovada.
>
> As quatro dão `OK` com a evidência atual.
