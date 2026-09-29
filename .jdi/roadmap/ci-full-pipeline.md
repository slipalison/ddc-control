---
order: 5.8
name: CI full pipeline
---
- **Slug:** ci-full-pipeline
- **Goal:** o `ci.yml` passa a ser uma chamada só ao `pipeline.yml` do `slipalison/github-workflows`, com a esteira inteira: qualidade (a UI ganha piso de cobertura), segurança (Gitleaks, TruffleHog, Semgrep, Trivy, SBOM e CodeQL de Rust, JS/TS e Actions), Sonar com Quality Gate (Rust pelas regras do Sonar e LCOV, JS por LCOV) e o `Portao` como check único; no template, app desktop entra pelo `pipeline.yml` e o Sonar só se desliga com motivo escrito
- **Reason:** pedido do usuário, 2026-09-28: "o CI [...] está sem test, validação de segurança, sonarqube [...] seguir os padrões de validação e de qualidade que o github-workflow exige (se não estiver exigindo deixe [...] claro a exigência)"
