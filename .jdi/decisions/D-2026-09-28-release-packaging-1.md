D-2026-09-28-release-packaging-1 (2026-09-28): Arquitetura geral da phase e arquivo de evidência.

O card ("completei a PR e mesmo assim não gerou um pacote/release") exige que o push desta phase na `main` produza a tag `v0.1.0` e a GitHub Release com os pacotes anexados. Hoje não existe nenhuma tag no repositório, então o `versao` calcula `salto=inicial` e `versao=0.1.0` (`versao_inicial: "0.1.0"` já casa com `[workspace.package] version` do `Cargo.toml`, herdado de `ci.yml`).

**Branch do `github-workflows`:** nova branch criada a partir de `origin/qualidade-rust-windows` (head `9e91d1ac9604aa581ac17438c54d75a2fcc213fa`, PR #13 ainda aberto), empilhada sobre ela — nunca a partir de `origin/main` direto, porque as mudanças desta phase (`lancar.yml`, `qualidade.yml`) dependem dos campos que o #13 introduz (`so`, `pacotes_sistema`, `auditoria`, `build_release`). PR próprio para a `main` do `github-workflows`, com o corpo dizendo "inclui o #13; mergear o #13 antes" (mesmo padrão do #13 em relação ao que vier antes dele). Se o #13 for mergeado (squash) durante a phase, a branch é rebaseada sobre `origin/main` de lá, e as referências passam a apontar só para o novo SHA resultante.

**`ci.yml` do ddc-control:** todo `uses:` de `slipalison/github-workflows/...` continua fixado no SHA do head do PR novo (nunca `@main`), como em D-2026-09-27-ci-crossbuild-1. A troca por `@main` é Deferred, só depois do merge do PR do `github-workflows` desta phase.

**Arquivo de evidência:** `.jdi/phases/release-packaging/release-evidence.env`, `KEY=VALUE` por linha, versionado, reescrito a cada run relevante (o Gate 8 do reviewer consulta sempre o que está ali, nunca um id fixo no CONTEXT — mesma regra de D-2026-09-27-ci-crossbuild-1/Notes). Chaves mínimas: `REPO=slipalison/ddc-control`, `RUN_ID` (run real de `pull_request` que exercita `qualidade`+empacotamento e mostra `lancar` como `skipped`), `HEAD_SHA`, `WORKFLOWS_BRANCH`, `WORKFLOWS_PR`, `WORKFLOWS_SHA`, e as chaves da rehearsal de release (D-8): `DRAFT_TAG`, `DRAFT_RELEASE_ID`.

Os dois repositórios continuam clonados lado a lado em `/home/slipalison/repos/` (ddc-control e github-workflows), como na phase `ci-crossbuild`.
