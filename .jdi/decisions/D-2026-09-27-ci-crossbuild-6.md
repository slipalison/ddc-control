D-2026-09-27-ci-crossbuild-6 (2026-09-27): `.cargo/audit.toml` novo no ddc-control, com exatamente 4 entradas em `[advisories] ignore`: RUSTSEC-2018-0005, RUSTSEC-2024-0370, RUSTSEC-2024-0320 e RUSTSEC-2024-0429.
- Cada entrada tem comentário com: o caminho de dependência medido em `cargo tree -i`; por que não é alcançável pelo ddc-control; e quando reavaliar (troca do `ddc-hi`/`mccs-db`, do tauri/gtk etc.).
- Nenhum outro mecanismo pode esconder achado: sem `severity_threshold`, sem `informational_warnings` vazio para calar a vulnerabilidade, sem `--ignore` na linha de comando.
- `cargo audit`, rodado da raiz (lê `.cargo/audit.toml` sozinho), sai 0 hoje. Atenção: `--file` do cargo-audit é o `Cargo.lock`, não o config.
