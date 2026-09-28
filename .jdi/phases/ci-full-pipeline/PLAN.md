# Phase 8: CI full pipeline — Plan  (slug: ci-full-pipeline)

## Goal
`ci.yml` = uma chamada ao `pipeline.yml`, com qualidade, segurança, CodeQL, Sonar e `Portao`. No gw, o app desktop entra pelo pipeline e o Sonar sem motivo reprova.

## Locked decisions (from CONTEXT.md)
- -1: um job `esteira` → `pipeline.yml`, com inputs da `release-packaging`.
- -2: segurança inteira, CodeQL rust/js-ts/actions.
- -3: Sonar pelo CI, com `scripts/ci/sonar-coverage.sh`.
- -4: UI com piso 80.
- -5: gw #15, provado pela branch `teste/esteira-app-desktop`.
- -6: ruleset só com o OK do usuário.

## Tasks

### Wave 1: github-workflows (branch `esteira-app-desktop`, PR #15 empilhado no #14)
- **T1** `sonar.yml`: input `pacotes_sistema` (regex do qualidade, por `env`, depois do `preparar`).
- **T2** `pipeline.yml`:
  - `timeout_qualidade`;
  - a `qualidade` espera o `versao` (`needs` + `!cancelled()`) e recebe `versao`;
  - `artefatos_release`, `changelog` e `ramo_de_ensaio` → `lancar` (`rascunho` fora da produção, e a `vN` só na produção);
  - `sonar_versao_linguagem` e `sonar_pacotes_sistema`.
- **T3** `Portao`: sem `sonar_projeto` e sem `sonar_dispensa`, reprova; com os dois, também. A dispensa fica numa linha só. Provar o script extraído em 6 casos.
- **T4** README ("O mínimo exigido", "Pacotes na release", pré-requisitos) e `exemplos/ci-rust-desktop.yml` pelo pipeline.
- **T5** `actionlint` 1.7.12 e `pinar_actions.py --verificar`. Branch descartável `teste/esteira-app-desktop`: os aninhados fixados no head.

### Wave 2: ddc-control (branch `phase/ci-full-pipeline`, a partir da `main`)
- **T6** `apps/ddc-tray/package.json`: `test:unit` (node --test + LCOV em `coverage/`) e `test` = `test:unit` + Playwright. `.gitignore`: `coverage/` e `/resultados/`.
- **T7** `scripts/ci/sonar-coverage.sh`: no CI, instala o `cargo-llvm-cov` 0.9.1 e o Node 24.18.0 com sha256; roda o `cargo llvm-cov --workspace` e o `npm run test:unit`; reescreve o `SF:` da UI para a raiz.
- **T8** `.github/workflows/ci.yml`: um job `esteira` com o pipeline no `WORKFLOWS_TEST_SHA`, as permissões e os inputs da D-1..D-4.
- **T9** README (seção CI) e CHANGELOG (`## [0.1.0] - 2026-09-28` para o que foi publicado, e a mudança nova em `[Unreleased]`).

### Wave 3: prova
- **T10** PR no ddc-control. O run real do PR vira `pipeline-evidence.env`. Achado das varreduras ou do Sonar: consertar na causa (D-2).
- **T11** Anotar o run no gw #15 e tirar o #15 de rascunho.

## Regras
- ddc-control: commits em inglês, scope `ci-full-pipeline`, `.jdi/` separado de código; nunca `--no-verify` nem `JDI_*`.
- gw: `tipo(escopo): assunto` em pt-BR, comentários sem acento. Nunca tocar `.github/actions/**` nem `bin/**`.
- Local: nunca `sudo`. O script de cobertura só instala com `CI=true`.
- Gates em todo commit de código: fmt, clippy `-D warnings`, `cargo test --workspace --locked` e `npm test`.
