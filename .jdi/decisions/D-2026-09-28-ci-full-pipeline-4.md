D-2026-09-28-ci-full-pipeline-4 (2026-09-28): A UI ganha piso de cobertura de 80%.

- `npm run test:unit` roda o `node --test` com cobertura (`--test-coverage-include="src/**"`) e escreve `coverage/lcov.info`. `npm test` = `test:unit` + Playwright.
- O componente `node-ui` passa a `"cobertura":80`.
- Medido em 2026-09-28: 88,15% de linhas, 161 testes.
- Antes, o job do `node-ui` saía verde com a anotação de erro "Nenhum relatorio de cobertura encontrado em: apps/ddc-tray/coverage/lcov.info" (run 36453793875).
- Limite escrito: o número cobre os módulos que os testes `node --test` carregam. O `app.js`, a cola com o DOM, é exercido pelo Playwright e fica fora dele. No Sonar ele aparece sem cobertura, o que é honesto.
