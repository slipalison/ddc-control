# Phase 6: CI cross-build — Plan  (slug: ci-crossbuild)

## Goal
GitHub Actions no `slipalison/ddc-control`, a cada PR e push na `main`, em `ubuntu-latest` e `windows-latest`: fmt, clippy `-D warnings`, testes com cobertura (piso de 80% no Linux), build release do `ddc-tray` nos dois SOs, `cargo audit` com exceções justificadas e a suíte da UI (`node --test` + Playwright). Tudo pelos templates do `slipalison/github-workflows`, estendidos lá e nunca copiados para cá.

## Locked decisions (from CONTEXT.md)
- ci-crossbuild-1: um único caller, `.github/workflows/ci.yml`: gatilhos `push` na `main`, `pull_request` e `workflow_dispatch`; `concurrency` com `cancel-in-progress`; `permissions: contents: read`; jobs `versao` e `qualidade`. Durante a phase, os `uses:` ficam em `@<WORKFLOWS_SHA>`, o head do PR do gw. PR rascunho aberto cedo. Evidência em `ci-evidence.env`.
- -2: `qualidade.yml` ganha campos opcionais e retrocompatíveis: `so`, `pacotes_sistema` (validado por regex, passado por `env`, só Linux), `cargo-llvm-cov` 0.9.1 por SO, `auditoria` (passo `cargo audit`, binário fixo) e build de release (passo `Build <pacote> (release)`). A mudança fica só no arquivo de workflow: composites e `bin/` não mudam.
- -3: componentes `rust-linux` (80%, pacotes do Tauri, auditoria, release), `rust-windows` (`so: windows-latest`, cobertura 0, release, sem auditoria) e `node-ui` (`apps/ddc-tray`, `npm test` = `node --test` TAP + Playwright; `--with-deps` só no runner).
- -4: `cargo build -p ddc-tray --release --locked` nos dois SOs. -5: fix do tauri#13419 no `build.rs`, decidido por `CARGO_CFG_TARGET_OS`/`_ENV`.
- -6: `.cargo/audit.toml` com exatamente 4 `ignore` comentados. -7: prova do negativo no CI real (commit só em `audit.toml`) e local (worktree). -8: `versao` só calcula; os READMEs dos dois repositórios ganham as seções novas.

## Assumptions (cadeia autônoma, sem perguntas)
- **A-1** Campo de build de release no template: `build_release` (valor = nome do pacote).
- **A-2** Componentes rust com `versao` = a minor do `rustc --version` local (esperado 1.98), para o clippy do CI ser o mesmo do local.
- **A-3** `node-ui` com `versao: "24"`: o README diz que o `node --test` roda em Node 24, e o glob em `--test` exige Node ≥ 21.
- **A-4** Os browsers do Playwright são instalados por `apps/ddc-tray/scripts/playwright-browsers.mjs`, chamado pelo `npm test`. `--with-deps` só com `GITHUB_ACTIONS=true` e `RUNNER_ENVIRONMENT=github-hosted`. Fora disso, `playwright install chromium` no cache do usuário, sem sudo (D-3).
- **A-5** `timeout: 60` no job `qualidade`: o Tauri compila 3 vezes no Windows (clippy, llvm-cov, release), e o 1º run não tem cache.
- **A-6** `defaults.run.shell: bash` no `qualidade.yml`, porque o shell padrão do Windows é o pwsh e os scripts usam `set -euo pipefail`.
- **A-7** As composites (`preparar`, `relatar-cobertura`) rodam sempre `@main`, então nenhum SHA de PR as exercita. Um problema de Windows se contorna no `qualidade.yml`: por exemplo, se faltar `python3` no Git Bash, entra um passo só-Windows antes de `Cobertura`. Se não houver contorno no workflow, a T-6 fica `blocked`, com o motivo medido (D-2).
- **A-8** Branch do gw: `qualidade-rust-windows`, criada a partir de `origin/main`.

