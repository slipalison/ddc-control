D-2026-09-28-release-packaging-7 (2026-09-28): Wiring do `.github/workflows/ci.yml` do ddc-control para pacotes e release.

- **Gatilho:** `on.push.branches` passa a `[main, 'ensaio-release/**']`. `pull_request` e `workflow_dispatch` continuam. O push numa branch `ensaio-release/*` usa o `ci.yml` do PRÓPRIO commit empurrado, então o ensaio da D-8 não depende de nada estar na `main`. O `workflow_dispatch` exige o workflow na branch padrão e por isso não serve.
- **`qualidade`:** ganha `needs: [versao]` e recebe `versao: ${{ needs.versao.outputs.versao }}` (D-10).
  - `rust-linux` passa a `so: ubuntu-22.04` (D-9), com `empacotar_tauri: true`, `caminho_tauri: apps/ddc-tray/src-tauri`, `binarios_extra: ddc-cli` e `carimbar_versao: true`.
  - `rust-windows` recebe os mesmos campos, sem `so` novo.
  - `node-ui` fica igual.
- **Job novo `lancar`:**

```yaml
lancar:
  needs: [versao, qualidade]
  if: >-
    github.event_name == 'push'
    && (github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/heads/ensaio-release/'))
    && needs.versao.outputs.salto != 'nenhum'
  permissions:
    contents: write
    packages: write
  uses: slipalison/github-workflows/.github/workflows/lancar.yml@<WORKFLOWS_SHA>
  with:
    versao: ${{ needs.versao.outputs.versao }}
    tag: ${{ needs.versao.outputs.tag }}
    versao_inicial: "0.1.0"
    artefatos: "pacotes-*"
    changelog: CHANGELOG.md
    rascunho: ${{ github.ref != 'refs/heads/main' }}
```

- Em `pull_request` o `lancar` nunca roda (`event_name == 'push'`). Na `main`, a release é publicada; numa `ensaio-release/*`, sai em rascunho.
- `needs` garante que o `lancar` só roda com a `qualidade` inteira verde (testes, cobertura, audit, pacotes nos dois SOs).
- `packages: write` é concedido mesmo sem imagem: a checagem de permissão do GitHub é estática. O motivo está no cabeçalho do `lancar.yml`.
- `versao_inicial: "0.1.0"` é o MESMO literal do job `versao`. Se um dia mudar, muda nos dois `with:` no mesmo commit.
- **Segurança:** quem pode empurrar uma branch `ensaio-release/*` já pode empurrar na `main` (escrita no repositório). O ensaio só cria RASCUNHO, que não é público e não cria tag.
