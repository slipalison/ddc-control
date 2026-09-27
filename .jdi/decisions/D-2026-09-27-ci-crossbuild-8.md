D-2026-09-27-ci-crossbuild-8 (2026-09-27): Nesta phase, `versao` só calcula (`versao_inicial: "0.1.0"`, igual ao `[workspace.package]`): nunca cria tag nem release. A documentação muda em dois lugares:
- o README do github-workflows (seção Rust: Windows, `pacotes_sistema`, `auditoria`, build de release) e um exemplo em `exemplos/` validado pelo actionlint do CI de lá;
- o README do ddc-control ganha uma seção de CI.

O texto é revisado por humano no PR (Deferred to PR review), e não por grep.
