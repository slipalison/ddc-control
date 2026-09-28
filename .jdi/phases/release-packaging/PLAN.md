# Phase 7: Release packaging — Plan  (slug: release-packaging)

## Goal
Bundles do Tauri (MSI/NSIS no Windows; deb/rpm/AppImage no Linux, com a regra udev) e os binários do `ddc-cli`, mais o release automático: versão pelos commits, tag e GitHub Release com os pacotes e as notas do CHANGELOG. Tudo pelos templates de `slipalison/github-workflows`.

## Locked decisions (from CONTEXT.md)
- -1: branch `pacotes-tauri-release` do gw empilhada sobre `origin/qualidade-rust-windows` (#13); `uses:` do `ci.yml` no SHA do head do PR novo; evidência em `release-evidence.env`.
- -2 (+emenda): `lancar.yml` ganha `artefatos`, `rascunho` e `changelog`, apaga o rascunho velho da mesma tag e anexa `SHA256SUMS`. O egresso é medido e fechado em `block`.
- -3: `qualidade.yml` ganha `empacotar_tauri`, `caminho_tauri`, `binarios_extra` e os passos de pacote, com artifact `pacotes-<nome>`.
- -4 (+emenda): deb e rpm levam a regra em `/usr/lib/udev/rules.d/` e o `i2c-dev` em `/usr/lib/modules-load.d/ddc-control.conf`, com pós-instalação best-effort e `/usr/bin/ddc-tray`. O AppImage fica sem a regra.
- -5: arquivos do CLI com exatamente `{ddc-cli[.exe], LICENSE}`. -6: 7 categorias fechadas, por padrão de nome.
- -7 (+emenda): gatilho `ensaio-release/**`; `qualidade` com `needs: [versao]`; job `lancar` literal, com rascunho fora da `main`.
- -8 (reescrita): PR com `lancar` skipped, mais um ensaio real por push em `ensaio-release/*`, que cria o RASCUNHO `v0.1.0`. Nenhuma tag `v*`.
- -9: `rust-linux` em `ubuntu-22.04` (glibc 2.35). -10: repositório em `0.0.0`; o CI roda `Carimbar versao` e `Conferir versao dos binarios`.

## Premissas (cadeia autônoma, sem perguntas)
- **A-1 `tauri-cli` 2.12.0**, a minor do `tauri` 2.12.0 do `Cargo.lock`. Binário pronto da release `tauri-cli-v2.12.0`, por alvo; nome e sha256 lidos de `gh release view tauri-cli-v2.12.0 --repo tauri-apps/tauri --json assets` (campo `digest`). Sem binário para o alvo, ou se ele não rodar no 22.04 → `blocked` (a D-3 proíbe `cargo install`).
- **A-2 `mainBinaryName: "ddc-tray"`** desde o início: o `kwin/anchor.js:12` casa `resourceClass === "ddc-tray"`, e o `.desktop` precisa de `Exec=ddc-tray` (D-4).
- **A-3 Nome publicado:** o prefixo do produto (antes de `[_-]<versao>[_-]`) vai para minúsculas, com `-` no lugar de espaço; o resto fica como o bundler gerou. Esperado: `ddc-control_<v>_amd64.deb`, `ddc-control-<v>-1.x86_64.rpm`, `ddc-control_<v>_amd64.AppImage`, `ddc-control_<v>_x64_en-US.msi`, `ddc-control_<v>_x64-setup.exe`. Casa a D-6 e os `Verify:`; nome sem a versão reprova o passo.
- **A-4 `dist/` plano** na raiz do checkout; o `Guardar pacotes` sobe só `dist/*`. Num workspace, o `target/` fica na raiz (via `cargo metadata`), e o caminho da D-3 não acharia nada. O `lancar` também precisa de arquivos soltos depois do `merge-multiple`.
- **A-5 O `lancar` baixa os pacotes ANTES da release** e os passa ao `gh release create`: com anexos, o `gh` cria rascunho, sobe tudo e só então publica, então uma release publicada nunca fica sem pacote. Substitui o `gh release upload` posterior da D-2, com o mesmo resultado; é desvio de mecanismo, registrado no SUMMARY.
- **A-6 Ensaio em dois tempos:** `git push origin origin/main:refs/heads/ensaio-release/v0.1.0-<sha7>` (o `ci.yml` da `main` não dispara aí) e depois `git push origin <HEAD_SHA>:refs/heads/ensaio-release/v0.1.0-<sha7>`. Assim `github.event.before` = `main`, e o `versao` julga só os commits da phase, como no push do merge; com o `before` zero, julgaria o histórico inteiro. Uma branch nova por tentativa.
- **A-7 Cache:** a chave e as `restore-keys` do `Cache do target` ganham `matrix.c.so || 'ubuntu-latest'`. Sem isso, o 22.04 restauraria o `target/` do 24.04 (glibc 2.39) e anularia a D-9.

## Regras de execução (todas as tasks)
- **gw** = `/home/slipalison/repos/github-workflows`, com `core.hooksPath hooks` ativo; commits `tipo(escopo): assunto` e comentários em pt-BR sem acento, com o porquê medido. Nunca tocar `.github/actions/**` nem `bin/**`: as composites rodam `@main` (DoD 2). Se o #13 for mergeado: `git rebase --onto origin/main 9e91d1a`, `--force-with-lease`, repin e evidência refeita.
- **ddc-control:** Conventional Commits em inglês, scope `release-packaging`, header ≤ 72, D-XX no corpo. O hook proíbe `.jdi/` com código/`.github/` e duas phases num commit. Nunca `--no-verify`, `JDI_ALLOW_MIXED` nem `JDI_GATE_DISABLE`. O PLAN.md é commitado antes do 1º commit de código (gate do pre-commit).
- **Trailers** em todo commit dos dois repos: `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` e `Claude-Session: https://claude.ai/code/session_01RHSZzNqTEuMKWTdwP57BRL`.
- **Local:** nunca `sudo`; nunca VCP em monitor real; nunca instalar pacote nem abrir o app (o AppImage só com `--appimage-extract`). actionlint 1.7.12 no scratchpad.
- **Gates em todo commit de código do ddc-control:** `cargo fmt --all --check`, clippy com `--all-features -D warnings`, `cargo test --workspace --locked`, cobertura ≥ 80 e `cd apps/ddc-tray && npm test`.
- **Runs:** nunca `gh run rerun` para evidência; nunca push enquanto um run de evidência não termina (`cancel-in-progress`); falha se lê com `--log-failed`.
- **Release:** nenhuma tag `v*` (nada de `--tags`, release à mão ou publicação); o rascunho do ensaio fica para o humano.
- **Verify:** sempre em `bash`, literal do CONTEXT. O crítico do DoD roda forçado: só se entrega com os 9 `OK`.

## Tasks

### Wave 1 (parallel-eligible: repositórios e arquivos disjuntos)

#### T-1 (gw): `qualidade.yml` empacota Tauri e binários extra, e carimba a versão
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `gw:.github/workflows/qualidade.yml`, `gw:README.md`, `gw:exemplos/ci-rust-desktop.yml`
- **Acceptance:**
  - `git switch -c pacotes-tauri-release origin/qualidade-rust-windows` (`9e91d1a`), depois do fetch. O input `versao` (default `""`) é a versão do PRODUTO; o `versao` do componente continua sendo a da linguagem, e o cabeçalho diz isso.
  - **`Conferir componente`** (valores por `env`, booleanos por `toJSON`) reprova: `empacotar_tauri`/`carimbar_versao` fora de rust ou fora de `true`/`false`; Tauri sem `build_release` ou `caminho_tauri`; `caminho_tauri` fora de `^[A-Za-z0-9_./-]+$` ou com `..`; `binarios_extra` fora de `^[A-Za-z0-9_-]+( [A-Za-z0-9_-]+)*$`; `carimbar_versao` com `versao` vazia ou fora de `^[0-9]+\.[0-9]+\.[0-9]+$`; com Tauri, major/minor > 255 ou patch > 65535 (MSI).
  - **Ordem, depois dos testes (nomes = contrato do DoD 4):** `Carimbar versao`, `Build <pacote> (release)`, `Build binarios extra (release)`, `Conferir versao dos binarios`, `Empacotar binarios extra`, `Instalar tauri-cli`, `Empacotar <pacote> (Tauri)`, `Guardar pacotes`. O nome do passo Tauri sai de `format()`, com texto fixo sem o campo (W-5).
  - **`Carimbar versao`:** awk só no `version` do `[workspace.package]` (ou `[package]`) do `Cargo.toml` da raiz; lock acertado offline, pelo comando que o run medir. Imprime o `git diff` e reprova se o lock mudar algo além das linhas `version` dos membros.
  - **`Conferir versao dos binarios`:** `target/release/<bin>[.exe] --version` contém a versão; no Windows, executa o `.exe`.
  - **`Empacotar binarios extra`:** `dist/<bin>-<alvo>.tar.gz` (Linux) ou `.zip` (Windows), com exatamente `{<bin>[.exe], LICENSE}` na raiz, sem `./` nem diretório. A tabela `$RUNNER_OS-$RUNNER_ARCH` → alvo existe uma vez só, num passo com `id` lido pelos outros (D-5). No Git Bash não há `zip`: `7z` ou o `tar.exe` do Windows, o que o run medir.
  - **`Instalar tauri-cli`:** A-1, no padrão do cargo-llvm-cov.
  - **`Empacotar <pacote> (Tauri)`:** `cargo tauri build --bundles "$BUNDLES"` (`deb,rpm,appimage` ou `msi,nsis` por `$RUNNER_OS`), com `--locked` repassado ao cargo. Copia para `dist/` (A-3, A-4), exige 1 arquivo por bundler pedido e reprova se sobrar espaço no nome.
  - **`Guardar pacotes`:** upload-artifact (SHA do lock), `pacotes-<nome>`, `dist/*`, `if-no-files-found: error`. Cache: A-7. **Retrocompatível:** sem os campos novos, os passos novos ficam pulados e o resto não muda.
  - **Local:** `python3 bin/pinar_actions.py --verificar` OK (lock intocado); actionlint limpo; o `Conferir componente` extraído do YAML roda em ≥ 12 combinações, inclusive valor com quebra de linha.
  - **Docs:** README com a tabela de campos e a seção "Empacotamento Tauri e versão carimbada"; exemplo com os campos e o `ubuntu-22.04` (com o porquê). **Commit:** `feat(qualidade): empacota tauri, binarios extra e carimba a versao`.
- **Dependencies:** none
- **Test:** script extraído + pins + actionlint; DoD 3/4 na T-5
- **Status:** completed

#### T-2: arquivos de `packaging/linux` e bundle do `tauri.conf.json`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `packaging/linux/60-ddc-control-i2c.rules`, `packaging/linux/ddc-control-i2c-dev.conf`, `packaging/linux/postinstall.sh`, `apps/ddc-tray/src-tauri/tauri.conf.json`
- **Acceptance:**
  - O `.rules` é só a linha da regra de `docs/linux-ddc-setup.md:46` + `\n` (D-4). O `.conf` é `i2c-dev\n`.
  - **`postinstall.sh`** (POSIX sh): `modprobe i2c-dev`, `udevadm control --reload` e `udevadm trigger --subsystem-match=i2c-dev`, cada um com `2>/dev/null || true`, e `exit 0` no fim. `sh -n` passa, e `env -i PATH=/nonexistent /bin/sh packaging/linux/postinstall.sh` sai 0.
  - **`tauri.conf.json`:** `mainBinaryName: "ddc-tray"` (A-2), `bundle.active: true` e `bundle.license: "MIT"` (para o rpm). Em `bundle.linux.deb` e `.rpm`: `files` com os 2 destinos da D-4 (`../../../packaging/linux/...`) e `postInstallScript: "../../../packaging/linux/postinstall.sh"`. `productName`, `identifier`, ícones e CSP intactos.
  - `cargo build -p ddc-tray --locked` aceita o config (o tauri-build valida). Gates verdes. **Commit:** `feat(release-packaging): bundle packages with udev rule and i2c-dev`.
- **Dependencies:** none
- **Test:** gates locais + os dois comandos do `postinstall.sh`; DoD 3 na T-5
- **Status:** completed

### Wave 2

#### T-3 (gw): `lancar.yml` com pacotes, somas e changelog, e o PR do gw
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `gw:.github/workflows/lancar.yml`, `gw:README.md`, `gw:exemplos/ci-rust-desktop.yml`
- **Acceptance:**
  - **Inputs** `artefatos` (`""`), `rascunho` (`false`) e `changelog` (`""`). Sem eles, os passos e comandos são os de hoje, e o `ci.yml` do gw não muda.
  - **`Conferir as entradas` do job `lancar`:** `env` `VERSAO`, `TAG`, `ARTEFATOS`, `CHANGELOG` (contrato do DoD 8); `[[ =~ ]]` com `^[A-Za-z0-9_.*-]+$` para `artefatos`, e `^[A-Za-z0-9_./-]+$` sem `..` para `changelog`. Sem arquivo, sem rede e sem outra variável: o DoD 8 o roda sozinho, sob `bash -c`.
  - O passo homônimo do job `alias` vira `Conferir imagem e digest`, e `grep -c 'name: Conferir as entradas'` = 1: o awk do DoD 8 pega a 1ª ocorrência, que hoje é a linha 91, no `alias`.
  - **Ordem (A-5):** (1) `Baixar pacotes`: download-artifact v8.0.1 (SHA do lock), `pattern`, `merge-multiple: true`, `path: pacotes-baixados`. (2) `Somas SHA256`: `SHA256SUMS` feito de dentro da pasta, só com nomes. (3) `Montar as notas`: `## [<versao>]` ou, sem ela, `## [Unreleased]`, por awk, seguido das notas do `versao`; arquivo ausente reprova. (4) `Apagar rascunho velho da mesma tag`: `draft` e `tag_name == TAG`, pelo id; publicada nunca. (5) `Criar a tag e a release`: com os arquivos e `SHA256SUMS`, mais `--draft` quando `rascunho`. (6) `Resumo`, com o id e a URL.
  - O `timeout-minutes` é ajustado ao tempo medido. Egresso do job: `egresso: audit` PROVISÓRIO, com comentário apontando a T-6.
  - **Local:** Verify 8 com `WORKFLOWS_SHA` = o commit local dá `OK`; actionlint e pins limpos.
  - **Docs:** README com os inputs do `lancar` e a seção "Pacotes na release" (rascunho, somas, changelog, rascunho velho); exemplo com o job `lancar` da D-7 e o porquê de `ensaio-release/**`.
  - **Commit e PR:** `feat(lancar): anexa pacotes, somas e changelog a release`; push `-u`; `gh pr create --repo slipalison/github-workflows --base main`. Corpo em pt-BR, abrindo com "inclui o #13; mergear o #13 antes", dizendo o que muda e por que é retrocompatível, com uma tabela de runs por head (T-5/T-6).
  - **CI do gw verde no head:** `Scripts`, `Pins das actions`, `Sintaxe dos workflows` e `versao`.
- **Dependencies:** T-1 (mesmo README e exemplo)
- **Test:** Verify 8 local + CI do gw
- **Status:** completed

### Wave 3

#### T-4: `ci.yml`, workspace em `0.0.0` e o PR rascunho
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`
- **Acceptance:**
  - **D-7:** `push.branches: [main, 'ensaio-release/**']`; `qualidade` com `needs: [versao]` e `versao: ${{ needs.versao.outputs.versao }}`; job `lancar` literal da D-7, com o mesmo `versao_inicial: "0.1.0"` do `versao`.
  - **Componentes:** `rust-linux` e `rust-windows` recebem `empacotar_tauri: true`, `caminho_tauri: apps/ddc-tray/src-tauri`, `binarios_extra: ddc-cli` e `carimbar_versao: true`; o `rust-linux` vai para `so: ubuntu-22.04` (D-9); `node-ui` igual.
  - **Pins e comentários:** os 3 `uses:` em `@<WORKFLOWS_SHA>` (40 caracteres, head do PR da T-3). Comentários em inglês com o porquê: glibc, o ensaio, `packages: write` estático e a troca para `@main` depois do merge (Deferred).
  - **D-10:** `[workspace.package] version = "0.0.0"`. O `Cargo.lock` sai do cargo, com só as 4 linhas `version` dos membros. `cargo run -q -p ddc-cli -- --version` imprime `0.0.0`. actionlint limpo e gates verdes.
  - **Commit e PR:** `ci(release-packaging): build packages and release from CI`; `git push -u origin phase/release-packaging`; `gh pr create --draft --base main --title "feat(release-packaging): packages and automatic GitHub Release"`, com corpo curto em inglês e a linha de atribuição.
  - O 1º run nasce com `versao`, os 3 jobs `qualidade /` e `lancar` skipped, sem `startup_failure`.
- **Dependencies:** T-2, T-3
- **Test:** 1º run criado
- **Status:** completed

### Wave 4

#### T-5: convergência contra os runs reais de PR
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** definidos pelos runs, só em `gw:.github/workflows/qualidade.yml`, `.github/workflows/ci.yml` (o `@SHA` e o `pacotes_sistema`, com o porquê de cada pacote), `apps/ddc-tray/src-tauri/tauri.conf.json` e `packaging/linux/*`
- **Acceptance:**
  - **Laço:** push → `gh run watch --exit-status` → `--log-failed` → correção. As falhas de um run se corrigem juntas: vários commits, um push. Cada correção é 1 commit: `fix(release-packaging): …`, ou `fix(qualidade): …` no gw seguido de `ci(release-packaging): pin workflows to <sha7>`. O SUMMARY registra run → falha → correção → commit.
  - **Esperado:** nome do binário (A-2), dependências do deb, AppImage no 22.04 (linuxdeploy, FUSE, `librsvg2-dev`/`file`), NSIS e zip no Windows. WiX com VBScript no `windows-latest`: primeiro um passo no template; só sem saída, `so: windows-2022`, como desvio medido da D-7.
  - **Não vale:** pular bundler, `continue-on-error`, `|| true` nos passos de pacote, afrouxar `if-no-files-found` ou tirar `--locked`.
  - **Fim:** o run de PR fica verde. Com um `release-evidence.env` provisório, não commitado, os Verify 1, 3, 4 e 8 dão `OK` em `bash`. `objdump -T` do `usr/bin/ddc-tray` do deb e do `ddc-cli` do tar.gz não mostra nenhum `GLIBC_` > 2.35 (D-9). O corpo do PR do gw ganha os runs.
- **Dependencies:** T-4
- **Test:** ensaio dos Verify 1, 3, 4 e 8 + `objdump`
- **Status:** completed

### Wave 5 (parallel-eligible: arquivos disjuntos)

#### T-6: medir o egresso do `lancar` e fechar em `block` (D-2)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `gw:.github/workflows/lancar.yml`, `gw:README.md` (Segurança e `lancar`), `.github/workflows/ci.yml` (só o repin)
- **Acceptance:**
  1. **Ensaio em `audit`:** push em dois tempos (A-6) do HEAD atual. O run sai verde e o workflow cria o rascunho `v0.1.0`.
  2. **Leitura:** os `host:porta` do job `lancar / Tag e release` vêm do insights do StepSecurity, pelo link do harden-runner no log ou no resumo (WebFetch ou curl); se não der, do log do pós-passo. O SUMMARY registra o run, a fonte e a lista.
  3. **gw:** commit `fix(lancar): fecha o egresso com a lista medida`, com `egresso: block` e só hosts medidos. `uploads.github.com`, `results-receiver.actions.githubusercontent.com` e `*.blob.core.windows.net` são candidatos, não fatos. Cada host tem um comentário YAML acima da chave, dizendo qual passo o usa; nunca dentro do bloco `|`, onde viraria endpoint. O README diz o mesmo.
  4. **Repin e novo ensaio:** `ci(release-packaging): pin workflows to <sha7>` e push. Novo ensaio, numa branch nova, em `block`: verde, sem chamada bloqueada, com o rascunho recriado e o anterior apagado pelo próprio workflow. Se algo for bloqueado, entra o host medido e repete.
  - Com o env provisório, os Verify 2, 5, 6 e 7 dão `OK`.
- **Dependencies:** T-5
- **Test:** ensaio dos Verify 2, 5, 6 e 7
- **Status:** completed

#### T-7: README, doc do Linux e CHANGELOG
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `docs/linux-ddc-setup.md`, `CHANGELOG.md`
- **Acceptance:**
  - **README (en), Install/Download:** página de Releases com os 7 arquivos por categoria; `sha256sum -c --ignore-missing SHA256SUMS`; instaladores sem assinatura (aviso do SO); deb e rpm já trazem regra, módulo e pós-instalação, e o AppImage não (vale o doc); o `cargo install` fica.
  - **README, resto:** Status e "no installer yet" atualizados. Na seção CI: a `main` publica, `ensaio-release/*` gera rascunho e PR nunca publica; o `0.0.0` é carimbado, e o `--version` local diz `0.0.0`; o 22.04 é pela glibc.
  - **`docs/linux-ddc-setup.md`:** nos passos 2 e 3, o deb e o rpm já fazem isso (com os caminhos); AppImage e build pelo cargo pedem o passo manual. A regra do doc continua idêntica.
  - **CHANGELOG `[Unreleased]`:** Added (pacotes, release, regra/módulo empacotados, arquivos do CLI, `SHA256SUMS`) e Changed (`0.0.0` carimbado, 22.04, a linha "(no installer yet)"). Cada item numa linha `- `, porque o DoD 6 compara linha a linha.
  - Nenhuma palavra todo/fixme (DoD 9). Nome e número sempre de um run. **Commit:** `docs(release-packaging): document packages and releases`.
- **Dependencies:** T-5 (nomes medidos)
- **Test:** Verify 9 + revisão humana (Deferred)
- **Status:** completed

### Wave 6

#### T-8: sequência final de evidência
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.jdi/phases/release-packaging/release-evidence.env` (+ status do PLAN/SUMMARY, no mesmo commit só de `.jdi/`)
- **Acceptance:**
  1. **Pré-condição:** T-5..T-7 fechadas e nada pendente de push. O head do PR do gw é o `WORKFLOWS_SHA`, com o CI verde, sem check falho nem cancelado, e o `ci.yml` aponta para ele.
  2. **Run de PR:** push, e o run no `HEAD_SHA` termina `success` sem push no meio. Guardar `RUN_ID` e `HEAD_SHA`.
  3. **Ensaio:** push em dois tempos (A-6) do MESMO `HEAD_SHA` numa `ensaio-release/v0.1.0-<sha7>`. O run dá `success`, e pela API sobra um único rascunho com `tag_name` `v0.1.0`: o id dele é o `DRAFT_RELEASE_ID`.
  4. **Env:** `KEY=VALUE`, sem aspas nem espaços, com `REPO VERSAO RUN_ID HEAD_SHA WORKFLOWS_BRANCH WORKFLOWS_PR WORKFLOWS_SHA ENSAIO_BRANCH ENSAIO_RUN_ID DRAFT_RELEASE_ID`. Sem `DRAFT_TAG`.
  5. **Verify:** os 9 `Verify:` do CONTEXT, literais, em `bash`, da raiz, dão 9 `OK` antes do commit. A baseline do PROJECT também passa.
  6. **Commit e limpeza:** `docs(release-packaging): record release evidence`, só `.jdi/`; push (o run que ele dispara não é evidência); apagar as branches `ensaio-release/*`. O rascunho FICA: sem tag e sem publicação.
  - **Depois do `HEAD_SHA`:** nada muda em `crates apps packaging Cargo.* .cargo .github CHANGELOG.md LICENSE`, nas configs de lint nem no toolchain. README e docs também entram antes.
- **Dependencies:** T-6, T-7
- **Test:** os 9 `Verify:` do DoD
- **Status:** completed

## Execution
- 8 tasks em 6 waves. W1 e W5 são paralelas (arquivos disjuntos); speedup ≈ 1.3x. T-5, T-6 e T-8 fazem vários commits de propósito, cada um atômico.
- **DoD × task:** 1 → T-4/5/8 · 2 → T-1/3/6/8 · 3 → T-1/2/4/5 · 4 → T-1/4/5 · 5 → T-3/4/6/8 · 6 → T-3/6/7/8 · 7 → T-8 (todas) · 8 → T-3 · 9 → todas, T-7.

## Riscos
- **R-1** `tauri-cli` sem binário ou com glibc nova demais (A-1). **R-2** WiX no `windows-latest` (T-5). **R-3** AppImage no 22.04: o linuxdeploy vem do GitHub, e o egresso do `qualidade` é `audit`.
- **R-4** `versao` numa branch nova julga todo o histórico (A-6). **R-5** Cache do 24.04 no 22.04 (A-7 + `objdump`).
- **R-6** `cancel-in-progress` durante a evidência; o ensaio roda em outro ref, portanto noutro grupo. **R-7** Merge do #13 no meio: rebase, novo `WORKFLOWS_SHA` e evidência refeita.
- **R-8** Corpo da release (CHANGELOG + notas do histórico) abaixo de 125 mil caracteres, medido no ensaio. **R-9** Cada ensaio é um CI inteiro: correções agrupadas e Verify ensaiados antes de cada push.

## Files modified (all tasks)
- **ddc-control:** `packaging/linux/{60-ddc-control-i2c.rules,ddc-control-i2c-dev.conf,postinstall.sh}`, `apps/ddc-tray/src-tauri/tauri.conf.json`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `README.md`, `docs/linux-ddc-setup.md`, `CHANGELOG.md`, `.jdi/phases/release-packaging/release-evidence.env`.
- **gw:** `.github/workflows/{qualidade,lancar}.yml`, `README.md` e `exemplos/ci-rust-desktop.yml`. O `actions.lock.json` fica intocado: o download-artifact v8.0.1 já está nele.

## Test requirements
- **Rust:** `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --locked`.
- **Cobertura:** `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`.
- **UI:** `cd apps/ddc-tray && npm test` + Gate 7 (`npm ci --ignore-scripts && npx playwright test`).
- **gw:** `python3 bin/pinar_actions.py --verificar`, actionlint 1.7.12 e os scripts de validação extraídos do YAML.
- **CI:** os 9 `Verify:` da CONTEXT dão `OK`, em `bash`, contra o `release-evidence.env` commitado. Mínimo de cobertura: 80% de linhas (PROJECT.md).
