D-2026-09-28-release-packaging-9 (2026-09-28): O componente `rust-linux` roda em `ubuntu-22.04`, e não mais no `ubuntu-latest`, que hoje é o 24.04.

- **Por quê:** os pacotes Linux (deb, rpm e AppImage) ligam dinamicamente na glibc do sistema onde foram CONSTRUÍDOS. Construídos no 24.04 (glibc 2.39), não rodariam no Ubuntu 22.04, no Debian 12 (2.36) nem no Mint 21. No 22.04 (glibc 2.35), rodam nesses e em todos os mais novos (Fedora do usuário incluso).
- **O que muda junto:** testes, cobertura e audit do Linux passam a rodar no 22.04 no mesmo job. É um job a menos e o mesmo binário que vai para o pacote. O `webkit2gtk-4.1` existe no 22.04.
- **O `so: ubuntu-22.04` já é aceito** pela validação do `qualidade.yml` (D-2026-09-27-ci-crossbuild-2). O conjunto de testes de plataforma congelado na ci-crossbuild não muda, porque é o mesmo SO.
- **Quando revisar:** o GitHub aposenta os runners 22.04 perto do fim do suporte da Canonical (abril de 2027). Nessa hora, sobe para o próximo LTS e se reavalia.