## Regras de execução (todas as tasks)
- **Dois repositórios.** `gw:` = `/home/slipalison/repos/github-workflows/`. Antes do 1º commit lá: `git -C /home/slipalison/repos/github-workflows config core.hooksPath hooks`. Commits `tipo(escopo): assunto`, pt-BR sem acento, ≤ 72 caracteres; comentários em pt-BR sem acento, com o porquê medido.
- **ddc-control.** Conventional Commits em inglês, scope `ci-crossbuild`, header ≤ 72 e D-XX no corpo. Nunca `.jdi/` junto de código/`.github/`. Nunca `--no-verify`, `JDI_ALLOW_MIXED` nem `JDI_GATE_DISABLE`. O `revert` não é tipo aceito pelo hook: reversão = `ci(ci-crossbuild): …`.
- **Trailers.** Todo commit, nos dois repositórios, termina com `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` e `Claude-Session: https://claude.ai/code/session_01RHSZzNqTEuMKWTdwP57BRL`.
- **Máquina local.** Nunca `sudo`; nunca VCP em monitor real (esta phase não roda teste de hardware). O actionlint 1.7.12 fica no scratchpad.
- **Gates locais em todo commit de código do ddc-control:** `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --locked`; cobertura ≥ 80 (ver Test requirements); `node --test`; Gate 7 quando `apps/ddc-tray` mudar; `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` quando `crates/` mudar.
- **Runs.** Nunca `gh run rerun` para gerar evidência: o reutilizável fica preso à resolução original (README do gw). Nunca dar push enquanto um run que vai virar evidência não terminou, porque o `cancel-in-progress` o cancela. Falhas se leem com `gh run view <id> --log-failed`.
- **Custo do verify.** O crítico do DoD roda forçado a cada verify, então o doer só entrega depois do ensaio dos 9 `Verify:` com `OK` (8 até a iteração 1).

## Tasks
Specialist único: `jdi-doer-ddc-control` (glob `**/*`), que também cuida dos arquivos `gw:`.

### Wave 1 (parallel-eligible)

#### T-1: `.cargo/audit.toml` com as 4 exceções justificadas
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.cargo/audit.toml`
- **Acceptance:**
  - Só `[advisories]`, com `ignore` = RUSTSEC-2018-0005, -2024-0370, -2024-0320 e -2024-0429. Nenhuma outra chave (`severity_threshold`, `informational_warnings`…).
  - Uma entrada por linha, com vírgula final. Acima de cada uma, um comentário com o caminho medido em `cargo tree -i <crate> --target all --locked`, por que o ddc-control não alcança o código afetado e quando reavaliar. O ID 2018-0005 só aparece no bloco dele: tirá-lo é só apagar linhas, e o TOML continua válido (T-8, DoD 7 e 8).
  - O Verify do DoD 8 imprime `OK` localmente.
- **Dependencies:** none
- **Test:** Verify do DoD 8 · commit `ci(ci-crossbuild): add cargo audit policy with justified ignores`
- **Status:** completed

#### T-2: Manifesto do Windows embutido em todo binário do `ddc-tray` (tauri#13419)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/build.rs`, `apps/ddc-tray/src-tauri/windows-app-manifest.xml`
- **Acceptance:**
  - O `.xml` é cópia de `~/.cargo/registry/src/*/tauri-build-2.7.0/src/windows-app-manifest.xml` (Common-Controls v6 + DPI): o binário de release continua com o mesmo manifesto.
  - O `build.rs` usa `windows_attributes(WindowsAttributes::new_without_app_manifest())`. Só com `CARGO_CFG_TARGET_OS == "windows"` e `CARGO_CFG_TARGET_ENV == "msvc"` (lidos com `std::env::var`, nunca por `#[cfg(windows)]`), emite `cargo::rustc-link-arg=/MANIFEST:EMBED`, `cargo::rustc-link-arg=/MANIFESTINPUT:<CARGO_MANIFEST_DIR>/windows-app-manifest.xml` e o `rerun-if-changed` do arquivo. Sem `unwrap`/`expect`: erro → `ExitCode::FAILURE`. Um comentário cita tauri#13419 e a D-5.
  - No Linux nada muda: `cargo build -p ddc-tray --locked -vv 2>&1 | grep -c MANIFEST` = 0, e os gates locais seguem verdes. A prova no Windows é o DoD 5 (T-6).
