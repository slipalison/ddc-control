D-2026-09-28-ci-full-pipeline-3 (2026-09-28): Sonar pelo CI, com Rust e cobertura.

- O projeto `slipalison_ddc-control` já existe no SonarQube Cloud, mas só com a Análise Automática. Ela mediu css, js, json, shell, web, xml e yaml, e nenhuma linha de Rust nem de cobertura.
- A análise passa ao CI: `sonar_projeto: slipalison_ddc-control`, `sonar_linguagem: rust` e `sonar_versao_linguagem: "1.98"`. O Clippy que o Sonar roda usa o mesmo toolchain.
- `sonar_pacotes_sistema: "libwebkit2gtk-4.1-dev libudev-dev"`: o Clippy compila o workspace. O librsvg só serve ao AppImage.
- `sonar_comando_testes: bash scripts/ci/sonar-coverage.sh` escreve dois relatórios:
  - `resultados/lcov.info`, do `cargo llvm-cov` do workspace;
  - `resultados/lcov-ui.info`, do `node --test` da UI, com os caminhos reescritos para a raiz.
- O script instala o `cargo-llvm-cov` 0.9.1 e o Node 24.18.0 com sha256 conferido, e só no CI (`CI=true`). Fora do CI, usa o que a máquina tem.
- Fontes e testes em conjuntos disjuntos: `**/tests.rs` e `**/tests/**` são teste.
- Pré-requisitos do usuário, fora do repositório: o secret `SONAR_TOKEN` e a Análise Automática desligada no projeto.
