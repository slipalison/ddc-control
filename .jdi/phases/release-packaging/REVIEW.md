# Phase 7: Review  (slug: release-packaging)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 1. Escrito pelo orquestrador a partir do resultado integral do reviewer: o harness nega escrita de `.md` ao subagente.
>
> O HEAD local é `6c5d69b`, e o `HEAD_SHA` da evidência é `299c3b8`. Entre os dois só mudou `.jdi/` (`git diff --quiet 299c3b8 HEAD -- . ':!.jdi'` sai 0). No github-workflows, o PR #14 (`pacotes-tauri-release`) está aberto, com head `3688e06` = `WORKFLOWS_SHA`, empilhado sobre o #13 (`9e91d1a`), também aberto. Os gates rodaram em `bash`, no Fedora (rustc 1.98.1).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0. `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked` sai 0 para `x86_64-unknown-linux-gnu` e para `x86_64-pc-windows-msvc`. O `ddc-tray` no Windows só é provado pelo CI: no `RUN_ID` 36435173929, `qualidade / rust-windows` deu success, com `Build ddc-tray (release)` e os pacotes. |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 binários. |
| Coverage | PASS | 83.36% de linhas (`main.rs`/`build.rs` excluídos); piso 80. Sem exclusão (Verify literal do PROJECT): 82.93%. A phase não mexeu em nenhum `.rs`. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0, também com `--all-features`. Nenhum `#[allow(` fora de testes. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.4 e 5.8 sem saída. 5.5a e 5.6 só com hits que já existiam. 5.7: `Confirm::Yes` só nas bordas, e os testes de hardware têm `#[ignore]` e `DDC_HW_TESTS`. 5.9 limpo. 5.10: `cargo audit` 0.22.2 sai 0 (1273 advisories); o `Cargo.lock` só mudou nas 4 linhas `version` dos membros. 5.11 limpo. |
| Consistency | WARN | Os 11 arquivos de código do plano aparecem nos commits, com escopos e tipos certos. D-1, D-2 e D-2026-09-28-release-packaging-1..10 conformes, contando as emendas. Desvios registrados no PLAN (A-4/-5/-7) e no SUMMARY. W-1 e W-3 ficam com a D-2. |
| Segurança do template (gw#14) | WARN | `pinar_actions.py --verificar` OK, e o `actions.lock.json` não mudou. actionlint 1.7.12 com shellcheck limpo. Nenhum `${{ }}` novo em `run:`. Sem acento. `tauri-cli` e `cargo-audit` musl batem com o `digest` das releases. W-1..W-5 abaixo. |
| UI Validation | PASS | `npm ci --ignore-scripts` + `npx playwright test`: 138 passed, 6 skipped, 0 failed. `node --test`: 161/161, `# fail 0`. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto (9 CONTEXT + 3 PROJECT), literais em `bash`; 2 manuais pendentes |

## Blockers
- nenhum

## Warnings
- **W-1 (segurança, gw `lancar.yml:414-415`, D-2): `Mover a tag de major` ignora o `rascunho`.**
  - O passo tem só `if: inputs.tag_movel_major`. Com `tag_movel_major: true` e `rascunho: true`, um ENSAIO cria ou move por força a tag real `vN` para um commit fora da main.
  - Isso contradiz a descrição do input (`lancar.yml:91`), o README e a D-2 ("nenhuma tag é criada").
  - O próprio gw chama o `lancar` com `tag_movel_major: true` (`ci.yml:226` do gw).
  - Não atinge o ddc-control.
  - Corrigir antes do merge do gw#14: `if: inputs.tag_movel_major && !inputs.rascunho`, ou reprovar a combinação em `Conferir as entradas`.
- **W-2 (segurança e cadeia de suprimento, gw `qualidade.yml:759`): o AppImage publicado leva binários baixados sem hash.**
  - O `cargo tauri build` do tauri-cli 2.12 baixa no job, sem conferir hash, o `AppRun-x86_64` (`apprun-old`), o `linuxdeploy-07333c6` e o `linuxdeploy-plugin-appimage` da release `continuous`, que muda com o tempo (job 108971248763, linhas ~2102-2104; `prepare_tools` do tauri-bundler).
  - O `AppRun` vira o ponto de entrada do `.AppImage` publicado.
  - Sugestão: pré-semear essas ferramentas com sha256 conferido no diretório de ferramentas do tauri, porque o bundler pula o download quando o arquivo existe. No mínimo, declarar a exceção na seção "Segurança" do README do gw.
- **W-3 (gw `lancar.yml:237-238` e `qualidade.yml:215-219`, D-2): `changelog` e `caminho_tauri` aceitam caminho absoluto.**
  - `CHANGELOG=/etc/passwd` passa em `Conferir as entradas` (script extraído do `3688e06`), embora o comentário e o README digam "relativo".
  - O trecho `## [Unreleased]` de qualquer arquivo do runner iria para o corpo público da release.
  - Recusar `/` no começo.
- **W-4 (gw README linha 201 e comentário `lancar.yml:193-208`): o egresso efetivo é maior que os "quatro destinos".**
  - O harden-runner 2.21.1 em `block` libera os domínios meta do GitHub: `github.com`, `*.github.com`, `*.githubapp.com`, `ghcr.io` e 20 contas `productionresultssa*`.
  - A lista explícita está certa. O texto de segurança precisa dizer isso.
- **W-5 (gw `lancar.yml:357-378`, baixo): corrida entre ensaio e main.**
  - Um ensaio da mesma versão rodando junto com o `lancar` da main apagaria, no `Apagar rascunho velho`, o rascunho em trânsito que o `gh release create` cria enquanto sobe os anexos.
  - A release da main falharia sem publicar, então não há perda de segurança.
  - Evitar `ensaio-release/*` perto do merge, ou filtrar também por `target_commitish`.
- **W-6 (docs do gw):**
  - `qualidade.yml:72` e o README (linha 751) dizem que a única diferença é a chave do cache, mas o `Alvo do runner` agora roda em todo componente rust.
  - Em `exemplos/ci-rust-desktop.yml:32-44`, os parágrafos "A RELEASE…" e "A VERSAO…" partem a lista "O QUE CADA COMPONENTE PROVA".
- **W-7 (texto, Deferred):**
  - O corpo do rascunho mistura o CHANGELOG em inglês com os títulos em pt-BR das notas do `versao`.
  - Sem tag anterior, as notas listam o histórico inteiro.
  - O README do ddc-control diz "Every push to `main` publishes a GitHub Release", mas só publica com `salto != nenhum`.
- **W-8 (herdado da ci-crossbuild, continua aberto):** `${{ }}` em `run:` nos passos dotnet/npm/go e `gocover-cobertura@latest` no `qualidade.yml` do gw (W-3 da ci-crossbuild); `so` aceita qualquer sufixo `ubuntu-*`/`windows-*` (W-6 da ci-crossbuild). Nenhum atinge o ddc-control.

## Notas

### Pontos de segurança pedidos (conferidos)
- **Lista `block` do `lancar`:** são 4 destinos, com comentário fora do bloco `|`. O ensaio final rodou em `block` sem bloqueio, e todos os `endpoint called` estão liberados. Ver W-4.
- **Apagar rascunho velho:**
  - filtra `select(.draft == true and .tag_name == "$TAG")` e apaga pelo id;
  - a `TAG` já passou pela regex `v[0-9]+.[0-9]+.[0-9]+`;
  - uma release publicada nunca casa, e `A tag ainda nao existe?` reprova antes;
  - risco residual: W-5.
- **Notas do CHANGELOG:** o `awk` lê o arquivo, e o título entra por `-v` só com dígitos. O trecho passa pelo `printf` builtin até `$RUNNER_TEMP/notas-release.md`, e o `gh` recebe só `--notes-file`. Nada de texto de terceiro chega ao argv de processo externo.
- **Anexos:**
  - nomes restritos a `[A-Za-z0-9._+-]`, sem subdiretório e sem `SHA256SUMS` duplicado;
  - `sha256sum --` e o prefixo `pacotes-baixados/`;
  - o download vem antes da criação (A-5).
- **Gatilho `ensaio-release/**`:**
  - o `lancar` exige `push` e ref `main` ou `ensaio-release/`, com `rascunho: ${{ github.ref != 'refs/heads/main' }}`;
  - PR e `workflow_dispatch` nunca chegam ao `lancar`;
  - push de tag não dispara;
  - o `download-artifact` sem `run-id` só vê o próprio run.
- **`postinstall.sh` (root):**
  - só `modprobe i2c-dev`, `udevadm control --reload` e `udevadm trigger --subsystem-match=i2c-dev`, cada um com `2>/dev/null || true`, e `exit 0`;
  - `sh -n` e shellcheck limpos;
  - o `postinst` do deb e o `%post` do rpm são idênticos ao arquivo.
- **Endurecimento sugerido (emenda da D-7):** `artefatos: "pacotes-*"` casaria um `pacotes-*` subido por qualquer job do run, como o `node-ui` com código npm de terceiros. Usar `pacotes-rust-*`.

### Rascunho 398335754
- `draft: true`, `tag_name: v0.1.0`, alvo `299c3b8…`, autor `github-actions[bot]`, `published_at: null`.
- É a única release do repositório, que tem 0 tags.
- 8 anexos. O `digest` da API bate com o `SHA256SUMS`.
- Deb: `Version: 0.1.0`, `Maintainer: Alison Amorim`, `Depends: libwebkit2gtk-4.1-0, libgtk-3-0`, `Exec=ddc-tray`, GLIBC máx. 2.34.
- Rpm: lista `/usr/bin/ddc-tray`, a regra udev, o `modules-load.d` e o `.desktop`.
- Corpo de 20.394 caracteres (W-7).

### Conformidade D-XX (resumo)
- **D-1:** branch sobre `9e91d1a`; 3 `uses:` em `@3688e06…`; `release-evidence.env` com as 10 chaves.
- **D-2:** atendida pela A-5 e com o egresso medido; ressalvas W-1 e W-3.
- **D-3:** atendida. Desvios documentados: `dist/*`, chave do cache com o runner, 7z no Windows.
- **D-4:** regra byte a byte igual, `modules-load.d`, `postinst` best-effort, `/usr/bin/ddc-tray`, `mainBinaryName`, limite do AppImage documentado.
- **D-5/-6:** conjuntos de membros e as 7 categorias.
- **D-7:** literal; `qualidade` com `needs: [versao]`.
- **D-8:** PR com `lancar` skipped, ensaio verde, rascunho mantido, 0 tags, branch de ensaio apagada.
- **D-9:** ubuntu-22.04, GLIBC máx. 2.34.
- **D-10:** `0.0.0` no repositório; `Carimbar versao` e `Conferir versao dos binarios` em success nos dois SOs.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run real de `pull_request` (`RUN_ID`) no `HEAD_SHA`: success, árvore igual, `versao`/`qualidade` success, `rust-linux` em ubuntu-22.04, `salto=inicial versao=0.1.0`, `lancar` skipped | CONTEXT | Auto | PASS | `OK` (bash). Run 36435173929 |
| 2 | Template que rodou = PR #14 (`WORKFLOWS_SHA`): `referenced_workflows`, composites `@main` sem diff, CI do gw verde | CONTEXT | Auto | PASS | `OK` (bash). Head do #14 = `3688e06`; 3/3 checks success |
| 3 | `pacotes-rust-linux`: 4 arquivos sem espaço; deb, rpm, AppImage e tar.gz inspecionados; `ddc-cli --version` executa; `Cargo.toml` em `0.0.0` | CONTEXT | Auto | PASS | `OK` (bash). `postinst`/`%post` idênticos, GLIBC máx. 2.34 (conferido à parte) |
| 4 | `pacotes-rust-windows`: MSI, NSIS e zip `{ddc-cli.exe, LICENSE}`; 5 passos em success nos dois jobs rust | CONTEXT | Auto | PASS | `OK` (bash) |
| 5 | Ensaio `ENSAIO_RUN_ID`: push do mesmo `HEAD_SHA` em `ensaio-release/*`, success, `salto=inicial`, `lancar / Tag e release` em success | CONTEXT | Auto | PASS | `OK` (bash). Run 36435174227 |
| 6 | Rascunho `DRAFT_RELEASE_ID`: draft/tag/alvo; assets = artefatos (7) + `SHA256SUMS`; `sha256sum -c --strict`; byte a byte; itens do `[Unreleased]` no corpo | CONTEXT | Auto | PASS | `OK` (bash). 398335754 |
| 7 | Nenhuma tag `v*` | CONTEXT | Auto | PASS | `OK` (bash). 0 tags |
| 8 | Validação de entrada do `lancar.yml` (script extraído do `WORKFLOWS_SHA`) | CONTEXT | Auto | PASS | `OK` (bash). Não cobre caminho absoluto; ver W-3 |
| 9 | Nenhum `TODO`/`FIXME` sem issue | CONTEXT | Auto | PASS | `OK` (bash) |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386 passed / 0 failed / 9 ignored |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 82.93% sem exclusão; 83.36% com a exclusão do Gate 3 |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` (bash, literal) |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (`CHANGELOG.md:8`) com 4 itens em Added e 3 em Changed; `## [0.1.0]` fica para a primeira release real |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção Install com os 7 arquivos, `sha256sum -c --ignore-missing SHA256SUMS`, aviso de instalador sem assinatura, Status e seção CI; a frase "Every push to `main` publishes" é imprecisa (W-7) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod release-packaging` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Nenhum gate bloqueia. Os 9 `Verify:` do CONTEXT e os 3 da baseline dão `OK` em `bash`. O rascunho 398335754 contém o que a D-8 pede, e os pontos de segurança pedidos se sustentam.

Antes do merge do gw#14, corrigir:
- W-1, com prioridade;
- W-3;
- W-4 e W-6, no texto;
- W-2: pelo menos declarar a exceção, e de preferência pré-semear as ferramentas com sha256.

Cada correção muda o `WORKFLOWS_SHA` e pede repin e novos runs de PR e de ensaio.

Ordem humana:
1. Instalar os pacotes do rascunho. Não clicar em "Publish".
2. Mergear o gw#13.
3. Rebasear e mergear o gw#14.
4. Trocar `@…` por `@main` no `ci.yml`.
5. Mergear o #11.

## DoD Critic (enhanced)

- DoD row «8»: o `Verify:` extraía só o corpo `run:` do `Conferir as entradas` e injetava `ARTEFATOS`/`CHANGELOG` por conta própria. O mapeamento `env:` do passo e a ausência de `continue-on-error`/`if` nunca eram conferidos. O crítico demonstrou dois mutantes no gw (clone descartável), e nos dois o `Verify:` imprimiu OK, com actionlint e `pinar_actions` verdes:
  - `ARTEFATOS: ${{ inputs.changelog }}` só nesse passo;
  - `continue-on-error: true` no passo.
- DoD row «5» (fragilidade, não objetiva): o `J=$(...);` fechava a cadeia `&&` com `;`.
- As outras 10 linhas Auto foram julgadas não ocas. A linha 6 discrimina de fato: os 41 itens do `[Unreleased]` só casam pela seção do CHANGELOG, e não pelas notas do `versao`. A linha 9 resistiu a 9 mutações.

**Verdict:** BLOCKED

> Correção do orquestrador (verify reset iter 1 → 2, CONTEXT):
> - a linha 8 lê o `lancar.yml` do `WORKFLOWS_SHA` com PyYAML e exige, no passo `Conferir as entradas` do job `lancar`, `env` `VERSAO`/`TAG`/`ARTEFATOS`/`CHANGELOG` = os inputs certos, nenhum `if`/`continue-on-error` (nem no job) e o passo antes do `gh release create`;
> - o script testado é o `run` parseado;
> - passam a reprovar também `changelog` `/etc/passwd` e `/home/runner/x` (W-3);
> - a linha 5 trocou o `;` por `&&`.
>
> Contra o `3688e06` a linha 8 agora NÃO imprime OK, porque o caminho absoluto ainda é aceito. A correção vem na iteração 2.
