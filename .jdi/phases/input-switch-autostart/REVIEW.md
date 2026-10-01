# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Rodada 2, iteração 4 do loop (9ª iteração absoluta, depois do auto-reset 1/3).
> - Branch `phase/input-switch-autostart`, HEAD `8626fa6`, base `origin/main` = merge-base `134b665`.
> - O reviewer produziu o conteúdo e o orquestrador o gravou, porque o harness nega escrita de `.md` ao subagente.
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT.
> - O prj2 (`cargo llvm-cov … → coluna Lines >= 80%`) não é comando de shell e fica com o Gate 3. As 2 linhas Manual não têm comando.
> - Prefixos sha256 dos comandos: ctx1 `25d000812f43`, ctx2 `81b3ba163edc`, ctx3 `8f648d4f4fe0`, ctx4 `05ff88d61fb8`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `4de5122d7b32`, ctx9 `ae65d59b93ea`, ctx10 `24da480a271e`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - As linhas 2 a 7, 9 e 10 e as do PROJECT batem byte a byte com a extração de `b53119f`.
> - A linha 1 ganhou três coisas: o corpo fixado de `WorkerClient::write_vcp` (awk), o conjunto do dep-info e o SHA-256 novo de `worker/tests.rs` (`79d40888…`).
> - A linha 8 só ganhou o conjunto do dep-info (D-2026-10-01-input-switch-autostart-6(d)(e)).
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`, tirados do ambiente.
>
> Os 10 do CONTEXT, o prj1 e o prj3 rodaram LITERALMENTE duas vezes: dentro do `bwrap`, com o observador ativo, e no nativo, depois da prova.
> - A linha 4 rodou com `npm ci` e o Playwright inteiro.
> - A linha 9 rodou 4 vezes literalmente (3 no `bwrap` e 1 no nativo), mais 1 execução direta dos dois scripts no `bwrap`.
> - A cobertura reusa o Gate 3.
>
> **Herança, só por diff.** `git diff b53119f HEAD -- . ':!.jdi'` lista só `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs` (+131 −47, `3a3afa6`).
> - Para esse arquivo nada foi herdado. Foram refeitos: o tempo e a contagem de leituras do (f), todas as mutações da linha 1 (as 12 do critic e as antigas), as da linha 8 e a busca de variantes dentro das faixas declaradas.
> - Também foram reproduzidas nesta iteração as sondas baratas das linhas 2, 6 e 10: a de 1 byte, a M6, M10c, M10n, M10g e M10ctl.
> - Herdado, porque os arquivos e os Verify não têm diff desde `b53119f`:
>   - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
>   - as mutações A, R, B1 e B2 da linha 9;
>   - M4s, M4g, M4ge, M4u, M3f e M4f das linhas 3 e 4.
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
>
> **Estado da máquina.**
> - O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim.
> - A listagem do `~/.config/autostart` real tem os mesmos sha256 antes e depois, no locale original: `ls -A` `bf0a7045a8fb…` e `ls -la` `f69da4fd3ea5…`. O `mtime` do diretório segue em 2026-09-30 22:21:43.
> - No fim não sobrou nenhum `dbus-run-session` privado, watcher, servidor HTTP do Playwright, tempdir `/tmp/smoke-*`, laço de CPU ou observador inotify.
> - `/var/tmp/rv9` foi apagado (7,3 GB): cópia, `target/`, arquivos que cobriam os `/dev/i2c-*` e logs.
>
> **Monitor real.**
> - Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados.
> - As mutações mexeram só em código do adapter sobre `FakeDisplays`: `settle_input`, `input_unsettled`, `write_budget`, `WorkerClient::{spawn, call, transact, write_vcp}`, `Worker::run` e métodos novos do trait `Clock`.
> - Fora do adapter, mexeram em atributos de módulo, em cópias de arquivos de teste, em macros plantadas (`ddc_hi_backend.rs`, `lib.rs`, `identity.rs`), no `tray.rs` (M6), em 1 byte do `ddc-core` e em comentários de `app.js`, `en.js` e `private-bus.sh`.
> - As sondas `rv_probe` e `rv_fix…` só usam `WorkerClient<FakeDisplays>`.
> - Nenhuma mutação criou teste que fale com o backend real.
>
> **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. A prova por inotify em `bwrap` rodou ANTES de qualquer execução nativa da suíte (ver 5.7).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0, e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` também, ambos no `bwrap`. Única nota: o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados), no `bwrap`, às 12:53, antes de qualquer execução nativa. O prj1 nativo literal também dá `OK`. Os 9 são os 7 de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs`, todos `needs the dev monitor attached; run with DDC_HW_TESTS=1`. Igual à iteração anterior; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed`. Blocos `Doc-tests`: só `ddc_cli`, `ddc_core` e `ddc_tray`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0 -`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. Rodou no `bwrap` (413/0/9 dentro dele). Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0 no repositório e também na árvore limpa do HEAD dentro da cópia descartável (`bwrap`). O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a leitura do teste alterado e a prova por inotify em `bwrap`. Houve zero aberturas de `/dev/i2c-*` em gates, Verify, smokes, Playwright, 70 execuções do (f), todas as mutações e sondas, com controle positivo. |
| Consistency | PASS (com warnings) | D-2026-10-01-6 (a) a (e) está conforme no código, nos testes e nos Verify. Há 3 commits desde `b53119f`, com tipo, escopo e tamanho corretos. W-25 e W-26 são falta de força do DoD da linha 1, não violação de D-XX no código. Mas a W-25 mostra que um teto de janela posto só no caminho de produção quebraria a D-3 (3 s) sem nenhum teste vermelho. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app, no `bwrap`: 152 passed, 6 skipped, 0 failed. Os 6 skipped são só `screenshots.spec.mjs:24/34/47` em `light`/`dark` (`test.skip` condicionado a `SCREENSHOTS=1`). `input-notice.spec.mjs` tem 10 `✓`. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduos: **W-25** e **W-26** (linha 1, objetivas prováveis, dentro das faixas declaradas, Verify 1 `OK` 3/3), **W-24 reaberta** (linhas 1 e 8, não objetiva: `#[path]` multilinha para nome fora do charset da regex + `include_str!`) e nota de precisão P1 (linha 1). |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`). É costura interna do adaptador de entrada (D-11), não port do core. Arquivo sem diff. Só nota.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O `grep -RL` não lista nenhuma raiz sem `forbid(unsafe_code)`, e `[workspace.lints.rust] unsafe_code = "deny"` está em `Cargo.toml:24`. Não há `unsafe {` em `ddc-adapters`.
- **5.6** Todos os hits de `unwrap`/`expect`/`panic` ficam depois do primeiro `#[cfg(test)]` de cada arquivo ou em comentário:
  - `autostart.rs` 200/213/274 (cfg na linha 172);
  - `lib.rs` 282/359/360 (cfg na 252);
  - `tray.rs:380` (cfg na 207);
  - `status_item.rs` 304/320 (cfg na 233);
  - `worker.rs:141` (comentário);
  - `stop_signals.rs` 105/108 (cfg na 82);
  - `kwin_placement.rs` 246/256/260/267 (cfg na 195).

  Nenhum hit em `ddc-core`. Os lints `unwrap_used`/`expect_used`/`panic` = `warn` seguem no workspace, e `clippy.toml` só libera em testes.
