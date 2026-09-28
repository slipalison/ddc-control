D-2026-09-28-ci-full-pipeline-1 (2026-09-28): O `ci.yml` é uma chamada só ao `pipeline.yml`.

- Um job, `esteira`, com `uses: slipalison/github-workflows/.github/workflows/pipeline.yml@<ref>`. Saem os `versao`, `qualidade` e `lancar` avulsos da D-2026-09-27-ci-crossbuild-1 e da D-2026-09-28-release-packaging-7.
- Motivo: chamadas avulsas publicaram a v0.1.0 sem varredura de segurança, sem Sonar e sem portão, e o painel não mostrava nenhuma dessas ausências. Com `imagem` e `app` vazios, o pipeline pula o build de imagem e o deploy, e o resto é a esteira inteira.
- Permissões do job: `contents: write` (tag e release), `packages: write` (checagem estática do `alias` do `lancar.yml`), `security-events: write` (SARIF) e `actions: read` (CodeQL).
- O que a `release-packaging` já decidiu continua igual, agora como inputs do pipeline:
  - componentes, carimbo de versão, `ubuntu-22.04` e o ensaio em `ensaio-release/**`;
  - `timeout_qualidade: 60`, `artefatos_release: "pacotes-rust-*"`, `changelog: CHANGELOG.md` e `ramo_de_ensaio: "ensaio-release/"`.
