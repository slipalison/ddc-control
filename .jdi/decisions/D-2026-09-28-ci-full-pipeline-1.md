D-2026-09-28-ci-full-pipeline-1 (2026-09-28): O `ci.yml` é uma chamada só ao `pipeline.yml`.

- Um job, `esteira`, com `uses: slipalison/github-workflows/.github/workflows/pipeline.yml@<ref>`. Saem os `versao`, `qualidade` e `lancar` avulsos da D-2026-09-27-ci-crossbuild-1 e da D-2026-09-28-release-packaging-7.
- Motivo: chamadas avulsas publicaram a v0.1.0 sem varredura de segurança, sem Sonar e sem portão, e o painel não mostrava nenhuma dessas ausências. Com `imagem` e `app` vazios, o pipeline pula o build de imagem e o deploy, e o resto é a esteira inteira.
- Permissões do job: `contents: write` (tag e release), `packages: write` (checagem estática do `alias` do `lancar.yml`), `security-events: write` (SARIF) e `actions: read` (CodeQL).
- Emenda (review da iteração 1, W2): o `lancar` do `pipeline.yml` roda em qualquer evento que não seja `pull_request`, e não só em `push` como na D-2026-09-28-release-packaging-7. Um `workflow_dispatch` na `main` publica, se `salto != nenhum`; numa `ensaio-release/*`, cria rascunho. Mantido de propósito: disparar o workflow exige escrita no repositório, e os portões são os mesmos.
- O que a `release-packaging` já decidiu continua igual, agora como inputs do pipeline:
  - componentes, carimbo de versão e `ubuntu-22.04`;
  - `timeout_qualidade: 60`, `artefatos_release: "pacotes-rust-*"` e `changelog: CHANGELOG.md`.
- Emenda (2026-09-28, run 36498449337): sai o ensaio em `ensaio-release/**` da D-2026-09-28-release-packaging-8.
  - Pelo pipeline, o rascunho passa pelo Sonar, e o SonarQube Cloud Free só analisa a `main` e os PRs: o relatório da branch subiu, a consulta do Quality Gate voltou "Not authorized or project not found", e o `lancar` ficou pulado.
  - Pular o Sonar no ensaio abriria a porta que esta phase fecha. O gw#15 tirou o `ramo_de_ensaio` antes de ele entrar na `main`.
  - Os pacotes de cada PR continuam nos artefatos do run. O `lancar.yml` é o mesmo que publicou a v0.1.0; só a fiação do `pipeline.yml` até ele roda pela primeira vez no push do merge (review W3, risco aceito).