- **5.7 (segurança de escrita no monitor, delegada pela linha 8).**
  - Greps do gate:
    - `Dangerous` existe no core (`error.rs:33`, `capabilities/tests.rs`);
    - não há `Confirm::Yes` fora de fronteira humana ou teste;
    - `DdcHiMonitorBackend` em `crates/*/tests` e `apps` só aparece em `crates/ddc-adapters/tests/real_monitor.rs` (7 chamadas a `new()`) e no composition root (`lib.rs:29`, `lib.rs:79`);
    - `real_monitor.rs` tem 7 `#[test]`, todos com `#[ignore = "…DDC_HW_TESTS=1"]`, e 0 bytes de diff contra a base.
  - **Leitura do único teste alterado, `worker/tests.rs` (`3a3afa6`):**
    - **(b)** (`tests.rs:1200`) segue pelo mesmo caminho: as 10 políticas de `custom_settle_table()` (`1026`, linhas `1032`–`1044`) passam por `write_then_read_with` (`1073`). Este chama `on_bus_with` (`422`), que monta um `Worker<FakeDisplays, VirtualClock>` sobre `FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)])`. Só mudou a comparação dos sleeps (`assert!` com `runs()`, equivalente).
    - **(e)** (`1336`) usa `client_of` (`1298`), que é `WorkerClient::spawn(FakeDisplays::with([…Behaviour::Answer]), budgets, policies)` e dá um `WorkerClient<FakeDisplays>`. Cria 2 clientes por política (`short_vcp_budgets` `1280`, `ample_vcp_budgets` `1286`). `assert_write_budgets` (`1306`) só chama `write_budget_of`, sem pedido e sem sleep. Os dois `write_then_read_with` seguem em relógio virtual.
    - **(f)** só ganhou o limite de baixo (`1431`).
    - O arquivo cita `DdcHi` só em `DdcHiBudgets`, e `/dev/i2c` só num comentário (950), que já estava na base. Não há `DdcHiDisplays` nem `DdcHiMonitorBackend`.
  - `ddc_hi_backend/tests.rs` (sem diff desde `b53119f`) constrói `DdcHiMonitorBackend::new()` e só lê `client.policies`, sem nenhum pedido. Coberto pela prova abaixo.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`. Ali dentro, `stat` mostra os 16 como "arquivo comum vazio". Um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) ficou ligado nesses arquivos das 12:53:05 às 13:20:14.
    - Controle positivo: `cat /dev/i2c-3` no sandbox gerou 1 evento (`mask=0x20`, 12:53:10).
    - `cargo test --workspace --locked` no sandbox, às 12:53:18, antes de qualquer execução nativa: `SUITE_EXIT=0`, 413 passed, 0 failed, 9 ignored, ZERO eventos.
    - Também com ZERO eventos, tudo no sandbox: build, cross-check, fmt, clippy, `cargo llvm-cov`, `cargo audit`, os 12 Verify literais, a coleta de evidência, os dois smokes diretos do binário release (`DDC_TRAY_FAKE=1`, `A_EXIT=0`, `B_EXIT=0`), o Playwright, as 70 execuções do (f) e todas as mutações e sondas.
    - No log inteiro há 1 evento, o do controle.
    - As execuções nativas (12:57:23–12:57:54) vieram depois da prova e rodam o mesmo código. O observador não as vê, porque os nós reais são outros inodes.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` 0.22.2 (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, warning permitido que já estava no lock da base. `Cargo.lock` sem diff desde `b53119f`.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração anterior** (`b53119f..HEAD`), 3 ao todo:
  - `117901f` docs (63 caracteres): a decisão `D-2026-10-01-input-switch-autostart-6.md` e o CONTEXT (linhas 1 e 8);
  - `3a3afa6` test (70 caracteres): só `worker/tests.rs`;
  - `8626fa6` docs (66 caracteres): CONTEXT (SHA-256 recongelado), PLAN e SUMMARY.

  Escopo = slug, `.jdi/` nunca no mesmo commit que código (D-12), e os 3 trazem `Claude-Session:`.
- **Zero byte de diff desde `b53119f`** fora de `worker/tests.rs`, incluindo `Cargo.lock`, `README.md`, `CHANGELOG.md` e `docs/`. Código de produção não mudou nesta iteração. Contra a base seguem com 0 bytes:
  - `crates/ddc-core`;
  - `apps/ddc-tray/scripts/smoke-sni.sh`;
  - `real_monitor.rs`;
  - `capabilities/`.
- **D-2026-10-01-6.** Conforme:
  - (a) A tabela tem 10 políticas (`tests.rs:1032-1044`), com `1 µs/100 ms` e `1 dia/1 µs`.
  - (b) Há dois clientes por política: `min(70 ms, janela/2)` e `1000·janela + 1 s`, com `capabilities` = 2·vcp e `enumerate` = 3·vcp. Os dois exigem `vcp + janela` para `0x60` e `vcp` para os 255 outros códigos (`1314`).
  - (c) O (f) ganhou `reads * 2 >= ceil(janela/passo)` (`1431`).
  - (d) O awk do Verify 1 casa exatamente `worker.rs:538-543`.
  - (e) O dep-info dos Verify 1 e 8 lista os 10 `.rs` esperados.
  - Os nomes dos 7 testes não mudaram.
  - **Mas** a conclusão de (d), "o orçamento real é exatamente o que (e) prova", não se sustenta: o pós-processamento pode ir para `call`/`transact` (W-26). E (e), "um `#[path]` em qualquer forma muda esse conjunto", também não: ver W-24.
- **D-17(a)** ("cliente e worker leem a janela e o passo SÓ da `RetryPolicies`"). Conforme no código:
  - `settle_input` lê só `self.policies.input_settle` (`worker.rs:313`);
  - o `sleep` usa `settle.step` (`worker.rs:320`);
  - `write_budget` usa `policies.input_settle.window` (`worker.rs:190`).
- **D-3 (250 ms / 3 s).** O código e o `Default` estão conformes. A W-25 mostra que um teto de 2 s posto só em `Worker<_, SystemClock>::run` reduziria a janela de produção para 2 s com tudo verde.
- **D-14 / D-19 / D-2026-10-01-1 a -5.** Sem diff de código. `write_budget_of` (`worker.rs:464`) segue sendo o único caminho, e `write_vcp` a chama (`worker.rs:539`). As linhas `INPUT_SETTLE_*` são exatamente as 4 esperadas, em `retry.rs:27/32/112/113`, mais o comentário `///` da linha 22.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo.
- **PLAN x commits.**
  - Os 33 arquivos do diff da phase seguem em `## Files modified (all tasks)`.
  - A entrada "Rodada 2, iteração 4 (D-2026-10-01-input-switch-autostart-6)" foi acrescentada (`PLAN.md:139`).
  - Resíduo cosmético que segue: `PLAN.md:153` ainda diz "emendados por D-13, D-14, D-15, D-16 e D-17".
- **SUMMARY.** A seção "Rodada 2, iteração 4" (`SUMMARY.md:289`) bate com o reproduzido aqui:
  - M1r32, M1g20 e M1gs1 caem na linha `1µs, 100ms` (32, 20 e 100 contra 100 000);
  - M1bv dá `202s != 101.1s`;
  - N1 e M1gw100 dão sleeps `10.000001s` e `100ms` contra `1µs`;
  - N2spawn50 dá 2 leituras e N2clk16 dá 7;
  - N3 dá `0x04: 150ms != 50ms`;
  - N4slack e N4floor deixam a lib em `76 passed`, mas o Verify 1 sai sem `OK`;
  - (f) 30/30 em 0,32 s com 20 leituras; (b) em 0,03 s e (e) em 0,04 s.

### Achados do critic da iteração anterior, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| W-23: M1r32, M1r20, M1g20, M1g32, M1cw32 | FECHADO | (b), linha `1µs, 100ms`: leituras 32, 20, 20, 32 e 32 contra 100 000. M1cw32 também cai em (e) (`VCP 50ms: 0x60`). Verify 1 sem `OK` |
| W-23: M1bv (piso atrelado ao VCP) | FECHADO | (e), cliente folgado: `5ms, 100ms, VCP 101s: 0x60`, `202s != 101.1s` |
| N1 `window.max(step)` | FECHADO | (b) `86400s, 1µs: sleeps [(10.000001s, 1)] != [(1µs, 1)]`; (e) `86400.0000005s != 1.5µs` |
| N2: N2spawn50, N2clk16 | FECHADO na faixa declarada | (f): `2 reads…` e `7 reads of the input, fewer than half of the 20 that fit`. Limiares conferidos: N2spawn12 vermelha (9 leituras), N2spawn11 verde (≤ ~11 ms, declarado não afirmado), N2spawnc45 vermelha (22), N2spawnc48 verde (≥ ~4,8 ms). **Resíduo: W-25** |
| N3 (reset orçado como input) | FECHADO | (e) `0x04: 150ms != 50ms` |
| N4: N4slack, N4floor (no `write_vcp`) | FECHADO nessa forma | lib `76 passed`; Verify 1 sem `OK` pelo corpo fixado. **Resíduo: W-26** (o mesmo, um nível abaixo) |
| Abaixo do menor valor: M1gs1, M1gw100 | FECHADO | M1gs1: (b) 100 ≠ 100 000. M1gw100: (b) sleeps `[(100ms, 1)] != [(1µs, 1)]` e (e) |
| W-24: M8p3 (`#[path]` multilinha sob `#[rustfmt::skip]`) | FECHADO nessa forma | Verify 8 e Verify 1 sem `OK`, pelo dep-info (`settle_checks.rs` entra no conjunto). **Resíduo não objetivo: W-24 reaberta** (M8p6–M8p8) |

### Mutações reproduzidas pelo reviewer
**Montagem.** Todas rodaram num repositório git descartável, `/var/tmp/rv9/repo`, já apagado, com `target/` próprio:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1.

A cópia é fiel ao repositório real:
- árvores idênticas: base `0c8815c7…` e HEAD `043a21dd…`;
- `diff --stat` idêntico: 33 arquivos, `2799 insertions(+), 87 deletions(-)`.

**Execução.**
- Mutações aplicadas por substituição exata, com 1 ocorrência conferida, e formatadas com `cargo fmt`.
- Para cada uma: `fmt --check`, `cargo clippy -p ddc-adapters --all-targets -- -D warnings`, lib inteira e Verify 1 ou 8 literal.
- Reversão: `git checkout -- .` + `git clean -fd`. `git status` ficou limpo depois de cada uma.
- Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados.
- Baselines na cópia, todas `OK`: linha 1 (5,30 s a frio e depois 1,55 s), linha 8 (1,86 s) e linha 10 (0,02 s).
- Todas as mutações abaixo saem com `fmt` 0 e clippy 0. Verdes repetidas 3/3.

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | As 12 do critic (M1r32, M1g20, M1cw32, M1bv, N1, N2spawn50, N2clk16, N3, N4slack, N4floor, M1gs1, M1gw100) | Todas VERMELHAS no Verify 1. Ver a tabela anterior |
| 1 | Antigas: M1g, M1g16, M1g32, M1gw, M1c, M1s, M1s4, M1s2, M1s23, M1d, M1bf150, M1m, M1f, M1r16, M1r20, M1gc, M1t3, M1x, M1xr, M1cv, M1cd, M1call | Todas VERMELHAS. Por quem: M1g pela (b) `5ms, 100ms` (2 ≠ 20) e pelo (f); M1c pela (b) `2s, 10s` (3 ≠ 5) e pelo (e); M1s pela (b) (10 ≠ 5); M1s2 e M1s23 pela (b) `7s, 7s` (2 ≠ 1); M1d pela (b) e pelo (e) (`4.32s`); M1bf150 pelo (e) (`200ms`); M1m pela (b), pelo (e) (`1.05s`) e pelo (f); M1gc pelo (e) (`1.05s`); M1t3 e M1x pela (b) (21 ≠ 20); M1xr pela (c) e por `a_panic_or_a_refusal…`; M1cv pelo (f) (`tests.rs:1419`) e pelo corpo fixado; M1cd pelo (f) (`1439`) e pelo corpo fixado; M1call pelo (e) (`0x00: 150ms != 50ms`) e pelo (f) |
| 1 | M1t (janela contada de antes do write) | VERDE 1/1. Declarado não afirmado ("a origem da janela") |
| 1 | P1k99900 `step.max(window / 99_900)` | VERMELHA: (b) 99 901 ≠ 100 000 |
| 1 | P2 `window.min(budgets.vcp * 1_199_999)`, P3 `window.max(budgets.vcp / 1_000_000)`, P3e `window.max(budgets.enumerate / 1_000_000)` | VERMELHAS no (e): `12h, 1 dia, VCP 70ms: 84000s != 86400.07s`; `1.001001001s != 1.001001s`; `5.5µs != 1.5µs`. As fronteiras declaradas de (e) se confirmam |
| 1 | **P1k99950** `step.max(window / 99_950)` | **VERDE 3/3.** `100 ms / 99 950` trunca para 1 µs em `Duration`. Nota P1 (precisão, não objetiva) |
| 1 | **N5w**: teto de 2 s na janela em `Worker<_, SystemClock>::run` (`MAX_WORKER_HOLD`) | **VERDE: lib `76 passed`, Verify 1 `OK` 3/3.** No workspace da cópia: 413/0/9, clippy 0, Verify 8 e 10 `OK`. Ver W-25 |
| 1 | **N5s**: teto de 100 ms no passo em `run`; **N5wf**: piso de 100 ms na janela em `run`; **N5sp**: o teto de 2 s no `spawn`, com o cliente guardando a política original | **VERDES 3/3 cada.** W-25 |
| 1 | **N5clk**: método `Clock::max_hold()` (`Duration::MAX` por padrão, 2 s no `SystemClock`) usado em `settle_input` | **VERDE 3/3**, e sobrevive também à correção proposta. W-25 |
| 1 | **N4call**: `answer.recv_timeout(budget + QUEUE_SLACK)` (50 ms) em `WorkerClient::call` | **VERDE 1+3.** W-26 |
| 1 | **N4tfloor**: `self.call(budget.max(MIN_CALL_BUDGET), …)` (100 ms) em `WorkerClient::transact`; **N4tslack**: `budget + QUEUE_SLACK` no mesmo lugar | **VERDES 1+3 cada.** N4tfloor também no workspace da cópia: 413/0/9, clippy 0, Verify 8 `OK`. W-26 |
| 1 | **Sonda de efeito, só na cópia e só com `FakeDisplays`** (`rv_probe`): cliente `Default`, monitor que mantém o input antigo, mais um write de brilho preso com VCP de 20 ms | HEAD: worker `{250ms, 3s}`, write de `0x60` em **3,00 s / 12 leituras**, brilho em `Timeout` após 20,07 ms. **N5w: worker `{250ms, 2s}`, 2,00 s / 8 leituras.** N5s: 30 leituras. N5clk: 2,00 s / 8. **N4tfloor: `Timeout` após 100,07 ms. N4call: após 70,06 ms** |
| 1 | **Correção proposta para a W-25, só na cópia**: teste que, para as 10 linhas da tabela + `Default` + `without_backoff()`, lê `worker.policies` pela fila do cliente (`client.call(UNHURRIED, \|worker, _\| Ok::<_, DdcError>(worker.policies))`) e exige igualdade | HEAD verde em 0,00 s. VERMELHAS: N5w, N5s, N5wf, N5sp e N2spawn50. Sobra N5clk, que o texto precisa declarar |
| 8 | M8p `#[path = "worker/settle_checks.rs"]`, M8p2 `#[ path="…" ]`, M8p4 `#[cfg_attr(all(), path = "…")]` | lib `77 passed` (`…worker::tests::compiled_from_settle_checks ... ok`). VERMELHAS nos Verify 8 e 1 |
| 8 | M8p3 (W-24): `#[rustfmt::skip]` + `#[` / `path = "worker/settle_checks.rs"` / `]` | lib `77 passed`. **VERMELHA nos Verify 8 e 1, pelo dep-info** |
| 8 | M8m `macro_rules! planted` + `planted!(test);`; M8x `vcp_table!` com `#[test]` chamado sem `cfg` em `worker.rs`; M8i `mod extra { #[cfg(test)] mod tests {…} }` em `identity.rs` | VERMELHAS no Verify 8 pela âncora da `--list` (`ddc_hi_backend::planted`, `…worker::the_input_source_is_vcp_0x60`, `…identity::extra::tests::planted`) |
| 8 | M8ctl (controle) `#[test] fn planted() {}` em `identity/tests.rs` | Verify 8 `OK` e Verify 1 `OK`, como deveriam |
| 8 | M8p5: `#[path]` multilinha para `../settle_checks.rs` + `include_str!("ddc_hi_backend/worker/tests.rs")`; M8p5noinc, o mesmo sem `include_str!` | VERMELHAS: o dep-info grava `ddc_hi_backend/../…` sem normalizar, e o arquivo congelado também some do conjunto |
| 8 | **M8p6**: `#[rustfmt::skip]` + `#[` / `path = "worker/settle-checks.rs"` / `]`, com o arquivo copiado + `const _: &str = include_str!("tests.rs");` + teste marcador | **`fmt` 0, clippy 0, Verify 8 `OK` e Verify 1 `OK`** |
| 8 | **M8p7 (composta)**: M8p6 com a cópia sem as linhas `5ms/100ms` e `1µs/100ms` e sem o limite de baixo do (f), **mais a M1g (piso de 50 ms no passo) em produção** | **lib `77 passed`, `fmt` 0, clippy 0, Verify 8 `OK` 2/2 e Verify 1 `OK` 2/2.** `worker/tests.rs` segue `79d40888…` e no dep-info. Ver W-24 |
| 8 | **M8p8**: a mesma composta com `worker/settle_checks.txt` | **Verify 8 e 1 `OK` 2/2** |
| 8 | Endurecimento testado: o conjunto de TODOS os tokens do dep-info sob `crates/ddc-adapters/src`, normalizados com `realpath -m` e em qualquer extensão | HEAD: 15 entradas. M8p5, M8p7 e M8p8 acrescentam exatamente o arquivo trocado, então falhariam |
| 2 | M2byte: 1 caractere em `crates/ddc-core/src/lib.rs` | VERMELHA: Verify 2 sem `OK` |
| 6 | M6 `toggle_or_report(&**entry, \|_, _\| {})` em `tray::flip_autostart` | O teste passa sozinho, mas a linha `ddc-tray: could not change …` some. VERMELHA: Verify 6 sem `OK` |
| 10 | M10c `// TODO:` em `app.js`, M10n `# todo:` em `private-bus.sh`, M10g `/* labels */` em `en.js` | VERMELHAS. M10ctl `// TODO(#12):` dá `OK` |

**Limite de leituras do (f) no HEAD, entre 10 e 21.**
- 30 execuções isoladas (`--exact …input_write_through_the_client_outlives_the_vcp_budget`), em `bwrap`, na cópia com a árvore do HEAD: 30/30 `ok`, todas em 0,32 s.
- Mais 30 execuções numa cópia instrumentada, que só ganhou um `eprintln!` da contagem: 30/30 `ok`, todas com `rv-reads=20` (limites de 10 a 21) e `waited` de 100,05 a 100,12 ms.
- Sob 24 laços de CPU (cada um sob `timeout 75`, conferidos encerrados): 10/10 `ok`, em 0,32–0,33 s, sempre com 20 leituras.
- A folga até o limite de 0,5 s do Verify é de 0,17 s.

**Linha 9:** 4/4 execuções literais `OK` (`bwrap` 21,06 s com o build release, depois 5,63 e 5,61 s; nativo 5,61 s). Mais 1 execução direta dos dois scripts DENTRO do `bwrap`: `A_EXIT=0`, `B_EXIT=0`, zero eventos. A W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração anterior:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. Atenção: a "correção" natural disso (N5w, teto só no `run`) passa em tudo e quebra a D-3, ver W-25.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O README documenta.
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final: `Exec=…/autostart_entry-c0e56ec32fbab8ff $` no `cat -A`.
  - `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN): FECHADO, com resíduo cosmético.** `PLAN.md:153` ainda diz "emendados por D-13 … D-17".
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.**
  - Tempo: 0,32 s em repouso (30/30) e 0,32–0,33 s sob carga (10/10). A folga até os 0,5 s do Verify é de 0,17 s.
  - Leituras: 20 em 30/30 e em 10/10, dentro de 10..21.
  - O `rust-windows` mostra primeiro.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 4/4 literais + 1 direta no `bwrap` OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-20, W-21, W-22: FECHADOS** e seguem vermelhos.
- **W-23 (linha 1): FECHADO.** M1r32, M1r20, M1g20, M1g32, M1cw32 e M1bv ficam VERMELHAS.
- **Achados do critic anterior (N1–N4 e abaixo do menor valor): FECHADOS na forma relatada**, ver a tabela. O resíduo segue nas W-25 e W-26.
- **W-9, W-10: TRATADOS** (iteração 3). **W-12, W-13: FECHADOS** (iteração 4). **W-15, W-17, W-18, W-19: FECHADOS** (iterações 5 e seguintes).

- **W-24 (linhas 1 e 8, não objetiva: evasão deliberada; D-2026-10-01-6(e)): FECHADA na forma M8p3, REABERTA noutra forma.**
  - O que as linhas afirmam: "pelo dep-info … um `#[path]` em qualquer forma falha".
  - Por que escapa: o filtro `grep -oE 'src/ddc_hi_backend(/[A-Za-z0-9_/]+)?\.rs$'` não enxerga um arquivo trocado cujo nome tenha hífen (`worker/settle-checks.rs`) ou outra extensão (`worker/settle_checks.txt`). E `include_str!("tests.rs")` mantém o arquivo congelado no dep-info sem compilá-lo como código.
  - **M8p7:** com M1g em produção e o harness enfraquecido, dá Verify 1 `OK` e Verify 8 `OK` (2/2), com `fmt`, clippy e lib verdes. O SHA-256 `79d40888…` segue intacto.
  - Precisa de 4 peças deliberadas (`#[rustfmt::skip]`, atributo multilinha, nome fora do charset e `include_str!`). Por isso fica como nota, como antes.
  - Endurecimento testado: comparar o conjunto de TODOS os tokens do dep-info sob `crates/ddc-adapters/src`, normalizados com `realpath -m` e em qualquer extensão, com a lista exata (15 no HEAD). Isso pega M8p5, M8p7 e M8p8.

Novos nesta iteração:
- **W-25 (linha 1, lacuna objetiva provável; D-3, D-17(a), D-2026-10-01-6).**

  **O que a linha afirma.** A (b) declara faixas "(um worker que lesse passo ou janela de outro lugar que não a política, ou lhes impusesse, com qualquer nome, um limite DENTRO destas faixas, falha aqui)". Na janela, isso inclui "teto absoluto abaixo de 1 dia". O "NÃO afirmado" só exime, no caminho de produção, o "piso no passo só no caminho de produção de ~11 ms ou menos".

  **O que os testes cobrem.** A tabela roda só em `Worker::new` + `VirtualClock`. O (f) é o único teste do caminho de produção (`spawn`, `Worker<_, SystemClock>::run`, `SystemClock`), com uma única política de 5 ms/100 ms e leituras aceitas de 10 a 21. Por isso escapam:
  - qualquer teto de janela acima de cerca de 50 ms;
  - qualquer piso de janela de até cerca de 105 ms;
  - qualquer teto de passo acima de cerca de 4,8 ms,

  desde que postos só no caminho de produção.
  - **N5w**, 1 constante e 2 linhas, com desculpa natural (a própria W-4):
    ```rust
    /// The longest one input switch may hold the worker: every other monitor waits behind it.
    const MAX_WORKER_HOLD: Duration = Duration::from_secs(2);
    // Worker<S, SystemClock>::run, worker.rs:210
    fn run(mut self, jobs: Receiver<Job<S>>) {
        let settle = &mut self.policies.input_settle;
        settle.window = settle.window.min(MAX_WORKER_HOLD);
        for job in jobs {
    ```
    `fmt` 0, clippy `-D warnings` 0 (workspace), 413/0/9, **Verify 1 `OK` 3/3**, Verify 8 e 10 `OK`. O (h) só lê `client.policies`, que o `run` não toca.
  - **Efeito em produção, medido com `FakeDisplays`:** a troca de input com o `Default` assenta em **2,00 s / 8 leituras, em vez de 3,00 s / 12**. Isso quebra a janela de 3 s da D-3 sem nenhum teste vermelho.
  - Mesma família, todas com Verify 1 `OK` 3/3:
    - N5s (teto de 100 ms no passo, 30 leituras em vez de 12);
    - N5wf (piso de 100 ms na janela);
    - N5sp (o teto de 2 s no `spawn`);
    - N5clk (`Clock::max_hold()` de 2 s no `SystemClock`).
  - **Correção testada na cópia:** um teste que lê `worker.policies` pela fila do cliente, para cada linha da tabela, mais `Default` e `without_backoff()`, e exige igualdade com a política dada. Fora do conjunto `input_write_*` ou dentro do (e):
    ```rust
    for policies in custom_settle_table().into_iter().chain([RetryPolicies::default(), no_backoff()]) {
        let client = client_of(policies, short_vcp_budgets(policies.input_settle));
        let held = client.call(UNHURRIED, |worker, _| Ok::<_, DdcError>(worker.policies)).unwrap();
        assert_eq!(held, policies);
    }
    ```
    HEAD verde (0,00 s). N5w, N5s, N5wf, N5sp e N2spawn50 ficam VERMELHAS.
  - **Para convergir, o texto também precisa mudar.** O "NÃO afirmado" deve incluir os limites postos só pelo relógio de produção, isto é, um método de `Clock` exclusivo do `SystemClock` que mantenha a política de 5 ms/100 ms do (f) entre 10 e 21 leituras (N5clk). Um teste do caminho de produção com a janela do `Default` custaria 3 s reais.
  - Pede novo SHA-256 de `worker/tests.rs` e nova D-XX.

- **W-26 (linha 1, lacuna objetiva provável; D-2026-10-01-6(d)).**
  - **O que a linha afirma:** "o corpo de `WorkerClient::write_vcp` é EXATAMENTE `let budget = self.write_budget_of(code);` seguido do `transact` (o Verify o fixa), então o orçamento real de um write é o que este teste prova". A decisão diz o mesmo: "Assim, o orçamento real é exatamente o que (e) prova".
  - **Por que escapa:** o Verify fixa só o corpo de `write_vcp`. `WorkerClient::transact` (`worker.rs:492`) e `WorkerClient::call` (`worker.rs:470`, `recv_timeout` na 483) seguem livres. E as margens do (f) (< `vcp + janela + 1 s` no `0x60`, < `vcp + janela` nos outros códigos) absorvem até 100 ms.
  - **N4tfloor** ("abaixo de uma ida e volta DDC/CI nenhuma resposta chega"):
    ```rust
    /// Below one DDC/CI round trip no answer can come.
    const MIN_CALL_BUDGET: Duration = Duration::from_millis(100);
    // WorkerClient::transact, worker.rs:495
    self.call(budget.max(MIN_CALL_BUDGET), move |worker, deadline| op(worker, &id, deadline))
    ```
    `fmt` 0, clippy 0 (workspace), 413/0/9, **Verify 1 `OK` 1+3**, Verify 8 `OK`. A sonda mostra que um write de brilho preso, com VCP de 20 ms, desiste após **100,07 ms em vez de 20,07 ms**. É um piso absoluto bem acima dos 0,5 µs que (e) declara pegar.
  - **N4call** (`recv_timeout(budget + QUEUE_SLACK)`, 50 ms em `call`): verde, 70,06 ms na sonda. **N4tslack** (`budget + QUEUE_SLACK` em `transact`): verde.
  - **Correção:** fixar no Verify 1 também os corpos de `WorkerClient::call` e `WorkerClient::transact` (mesma técnica do awk), ou passar o texto a afirmar só "o orçamento real é o de `write_budget_of` mais, no máximo, a tolerância do (f)". Pede emenda do DoD e nova D-XX.

Notas, sem ação no código:
- **P1 (linha 1, precisão, não objetiva).** "piso relativo `passo ≥ janela/k` com k < 100 000": com `Duration`, `100 ms / k` trunca para 1 µs quando k > 99 900. A P1k99950 fica verde e a P1k99900 fica vermelha. Escrever "k ≤ 99 900" ou "até o truncamento em ns".
- **M1t (janela contada de antes do write).** Fica verde e é declarada não afirmada.
- **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. Manter, nas próximas rodadas (reviewer, critic, `/jdi-ship`), a leitura do 5.7 e a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **`.jdi/DECISIONS.md`.** É uma visão gerada e ignorada (`.gitignore:31`). As fontes estão em `.jdi/decisions/` (inclusive `D-2026-10-01-input-switch-autostart-6.md`), e `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter, com 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado e em menos de 0,5 s. (b) faz uma TABELA de 10 políticas em relógio virtual, com faixas declaradas. (e) usa dois clientes por política e os 256 códigos. (f) usa o `WorkerClient` real, com leituras entre `ceil(janela/passo)/2` e `+1`. (g) fixa `Default`/`without_backoff()` e (h) cobre a fiação. As linhas `INPUT_SETTLE_*` são exatamente as 4. O corpo de `write_vcp` é fixado, o dep-info lista os 10 `.rs` e 3 arquivos de teste são congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,57 s (`bwrap`) e 1,57 s (nativo). Os 7 nomes listados, cada um `1 passed; 0 failed`: (a), (c), (d) e (g) em 0,00 s; (b) em 0,03 s; (e) em 0,04 s; (f) em 0,32 s (30/30 em repouso; 0,32–0,33 s sob 24 laços; 20 leituras, dentro de 10..21). (h) listado e passa. `git grep` das constantes = `retry.rs:27/32/112/113` (o `///` da 22 fica excluído). Corpo de `write_vcp` = `worker.rs:538-543`. Dep-info = os 10 `.rs`. SHA-256 `79d40888…`, `ec2d2980…` e `12ac10db…` batem. As 12 do critic e as 22 antigas ficam VERMELHAS. **Resíduos: W-25 (N5w, N5s, N5wf, N5sp, N5clk) e W-26 (N4tfloor, N4call, N4tslack) dão `OK`; nota P1.** |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,11 s. Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base sai 0; sem `sleep`. Sonda de 1 caractere (M2byte) reproduzida: VERMELHA. |
| 3 | View-model puro + 4 testes nomeados, com o texto inteiro conferido por igualdade com frases LITERAIS (papéis de mantida e pedida) em `en`/`pt-BR`, para o aviso de input e o genérico; `tests/ui` inteira verde; `view-model.test.mjs` congelado por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 0,25 s. `npm run test:unit`: `# tests 165 / # pass 165 / # fail 0`, 88.48%. SHA-256 `7c650133…` bate. M4s, M4g, M4ge, M4u e M3f herdadas (arquivos e Verify sem diff desde `b53119f`). |
| 4 | Aviso visível de ponta a ponta. `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; texto inteiro de cada toast contra a frase literal com papéis; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 18,58 s (`bwrap`) e 18,73 s (nativo), com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed; 10 `✓` em `input-notice.spec.mjs`. SHA-256 `92f99e00…`/`ee930894…` batem. M4s, M4g, M4u e M4f herdadas (arquivos sem diff). |
| 5 | Menu nativo: rótulo en/pt-BR; item nas 2 plataformas com marca = estado do SO; `from_id`; fiação LINUX (ksni) e testes de `tray`. Fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 2,99 s (`bwrap`) e 1,94 s (nativo). Os 6 nomes existem em `--list` e passam; `menu:: i18n::` dá `20 passed; 0 failed; 0 ignored`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); o chamador de PRODUÇÃO imprime a linha do `crate::report`; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,41 s. Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` imprime 2 vezes a linha `ddc-tray: could not change the start-with-system entry: read-only home`. M6 reproduzida: o teste passa sem a linha, e o Verify fica VERMELHO. |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 1,31 s (`bwrap`) e 0,50 s (nativo). `[Desktop Entry]`, `Name=test` e `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` impressos; listagem real idêntica (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: tokens de hardware só nos 3 arquivos de produção; nesses 3, só `#[cfg(test)]` + `mod tests;`, sem atributo com `test` ou `path`; `doctest = false` efetivo e sem bloco `Doc-tests ddc_adapters`; dep-info com os 10 `.rs`; todo teste listado sob `ddc_hi_backend::` num módulo `tests` ancorado; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 1,33 s (`bwrap`) e 1,08 s (nativo). Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `cargo metadata`: `["lib"] ddc_adapters doctest=false`, sem `Doc-tests ddc_adapters`. Dep-info: os 10 `.rs`. A `--list` sob `ddc_hi_backend::` tem 51 testes: `tests` (4), `worker::tests` (32), `hardware::tests` (10) e `identity::tests` (5). `capabilities/` só tem `default.json`, sem diff e sem `autostart`. `tauri-plugin-autostart v2.6.0` -> `ddc-tray`. M8p, M8p2, M8p4 e **M8p3 (W-24)** ficam VERMELHAS; M8m, M8x e M8i também; M8ctl dá `OK`. **Resíduo não objetivo (W-24 reaberta): M8p6, M8p7 e M8p8.** O "nenhum teste fala com monitor real" está no 5.7: leitura + inotify, zero aberturas. |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B; monitor simulado exigido nas duas; marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **4/4 execuções literais**: `bwrap` 21,06 s (build release), 5,63 s e 5,61 s; nativo 5,61 s. Execução direta no `bwrap` com zero eventos de `/dev/i2c-*`. Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor`, `smoke-sni: the popup was still shown 1.5 s later` e `smoke-sni: OK — PID 3092952 registered its tray item, … and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor`, checkmark desmarcado; o 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray`, e o 2º remove; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. PID 24429 intacto. W-16 não apareceu. A, R, B1 e B2 herdadas. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase (caminhos fixos + `crates/ddc-adapters/Cargo.toml` UNIDOS aos não-Rust do diff); `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,03 s. Reproduzidas: M10c (`// TODO:` em `app.js`), M10n (`# todo:` em `private-bus.sh`) e M10g (`/* labels */` em `en.js`) ficam VERMELHAS; M10ctl (`TODO(#12)`) dá `OK`. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,05 s (`bwrap`) e 1,06 s (nativo). Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,01 s. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` (linha 10, "Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18). O último release é `## [0.1.0] - 2026-09-28` (linha 31). Nenhum heading de release novo nesta phase; sem diff desde `b53119f`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330, 333 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento (250 ms, no máximo 3 s); os smokes em `scripts/` são citados (linha 387); as limitações do tray estão em "Known limitations of the tray app" (linha 392). Sem diff desde `b53119f`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers pelos gates. Os 13 Verify automáticos dão `OK` no HEAD, rodados literalmente no `bwrap` e no nativo. Os achados do critic anterior estão fechados por mutação:
- W-23, N1, N2 (na faixa declarada), N3, N4 (no `write_vcp`), M1gs1 e M1gw100 ficam vermelhos, e as 22 antigas também;
- o `#[path]` multilinha da W-24 (M8p3) fica vermelho pelo dep-info nos Verify 1 e 8;
- os controles (M8ctl, M10ctl) seguem sem falso positivo.

A prova por inotify em `bwrap` deu zero aberturas de `/dev/i2c-*`. O (f) leu 20 vezes, dentro de 10..21, em 30/30 e sob carga.

**Antes de seguir para o critic, fechar a W-25 e a W-26** (linha 1, objetivas prováveis, dentro das faixas que a própria linha declara). Se o loop seguir sem elas, o reviewer espera BLOCKED pela linha 1.
- **W-25:** um teto de janela de 2 s posto só em `Worker<_, SystemClock>::run` (N5w, com a desculpa da W-4) passa em fmt, clippy, workspace e Verify 1/8/10. Em produção, a troca de input assenta em 2,00 s em vez dos 3 s da D-3.
  - Correção testada na cópia: um teste que lê `worker.policies` pela fila do cliente, para as 10 linhas da tabela, mais `Default` e `without_backoff()`, e exige igualdade. Ele pega N5w, N5s, N5wf, N5sp e N2spawn50.
  - Declarar no "NÃO afirmado" os limites postos só por um método de `Clock` exclusivo do `SystemClock` que mantenham o (f) entre 10 e 21 leituras (N5clk).
- **W-26:** fixar no Verify 1 também os corpos de `WorkerClient::call` e `WorkerClient::transact`, ou deixar de afirmar que "o orçamento real de um write é o que este teste prova". Hoje um piso de 100 ms em `transact` (N4tfloor) ou uma folga de 50 ms em `call` (N4call) passam em tudo.
- Os dois pedem novo SHA-256 de `worker/tests.rs` (se o teste da W-25 entrar) e nova D-XX.

Outros pontos:
- **W-24** (não objetiva): trocar, nas linhas 1 e 8, o filtro `src/ddc_hi_backend(/[A-Za-z0-9_/]+)?\.rs$` pelo conjunto exato de TODOS os tokens do dep-info sob `crates/ddc-adapters/src`, normalizados com `realpath -m` e em qualquer extensão (15 no HEAD). Isso pega M8p5, M8p7 e M8p8.
- **P1:** escrever "k ≤ 99 900" na faixa do piso relativo do passo.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL em `/dev/i2c-*`, manter a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2). Um `22 reads` ou um (f) lento lá aponta para a W-11;
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura; rodada 2, iteração 4). Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados, e o único evento registrado foi o do controle. Linhas 3–7 e 9–13: `hollow=false`.