- **Dependencies:** none
- **Test:** gates locais + DoD 5 · commit `fix(ci-crossbuild): embed the Windows app manifest in test binaries`
- **Status:** completed

#### T-3: `npm test` roda `node --test` e o Playwright, sem sudo local
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/package.json`, `apps/ddc-tray/scripts/playwright-browsers.mjs`, `apps/ddc-tray/tests/ui/playwright-browsers.test.mjs`
- **Acceptance:**
  - O script `test` roda, em ordem: `node --test --test-reporter=tap "tests/ui/**/*.test.mjs"` (glob entre aspas, expandido pelo Node), `node scripts/playwright-browsers.mjs` e `playwright test`. Sem pre/post-script; `devDependencies` e lock intocados.
  - `playwright-browsers.mjs` exporta a função pura `installArgs(env)` (A-4). Chama o `playwright` por `spawnSync` com array de argumentos, sem shell, e propaga o exit code. O teste fixa por igualdade as 3 combinações: local, self-hosted e github-hosted.
  - Local, `cd apps/ddc-tray && npm test` (sem sudo) dá TAP com `# fail 0` e `# pass` ≥ 158, e Playwright com ≥ 138 `passed`.
- **Dependencies:** none
- **Test:** `npm test` + Gate 7 · commit `test(ci-crossbuild): run node --test and Playwright from npm test`
- **Status:** completed

#### T-4 (gw): `qualidade.yml` estendido, README e exemplo, num PR no github-workflows
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `gw:.github/workflows/qualidade.yml`, `gw:README.md`, `gw:exemplos/ci-rust-desktop.yml`
- **Acceptance:**
  - `git fetch origin && git switch -c qualidade-rust-windows origin/main`, com o hook ativo.
  - **Workflow** (D-2, A-1, A-6, A-7):
    - `runs-on: ${{ matrix.c.so || 'ubuntu-latest' }}`.
    - 1º passo, `Conferir componente` (valores por `env`): reprova com `::error::` um `so` fora de `ubuntu-*`/`windows-*`, `pacotes_sistema` fora da regex da D-2 ou fora de Linux, `auditoria` fora de rust + Linux, e pacote de release fora de `^[A-Za-z0-9_-]+$`.
    - `Instalar pacotes do sistema`: `apt-get`, só em Linux e só se o campo estiver preenchido.
    - `Instalar cargo-llvm-cov` 0.9.1: o `.tar.gz` do SO (`x86_64-unknown-linux-gnu` ou `x86_64-pc-windows-msvc`, os dois publicados), conferido pelo sha256 do `digest` da release antes de extrair em `${CARGO_HOME:-$HOME/.cargo}/bin`.
    - `cargo audit`: cargo-audit 0.22.2 (`cargo-audit-x86_64-unknown-linux-gnu-v0.22.2.tgz`), sha256 do `digest` da release `cargo-audit/v0.22.2`. Sem `cargo install`, `latest`, `|| true` ou `--ignore`. Roda no `caminho` e lê o `.cargo/audit.toml` do chamador.
    - `Build <pacote> (release)`: `cargo build -p "$PACOTE" --release --locked`.
    - Os nomes de passo são exatos: são o contrato do DoD 3.
  - **Retrocompatível:** um componente sem os campos novos roda os mesmos passos e comandos em `ubuntu-latest`. A única diferença é o `shell: bash` explícito, que liga o `pipefail`; os `run:` com pipe já fazem `set -euo pipefail`, e um comentário registra isso. O cabeçalho documenta os campos novos.
  - **README:** tabela de campos; seção Rust com Windows, `pacotes_sistema`, `auditoria`, build de release e o porquê de binário fixo com sha256; o exemplo novo na lista do topo. **Exemplo:** app Rust/Tauri em Linux e Windows + UI node, com `versao` e `qualidade` em `@main`, comentado.
  - **Local:** `python3 bin/pinar_actions.py --verificar` OK (sem action nova, `actions.lock.json` intocado); actionlint limpo em `.github/workflows/*.yml exemplos/*.yml`; o laço "exemplos apontam para workflows que existem" passa.
  - **Commit e PR:** 1 commit `feat(qualidade): …`, push, `gh pr create --repo slipalison/github-workflows --base main` (corpo em pt-BR). O CI de lá fica verde no head: `Scripts`, `Pins das actions`, `Sintaxe dos workflows` e `versao`.
