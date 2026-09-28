D-2026-09-28-release-packaging-5 (2026-09-28): Contrato dos arquivos de binário cru do `ddc-cli`.

`binarios_extra: "ddc-cli"` (D-2026-09-28-release-packaging-3) produz exatamente dois arquivos, um por SO do CI, nomeados pelo par `$RUNNER_OS-$RUNNER_ARCH` já calculado em `qualidade.yml` para `cargo-llvm-cov`/`cargo-audit` (reaproveitado, nunca uma segunda tabela de alvo):

- Linux: `ddc-cli-x86_64-unknown-linux-gnu.tar.gz`, contendo exatamente dois membros: `ddc-cli` (o binário de `target/release/ddc-cli`, com permissão de execução preservada pelo `tar`) e `LICENSE` (cópia do `LICENSE` da raiz do repositório, MIT).
- Windows: `ddc-cli-x86_64-pc-windows-msvc.zip`, contendo exatamente dois membros: `ddc-cli.exe` (de `target/release/ddc-cli.exe`) e `LICENSE`.

Nenhum outro arquivo dentro do arquivo compactado (sem `README`, sem diretório extra, sem o binário do `ddc-tray`) — o conjunto é fechado e auditável (mesma filosofia de D-2026-09-26-full-osd-control-1 para o catálogo MCCS: só o que está listado aqui existe, extensão futura é decisão nova). `LICENSE` entra porque um binário standalone distribuído fora do gerenciador de pacote (o caso do `.tar.gz`/`.zip`, ao contrário do `.deb`/`.rpm` que carregam a licença no metadado do pacote) precisa da licença ao lado para ser redistribuível sem ambiguidade.

Rationale de nomear com o `<alvo>` completo (`x86_64-unknown-linux-gnu`) em vez de algo mais curto (`linux-x64`): é o mesmo vocabulário que `rustc --print target-list` e os nomes de artefato do `cargo-llvm-cov`/`cargo-audit` já usam nesta esteira — quem baixa da release já reconhece o formato de quem baixa toolchains Rust.
