D-2026-09-28-release-packaging-10 (2026-09-28): A versão do pacote é carimbada pelo CI, sem commit de bot.

- **No repositório**, `[workspace.package] version` passa a `"0.0.0"`, marca de build de desenvolvimento. É a mesma convenção do `helm-charts` citada no README do github-workflows ("o campo e `0.0.0` e o CI carimba"). O `Cargo.lock` acompanha. Um `ddc-cli --version` local diz `0.0.0`, e isso é o esperado.
- **`qualidade.yml` ganha o input `versao`** (string, default `""`) e o campo de componente `carimbar_versao` (boolean, só rust). Com os dois dados, um passo `Carimbar versao` roda ANTES do `Build <pacote> (release)`:
  - reescreve o `version` do `[workspace.package]` (ou do `[package]` sem workspace) do `Cargo.toml` da raiz do checkout, e só dele;
  - acerta o `Cargo.lock` para que `--locked` continue passando (`cargo update --workspace --offline`, ou o equivalente que o run medir);
  - a versão é validada por regex (`^[0-9]+\.[0-9]+\.[0-9]+$`) e entra por `env`;
  - o Tauri lê a versão do crate (o `tauri.conf.json` não tem `version`), então MSI, NSIS, deb, rpm e AppImage saem com ela;
  - um passo `Conferir versao dos binarios` roda `<binario> --version` de cada `binarios_extra` e reprova se a saída não contém a versão.
- **Limite do MSI:** major e minor até 255, patch até 65535. O passo reprova cedo se a versão estourar.
- **Em pull_request** a versão carimbada é a que o push VAI ter (`versao` só calcula), então os artefatos do PR já são os pacotes da próxima release.