- **Linha 1 (objective).** W-25 e W-26 confirmadas, mais duas variantes novas. A sonda usa só `FakeDisplays` e a política `Default`; com cada mutação, o `0x60` assenta em 2,000 s com 8 leituras, quando no HEAD são 3,000 s com 12.
  - N5w: teto de 2 s na janela, em `Worker<_, SystemClock>::run`.
  - N5dl: `deadline = now + budget.min(2 s)` em `WorkerClient::call`.
  - N5serve: `op(self, deadline.min(now + 2 s))` em `Worker::serve`.
  - N4tfloor e N4call: um write de brilho preso, com VCP de 20 ms, desiste depois de 100 ms e 70 ms.

  Com essas mutações, o fmt e o clippy saem 0 e os Verify 1 e 8 imprimem `OK`. As correções pontuais propostas não bastam:
  - o teste que lê `worker.policies` pela fila pega N5w, mas não pega N5dl nem N5serve;
  - fixar `call`/`transact` não pega N5serve.

  Congelar por SHA-256 os arquivos de produção do adapter fecharia a família inteira. O que ainda escaparia:
  - uma troca de módulo só no build de produção, via `#[cfg_attr(not(test), path = ...)]` em `lib.rs`, apontando para arquivos `.md` (deliberado);
  - `hardware.rs`/`identity.rs` fora do congelamento;
  - camadas acima do adapter;
  - o nível do workspace.
- **Linha 2 (objective, baixa gravidade).** O texto diz "o assentamento só existe no adapter", mas o Verify só olha `crates/ddc-core`. Mutação r2app: uma 2ª espera de assentamento em `apps/ddc-tray/src-tauri/src/commands.rs` (`sleep(500 ms)` + releitura), com o fmt e o clippy em 0, os testes passando e o Verify 2 `OK`.
- **Linha 8 (hollow, NÃO objetiva).** W-24 confirmada sem `#[rustfmt::skip]`: um `macro_rules!` emite `#[path = ...] mod tests;` para uma cópia enfraquecida com `include_str!("tests.rs")`. H8: um helper só de teste, sem `cfg`, com `#[allow(dead_code)]` em `worker.rs`. As duas formas são evasão deliberada.

**Verdict:** BLOCKED
