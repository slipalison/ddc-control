# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Rodada 2, iteração 5 do loop (10ª iteração absoluta, depois do auto-reset 1/3).
> - Branch `phase/input-switch-autostart`, HEAD `41738bd`, base `origin/main` = merge-base `134b665`.
> - O reviewer produziu o conteúdo e o orquestrador o gravou, porque o harness nega escrita de `.md` ao subagente.
> - **O código não mudou desde a iteração anterior.** `git diff 0ea47ec HEAD -- . ':!.jdi'` e `git diff 3a3afa6 HEAD -- . ':!.jdi'` saem vazios. Só o DoD mudou, em `41738bd` (D-2026-10-01-input-switch-autostart-7).
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT.
> - O prj2 (`cargo llvm-cov … → coluna Lines >= 80%`) não é comando de shell e fica com o Gate 3. As 2 linhas Manual não têm comando.
> - Prefixos sha256 dos comandos: ctx1 `ea8c5521806e`, ctx2 `59b02ce9ed66`, ctx3 `8f648d4f4fe0`, ctx4 `05ff88d61fb8`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `e4dc06eaa489`, ctx9 `ae65d59b93ea`, ctx10 `24da480a271e`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - Comparação por programa com a extração de `0ea47ec`:
>   - as linhas 3 a 7, 9 e 10 e as do PROJECT batem byte a byte;
>   - a linha 1 e a linha 8 ganharam a mesma cláusula final: árvore `e87361dd…` de `crates/ddc-adapters`, blobs `a32052f8…`/`56607a7f…` de `Cargo.toml`/`Cargo.lock`, `git diff --quiet HEAD` nesses caminhos e nenhum arquivo não rastreado sob o crate;
>   - a linha 2 ganhou `crates/ddc-cli apps/ddc-tray/src-tauri/src/commands.rs` no pathspec do `git diff --quiet`.
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`, tirados do ambiente (os mesmos do processo do `ddc-tray` do usuário).
>
> Os 10 do CONTEXT, o prj1 e o prj3 rodaram LITERALMENTE duas vezes: dentro do `bwrap`, com o observador ativo, e no nativo, depois da prova.
> - A linha 4 rodou com `npm ci` e o Playwright inteiro.
> - A linha 9 rodou 2 vezes literalmente (1 no `bwrap` e 1 no nativo), mais 1 execução direta dos dois scripts no `bwrap`.
> - A cobertura reusa o Gate 3.
>
> **Herança, só por diff.** Como o código não mudou desde `3a3afa6`, herdo a evidência que depende só do código e dos testes:
> - as fronteiras P1k99900 (vermelha) e P1k99950 (verde);
> - o resultado, no nível dos testes, das 12 mutações do critic da iteração 4 e das 22 antigas;
> - os limiares N2spawn11/12 e N2spawnc45/48;
> - as 30 + 30 + 10 execuções do (f), com 20 leituras dentro de 10..21;
> - a sonda de efeito com `FakeDisplays`: N5w assenta em 2,00 s / 8 leituras, contra 3,00 s / 12 no HEAD;
> - W-2 (clippy do `notification_area.rs` para `x86_64-pc-windows-gnu`);
> - A, R, B1 e B2 da linha 9;
> - M4s, M4g, M4ge, M4u, M3f, M4f, M6 e M10*;
> - a cobertura de UI (88.48%).
>
> **Refeito nesta iteração:**
> - todos os gates (baratos: o cache de compilação foi reaproveitado, sem diff de código);
> - os 12 Verify literais nos dois ambientes;
> - a evidência da linha 1: tempo de cada teste, mais 10 execuções do (f);
> - as 9 mutações do critic anterior, cada uma em duas formas (working tree e commitada);
> - 3 sondas novas de escape: `paths` na raiz, `paths` aninhado e `assume-unchanged`;
> - o teste do endurecimento proposto.
>
> **Estado da máquina.**
> - O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim.
> - `~/.config/autostart` real, comparado antes e depois:
>   - o sha256 de `ls -A` é o mesmo nas duas vezes (`bf0a7045a8fb…`);
>   - o `mtime` do diretório segue em 2026-09-30 22:21:43, e as 4 entradas têm os mesmos tamanhos e `mtime` em `full-iso`;
>   - o sha256 de `ls -la` mudou só na linha `..` (o `mtime` de `~/.config`, 14:03:38). Quem escreveu foi o `spectaclerc` da sessão KDE do usuário, não esta revisão.
> - No fim não sobrou nenhum `dbus-run-session` privado, watcher, servidor HTTP do Playwright, tempdir `/tmp/smoke-*` ou observador inotify.
> - `/var/tmp/rv10` foi apagado (9,3 GB): cópia, `target/`, arquivos que cobriam os `/dev/i2c-*` e logs.
>
> **Monitor real.**
> - Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados.
> - As mutações mexeram só em:
>   - código do adapter sobre `FakeDisplays` (`worker.rs`, `lib.rs` com cópias `.md`, uma cópia de `worker/tests.rs`);
>   - `commands.rs` (r2app);
>   - uma cópia do crate em `vendor/` mais um `.cargo/config.toml` (sondas da W-27).
> - Nenhuma mutação criou teste que fale com o backend real.
>
> **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. A prova por inotify em `bwrap` rodou ANTES de qualquer execução nativa da suíte (ver 5.7).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` saem 0, ambos no `bwrap`. A única nota é o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados). Rodou no `bwrap` das 13:55:09 às 13:55:12, antes de qualquer execução nativa. O prj1 nativo literal também dá `OK`. Os 9 ignorados são os 7 de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs`, todos com `#[ignore = "needs the dev monitor attached; run with DDC_HW_TESTS=1"]`. O número é igual ao da iteração anterior; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed` em 0,50 s. Blocos `Doc-tests`: só `ddc_cli`, `ddc_core` e `ddc_tray`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0 -`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. Rodou no `bwrap`. Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. UI: `node --test` dá 165/165; a cobertura de 88.48% é herdada, porque o JS não tem diff. |
| Lint | PASS | `cargo fmt --all --check` sai 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0, ambos no `bwrap`. O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a prova por inotify em `bwrap`: zero aberturas de `/dev/i2c-*` em gates, Verify, smokes, Playwright, 10 execuções do (f), todas as mutações e sondas, com controle positivo. |
| Consistency | PASS (com warnings) | D-2026-10-01-7 está conforme no CONTEXT (linhas 1, 2 e 8), no PLAN e no SUMMARY. Há 2 commits desde `8626fa6`, ambos `docs`, com escopo e tamanho corretos. O resíduo de `PLAN.md:153` (W-8) fechou. Um resíduo novo, não objetivo: W-27. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app, no `bwrap`: 152 passed, 6 skipped, 0 failed (18,0 s). Os 6 skipped são só `screenshots.spec.mjs:24/34/47` em `light`/`dark` (`test.skip` condicionado a `SCREENSHOTS=1`). `input-notice.spec.mjs` tem 10 `✓`. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. **W-24, W-25 e W-26 estão FECHADAS** pelo congelamento: as 9 mutações do critic ficam vermelhas nas duas formas. Resíduos: **W-27** (linhas 1 e 8, não objetiva: override de `paths` num `.cargo/config.toml` aninhado), uma nota de estado local (`assume-unchanged`) e a P2 (redação da linha 2). |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`). É costura interna do adaptador de entrada (D-11), não port do core. Só nota.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O `grep -RL` não lista nenhuma raiz sem `forbid(unsafe_code)`, e `[workspace.lints.rust] unsafe_code = "deny"` está no `Cargo.toml` da raiz. Não há `unsafe {` em `ddc-adapters`.
- **5.6** Os hits de `unwrap`/`expect`/`panic` são os mesmos da iteração anterior. Todos ficam depois do primeiro `#[cfg(test)]` de cada arquivo ou em comentário:
  - `autostart.rs` 200/213/274 (cfg na linha 172);
  - `lib.rs` 282/359/360 (cfg na 252);
  - `tray.rs:380` (cfg na 207);
  - `status_item.rs` 304/320 (cfg na 233);
  - `worker.rs:141` (comentário);
  - `stop_signals.rs` 105/108 (cfg na 82);
  - `kwin_placement.rs` 246/256/260/267 (cfg na 195).

  Nenhum hit em `ddc-core`.
- **5.7 (segurança de escrita no monitor, delegada pela linha 8).**
  - Greps do gate:
    - `Dangerous` existe no core (`error.rs:33`, `capabilities/tests.rs`);
    - não há `Confirm::Yes` fora de fronteira humana ou teste;
    - `DdcHiMonitorBackend` em `crates/*/tests` e `apps` só aparece em `real_monitor.rs` (7 chamadas a `new()`, as 7 com `#[ignore = "…DDC_HW_TESTS=1"]`, 0 bytes de diff contra a base) e no composition root (`lib.rs:29`, `lib.rs:79`).
  - Os testes não mudaram desde a iteração anterior: `worker/tests.rs` `79d40888…`, `ddc_hi_backend/tests.rs` `ec2d2980…` e `retry/tests.rs` `12ac10db…`. A leitura de (b), (e), (f) e (h) da iteração 4 continua valendo: tudo sobre `FakeDisplays`/`WorkerClient<FakeDisplays>`, e o (h) só lê `client.policies`.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`. Ali dentro, `stat` mostra os 16 como "regular empty file". Um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) ficou ligado nesses arquivos das 13:53:59 até o fim da revisão (por volta das 14:13).
    - Controle positivo: `cat /dev/i2c-3` DENTRO do sandbox gerou 1 evento (`mask=0x20`, 13:54:25).
    - `cargo test --workspace --locked` no sandbox, às 13:55:09, antes de qualquer execução nativa: `SUITE_EXIT=0`, 413/0/9, ZERO eventos.
    - Também com ZERO eventos, tudo no sandbox:
      - build, cross-check, fmt, clippy, `cargo llvm-cov` e `cargo audit`;
      - os 12 Verify literais e a coleta de evidência;
      - 10 execuções do (f);
      - os dois smokes diretos do binário release (`DDC_TRAY_FAKE=1`, `A_EXIT=0`, `B_EXIT=0`) e o Playwright;
      - todas as mutações e sondas na cópia.
    - No log inteiro há 1 evento, o do controle.
    - As execuções nativas (13:57:55–13:58:30) vieram depois da prova e rodam o mesmo código.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, aviso permitido que já estava no lock da base. O `Cargo.lock` é o blob `56607a7f…`.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração anterior** (`8626fa6..HEAD`), 2 ao todo:
  - `0ea47ec` docs (52 caracteres): REVIEW e LOOP da iteração 4;
  - `41738bd` docs (67 caracteres): `D-2026-10-01-input-switch-autostart-7.md`, CONTEXT (linhas 1, 2 e 8), PLAN e SUMMARY.

  Escopo = slug, `.jdi/` nunca no mesmo commit que código (D-12), e os 2 trazem `Claude-Session:`.
- **D-2026-10-01-7, conforme:**
  - As linhas 1 e 8 exigem a árvore `e87361ddfc4bcd7e56290e460b8582c8d45b95b7`, os blobs `a32052f8…`/`56607a7f…`, `git diff --quiet HEAD` e nenhum arquivo não rastreado sob o crate. No HEAD: árvore e blobs batem, o diff sai 0 e não há arquivo não rastreado.
  - As fixações antigas seguem: SHA-256, corpo de `write_vcp` (`worker.rs:538-543`), conjunto `INPUT_SETTLE_*` (`retry.rs:27/32/112/113`) e dep-info (os 10 `.rs`).
  - O "NÃO afirmado" da linha 1 traz o monitor real (W-3) e as camadas acima do adapter.
  - A faixa P1 passou a "k ≤ 99 900", o que bate com P1k99900 vermelha e P1k99950 verde (herdado).
  - A linha 2 afirma e confere `crates/ddc-core`, `crates/ddc-cli` e `commands.rs` sem diff.
  - `PLAN.md:140` registra a iteração 5, e `PLAN.md:154` agora diz "D-13 a D-19 e D-2026-10-01-input-switch-autostart-1 a -7" (W-8 fechada).
  - O SUMMARY ("Rodada 2, iteração 5") bate com o reproduzido aqui: arquivo `.md` não rastreado sob o crate derruba as linhas 1 e 8; linha a mais em `lib.rs` derruba a linha 1; linha a mais em `commands.rs` derruba a linha 2.
- **O texto das linhas 1, 2 e 8 é verdadeiro no HEAD?**
  - **Linha 1: sim.** Além do Verify, o reviewer conferiu três coisas:
    - `git hash-object` de cada um dos 18 arquivos rastreados do crate e dos 2 da raiz é igual ao blob do HEAD;
    - `git ls-files -v` só tem `H`, ou seja, nenhum `assume-unchanged`/`skip-worktree`;
    - não existe `.git/info/attributes`, o `.gitattributes` só tem `text=auto eol=lf` e tipos binários, e o único filtro configurado é o LFS global.

    "Ids de conteúdo: valem também depois do squash-merge" é verdadeiro, com uma condição: o `main` não pode mudar o `Cargo.toml`/`Cargo.lock` da raiz antes do merge. Se mudar, as linhas 1 e 8 deixam de bater, o que é intencional pela D-7 ("até nova revisão").
  - **Linha 8: sim.** "nenhum código de teste, helper, macro ou atributo pode ser acrescentado [ao crate] sem esta linha falhar" se confirma pelas mutações W24mac, H8 e RelMd.
  - **Linha 2: sim na substância, com uma nota de precisão (P2).** A lista "as camadas que chamam o `set_feature` do core" não é exaustiva. Também o chama `apps/ddc-tray/src-tauri/src/panel.rs:213` (`write_brightness`, usado pelos itens "Brightness N%" e pela roda em `scroll.rs:148`). Esse arquivo tem 0 bytes de diff contra a base, mas o Verify não o confere. Como ele só escreve `VcpCode::BRIGHTNESS` (fixo), o input nunca passa por ali, e a afirmação sobre "o assentamento" se mantém.
- **D-3 / D-17(a) / D-14 / D-19 / D-2026-10-01-1 a -6.** Sem diff de código: conformes, pela leitura da iteração 4. `settle_input` lê só `self.policies.input_settle` (`worker.rs:313`), `write_budget` usa `policies.input_settle.window` (`worker.rs:190`), e `call`/`transact`/`serve`/`run` não alteram o orçamento nem a política no código congelado (`worker.rs:210-229` e `466-515`).
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo.

### Achados do critic da iteração anterior, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| W-25: N5w (teto de 2 s em `Worker::run`) | FECHADO por construção | Na cópia: `fmt` 0, clippy 0 (`-p ddc-adapters -p ddc-tray`), lib `76 passed`. Verify 1 e Verify 8 SEM `OK` nas duas formas: no working tree, `git diff --quiet HEAD` sai 1; commitada, a árvore é `c04d283f79c5…`, diferente de `e87361dd…`. N5s, N5wf, N5sp e N5clk editam o mesmo `worker.rs` e caem pelo mesmo mecanismo |
| Critic: N5dl (`deadline = now + budget.min(2 s)` em `call`) | FECHADO | lib `76 passed`; Verify 1/8 sem `OK` nas duas formas (árvore `a9b3d0c70890…`) |
| Critic: N5serve (`op(self, deadline.min(now + 2 s))` em `serve`) | FECHADO | lib `76 passed`; Verify 1/8 sem `OK` nas duas formas (árvore `0edf73c26387…`) |
| W-26: N4tfloor (`budget.max(100 ms)` em `transact`) | FECHADO | lib `76 passed`; Verify 1/8 sem `OK` nas duas formas (árvore `ed71898b9dbe…`). N4tslack edita o mesmo trecho |
| W-26: N4call (`recv_timeout(budget + 50 ms)` em `call`) | FECHADO | lib `76 passed`; Verify 1/8 sem `OK` nas duas formas (árvore `48d2600df4a3…`) |
| Critic: troca de módulo só em release, via `#[cfg_attr(not(test), path = "release/ddc_hi_backend.md")]` em `lib.rs` (RelMd) | FECHADO | `fmt` 0, clippy 0, lib `76 passed`. A troca está viva: o dep-info do build NÃO-teste lista `release/ddc_hi_backend.md` e `release/worker.md` (este com N5w). Verify 1/8 sem `OK`: no working tree, `untracked=[release/ddc_hi_backend.md release/worker.md]` e diff 1; commitada, árvore `94339b159225…` |
| W-24: macro que emite `#[`/`path = $file`/`]` + `#[cfg(test)] mod tests;` para `worker/settle-checks.rs` com `include_str!("tests.rs")` (W24mac) | FECHADO | `fmt` 0, clippy 0, lib `77 passed` (o marcador `compiled_from_settle_checks` vem da cópia). Verify 1/8 sem `OK`: no working tree, `untracked=[…/settle-checks.rs]`; commitada, árvore `3f3839da0418…`. M8p6, M8p7 e M8p8 também mexem no crate |
| H8 (helper só de teste, sem `cfg`, `#[allow(dead_code)]` com `// reason:`) | FECHADO | `fmt` 0, clippy 0, lib `76 passed`; Verify 1/8 sem `OK` nas duas formas (árvore `af224ed50a16…`) |
| Linha 2 (r2app): `sleep(500 ms)` + releitura em `commands.rs::write_feature` | FECHADO | `fmt` 0, clippy 0, testes do tray `152 + 2 passed`. Verify 2 sem `OK` nas duas formas (`commands.rs`, 16+/8−). Verify 1 e 8 seguem `OK`, corretamente: não afirmam `commands.rs` |
| P1 (k < 100 000) | FECHADO no texto | "k ≤ 99 900" |
| W-8 (resíduo `PLAN.md:153`) | FECHADO | `PLAN.md:154` atualizado |

### Mutações reproduzidas pelo reviewer
**Montagem.** Todas rodaram num repositório git descartável, `/var/tmp/rv10/repo`, já apagado, com `target/` próprio:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1.

A cópia é fiel ao repositório real:
- árvores idênticas: HEAD `755420ea…`, base `0c8815c7…`, `crates/ddc-adapters` `e87361dd…`, `Cargo.toml` `a32052f8…` e `Cargo.lock` `56607a7f…`;
- `diff --stat` idêntico: 67 arquivos, `3968 insertions(+), 88 deletions(-)`, incluindo `.jdi`.

Baselines na cópia, todas `OK`: Verify 1 (5,42 s a frio), 2, 8 e 10.

**Execução.**
- Mutações aplicadas por substituição exata, com 1 ocorrência conferida (`assert`), e formatadas com `cargo fmt`.
- Para cada uma: `fmt --check`, clippy `-p ddc-adapters -p ddc-tray --all-targets -D warnings`, lib `ddc-adapters` e Verify literal na forma working tree. Depois, commit e Verify literal de novo, na forma commitada.
- Reversão: `git reset --hard` para o commit 2 + `git clean -fd`.
- Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados, e zero eventos.

| Linha | Mutação | Resultado |
|---|---|---|
| 1, 8 | N5w, N5dl, N5serve, N4tfloor, N4call, RelMd, W24mac, H8 | Todas com `fmt` 0, clippy 0 e lib verde (76, ou 77 na W24mac). **Todas VERMELHAS nos Verify 1 e 8, nas duas formas.** Ver a tabela anterior |
| 2 | r2app | VERMELHA no Verify 2, nas duas formas; Verify 1 e 8 `OK` (esperado) |
| 1, 8 | **W27r**: `.cargo/config.toml` na RAIZ com `paths = ["vendor/ddc-adapters"]` + cópia do crate com N5w (não rastreada e commitada) | Verify 1 `OK` e Verify 8 `OK`, **mas** Verify 2 sai 1 e prj1 sai 101: o resolvedor dá `failed to select a version for ddc-core`/`ddc-adapters`, por causa da dev-dep do `ddc-core` sobre o `ddc-adapters`. O Gate 2 também falharia. Pega, mas por acaso. Com `vendor/ddc-core` também sobreposto, dá o mesmo |
| 1, 8 | **W27n**: `apps/ddc-tray/.cargo/config.toml` com `paths = ["../../vendor/ddc-adapters"]` + cópia do crate com N5w, commitada | **Da raiz: Verify 1, 2, 8 e 10 `OK` e prj1 `OK`** (testes do tray `152 passed`). O `cargo check -p ddc-tray` compila `crates/ddc-adapters/src/lib.rs` quando roda da raiz, mas `vendor/ddc-adapters/src/lib.rs` quando roda de `apps/ddc-tray` ou de `apps/ddc-tray/src-tauri`, que é o diretório de onde o Tauri CLI empacota. Ver W-27 |
| 1, 8 | **Nloc**: N5w no working tree + `git update-index --assume-unchanged worker.rs` | Verify 1 `OK` e Verify 8 `OK`. É só local: o commit não leva a mudança. Ver as notas |
| 1, 8 | Endurecimento testado: `[ "$(git ls-files -co -- ':(glob)**/.cargo/**' ':(glob)**/rust-toolchain' ':(glob)**/rust-toolchain.toml' \| sort -u)" = .cargo/audit.toml ]` | HEAD real e cópia: OK. W27n: FAIL. `.cargo/config` aninhado com `[profile.release] panic = "abort"`: FAIL |

**Linha 1, evidência do HEAD.** Cada teste rodou isolado no `bwrap`:
- (a), (c), (d), (g) e (h) em 0,00 s;
- (b) em 0,03 s e (e) em 0,04 s;
- (f) em 0,32 s, 10/10 vezes (a folga até o limite de 0,5 s do Verify é de 0,17 s).

O conjunto `input_write_*`/`writes_to_other_codes_*` listado tem exatamente os 7 nomes.

**Linha 9:** 2/2 execuções literais `OK` (`bwrap` 22,46 s com o build release; nativo 5,68 s). Mais 1 execução direta dos dois scripts DENTRO do `bwrap`: `A_EXIT=0`, `B_EXIT=0`, zero eventos. A W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração anterior:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado. Agora isso consta do "NÃO afirmado" da linha 1.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. A "correção" natural disso (N5w) agora derruba as linhas 1 e 8 até nova revisão.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O README documenta.
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final (`…/autostart_entry-c0e56ec32fbab8ff `).
  - `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN): FECHADO.**
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.** 0,32 s em 10/10 nesta iteração, mais 30 + 30 + 10 herdadas, sempre com 20 leituras (limites 10..21). A folga até os 0,5 s do Verify é de 0,17 s. O `rust-windows` mostra primeiro.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 2/2 literais + 1 direta no `bwrap` OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-24, W-25 e W-26: FECHADAS por construção (D-2026-10-01-7).** Toda a família, em qualquer forma, muda a árvore do crate. Foram reproduzidas 8 mutações dela: N5w, N5dl, N5serve, N4tfloor, N4call, RelMd, W24mac e H8.
- **W-20, W-21, W-22, W-23: FECHADOS** e seguem vermelhos (herdado, sem diff de código). **W-9, W-10, W-12, W-13, W-15, W-17, W-18, W-19: FECHADOS** em iterações anteriores.

Novo nesta iteração:
- **W-27 (linhas 1 e 8, NÃO objetiva: evasão deliberada; D-2026-10-01-7).**
  - **O que as linhas afirmam.** A linha 1 diz "Assentamento no adapter, sobre o código REVISADO e CONGELADO", e o (h) acrescenta "(a fiação de produção; sem ela o assentamento sumiria no app real…)". A D-7 diz "Sobre esse código congelado, os testes nomeados provam o comportamento afirmado".
  - **Por que escapa.** O congelamento prova o conteúdo de `crates/ddc-adapters`, do `Cargo.toml` e do `Cargo.lock`. Ele não prova qual `ddc-adapters` o binário do app linka. O cargo lê `.cargo/config.toml` nos ancestrais do diretório de onde o build roda, e um override `paths` troca um pacote de mesmo nome e versão sem tocar manifesto nem lock.
  - **W27n** (2 peças, ambas fora dos caminhos congelados):
    ```toml
    # apps/ddc-tray/.cargo/config.toml
    paths = ["../../vendor/ddc-adapters"]
    ```
    mais `vendor/ddc-adapters/` = cópia do crate com N5w em `worker.rs`. Commitada:
    - da raiz, os Verify 1, 2, 8 e 10 e o prj1 dão `OK`, e os gates rodados da raiz não veem nada;
    - o build do `ddc-tray` a partir de `apps/ddc-tray` ou de `apps/ddc-tray/src-tauri`, de onde o Tauri CLI empacota, compila `vendor/ddc-adapters/src/lib.rs`;
    - o cargo só imprime `warning: skipping duplicate package`, que nenhum Verify lê;
    - efeito do N5w medido na iteração 4 com `FakeDisplays`: a troca de input assenta em 2,00 s / 8 leituras, em vez de 3,00 s / 12 (D-3).
  - **Na raiz, a mesma ideia é pega, mas por acaso** (W27r). A dev-dep do `ddc-core` sobre o `ddc-adapters` quebra a resolução, e o Verify 2, o prj1 e o Gate 2 falham.
  - **Por que é não objetiva.** Exige uma cópia inteira do crate fora de `crates/` e um override sem motivo natural. Uma variante natural da mesma classe existe: `[profile.release] panic = "abort"` num `.cargo/config` aninhado, que o guia de tamanho do Tauri sugere. Ela desliga, no pacote, o `catch_unwind` de `isolated()`. Mas pânico não está entre as afirmações (a)–(h): a (c) cobre `Err` e valores errados.
  - **Endurecimento testado (1 linha, nas linhas 1 e 8):**
    ```sh
    [ "$(git ls-files -co -- ':(glob)**/.cargo/**' ':(glob)**/rust-toolchain' ':(glob)**/rust-toolchain.toml' | sort -u)" = .cargo/audit.toml ] || ok=0;
    ```
    HEAD OK; W27n FAIL; `panic = "abort"` aninhado FAIL. A alternativa é declarar no "NÃO afirmado" "a configuração do cargo (`.cargo/config*`) fora dos caminhos congelados". Qualquer das duas pede emenda do DoD e nova D-XX. Não bloqueia.

Notas, sem ação no código:
- **Nloc (linhas 1 e 8, estado local, não objetiva).** `git update-index --assume-unchanged`, `skip-worktree` ou um filtro local escondem uma mudança do working tree de `git diff --quiet HEAD`. Com N5w no working tree e a flag, os Verify 1 e 8 dão `OK` na cópia. A mudança nunca chega a commit, PR ou CI, porque o commit mantém a árvore congelada.
  - No checkout real conferido: `git ls-files -v` só tem `H`, e `git hash-object` dos 20 arquivos (18 do crate + os 2 da raiz) é igual aos blobs do HEAD.
  - Endurecimento opcional: `[ -z "$(git ls-files -v -- crates/ddc-adapters Cargo.toml Cargo.lock | grep -v '^H ')" ]`.
- **P2 (linha 2, precisão).** "as camadas que chamam o `set_feature` do core: `crates/ddc-core`, `crates/ddc-cli` e `commands.rs`" omite `panel.rs:213` (`write_brightness`, menu "Brightness N%" e roda), que tem 0 bytes de diff e só escreve `VcpCode::BRIGHTNESS`. Acrescentar `apps/ddc-tray/src-tauri/src/panel.rs` ao pathspec do Verify 2, ou escrever "as camadas que escrevem o input pelo `set_feature` do core".
- **Redação (linha 1).** Os "falha aqui" de (b) e (e), aplicados a limites postos só no caminho de produção (`run`, `spawn`, `call`, `serve`), agora são garantidos pelo congelamento (a LINHA falha), não pelo teste. Lendo "aqui" como "esta linha", o texto é verdadeiro.
- **Squash-merge.** Se outro PR mudar o `Cargo.toml` ou o `Cargo.lock` da raiz no `main` antes deste, as linhas 1 e 8 deixam de bater até nova revisão. É o comportamento pretendido pela D-7.
- **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. Manter, nas próximas rodadas (reviewer, critic, `/jdi-ship`), a leitura do 5.7 e a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **`.jdi/DECISIONS.md`.** É uma visão gerada e ignorada (`.gitignore:31`). As fontes estão em `.jdi/decisions/` (inclusive `D-2026-10-01-input-switch-autostart-7.md`), e `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter sobre o código REVISADO e CONGELADO: árvore de `crates/ddc-adapters` = `e87361dd…`, blobs `Cargo.toml`/`Cargo.lock` = `a32052f8…`/`56607a7f…`, sem diff do working tree e sem arquivo não rastreado sob o crate. Sobre ele: 7 testes nomeados (conjunto exato), cada um sozinho, não ignorado e em menos de 0,5 s; (b) com tabela de 10 políticas e faixas declaradas; (e) com dois clientes por política e os 256 códigos; (f) pelo `WorkerClient` real; (g) `Default`/`without_backoff()`; (h) fiação; `INPUT_SETTLE_*` = 4 linhas; corpo de `write_vcp` fixado; dep-info com os 10 `.rs`; 3 SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,64 s (`bwrap`) e 1,59 s (nativo). Os 7 nomes listados, cada um `1 passed; 0 failed`: (a), (c), (d) e (g) em 0,00 s; (b) 0,03 s; (e) 0,04 s; (f) 0,32 s (10/10). (h) listado e passa. `git grep` = `retry.rs:27/32/112/113` (o `///` da 22 fica excluído). Corpo de `write_vcp` = `worker.rs:538-543`. Dep-info = os 10 `.rs`. SHA-256 `79d40888…`, `ec2d2980…` e `12ac10db…` batem. Árvore `e87361dd…` e blobs batem, diff 0 e nenhum arquivo não rastreado. `hash-object` dos 20 arquivos = HEAD, e `ls-files -v` só tem `H`. N5w, N5dl, N5serve, N4tfloor, N4call, RelMd, W24mac e H8 ficam VERMELHAS nas duas formas. **Resíduo não objetivo: W-27 (W27n dá `OK`); nota Nloc.** |
| 2 | O assentamento não toca o `ddc-core` nem as camadas que chamam o `set_feature` do core: `crates/ddc-core`, `crates/ddc-cli` e `commands.rs` sem nenhum byte de diff desde a base; core só com `thiserror` e sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,12 s (`bwrap` e nativo). Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base nos 3 caminhos sai 0; sem `sleep`. r2app reproduzida: VERMELHA nas duas formas. Nota P2 (`panel.rs:213`, só brilho, 0 diff). |
| 3 | View-model puro + 4 testes nomeados, com o texto inteiro conferido por igualdade com frases LITERAIS (papéis de mantida e pedida) em `en`/`pt-BR`, para o aviso de input e o genérico; `tests/ui` inteira verde; `view-model.test.mjs` congelado por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 0,26 s (`bwrap` e nativo). `node --test`: `# tests 165 / # pass 165 / # fail 0`, sem skipped, todo nem cancelled. SHA-256 `7c650133…` bate. M4s, M4g, M4ge, M4u e M3f herdadas (sem diff). |
| 4 | Aviso visível de ponta a ponta. `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; texto inteiro de cada toast contra a frase literal com papéis; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 19,95 s (`bwrap`) e 21,05 s (nativo), com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed; 10 `✓` em `input-notice.spec.mjs`. SHA-256 `92f99e00…`/`ee930894…` batem. M4s, M4g, M4u e M4f herdadas. |
| 5 | Menu nativo: rótulo en/pt-BR; item nas 2 plataformas com marca = estado do SO; `from_id`; fiação LINUX (ksni) e testes de `tray`. Fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 3,27 s (`bwrap`) e 2,05 s (nativo). Os 6 nomes existem em `--list` e passam; `menu:: i18n::` dá `20 passed; 0 failed`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); o chamador de PRODUÇÃO imprime a linha do `crate::report`; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,54 s (`bwrap`) e 1,49 s (nativo). Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` imprime 2 vezes a linha `ddc-tray: could not change the start-with-system entry: read-only home`. M6 herdada (vermelha). |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 2,79 s (`bwrap`) e 0,54 s (nativo). Impressos: `[Desktop Entry]`, `Name=test` e `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff `. Listagem real idêntica (`ls -A` `bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: tokens de hardware só nos 3 arquivos de produção; nesses 3, só `#[cfg(test)]` + `mod tests;`; `doctest = false` efetivo e sem `Doc-tests ddc_adapters`; dep-info com os 10 `.rs`; todo teste sob `ddc_hi_backend::` num módulo `tests` ancorado; crate inteiro = árvore congelada da linha 1; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 1,39 s (`bwrap`) e 1,10 s (nativo). Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `cargo metadata`: `["lib"] ddc_adapters doctest=false`, sem `Doc-tests ddc_adapters`. Dep-info: os 10 `.rs`. A `--list` sob `ddc_hi_backend::` tem 51 testes: `tests` (4), `worker::tests` (32), `hardware::tests` (10) e `identity::tests` (5). Árvore e blobs batem. `capabilities/` só tem `default.json`, sem diff e sem `autostart`. `tauri-plugin-autostart v2.6.0` -> `ddc-tray`. **W24mac (W-24), H8 e RelMd ficam VERMELHAS**, assim como N5*/N4*. Resíduo não objetivo: W-27. O "nenhum teste fala com monitor real" está no 5.7: leitura + inotify, zero aberturas. |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B; monitor simulado exigido nas duas; marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **2/2 execuções literais**: `bwrap` 22,46 s (com o build release) e nativo 5,68 s. Execução direta no `bwrap`, com zero eventos de `/dev/i2c-*`. Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor`, `smoke-sni: the popup was still shown 1.5 s later` e `smoke-sni: OK — PID 3624814 registered its tray item, … and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor` e checkmark desmarcado; o 1º clique grava `/tmp/smoke-autostart-home.0zggOZ/.config/autostart/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray`, e o 2º remove; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. PID 24429 intacto. W-16 não apareceu. A, R, B1 e B2 herdadas. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase (caminhos fixos + `crates/ddc-adapters/Cargo.toml` UNIDOS aos não-Rust do diff); `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,03 s (`bwrap`) e 0,02 s (nativo). M10c, M10n e M10g (vermelhas) e M10ctl (`OK`) herdadas, sem diff. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,08 s (`bwrap`) e 1,10 s (nativo). Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,02 s (`bwrap`) e 0,01 s (nativo). |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` (linha 10, "Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18). O último release é `## [0.1.0] - 2026-09-28` (linha 31). Nenhum heading de release novo nesta phase; sem diff desde `b53119f`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330, 333 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento (250 ms, no máximo 3 s); os smokes privados são citados (linha 387); as limitações do tray estão em "Known limitations of the tray app" (linha 392, com o `Exec=` sem aspas na 399). Sem diff desde `b53119f`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers. Os 13 Verify automáticos dão `OK` no HEAD, rodados literalmente no `bwrap` e no nativo. A prova por inotify em `bwrap` deu zero aberturas de `/dev/i2c-*`, com controle positivo.

**O fechamento estrutural da D-2026-10-01-7 funciona.** As 9 mutações do critic anterior passam em `fmt`, clippy e testes, mas deixam o Verify da linha correspondente sem `OK`, tanto no working tree quanto commitadas:
- N5w, N5dl, N5serve, N4tfloor, N4call, a troca só em release via `cfg_attr` + `.md`, a W-24 por macro com `include_str!` e a H8, nas linhas 1 e 8;
- r2app, na linha 2.

Com isso, W-24, W-25 e W-26 ficam fechadas por construção. O texto das linhas 1 e 8 é verdadeiro no HEAD, e o da linha 2 também, na substância.

Resíduos, todos não bloqueantes:
- **W-27 (não objetiva):** um `.cargo/config.toml` ANINHADO em `apps/ddc-tray` com override `paths` para uma cópia do crate fora de `crates/` deixa todos os Verify `OK` quando rodados da raiz, enquanto o tray empacotado linka a cópia. Endurecimento de 1 linha testado nas linhas 1 e 8: o conjunto de `.cargo/**` e `rust-toolchain*` tem de ser exatamente `.cargo/audit.toml`. Alternativa: declarar a configuração do cargo no "NÃO afirmado". Qualquer das duas pede nova D-XX. Se o loop seguir para o critic sem isso, a W-27 é a única escapatória estrutural que este reviewer encontrou.
- **Nloc (estado local):** opcionalmente, exigir `git ls-files -v` só com `H` nos caminhos congelados.
- **P2:** acrescentar `apps/ddc-tray/src-tauri/src/panel.rs` ao Verify 2, ou restringir o texto às camadas que escrevem o input.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL em `/dev/i2c-*`, manter a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2). Um `22 reads` ou um (f) lento lá aponta para a W-11;
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff. Qualquer mudança em `crates/ddc-adapters`, no `Cargo.toml` ou no `Cargo.lock` da raiz derruba as linhas 1 e 8 até nova revisão, como a D-7 pretende.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura; rodada 2, iteração 5). Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados por inotify; fora o controle positivo, houve zero eventos. **Nenhuma linha está oca de forma objetiva.** As linhas 3–7 e 9–13 dão `hollow=false`. Nas linhas 1, 2 e 8 há só resíduos NÃO objetivos:

- DoD row «1 — assentamento no adapter» (hollow, NÃO objetiva).
  - **W-27**, reproduzida. Um `apps/ddc-tray/.cargo/config.toml` com `paths = ["../../vendor/ddc-adapters"]` aponta para uma cópia inteira do crate com N5w. Rodados da raiz, os Verify dão `OK`, e o tray compilado a partir de `apps/ddc-tray/src-tauri` linka a cópia.
    - A letra da linha continua verdadeira: a árvore congelada é a de `crates/ddc-adapters`, e as camadas acima do adapter estão no "NÃO afirmado".
    - A mutação é evasão deliberada.
    - Não há variante de boa-fé na mesma classe: perfil de release aninhado não toca (a)–(h), `[patch]` muda o lock congelado e `default-features = false` não compila.
  - **Nloc**, reproduzida. `assume-unchanged` esconde uma mudança do working tree, mas ela nunca chega a commit, PR ou CI.
  - A faixa P1 "k ≤ 99 900" fecha na aritmética.
  - O endurecimento proposto (conjunto `.cargo/**` = `.cargo/audit.toml`) devolve exatamente `.cargo/audit.toml` no checkout real.
- DoD row «2 — o assentamento não toca o core nem as camadas que o chamam» (hollow, NÃO objetiva).
  - **P2**: a lista não é exaustiva, porque `panel.rs:213` (`write_brightness`) também chama `set_feature`.
  - Mas esse caminho é fixo em `VcpCode::BRIGHTNESS`. Os únicos caminhos que escrevem o input são `commands.rs::write_feature` e `ddc-cli/src/run.rs:168`, e os dois estão no pathspec.
  - A lista depois dos dois-pontos é explícita e o Verify prova exatamente ela.
- DoD row «8 — segurança de hardware e superfície» (hollow, NÃO objetiva). Mesmos W-27 e Nloc da linha 1, reproduzidos.

**Verdict:** APPROVED_WITH_WARNINGS