- **Dependencies:** none
- **Test:** CI do gw + DoD 2
- **Status:** completed

### Wave 2

#### T-5: `.github/workflows/ci.yml` e o PR rascunho
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.github/workflows/ci.yml`
- **Acceptance:**
  - **D-1:** gatilhos, `concurrency` e `permissions`; `versao` = `versao.yml@<WORKFLOWS_SHA>` com `versao_inicial: "0.1.0"`; `qualidade` = `qualidade.yml@<WORKFLOWS_SHA>` com `timeout: 60`. O SHA tem 40 caracteres e é o head do PR da T-4. Comentários em inglês com o porquê, inclusive a troca para `@main` depois do merge do gw (Deferred).
  - **Componentes (D-3/-4, A-2/-3):**
    - `rust-linux`: `cobertura: 80`, `auditoria: true`, `build_release: "ddc-tray"` e `pacotes_sistema` mínimo — parte da lista da D-3, com o porquê de cada pacote em comentário; sai o que nenhum crate usa (ex.: appindicator, porque o Linux não tem `tray-icon`), quando um run verde provar.
    - `rust-windows`: `so: windows-latest`, `cobertura: 0`, release `ddc-tray`.
    - `node-ui`: `linguagem: node`, `versao: "24"`, `caminho: apps/ddc-tray`.
  - **Publicação:** actionlint local limpo; commit; `git push -u origin phase/ci-crossbuild`; `gh pr create --draft --base main --head phase/ci-crossbuild --title "feat(ci-crossbuild): CI on Linux and Windows via reusable workflows"`, com corpo curto em inglês e a linha de atribuição. O run de `pull_request` nasce com `versao` e `qualidade / {rust-linux,rust-windows,node-ui}`, sem `startup_failure`.
- **Dependencies:** T-1, T-2, T-3, T-4
- **Test:** 1º run criado · commit `ci(ci-crossbuild): run CI on Linux and Windows via reusable workflows`
- **Status:** completed

### Wave 3

#### T-6: Convergência contra os runs reais (Windows e Linux headless)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** definidos pelos runs, restritos a `gw:.github/workflows/qualidade.yml`; `.github/workflows/ci.yml` (o `@SHA` e `pacotes_sistema`); `apps/ddc-tray/src-tauri/**` (código `cfg(not(linux))` que nunca compilou: `tray/notification_area.rs`, `tray.rs`, `lib.rs`, `menu.rs`); `crates/*/src/**` e `crates/*/tests/**` (testes que dependem de SO, cache/`LOCALAPPDATA`); `Cargo.lock` só no caso do R-3.
- **Acceptance:**
  - **O laço:** push → `gh run watch <id> --exit-status` (ou poll de `gh run view --json status`) → `--log-failed` → correção. As falhas de um run se corrigem juntas: vários commits, um push. Tarefa de convergência: cada correção é 1 commit atômico — `fix(ci-crossbuild): …` aqui ou `fix(qualidade): …` no gw.
  - Uma correção no gw leva a push lá e, aqui, a `ci(ci-crossbuild): pin workflows to <sha7>` atualizando todos os `@` do `ci.yml`. O SUMMARY registra run → falha → correção → commit.
  - **O que não vale:** desligar teste ou pôr `#[ignore]` para o Windows passar; `#[cfg]` num teste, salvo quando o comportamento não existe naquele SO (justificado no SUMMARY); `allow` global para `dead_code` do Windows — o `cfg` vai no item.
  - **Fim:** o último run, com o `ci.yml` no SHA final do gw, terminou `success`; o head do PR do gw é esse SHA, com o CI de lá verde e sem check falho nem cancelado. Ensaio: com um `ci-evidence.env` provisório, não commitado (RUN_ID e HEAD_SHA desse run), os Verify 1 a 6 do DoD imprimem `OK`.
  - Os gates locais ficam verdes em todo commit de código, cobertura ≥ 80 inclusive.
- **Dependencies:** T-5
- **Test:** ensaio dos Verify 1–6 do DoD
- **Status:** completed

### Wave 4

#### T-7: README (seção CI) e CHANGELOG
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - **README (en)**, seção `## CI`: os gatilhos; os 3 componentes e o que cada um roda; o piso de 80% medido no Linux; `cargo audit` e a política do `.cargo/audit.toml` (como reavaliar); o que o `npm test` instala (A-4); como reproduzir localmente; os `uses:` presos a um SHA do gw até o merge dele.
  - **Resto do README:** Status e "Still to come" atualizados; a limitação "Windows is untested" reescrita com o que o CI prova agora e o que ainda não prova (tray num desktop Windows, `dxva2` real); Dev setup com `cargo install cargo-audit --locked`.
  - **CHANGELOG** `[Unreleased]`: Added (CI em Linux e Windows, política de audit) e Fixed (manifesto dos binários de teste no Windows e o que a T-6 corrigiu).
  - Todo número citado vem de um run. O commit não leva código nem `.jdi/`: `docs(ci-crossbuild): document the CI`.
- **Dependencies:** T-6
- **Test:** revisão humana (Deferred) + o run do push verde
- **Status:** completed

### Wave 5

#### T-8: Sequência de evidência — negativo → reversão → verde → `ci-evidence.env`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `.cargo/audit.toml` (2 commits), `.jdi/phases/ci-crossbuild/ci-evidence.env`
- **Acceptance:**
  1. **Pré-condição:** T-6 e T-7 fechadas, com o último run verde, nada pendente de push e o head do PR do gw = `WORKFLOWS_SHA`.
  2. **Negativo:** um commit que SÓ apaga as linhas da exceção RUSTSEC-2018-0005 (nenhuma acrescentada): `ci(ci-crossbuild): drop RUSTSEC-2018-0005 ignore to prove audit gate`. Push, e o run tem de TERMINAR sem nenhum push no meio. Confirmar `failure` no run, `cargo audit` do `rust-linux` em `failure` e o advisory no log. Guardar `NEG_SHA`/`NEG_RUN_ID`.
  3. **Reversão:** `git checkout "$NEG_SHA^" -- .cargo/audit.toml`, commit `ci(ci-crossbuild): restore RUSTSEC-2018-0005 audit exception`, push. Esperar `success` e guardar `RUN_ID`/`HEAD_SHA`.
  4. **`ci-evidence.env`** (`KEY=VALUE`, sem aspas nem espaços): `REPO=slipalison/ddc-control`, `RUN_ID`, `HEAD_SHA`, `WORKFLOWS_SHA`, `WORKFLOWS_PR`, `WORKFLOWS_BRANCH`, `NEG_RUN_ID`, `NEG_SHA`. Os 8 Verify do DoD imprimem `OK` antes do commit `.jdi`-only (`docs(ci-crossbuild): record CI evidence`, junto de PLAN/SUMMARY). O push vem depois; o run que ele dispara não é evidência.
  - Depois do `HEAD_SHA`, nada muda em `crates/`, `apps/`, `Cargo.*`, `.cargo/` nem `.github/`. Se o template mudar depois do negativo, refazer o negativo: o `ci.yml` do NEG tem de ser byte a byte o do HEAD_SHA.
- **Dependencies:** T-7
- **Test:** os 8 Verify do DoD
- **Status:** completed

## Execution
- 8 tasks em 5 waves. Só a W1 é paralela (T-1..T-3 aqui, T-4 no gw, arquivos disjuntos). Speedup ≈ 1.6x.
- T-6 e T-8 fazem vários commits de propósito: a convergência e a sequência de evidência exigidas pelo DoD. Cada commit continua atômico.
- **DoD × task:** 1 → T-5/T-6/T-8; 2 → T-4/T-6; 3 → T-4/T-5/T-6; 4 (painel ≥ 80) → T-6; 5 (Windows) → T-2/T-6; 6 (`node-ui`) → T-3/T-6; 7 (negativo no CI) → T-8; 8 (`audit.toml`) → T-1.

## Riscos
- **R-1 Cobertura no Linux.** O `qualidade.yml` não exclui `main.rs`: o literal medido local é ~82,9%, e no headless pode cair. Mitigação: mais testes de lógica, nunca filtro novo no template.
- **R-2 1º build real no Windows:** `dead_code`/`unused` sob `-D warnings`, `notification_area.rs` que nunca compilou, testes que dependem de caminho ou cache. Resolve-se na T-6.
- **R-3 Advisory novo antes da evidência.** O DoD 8 exige exatamente 4 `ignore`, então a saída é `cargo update -p` (Cargo.lock) num commit de correção. Se não houver versão corrigida → `blocked`.
- **R-4 `cancel-in-progress`** cancela run de evidência se houver push no meio (regra acima). **R-5** Custo de runner: correções agrupadas, ensaio local dos Verify, nada de `rerun`.
- **R-6 Windows.** O harden-runner tem suporte a Windows no tier comunitário (repositório público). O `python3` do `relatar-cobertura` pode faltar no Git Bash → A-7.

## Files modified (all tasks)
- ddc-control: `.cargo/audit.toml`, `.github/workflows/ci.yml`, `apps/ddc-tray/src-tauri/{build.rs,windows-app-manifest.xml}`, `apps/ddc-tray/{package.json,scripts/playwright-browsers.mjs,tests/ui/playwright-browsers.test.mjs}`, `README.md`, `CHANGELOG.md`, `.jdi/phases/ci-crossbuild/ci-evidence.env`, mais o que a T-6 medir (`apps/ddc-tray/src-tauri/**`, `crates/**`, `Cargo.lock`).
- gw: `.github/workflows/qualidade.yml`, `README.md`, `exemplos/ci-rust-desktop.yml` (`actions.lock.json` só se entrar action nova — não previsto).

## Test requirements
- Rust: `cargo test --workspace --locked`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo fmt --all --check`.
- Cobertura: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'` (local) e o painel `**Aprovado:** ≥ 80` do `rust-linux` (CI).
- UI: `cd apps/ddc-tray && npm test` e o Gate 7 (`npm ci --ignore-scripts && npx playwright test`).
- Auditoria: `cargo audit` sai 0 no HEAD, e o Verify do DoD 8 roda com o worktree descartável.
- CI: os 9 Verify do DoD da CONTEXT (8 até a iteração 1) imprimem `OK` contra o `ci-evidence.env` commitado.
- Mínimo de cobertura: 80% de linhas (PROJECT.md).

## Iteração 2 (ralph loop: rodada de avisos da REVIEW iter 1)
Entrada: a REVIEW iter 1 (W-1..W-5 e o DoD Critic) e o DoD da CONTEXT, agora com 9 linhas. A W-1 já tinha sido corrigida pelo orquestrador. Os commits seguem as regras de execução acima.

#### I2-1 (gw, W-2): `so: windows-*` só com `linguagem: rust`
- **Files modified:** `gw:.github/workflows/qualidade.yml`, `gw:README.md`, `gw:exemplos/ci-rust-desktop.yml`
- **Feito:**
  - O `Conferir componente` reprova com `::error::` qualquer `windows-*` com linguagem diferente de `rust`. O comentário diz por quê: só rust foi medido lá, e o `go test -race` pede cgo/gcc.
  - Cabeçalho, tabela do README, seção "Windows só com `rust`" e exemplo dizem a mesma regra.
  - O script, extraído do YAML, rodou em 20 combinações: 6 passam e 14 reprovam, e 5 delas são `windows-*` fora de rust.
  - Commit `33e00c7` `fix(qualidade): windows-* so com rust, a unica linguagem medida la`.
- **Status:** completed

#### I2-2 (gw, W-5): nome do passo de release sem o campo
- **Files modified:** `gw:.github/workflows/qualidade.yml`
- **Feito:**
  - `name: ${{ matrix.c.build_release && format('Build {0} (release)', matrix.c.build_release) || 'Build de release' }}`.
  - No run 36345684775: `Build ddc-tray (release)` nos dois jobs rust, e `Build de release` (skipped) no `node-ui`.
  - Commit `9e91d1a`. É o novo `WORKFLOWS_SHA`, com o CI do gw verde no run 36345116982.
- **Status:** completed

#### I2-3 (gw, W-4 + W-3): corpo do PR #13
- **Feito:**
  - `gh pr edit 13 --body-file`, com a tabela de runs por head (`2335b52`, `afc20cc`, `9e91d1a`).
  - Nota para o squash: não citar a ponte `python3` que o `afc20cc` tirou, com uma mensagem sugerida.
  - Seção "Follow-up, fora deste PR" com a W-3: `matrix.c.projeto` interpolado em `run:` e `gocover-cobertura@latest`. Esses caminhos ficam sem mudança nesta phase.
- **Status:** completed

#### I2-4: repin do `ci.yml`
- **Files modified:** `.github/workflows/ci.yml`
- **Feito:** os 2 `uses:` passam para `@9e91d1ac9604aa581ac17438c54d75a2fcc213fa`. Commit `384def7` `ci(ci-crossbuild): pin workflows to 9e91d1a`, com os gates locais verdes.
- **Status:** completed

#### I2-5 (fora do escopo pedido, achado pelo run NEG): teste com panic sensível a tempo no Windows
- **Files modified:** `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs` (escopo da T-6: testes que dependem de SO/ambiente)
- **Sintoma:**
  - No run NEG 36345194940, o `rust-windows` falhou em `cargo test com cobertura`.
  - Os testes `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving` e `maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors` deram `Timeout` em vez de `Transport("ddc-hi panicked: …")`.
  - Os 3 testes que provocam panic terminaram juntos, 1,2 s depois do início, inclusive o de relógio virtual. O próprio panic foi lento naquele runner; nos runs verdes anteriores, os mesmos testes levaram ~8 ms.
- **Reprodução:** um panic hook temporário que dorme 1,3 s faz os dois testes falharem localmente com a asserção exata do CI. Com a correção, os dois passam sob o mesmo hook, que depois foi removido.
- **Correção:**
  - `const UNHURRIED = 10 s`, que só limita um travamento, nas chamadas cuja resposta não depende de tempo.
  - O `stuck` continua com 250 ms, porque o `Timeout` dele é o que o teste prova.
  - Nenhuma asserção mudou, e nenhum teste foi ignorado.
  - Commit `2871dc6` `test(ci-crossbuild): give panic tests a budget that only bounds a hang`.
- **Status:** completed

#### I2-6: sequência de evidência refeita
- **Files modified:** `.cargo/audit.toml` (2 commits), `.jdi/phases/ci-crossbuild/ci-evidence.env`
- **Feito:**
  1. NEG `9ef86cc`: só apaga as 8 linhas da exceção RUSTSEC-2018-0005.
  2. O run NEG 36345194940 terminou `failure` antes de qualquer outro push. O passo `cargo audit` do `rust-linux` falhou com o advisory no log.
  3. Reversão `3446dd2` (`git checkout 9ef86cc^ -- .cargo/audit.toml`).
  4. I2-5 (`2871dc6`).
  5. O run verde 36345684775 no `HEAD_SHA` `2871dc6` usou o mesmo `ci.yml` do NEG.
  6. `ci-evidence.env` reescrito num commit só de `.jdi/`.
- Os 9 `Verify:` do DoD, rodados literalmente, imprimem `OK`.
- **Status:** completed
