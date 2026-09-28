# SUMMARY — release-packaging (loop iter 1)

> Escrito pelo orquestrador a partir do relatório do doer. O harness recusa a escrita do SUMMARY pelo subagente.

8/8 tasks concluídas, 0 bloqueadas. Pacotes reais saem verdes nos dois SOs, e o rascunho `v0.1.0` foi criado PELO WORKFLOW, com egresso `block`. Nenhuma tag `v*` existe. Os 9 `Verify:` do CONTEXT dão `OK` em `bash`. O 3 e o 4 dependiam do locale, pela ordem do `sort`; o orquestrador prefixou `export LC_ALL=C.UTF-8;` em todas as linhas, e as duas passaram a dar OK também com `pt_BR.UTF-8`.

## PRs
- github-workflows: [#14](https://github.com/slipalison/github-workflows/pull/14), branch `pacotes-tauri-release` a partir de `qualidade-rust-windows` (`9e91d1a`), head `3688e06`. Inclui o #13, que precisa ser mergeado antes. CI verde (run 36435144932).
- ddc-control: [#11](https://github.com/slipalison/ddc-control/pull/11) (rascunho).

## Commits
- gw:
  - `bedde2d` `feat(qualidade)`: empacota Tauri, binários extra e carimba a versão;
  - `7b3b560` `feat(lancar)`: anexa pacotes, somas e changelog;
  - `3f08bdf` `fix(qualidade)`: cargo-audit musl, que roda no 22.04;
  - `46f99c3` `fix(qualidade)`: Python 3.12 para o relatório no 22.04;
  - `3688e06` `fix(lancar)`: egresso fechado com a lista medida.
- ddc-control: `37e59e2` (bundles, udev, i2c-dev), `396711c` (CI de pacotes e release), `fa68125` (publisher e descrições), `ce37175` (docs), `8b093d1`/`2075f5e`/`299c3b8` (repin do gw; `299c3b8` = `HEAD_SHA`) e `515eb47` (evidência, só `.jdi/`).

## Runs
| Run | Commit (gw) | Tipo | Resultado | O que mostrou |
|---|---|---|---|---|
| 36429141171 | `396711c` (`7b3b560`) | PR | failure | Windows verde de primeira: WiX 3.14 sem problema de VBScript, NSIS e zip. O `cargo-audit` gnu pedia GLIBC_2.38/2.39 → `3f08bdf` (musl). |
| 36431331755 | `ce37175` (`3f08bdf`) | PR | failure | Os 4 pacotes Linux saíram. O `Cobertura` quebrou: `typing.Self` com o python 3.10 do 22.04 → `46f99c3`. |
| 36432823249 | `2075f5e` (`46f99c3`) | PR | success | `lancar` skipped. |
| 36432836623 | `2075f5e` | ensaio, egresso `audit` | success | O `versao` julgou `07323e5..2075f5e` (push em dois tempos, A-6); rascunho 398320036; medição do egresso. |
| **36435173929** | **`299c3b8` (`3688e06`)** | **PR (`RUN_ID`)** | **success** | `rust-linux` no ubuntu-22.04, `lancar` skipped, cobertura Linux de 82,99%. |
| **36435174227** | **`299c3b8` (`3688e06`)** | **ensaio (`ENSAIO_RUN_ID`)** | **success** | `lancar` em `block`, sem nenhuma chamada bloqueada. Apagou o rascunho anterior e criou o **398335754**, com 8 anexos e corpo de 20.394 caracteres. |

## O que os bundles revelaram
- **Dependência falsa de AppIndicator.** O tauri-cli 2.12 lê as entradas `tauri` de todos os targets do Cargo.toml, e o `tray-icon` do bloco `cfg(not(linux))` contaminava o Linux. Saiu a entrada redundante: o positioner já liga `tauri/tray-icon`, e o lock não mudou. O deb agora depende só de `libwebkit2gtk-4.1-0, libgtk-3-0`.
- **Metadados.** `publisher`, `category: Utility` e as descrições foram preenchidos. Antes o MSI e o deb saíam com `Author`/`Maintainer: github`.
- **Nomes.** Linux: `ddc-control_0.1.0_amd64.deb`, `ddc-control-0.1.0-1.x86_64.rpm`, `ddc-control_0.1.0_amd64.AppImage` e `ddc-cli-x86_64-unknown-linux-gnu.tar.gz`. Windows: `ddc-control_0.1.0_x64_en-US.msi`, `ddc-control_0.1.0_x64-setup.exe` e `ddc-cli-x86_64-pc-windows-msvc.zip`.
- **glibc.** No máximo GLIBC_2.34 no `ddc-tray` e no `ddc-cli` (D-9).
- **Carimbo.** O `ddc-cli --version` dá `0.1.0` no Linux (local) e no Windows (CI), enquanto o repositório diz `0.0.0`. O `cargo update --workspace --offline` só mexe nas 4 linhas `version` do lock.
- **Tempos.** Com cache, ~5,5 min por SO. Sem cache, Linux 11 min e Windows 14 min. O `lancar` leva ~30 s.

## Egresso medido (`lancar`)
| Destino | Quem usa |
|---|---|
| `github.com:443` | checkout |
| `api.github.com:443` | gh: tag, rascunho, release |
| `uploads.github.com:443` | gh: anexos |
| `results-receiver.actions.githubusercontent.com:443` | download-artifact |

O blob `productionresultssa*.blob.core.windows.net` é liberado sozinho pelo harden-runner 2.21.1 ("fetched GitHub meta domains"). O ensaio em `block` passou por outra conta de blob sem bloqueio.

## Desvios
- `apps/ddc-tray/src-tauri/Cargo.toml` saiu da lista da T-2: é a correção do AppIndicator.
- Comentário no `Cargo.toml` da raiz explicando o `0.0.0`.
- O `Carimbar versao` edita o Cargo.toml do `caminho` do componente, que aqui é a raiz.
- A tabela SO→alvo agora existe uma vez só (`Alvo do runner`).
- No gw: `cargo-audit` musl e o Python 3.12 por `setup-python` (já no lock) quando o `python3` é menor que 3.11. Nada em `bin/` nem nas composites.
- O `lancar` passa os anexos direto ao `gh release create` (A-5). O `Apagar rascunho velho` só roda com `artefatos`/`rascunho`. Timeout de 5 min.
- `librsvg2-dev` no apt do Linux, por leitura do plugin gtk do linuxdeploy, sem falha medida.
- T-6 e T-8 viraram um ensaio só.

## Gates locais
fmt, clippy (`--all-features -D warnings`), 386 testes, cobertura 83,36%, `npm test` (161 + 138), `cargo audit`, actionlint com shellcheck, `pinar_actions --verificar`. O `Conferir componente` do gw rodou em 33 combinações.

## Deferred (humano)
- Instalar os pacotes do rascunho 398335754 numa máquina real.
- Mergear o gw#13 e depois o gw#14, trocar `@3688e06…` por `@main` no `ci.yml`, e então mergear o #11. Esse merge cria a tag e a release `v0.1.0` e apaga o rascunho.
- Revisar os textos e o diff dos templates.

## Blocked tasks
- nenhuma

## Iteração 2 (fix do crítico na linha 8 + rodada de avisos)
Os 9 `Verify:` dão `OK` em `bash`. A linha 8, que falhava contra o `3688e06`, agora passa contra o `d6d340a`.

| | Valor |
|---|---|
| `HEAD_SHA` | `f6063d7` (depois dele, só `.jdi/`) |
| `WORKFLOWS_SHA` | `d6d340a` (gw#14, CI 36444714782 verde) |
| `RUN_ID` (PR) | 36445031966, success, `lancar` skipped |
| `ENSAIO_RUN_ID` | 36447884185, branch `ensaio-release/v0.1.0-f6063d7-2` (apagada) |
| `DRAFT_RELEASE_ID` | **398419349**, rascunho `v0.1.0`, 8 anexos; o 398335754 foi apagado pelo workflow |

- O primeiro ensaio (36446037437) caiu no Windows por queda de rede (`os error 10054`) ao baixar o `nsis_tauri_utils.dll`. Nada foi criado. O ensaio foi repetido em branch nova, sem `rerun`.
- gw:
  - `e0f00bb` W-1: `tag_movel_major` com `rascunho` reprova, e o `if` do passo ganhou `!inputs.rascunho`.
  - `5da296b` W-3: caminho absoluto em `changelog` e `caminho_tauri` reprova.
  - `0efa403` W-2: o passo `Ferramentas do AppImage` baixa de URL fixa, com sha256, `AppRun`, `linuxdeploy`, `linuxdeploy-plugin-appimage` (`1-alpha-20250213-1`, no lugar do `continuous`) e o runtime type2 (`20251108`, via `LDAI_RUNTIME_FILE`). Duas travas: nenhum `Downloading` do bundler, e o AppImage começa com o runtime conferido.
  - `d6d340a` W-4/W-5/W-6: textos.
- Prova da W-2 no job 109005130135: 0 linhas `Downloading` (eram 3), 4 sha256 `OK`, 0 conexões do bundler e runtime `2fca8b443c92` conferido.
- ddc-control:
  - `aaac501` `artefatos: "pacotes-rust-*"`;
  - `59d16ef` repin;
  - `f6063d7` README e CHANGELOG (W-7/W-5);
  - `23e14ec` evidência;
  - `bd278c9` PLAN.
- Gates locais verdes: 386 testes, cobertura 83,36%, 161 + 138 na UI, actionlint e pins. `Conferir as entradas` testado em 15 combinações e `Conferir componente` em 38.
- W-8 (herdado, fora do diff) continua aberto.

## Iteração 3 (sem código; reset das linhas 8 e 3 pelo orquestrador)
Os 9 `Verify:` dão `OK` (exit 0) antes e depois do commit `22e1153`, que só mexe no PLAN. A evidência é a da iteração 2: `RUN_ID` 36445031966, ensaio 36447884185, rascunho 398419349, `HEAD_SHA` `f6063d7` e `WORKFLOWS_SHA` `d6d340a`.

Controles negativos:
- A linha 8 sai 1 contra o `3688e06`, contra o `d6d340a` com `${{ inputs.artefatos }}` injetado no `::error::` e contra o `d6d340a` sem `::error::`.
- As checagens novas da linha 3 reprovam na `origin/main`, onde a versão ainda é `0.1.0`.
