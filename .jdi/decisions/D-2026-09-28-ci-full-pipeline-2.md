D-2026-09-28-ci-full-pipeline-2 (2026-09-28): Segurança completa, com CodeQL.

- `seguranca.yml` inteiro, em todo run: SBOM, Trivy (SCA), Semgrep, Gitleaks e TruffleHog no histórico inteiro. `seguranca_na_main` fica no padrão (`true`), porque o ruleset não exige PR atualizado.
- `linguagens_codeql: '["rust","javascript-typescript","actions"]'`. O repositório é público, então o CodeQL é gratuito e o SARIF vai para a aba Security.
- `caminho_iac` vazio: não há Dockerfile, manifesto nem chart.
- Um achado bloqueante se resolve no código ou na dependência. Se o caminho vulnerável for inalcançável, a exceção vai para `.trivyignore` com a mesma justificativa medida do `.cargo/audit.toml` (D-2026-09-27-ci-crossbuild-6), nunca afrouxando o portão do template.
