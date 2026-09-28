D-2026-09-28-release-packaging-6 (2026-09-28): Conjunto congelado de categorias de pacotes da release — por padrão, não por nome literal.

Lição de D-2026-09-27-ci-crossbuild-5 (REVIEW iter 2): comparar só QUANTIDADE é fraco, mas fixar um nome literal exato de saída de ferramenta de terceiro (o bundler Tauri/WiX/rpmbuild/dpkg) é frágil a mudanças de versão da ferramenta que não são desta phase. A decisão fixa CATEGORIAS com um padrão verificável por categoria, e o conjunto das 7 categorias é fechado:

| Categoria | Artefato (`pacotes-*`) | Padrão | Ferramenta de inspeção |
|---|---|---|---|
| Debian | `pacotes-rust-linux` | `^ddc-control[_-].*\.deb$` | `ar t`/`ar p` + `tar` |
| RPM | `pacotes-rust-linux` | `^ddc-control[_-].*\.rpm$` | `rpm2cpio` + `cpio -t` |
| AppImage | `pacotes-rust-linux` | `^ddc-control.*\.AppImage$` | magic bytes `41 49 02` no offset 8 (AppImage tipo 2) |
| CLI Linux | `pacotes-rust-linux` | exato: `ddc-cli-x86_64-unknown-linux-gnu.tar.gz` | `tar tzf` |
| MSI | `pacotes-rust-windows` | `\.msi$` | `file` reporta `MSI Installer` |
| NSIS | `pacotes-rust-windows` | `-setup\.exe$` | `file` reporta instalador Nullsoft/NSIS |
| CLI Windows | `pacotes-rust-windows` | exato: `ddc-cli-x86_64-pc-windows-msvc.zip` | `unzip -l` |

**A prova do DoD é: exatamente 4 arquivos em `pacotes-rust-linux` e exatamente 3 em `pacotes-rust-windows`, cada um casando com EXATAMENTE uma categoria da tabela (nenhuma categoria vazia, nenhum arquivo sobrando fora de todas), comparado como CONJUNTO de categorias, não como contagem solta.** Um bundler que passe a gerar um segundo `.deb` (ex. `-dbgsym`), um pacote a menos, ou um arquivo extra não previsto reprova a linha — mesmo que a contagem total "pareça" razoável.

Nomes literais exatos (`ddc-control_0.1.0_amd64.deb`, etc.) não são fixados aqui porque dependem de decisões internas do bundler Tauri (transformação de espaço em `productName`, sufixo de arquitetura) que podem mudar entre versões do `tauri-cli` sem nenhuma mudança de código desta phase — fixá-los seria acoplar o DoD a um detalhe de terceiro que ninguém aqui decidiu.
