D-2026-09-28-ci-full-pipeline-5 (2026-09-28): O template muda no `github-workflows` #15, e é provado aqui antes do merge.

- O gw #15 (branch `esteira-app-desktop`) vem empilhado no #14. Ele traz:
  - o app desktop pelo `pipeline.yml`;
  - `pacotes_sistema` no `sonar.yml`;
  - o `Portao` reprovando Sonar desligado sem `sonar_dispensa` (BREAKING para o `coluna`);
  - a seção "O mínimo exigido" no README.
- O `pipeline.yml` chama os workflows aninhados `@main`, e a `main` do gw não tem nem o #14. Para a prova ser do código do PR, o `ci.yml` aponta para a branch descartável `teste/esteira-app-desktop` (`WORKFLOWS_TEST_SHA`): o head do #15 (`WORKFLOWS_SHA`) com os aninhados fixados nele. As composites ficam `@main`, e o #14 e o #15 não as tocam.
- Depois do merge dos dois no gw, o `uses:` passa a `@main`.
