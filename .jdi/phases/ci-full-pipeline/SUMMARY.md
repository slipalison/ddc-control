# Phase 8: CI full pipeline — Summary  (slug: ci-full-pipeline)

## Resultado
O `ci.yml` virou uma chamada ao `pipeline.yml`, e o run real do PR #12 (36496391097, head `cccd78f`) saiu verde.

| Job | Resultado |
|---|---|
| `versao`, `qualidade` (`rust-linux`, `rust-windows`, `node-ui`) | ok |
| `Varreduras` | ok |
| CodeQL (`rust`, `javascript-typescript`, `actions`) | ok |
| `Sonar` | ok |
| `Portao` | ok |
| `lancar`, `imagem`, `publicar` | pulados, como devem ser num PR |

As 8 linhas do DoD passam contra esse run, e cada linha que lê o run reprova num run anterior (os mutantes estão nos commits de emenda).

## Números medidos
- **Varreduras:**
  - Gitleaks: 339 commits varridos, 0 achados;
  - Semgrep: 1.074 regras no conjunto;
  - code scanning do PR: CodeQL, Semgrep e Trivy, sem alerta aberto.
- **Trivy:** 2 avisos médios no `Cargo.lock` no primeiro run. São `serde_yaml` (GHSA-39vw-qp34-rmwf) e `glib` (GHSA-wrw7-89jp-8q8g), os mesmos advisories já aceitos no `.cargo/audit.toml`, e foram para o `.trivyignore` com referência cruzada.
- **Sonar** (PR 12): Quality Gate OK, cobertura 86,3% em 5.173 linhas; 0 bugs, 0 vulnerabilidades, 0 code smells, 0 hotspots.
- **UI:** 88,15% de linhas no `node --test` (piso 80). A anotação de erro "Nenhum relatorio de cobertura" sumiu.
- **Testes locais:** 386 passed, 0 failed, 9 ignored no Rust; 161 no `node --test`; 138 no Playwright.

## Desvios e correções
- **A premissa "o Sonar roda o Clippy" era falsa.** O sensor `Rust Enterprise` analisa sem compilar (4,6 s). D-3 foi emendada, e o texto foi corrigido nos dois repositórios. Os pacotes de sistema e o toolchain continuam necessários, porque o `cargo llvm-cov` do script compila o workspace.
- **Linha 4 do DoD:** o `grep clippy` passava por acaso, casando com a linha do `rustup`. Foi reescrita contra os sensores e a API do SonarCloud.
- **Linhas 3 a 6:** os `Verify:` foram corrigidos depois do primeiro run: escapes ANSI no log, SIGPIPE sob `pipefail` e diff com os prefixos `+`/`-`.
- **`SONAR_TOKEN`:** foi gerado pelo navegador a pedido do usuário. O token "ddc-control CI", sem expiração, ficou no secret do repositório. A Análise Automática já estava desligada.
- **Template:** o `pipeline.yml` chama os aninhados `@main`, e a `main` do gw não tem o #14. A prova usa a branch descartável `teste/esteira-app-desktop` (`a13c154`), que é o head do #15 (`ad8a96e`) com os aninhados fixados nele. O primeiro pin (`6bd2c63`) continua alcançável pela branch `teste/esteira-app-desktop-6bd2c63`.

## github-workflows #15 (empilhado no #14)
- `pipeline.yml` para app desktop: `timeout_qualidade`, versão carimbada, `artefatos_release`, `changelog`, `ramo_de_ensaio`, `sonar_versao_linguagem` e `sonar_pacotes_sistema`.
- `sonar.yml`: `pacotes_sistema`.
- `Portao` (BREAKING): reprova Sonar sem `sonar_projeto` e sem `sonar_dispensa`.
- README: "O mínimo exigido", o `Portao` no ruleset, a Análise Automática nos pré-requisitos e a correção do Clippy.

## Pendências do usuário
- **Merges:** gw#14, depois gw#15 (eu rebaseio o #15 sobre a `main`), depois trocar o `uses:` para `@main` neste PR, antes do merge dele.
- **Ruleset:** `esteira / Portao` como check obrigatório (D-6).
- **`coluna`:** uma linha de `sonar_dispensa`, ou o Sonar ligado, depois do gw#15.
