# Phase 7: Review  (slug: release-packaging)

**Verdict:** APPROVED_PENDING_MANUAL

> Loop iter 2. Escrito pelo orquestrador a partir do resultado integral do reviewer: o harness nega escrita de `.md` ao subagente.
>
> **Estado dos repositórios**
> - O HEAD local é `45c8433` e o `HEAD_SHA` da evidência é `f6063d7`. Entre os dois só mudou `.jdi/`.
> - No github-workflows, o PR #14 (`pacotes-tauri-release`) está aberto, MERGEABLE e com head `d6d340a` = `WORKFLOWS_SHA`. Ele continua empilhado sobre o #13 (`9e91d1a`).
>
> Os gates rodaram em `bash`, no Fedora, com rustc 1.98.1.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0. O `cargo check` de `ddc-core`, `ddc-adapters` e `ddc-cli` passa para Linux e para Windows msvc. O `ddc-tray` no Windows só o CI prova: no `RUN_ID` 36445031966, `rust-windows` deu success com os pacotes. |
| Tests | PASS | 386 passed, 0 failed, 9 ignored (hardware), em 15 binários. |
| Coverage | PASS | 83.36% (`main.rs`/`build.rs` excluídos) contra o piso de 80%. Sem exclusão: 82.93%. Nenhum `.rs` mudou na phase. |
| Lint | PASS | fmt OK. clippy `-D warnings` OK, também com `--all-features`. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.4, 5.8 e 5.9 sem saída. 5.5a e 5.6 com hits antigos. 5.7: bordas e hardware `#[ignore]`. 5.10: `cargo audit` 0.22.2 sai 0, e o `Cargo.lock` não mudou na iteração 2. 5.11 limpo. |
| Consistency | PASS | Os commits da iteração 2 têm scope e tipo coerentes e citam a D-XX no corpo. As D-2026-09-28-release-packaging-1..10 estão conformes, com as emendas. As ressalvas W-1 e W-3 à D-2 foram fechadas. |
| Segurança do template (gw#14, `3688e06..d6d340a`) | PASS | actionlint 1.7.12 e shellcheck 0.10.0 limpos. `pinar_actions.py --verificar` OK. `actions.lock.json`, `.github/actions` e `bin/` não mudaram. Todo `${{ }}` novo está em `env:`. W-1, W-2 e W-3 foram corrigidos e provados; W-4..W-6 corrigidos no texto. Restam W-9 e W-10, baixos. |
| UI Validation | PASS | Playwright: 138 passed, 6 skipped. `node --test`: 161/161. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto (9 CONTEXT + 3 PROJECT), literais em `bash`; 2 manuais pendentes |

## Blockers
- nenhum

## Warnings

### Resolvidos desde a iteração 1
- **Blocker do crítico (linha 8 do DoD):** o `Verify:` reescrito dá `OK` contra o `d6d340a`. Contra o `3688e06` sai 1, então discrimina.
- **Linha 5:** o `;` virou `&&`.
- **W-1:** `Conferir as entradas` reprova `tag_movel_major` junto com `rascunho` (`lancar.yml:228`), e `Mover a tag de major` tem `if: inputs.tag_movel_major && !inputs.rascunho` (`:449`).
- **W-2:** resolvido. As provas estão nas Notas.
- **W-3:** `changelog` (`lancar.yml:264`) e `caminho_tauri` (`qualidade.yml:226`) recusam `/` no começo.
- **W-4:** os domínios meta liberados pelo harden-runner estão escritos no comentário e no README do gw.
- **W-5:** documentado ("NÃO ENSAIE PERTO DO MERGE") no `lancar.yml`, no README do gw, no exemplo e no README do ddc-control. O risco residual é aceito: não há perda de segurança.
- **W-6:** o `Alvo do runner` está citado, e a lista do exemplo foi refeita.
- **W-7, parte do README:** "when its commits call for a new version", conferido contra `bin/versao.py:116-123`. O CHANGELOG também foi corrigido.
- **Endurecimento da D-7:** `artefatos: "pacotes-rust-*"`. No ensaio, dos 4 artefatos do run só os 2 `pacotes-rust-*` foram baixados.

### Abertos
- **W-7 residual (texto, Deferred):** o corpo do rascunho mistura o CHANGELOG em inglês com os títulos em pt-BR das notas do `versao`. Sem tag anterior, ele lista o histórico inteiro (21.160 caracteres).
- **W-8 (herdado da ci-crossbuild, fora do diff):** `${{ }}` em `run:` nos passos dotnet/npm/go, `gocover-cobertura@latest`, e `so` aceitando qualquer sufixo. Não atinge o ddc-control.
- **W-9 (baixo, operacional; gw `qualidade.yml:774-779`):** as ferramentas estão fixadas por hash, mas em tags que o dono reescreve.
  - Nenhuma das 4 releases é imutável. Os assets de `apprun-old` e `linuxdeploy-07333c6` já foram re-enviados antes, o último em 2026-09-20.
  - O passo falha fechado: um novo re-envio derruba o empacotamento Linux até alguém revisar e trocar o hash.
  - O README do gw declara a não-imutabilidade. Uma frase sobre o que fazer quando o sha256 falhar seria bem-vinda.
- **W-10 (cosmético; gw `qualidade.yml:912`):** uma linha de 133 caracteres no comentário do `Guardar pacotes`.

## Notas

### W-2: as ferramentas do AppImage (passo `Ferramentas do AppImage`, `qualidade.yml:728`)
- **Hashes contra os assets reais.** Baixei os 4 arquivos e calculei o sha256; todos batem.
  - `AppRun-x86_64` (31552 bytes): `digest` da API bate.
  - `linuxdeploy-07333c6-x86_64.AppImage`: `digest` bate.
  - `linuxdeploy-plugin-appimage.AppImage`: a API dá `digest: null` (asset de 2025-02), mas o tamanho bate.
  - `runtime-x86_64`: `digest` bate.
- **URLs.**
  - O plugin (`1-alpha-20250213-1`) e o runtime (`20251108`) vêm das últimas releases que não são `continuous`.
  - O `AppRun` e o linuxdeploy usam as URLs exatas do `prepare_tools` do tauri-bundler 2.10.0 (`LINUXDEPLOY_COMMIT_HASH = "07333c6"`), do qual o tauri-cli 2.12.0 depende (`=2.10.0`).
  - Os scripts do plugin gtk vêm por `include_bytes!`. Nada fica sem hash.
- **Trava 1 (`Downloading`, `qualidade.yml:870`).**
  - O padrão casa as 3 linhas reais do job 108971248763 (iteração 1), e dá 0 linhas nos jobs 109005130135 e 109014950511.
  - `pipefail` com `tee` preserva a falha do `cargo tauri build`.
- **Harden-runner.** As conexões do `cargo-tauri` e do `appimagetool` passaram de 4 para 0.
- **Trava 2 (runtime).** Matriz 2x2 com o `conferir_runtime` extraído do YAML: o AppImage novo passa contra o runtime fixado e reprova contra o `continuous` de hoje; o AppImage da iteração 1 faz o contrário.
  - As seções mascaradas somam 10.256 bytes (assinatura e update info). O `.text` é comparado inteiro.
  - O AppImage anexado ao rascunho passa, e o `AppRun.wrapped` dele é o `AppRun-x86_64` fixado.
- **O passo, testado à parte.** Com 1 hash adulterado, sai 1 antes do `mv`, e o passo Tauri reprova. Com alvo `aarch64`, `::error::`. As travas rodam antes do `Guardar pacotes`, então um AppImage não conferido nunca vira artefato.
- **Windows.** O WiX e o NSIS o bundler já baixa com hash fixo no código. Fora do escopo da W-2.

### W-1 e W-3: scripts extraídos do YAML parseado
- **`Conferir as entradas`:**
  - `MOVEL=true RASCUNHO=true` dá 1; as outras combinações dão 0.
  - Reprovam: `/etc/passwd`, `/home/runner/x`, `//etc/passwd`, `../CHANGELOG.md`, `docs/../../x`, `a b` e `C:x`.
  - Passam: `CHANGELOG.md`, `docs/CHANGELOG.md` e `./CHANGELOG.md`.
- **`Conferir componente`:** `apps/ddc-tray/src-tauri` dá 0. `/etc`, `/home/runner/x`, `//x`, `../x`, `a/../../b`, `a b` e vazio dão 1.

### Rascunho 398419349 e o 398335754
- **398419349:**
  - `draft: true`, `tag_name: v0.1.0`, alvo `f6063d7…`, autor `github-actions[bot]`, `published_at: null`;
  - 8 anexos e corpo de 21.160 caracteres;
  - é a única release do repositório, com 0 tags e nenhuma branch `ensaio-release/*`.
- **398335754:** a API responde 404. O log do `lancar` do ensaio registra "Rascunho 398335754 de v0.1.0 apagado."
- **Egresso do `lancar` no ensaio:** `block`, 0 bloqueios.

### Conformidade D-XX (resumo)
- **D-1:** os 3 `uses:` estão em `@d6d340a…`; `release-evidence.env` tem as 10 chaves.
- **D-2:** atendida com as emendas, W-1 e W-3.
- **D-3:** atendida. O passo `Ferramentas do AppImage` é extensão. Os 5 nomes do contrato não mudaram.
- **D-4/-5/-6/-10:** linhas 3 e 4 do DoD.
- **D-7 (emendada):** literal, com `pacotes-rust-*`.
- **D-8:** PR com `lancar` skipped, ensaio verde, rascunho mantido, 0 tags, branch de ensaio apagada.
- **D-9:** `ubuntu-22.04`.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Run real de `pull_request` (`RUN_ID`) no `HEAD_SHA`: success, árvore igual, `versao`/`qualidade` success, `rust-linux` em ubuntu-22.04, `salto=inicial versao=0.1.0`, `lancar` skipped | CONTEXT | Auto | PASS | `OK` (bash, literal). Run 36445031966 em `f6063d7` |
| 2 | Template que rodou = PR #14 (`WORKFLOWS_SHA`): `referenced_workflows`, composites `@main` sem diff, CI do gw verde | CONTEXT | Auto | PASS | `OK` (bash). Head do #14 = `d6d340a` |
| 3 | `pacotes-rust-linux`: 4 arquivos sem espaço; deb, rpm, AppImage e tar.gz inspecionados; `ddc-cli --version` executa; `Cargo.toml` em `0.0.0` | CONTEXT | Auto | PASS | `OK` (bash). À parte: o AppImage passa na trava do runtime fixado |
| 4 | `pacotes-rust-windows`: MSI, NSIS e zip `{ddc-cli.exe, LICENSE}`; 5 passos em success nos dois jobs rust | CONTEXT | Auto | PASS | `OK` (bash) |
| 5 | Ensaio `ENSAIO_RUN_ID`: push do mesmo `HEAD_SHA` em `ensaio-release/*`, success, `salto=inicial`, `lancar / Tag e release` em success, alias skipped | CONTEXT | Auto | PASS | `OK` (bash). Run 36447884185 |
| 6 | Rascunho `DRAFT_RELEASE_ID`: draft/tag/alvo; assets = artefatos (7) + `SHA256SUMS`; `sha256sum -c --strict`; byte a byte; itens do `[Unreleased]` no corpo | CONTEXT | Auto | PASS | `OK` (bash). 398419349; o 398335754 dá 404 |
| 7 | Nenhuma tag `v*` | CONTEXT | Auto | PASS | `OK` (bash) |
| 8 | Validação de entrada do `lancar.yml` (YAML parseado: `env`, sem `if`/`continue-on-error`, antes do `gh release create`; valores perigosos, inclusive `changelog` absoluto, reprovam) | CONTEXT | Auto | PASS | `OK` contra `d6d340a`; exit 1 contra `3688e06` (discrimina) |
| 9 | Nenhum `TODO`/`FIXME` sem issue | CONTEXT | Auto | PASS | `OK` (bash) |
| 10 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | 386 passed / 0 failed / 9 ignored |
| 11 | Coverage >= 80% of lines | PROJECT | Auto | PASS | 82.93% sem exclusão; 83.36% com a exclusão |
| 12 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | `OK` (bash, literal) |
| 13 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` em `CHANGELOG.md:8`, com o item da release corrigido e o do AppImage com ferramentas fixadas; `## [0.1.0]` fica para a primeira release real |
| 14 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: Install (`README.md:7`) com os 7 arquivos, `sha256sum -c --ignore-missing SHA256SUMS` e o aviso de instalador sem assinatura; "What publishes" (l.498) condiciona a publicação a `salto != nenhum`; "Do not rehearse near a merge" (l.499) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation** (while a manual item is still pending):
Run `/jdi-confirm-dod release-packaging` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Nenhum gate bloqueia. Os 9 `Verify:` do CONTEXT e os 3 da baseline dão `OK`. A linha 8 discrimina contra o head anterior. A iteração 2 fechou W-1..W-6, a parte do README da W-7 e o endurecimento da D-7. A W-2 foi provada de forma independente.

Opcionais antes do merge:
- W-10, reflow do comentário;
- W-9, uma frase sobre falha de sha256.

Qualquer mudança no gw pede repin, run de PR e ensaio.

Ordem humana:
1. `/jdi-confirm-dod release-packaging`.
2. Instalar os pacotes do rascunho 398419349. Não clicar em "Publish".
3. Mergear o gw#13; depois rebasear e mergear o gw#14.
4. Trocar `@d6d340a…` por `@main` no `ci.yml`.
5. Mergear o #11. Não ensaiar perto desse merge.
6. Na primeira release real, revisar o corpo (W-7 residual).

## DoD Critic (enhanced)

- DoD row «8»: o harness rodava o texto CRU do `run:` e aceitava QUALQUER saída diferente de zero como "reprovou". O GitHub interpola `${{ }}` no texto antes do bash.
  - Mutante demonstrado num clone descartável: a mensagem de erro do `lancar.yml:260` passa a citar `${{ inputs.artefatos }}`, como o `qualidade.yml` já faz em 7 passos.
  - O Verify imprimia OK, e o actionlint também passava.
  - No runner, `pacotes-$(id)` EXECUTARIA o `id` num job com `contents: write`.
- DoD row «3» (suspeita, não objetiva): um `version` no `tauri.conf.json` ou no `Cargo.toml` da tray daria `0.1.0` aos pacotes sem depender do carimbo.
- As outras 10 linhas Auto foram julgadas não ocas.

**Verdict:** BLOCKED

> Correção do orquestrador (verify reset iter 2 → 3, CONTEXT):
> - a linha 8 exige `"${{" not in run` no YAML parseado. O script roda como o runner o roda (`bash --noprofile --norc -eo pipefail`), e cada valor perigoso tem de ser recusado com o `::error::` do PRÓPRIO script. Um erro qualquer do bash dá `ERRO` e reprova a linha.
> - a linha 3 exige, no `HEAD_SHA`, `tauri.conf.json` sem `version` e `version.workspace = true` no crate da tray.
>
> Provas: contra o `d6d340a`, as duas linhas dão `OK`. O mutante do crítico, recriado num clone descartável do gw, NÃO imprime OK.
