# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Rodada 2, iteração 3 do loop (8ª iteração absoluta, depois do auto-reset 1/3).
> - Branch `phase/input-switch-autostart`, HEAD `714d2df`, base `origin/main` = merge-base `134b665`.
> - O reviewer produziu o conteúdo e o orquestrador o gravou, porque o harness nega escrita de `.md` ao subagente.
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT. O prj2 (`cargo llvm-cov … → coluna Lines >= 80%`) não é comando de shell e é coberto pelo Gate 3. As 2 linhas Manual não têm comando.
> - Prefixos sha256 dos comandos: ctx1 `3f63a511592d`, ctx2 `81b3ba163edc`, ctx3 `8f648d4f4fe0`, ctx4 `05ff88d61fb8`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `6ce495ae4075`, ctx9 `ae65d59b93ea`, ctx10 `24da480a271e`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - As linhas 2 a 7, 9 e 10 batem byte a byte com a extração de `c17b872`.
> - A linha 1 só difere no SHA-256 de `worker/tests.rs` (`124ba97c…` passou a `b1509c2a…`). Isso foi conferido trocando um hash pelo outro: o resto é igual.
> - A linha 8 só difere em `test` passar a `(test|path)` na regex de atributos (D-2026-10-01-input-switch-autostart-5(c)).
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`, tirados do ambiente.
>
> Os 10 do CONTEXT, o prj1 e o prj3 rodaram LITERALMENTE:
> - a linha 4 rodou com `npm ci` e o Playwright inteiro;
> - a linha 9 rodou 3 vezes;
> - a cobertura reusa o Gate 3.
>
> **Herança, só por diff.** `git diff c17b872 HEAD -- . ':!.jdi'` lista só `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs` (+138 −72, `e34e4fc`). Para esse arquivo nada foi herdado: foram refeitos o tempo e a contagem de leituras do (f), todas as mutações da linha 1 e as do orçamento do cliente que o (f) afirma pegar. O Verify 8 mudou, então M8m, M8x, M8i e M8ctl também foram refeitas, além do `#[path]`. Herdado, porque os arquivos e os Verify não têm diff desde `c17b872`:
> - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
> - as mutações A, R, B1 e B2 da linha 9;
> - a M6 da linha 6;
> - a sonda de 1 byte da linha 2;
> - M4s, M4g, M4ge, M4u, M3f e M4f das linhas 3 e 4;
> - M10c, M10n, M10g e M10ctl da linha 10.
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
>
> **Estado da máquina.**
> - O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim.
> - A listagem do `~/.config/autostart` real tem os mesmos sha256 antes e depois: `ls -A` `bf0a7045a8fb…` e `ls -la` `f69da4fd3ea5…`.
> - No fim não sobrou nenhum `dbus-run-session`, watcher, servidor HTTP do Playwright, tempdir `/tmp/smoke-*`, laço de CPU ou observador inotify.
> - `/var/tmp/rv8` foi apagado (1,7 GB): cópia, `target/`, arquivos que cobriam os `/dev/i2c-*` e logs.
>
> **Monitor real.**
> - Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados.
> - As mutações mexeram só em `settle_input`, `input_unsettled`, `write_budget` e `WorkerClient::write_vcp`, sempre sobre `FakeDisplays`. Também plantaram corpos vazios (`fn planted() {}`, `assert_eq!(0x60, 0x60)`), um `#[path]` para uma cópia do arquivo de teste e um `eprintln!` de contagem. O probe da correção roda em relógio virtual com `FakeDisplays`.
> - Nenhuma mutação criou teste que fale com o backend real.
> - Todo `cargo` da cópia rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados (ver 5.7).
>
> **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. A prova por inotify em `bwrap` rodou ANTES de qualquer execução nativa da suíte.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0, e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` também. Única nota: o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados), igual no `bwrap` e no nativo (prj1). Os 9 são os 7 de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs`, todos `needs the dev monitor attached; run with DDC_HW_TESTS=1`. Igual à iteração anterior; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed`. Blocos `Doc-tests`: só `ddc_cli`, `ddc_core` e `ddc_tray`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. Rodou dentro do `bwrap`, com o observador ativo. Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0 no repositório e também A FRIO, na cópia descartável com `target/` próprio (366 crates, 31 s, em `bwrap`). O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a leitura do teste alterado e a prova por inotify em `bwrap`. Houve zero aberturas de `/dev/i2c-*` na suíte, na cobertura, nos dois smokes do binário release, nas 70 execuções do (f) e em todas as mutações, com controle positivo. |
| Consistency | PASS (com warnings) | D-2026-10-01-5 (a), (b) e (c) estão conformes no código, nos testes e nos Verify. Há 3 commits desde `c17b872`, com tipo, escopo e tamanho corretos. A W-23 é falta de força de teste, não violação de D-XX no código. Ela mostra que a frase da linha 1, "piso ou teto, absoluto ou relativo", é mais larga que a faixa de razões janela/passo que a tabela cobre (1 a 20). |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped, 0 failed. Os 6 skipped são só `screenshots.spec.mjs:24/34/47` em `light`/`dark` (`test.skip` condicionado a `SCREENSHOTS=1`). `input-notice.spec.mjs` tem 10 `✓`. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduos novos: **W-23** (linha 1, objetiva provável: limite de leituras ou piso relativo com k ≥ 20 e piso do orçamento atrelado a `budgets.vcp`, todos Verify 1 `OK` 8/8) e **W-24** (linha 8, não objetiva: `#[` multilinha sob `#[rustfmt::skip]`). |

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
  - **Leitura do único teste alterado, `worker/tests.rs` (`e34e4fc`):**
    - **(b)** (`tests.rs:1177-1210`) roda as 8 políticas de `custom_settle_table()` (`1020-1051`) por `write_then_read_with` (`1067-1080`). Este chama `on_bus_with` (`422-440`), que monta um `Worker<FakeDisplays, VirtualClock>` sobre `FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)])`.
    - **(e)** (`1258-1286`) cria, para cada política, `WorkerClient::spawn(FakeDisplays::with([…Behaviour::Answer]), budgets, policies)` (`1266-1267`). Isso dá um `WorkerClient<FakeDisplays>`; o `spawn` não enumera nada até o primeiro pedido, e o teste só chama `write_budget_of`. Os dois `write_then_read_with` também rodam em relógio virtual.
    - **(f)** só mudou em `let most = settle_reads(settle) + 1;` (`1339`) e na mensagem.
    - O arquivo cita `DdcHi` só em `DdcHiBudgets`, e `/dev/i2c` só num comentário (950). Não há `DdcHiDisplays` nem `DdcHiMonitorBackend`.
  - **Demais testes Rust e JS.** Sem diff desde `c17b872`, e cobertos pela prova abaixo.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`. Ali dentro, `stat` mostra os 16 como "arquivo comum vazio". Um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) ficou ligado nesses arquivos das 11:42:37 às 12:02:17.
    - Controle positivo: `cat /dev/i2c-3` no sandbox gerou 1 evento (`mask=0x20`, 11:42:45).
    - `cargo test --workspace --locked` no sandbox, às 11:43, antes de qualquer execução nativa: `SUITE_EXIT=0`, 413 passed, 0 failed, 9 ignored, ZERO eventos.
    - Também com ZERO eventos, tudo no sandbox: `cargo llvm-cov`, build, clippy, os dois smokes do binário release (`DDC_TRAY_FAKE=1`, `A_EXIT=0`, `B_EXIT=0`), as 70 execuções do (f) e todas as mutações, probes e sondas das linhas 1 e 8.
    - No log inteiro há 1 evento, o do controle.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` 0.22.2 (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, warning permitido que já estava no lock da base. `Cargo.lock` sem diff desde `c17b872` (0 bytes).
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração anterior** (`c17b872..HEAD`), 3 ao todo:
  - `00c1b00` docs (70 caracteres): a decisão `D-2026-10-01-input-switch-autostart-5.md` e o CONTEXT (linhas 1 e 8);
  - `e34e4fc` test (69 caracteres): só `worker/tests.rs`;
  - `714d2df` docs (66 caracteres): CONTEXT (SHA-256 recongelado), PLAN e SUMMARY.

  Escopo = slug, `.jdi/` nunca no mesmo commit que código (D-12), e os 3 trazem `Claude-Session:`.
- **Zero byte de diff desde `c17b872`** fora de `worker/tests.rs`, incluindo `Cargo.lock`, `README.md`, `CHANGELOG.md` e `docs/`. Código de produção não mudou nesta iteração. Contra a base seguem com 0 bytes:
  - `crates/ddc-core`;
  - `scripts/smoke-sni.sh`;
  - `real_monitor.rs`;
  - `capabilities/`.
- **D-2026-10-01-5.** Conforme:
  - (a) A tabela tem as 8 políticas da decisão (`tests.rs:1024-1039`), todas diferentes do `Default` (`1043-1044`). Para cada uma, (b) exige `Ok`, 1 write, só leituras do input depois dele, `leituras == ceil(janela/passo)`, `sleeps == [passo; n−1] ++ [janela − passo·(n−1)]` e `Σ == janela` (`1195-1208`). (e) exige `write_budget_of(0x60) == vcp + janela` e `vcp` para brilho, power mode e `0xE1` (`1274-1284`).
  - (b) O limite do (f) passou a `ceil(janela/passo) + 1` (`1339`).
  - (c) A regex de atributos do Verify 8 passou a `(test|path)`.
  - Os nomes dos 7 testes não mudaram.
  - **Mas a frase da linha 1, "piso ou teto … absoluto ou relativo", vai além do que a tabela prova. A razão janela/passo da tabela vai de 1 a 20, e o `budgets.vcp` do (e) é fixo em 70 ms, abaixo de toda janela. Ver W-23.**
- **D-17(a)** ("cliente e worker devem ler a janela e o passo SÓ da `RetryPolicies`"). Conforme no código:
  - `settle_input` lê só `self.policies.input_settle` (`worker.rs:313`);
  - o `sleep` usa `settle.step` (`worker.rs:320`);
  - `write_budget` usa `policies.input_settle.window` (`worker.rs:190`).
- **D-3 / D-14 / D-19 / D-2026-10-01-1 a -4** Sem diff de código. `write_budget_of` (`worker.rs:464`) segue sendo o único caminho, e `write_vcp` a chama (`worker.rs:539`). As linhas `INPUT_SETTLE_*` são exatamente as 4 esperadas, em `retry.rs:27/32/112/113`, mais o comentário `///` da linha 22.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo.
- **PLAN x commits.**
  - Os 33 arquivos do diff da phase seguem em `## Files modified (all tasks)`.
  - A entrada "Rodada 2, iteração 3 (D-2026-10-01-input-switch-autostart-5)" foi acrescentada (`PLAN.md:138`).
  - Resíduo cosmético que segue: `PLAN.md:152` ainda diz "emendados por D-13, D-14, D-15, D-16 e D-17".
- **SUMMARY.** A seção "Rodada 2, iteração 3" (`SUMMARY.md:251`) bate com o reproduzido aqui:
  - M1g `reads 2 != 20`, M1g16 `16 != 20`;
  - M1gw pelo vetor de sleeps e `170ms != 100ms`;
  - M1c `3 != 5` e `5.07s != 10.07s`, M1s `10 != 5`;
  - M1s4 e M1d pelos vetores, e M1d também `4.5s != 4.25s`;
  - M1bf150 `220ms != 170ms`;
  - M1m e M1f em (b), (e) e (f);
  - (f) 30/30 em 0,32 s, e 413/0/9 com 85,25%.

### Achados do critic da iteração anterior, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| Piso no PASSO (W-22): M1g (50 ms), M1g90, M1g16 | FECHADO | M1g: (b) `InputSettle { step: 5ms, window: 100ms }: reads` 2 ≠ 20, Verify 1 sem `OK` 3/3. M1g16: 16 ≠ 20, sem `OK` 3/3. A M1g90 é coberta pelo mesmo motivo: qualquer piso acima de 5 ms muda a linha 5 ms/100 ms |
| Piso na janela + piso no passo: M1gw | FECHADO | (b) pelo vetor de sleeps (`[50ms; 20]` contra `[5ms; 20]`) e (e) (`170ms != 100ms`, `tests.rs:1276`). Verify 1 sem `OK` 3/3 |
| Teto na JANELA, no worker e no cliente: M1c (5 s) | FECHADO | (b) `2s, 10s: reads` 3 ≠ 5; (e) `5.07s != 10.07s` (`tests.rs:1274`). Sem `OK` 3/3 |
| Teto no PASSO: M1s (1 s), M1s4 (`window / 4`) | FECHADO | M1s: (b) `2s, 10s` 10 ≠ 5. M1s4: (b) `300ms, 1s: sleeps` `[250ms ×4]` contra `[300ms ×3, 100ms]`. Sem `OK` 3/3 cada |
| Passar da janela: M1d (`sleep(settle.step)`) | FECHADO | (b) `500ms, 4.25s: sleeps` `[500ms ×9]` contra `[500ms ×8, 250ms]`; (e) `4.5s != 4.25s`. Sem `OK` 3/3 |
| Piso no orçamento do CLIENTE: M1bf150 | FECHADO | (e) `220ms != 170ms` (`tests.rs:1275`). Sem `OK` 3/3 |
| M1m e M1f (W-20, já fechadas) | SEGUEM VERMELHAS | M1m: (b), (e) e (f). M1f: (b) 600 ≠ 20 leituras, (e) `170ms != 100ms`, (f) `32 reads of the input, more than the 21 allowed` (`tests.rs:1347`). Sem `OK` 3/3 cada |
| Não objetivo: `#[path = …]` antes de `#[cfg(test)] mod tests;` | FECHADO na forma de 1 linha | M8p, M8p2 (`#[ path="…" ]`) e M8p4 (`#[cfg_attr(all(), path = "…")]`) ficam VERMELHAS no Verify 8, com lib, fmt e clippy em 0. **Resíduo não objetivo: W-24** (`#[` multilinha sob `#[rustfmt::skip]`) |

### Mutações reproduzidas pelo reviewer
**Montagem.** Todas rodaram num repositório git descartável, `/var/tmp/rv8/repo`, já apagado, com `target/` próprio:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1.

A cópia é fiel ao repositório real:
- árvores idênticas: base `0c8815c7…` e HEAD `92c49e80…`;
- `diff --stat` idêntico: 33 arquivos, `2715 insertions(+), 87 deletions(-)`.

**Execução.**
- Mutações aplicadas por substituição exata, com 1 ocorrência conferida, e formatadas com `cargo fmt`.
- Para cada uma: `fmt --check`, `cargo clippy -p ddc-adapters --all-targets -- -D warnings`, lib inteira e Verify 1 ou 8 literal.
- Reversão: `git checkout -- .` + `git clean -fd`. `git status` ficou limpo depois de cada uma.
- Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados.
- Baselines na cópia, todas `OK`: linha 1 (4,76 s, a frio), linha 8 (1,74 s) e linha 10 (0,02 s).
- Todas as mutações abaixo saem com `fmt` 0 e clippy 0.

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | As 10 do critic, reproduzidas (M1g, M1g16, M1gw, M1c, M1s, M1s4, M1d, M1bf150, M1m, M1f) | Todas VERMELHAS, com Verify 1 sem `OK` em 3/3 cada. Ver a tabela anterior |
| 1 | M1r16 (controle): `const MAX_SETTLE_READS: u32 = 16;` + `for _ in 0..MAX_SETTLE_READS {` no lugar de `loop {` | VERMELHA: (b) 16 ≠ 20; (e); (f) `80.9ms < 100ms`. Sem `OK` 3/3 |
| 1 | M1s2 `step.min(window / 2)` e M1s23 `step.min(window * 2 / 3)` (teto relativo no passo com outro k) | VERMELHAS: (b) `7s, 7s: reads` 2 ≠ 1. Sem `OK` 3/3 cada |
| 1 | M1gc (piso só no cliente): `window.max(1 s)` em `write_budget` | VERMELHA: (e) `1.07s != 170ms`. Sem `OK` 3/3 |
| 1 | M1t3 (janela contada da 1ª leitura): `now + settle.step + settle.window` | VERMELHA: (b) 21 ≠ 20; (e) `105ms != 100ms`. Sem `OK` 3/3 |
| 1 | M1x (leitura extra quando a janela acaba) | VERMELHA: (b) 21 ≠ 20. Sem `OK` 3/3 |
| 1 | M1xr (releitura imediata de uma leitura que falha) | VERMELHA: (c) sleeps `[250ms ×3]` contra `[250ms ×6]`, e `a_panic_or_a_refusal…`. Sem `OK` 3/3 |
| 1 | M1cv (`write_budget_of(code).min(self.budgets.vcp)` em `write_vcp`) | VERMELHA: (f) `Err(Timeout)` (`tests.rs:1341`). Sem `OK` 2/2 |
| 1 | M1cd (`0x60` = `vcp` + janela do `Default`, sem passar por `write_budget_of`) | VERMELHA: (f) `3.06s >= 160ms + 1 s` (`tests.rs:1355`). Sem `OK` 2/2 |
| 1 | M1call (todo write orçado como o de `0x60`) | VERMELHA: (f) `160ms >= 60ms + 100ms` (`tests.rs:1361`). Sem `OK` 2/2 |
| 1 | **M1r32**: `/// No more reads of the input than this per switch: the bus is shared.` `const MAX_SETTLE_READS: u32 = 32;` + `for _ in 0..MAX_SETTLE_READS {` no lugar de `loop {` | **VERDE: lib `76 passed`, Verify 1 `OK` 8/8.** Ver W-23 |
| 1 | **M1r20**: o mesmo, com 20 | **VERDE: Verify 1 `OK` 3/3** |
| 1 | **M1g20**: `settle.step.max(settle.window / 20)` (piso relativo, k = 20) | **VERDE: Verify 1 `OK` 8/8** |
| 1 | **M1g32**: `settle.step.max(settle.window / 32)` | **VERDE: Verify 1 `OK` 3/3** |
| 1 | **M1cw32**: `settle.window.min(settle.step * 32)` no worker E em `write_budget` (teto relativo na janela) | **VERDE: Verify 1 `OK` 8/8** |
| 1 | **M1bv**: `budgets.vcp + policies.input_settle.window.max(budgets.vcp)` (piso do orçamento atrelado ao VCP) | **VERDE: Verify 1 `OK` 8/8** |
| 1 | M1gs1: `MIN_SETTLE_STEP` de 1 ms ("um passo zero giraria o worker") | VERDE 3/3. O piso fica abaixo do menor passo da tabela (5 ms) |
| 1 | M1gw100: `settle.window.max(100 ms)` | VERDE 3/3. O piso é igual à menor janela da tabela. Não objetivo |
| 1 | M1t (janela contada de ANTES do write): `let asked = self.clock.now();` antes do `transact`, e `end = asked + window` | VERDE 3/3. Não afirmado: a D-3 diz "depois de um write bem-sucedido", e "a janela conta a partir do write" está nas Notes ("não trava"). No relógio virtual o write custa 0 |
| 1 | **Correção proposta, só na cópia**: linha `(1 µs, 100 ms)` na tabela, com 100 000 leituras, mais, no (e), um 2º cliente por política com `DdcHiBudgets { vcp: settle.window * 3, ..budgets }`, exigindo `write_budget_of(0x60) == vcp + janela` | HEAD verde (`76 passed`), `fmt` 0, clippy 0, (b) em 0,03 s e (e) em 0,04 s. VERMELHAS: M1r20, M1r32, M1g20, M1g32, M1cw32 e M1gs1 (pela linha `1µs, 100ms`) e M1bv (pelo cliente folgado, `600ms != 400ms`). M1g, M1m e M1bf150 seguem vermelhas. Sobram M1gw100 e M1t |
| 8 | M8p `#[path = "worker/settle_checks.rs"]` antes de `#[cfg(test)]` em `worker.rs`, com o arquivo copiado de `worker/tests.rs` | lib `76 passed`, `fmt` 0, clippy 0. VERMELHA: Verify 8 exit ≠ 0 |
| 8 | M8p2 `#[ path="worker/settle_checks.rs" ]` e M8p4 `#[cfg_attr(all(), path = "…")]` | VERMELHAS: Verify 8 exit ≠ 0 |
| 8 | **M8p3**: `#[rustfmt::skip]` + `#[` / `    path = "worker/settle_checks.rs"` / `]`, em 3 linhas | **`fmt` 0, clippy 0, Verify 8 `OK`.** O rustfmt preserva a forma. Na composta, `settle_checks.rs` perde a linha 5 ms/100 ms e ganha um teste marcador, mais a M1g: `ddc_hi_backend::worker::tests::compiled_from_settle_checks ... ok` (77 passed), `worker/tests.rs` segue `b1509c2a…`, e **Verify 1 `OK` e Verify 8 `OK`**. Ver W-24 |
| 8 | M8m `macro_rules! planted` + `planted!(test);` em `ddc_hi_backend.rs` | `ddc_hi_backend::planted ... ok` (77 passed). VERMELHA: Verify 8 exit ≠ 0 |
| 8 | M8x `vcp_table!` com `#[test]` literal, definido em `lib.rs` e chamado sem `cfg` em `worker.rs` | `…worker::the_input_source_is_vcp_0x60 ... ok`. VERMELHA: Verify 8 exit ≠ 0 |
| 8 | M8i `mod extra { #[cfg(test)] mod tests { #[test] fn planted() {} } }` em `identity.rs` | `…identity::extra::tests::planted ... ok`. VERMELHA: Verify 8 exit ≠ 0 |
| 8 | M8ctl (controle) `#[test] fn planted() {}` no fim de `identity/tests.rs` | Verify 8 `OK`, como deveria |

**Limite de leituras do (f) no HEAD, agora com folga de 1.**
- 30 execuções isoladas (`--exact …input_write_through_the_client_outlives_the_vcp_budget`), em `bwrap`, na cópia com a árvore do HEAD: 30/30 `ok`, todas em 0,32 s.
- Mais 30 execuções numa cópia instrumentada, que só ganhou um `eprintln!` da contagem antes do `assert!`: 30/30 `ok`, e as 30 com `rv-reads=20 most=21`.
- Sob 24 laços de CPU (cada um sob `timeout 75`, conferidos encerrados): 10/10 `ok`, em 0,32–0,33 s.
- A folga até o limite de 0,5 s do Verify é de 0,17 s.

**Linha 9:** 3/3 execuções literais `OK` (5,63 / 5,61 / 5,64 s), mais 1 execução direta dos dois scripts DENTRO do `bwrap` (`A_EXIT=0`, `B_EXIT=0`, zero eventos). A W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração anterior:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. Código do assentamento inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O README documenta.
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final: `Exec=…/autostart_entry-c0e56ec32fbab8ff $` no `cat -A`.
  - `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN): FECHADO, com resíduo cosmético.** `PLAN.md:152` ainda diz "emendados por D-13 … D-17".
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.**
  - Tempo: 0,32 s em repouso (30/30) e 0,32–0,33 s sob carga (10/10). A folga até os 0,5 s do Verify é de 0,17 s.
  - Leituras: 20 de no máximo 21 em 30/30. A tolerância de 1 leitura (D-2026-10-01-5(b)) tirou a margem zero no Linux e cobre a ressalva do timer do Windows.
  - O `rust-windows` mostra primeiro. Um `22 reads` lá apontaria para outra premissa do timer, não para o worker.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 3/3 literais + 1 direta no `bwrap` OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-20 (linha 1): FECHADO.** M1f e M1m seguem VERMELHAS, em (b), (e) e (f).
- **W-21 (linha 8): FECHADO.** M8m, M8x e M8i seguem VERMELHAS pela âncora da `--list`, e o controle M8ctl segue `OK`.
- **W-22 (linha 1, piso no passo): FECHADO.** M1g e M1g16 ficam VERMELHAS pela linha 5 ms/100 ms da tabela. M1gw fica vermelha por (b) e (e).
- **Achados do critic anterior (oito sobreviventes + `#[path]`): FECHADOS**, ver a tabela acima. O resíduo segue nas W-23 e W-24.
- **W-9, W-10: TRATADOS** (iteração 3). **W-12, W-13: FECHADOS** (iteração 4). **W-15, W-17, W-18, W-19: FECHADOS** (iterações 5 e seguintes).

Novos nesta iteração:
- **W-23 (linha 1, lacuna objetiva provável; D-17(a), D-2026-10-01-5(a), D-19).**

  **O que a linha afirma:**
  - em (b): "um worker que … impusesse a eles [passo e janela] um piso ou um teto (com qualquer nome, absoluto ou relativo) … falha aqui";
  - em (e): "um cliente que … impusesse piso ou teto ao orçamento falha aqui".

  **Os testes cobrem só uma faixa de razões.** A razão janela/passo da tabela vai de 1 (7 s/7 s) a 20 (5 ms/100 ms, 20 leituras). Um piso no passo relativo à janela, ou um teto na janela relativo ao passo, com k ≥ 20, não muda nenhuma linha. O mesmo vale para um limite de leituras ≥ 20, que é a mesma coisa com outro nome. O (e) usa `budgets.vcp` fixo em 70 ms, abaixo de toda janela (asserção em `tests.rs:1265`). Por isso um piso atrelado ao VCP nunca dispara.
  - **M1r32**, 1 constante e 1 linha de produção, já formatadas pelo rustfmt:
    ```rust
    /// No more reads of the input than this per switch: the bus is shared.
    const MAX_SETTLE_READS: u32 = 32;
    // settle_input, worker.rs:315
    for _ in 0..MAX_SETTLE_READS {
    ```
    `fmt` 0, clippy `-D warnings` 0, lib `76 passed`, **Verify 1 `OK` 8/8, determinístico**: relógio virtual em (b) e (e), e no (f) 20 leituras, abaixo do teto de 32 e do limite de 21. A desculpa é natural (barramento compartilhado), como a dos 50 ms do DDC/CI na W-22.
  - Mesma família, todas com Verify 1 `OK`:
    - M1r20 (3/3);
    - M1g20 `step.max(window / 20)` (8/8) e M1g32 (3/3). São a M1g16 do critic anterior com k ≥ 20;
    - M1cw32 `window.min(step * 32)`, no worker E no cliente (8/8).
  - **No orçamento do cliente:** M1bv `budgets.vcp + window.max(budgets.vcp)` ("a janela cabe ao menos uma transação") dá Verify 1 `OK` 8/8.
  - **Efeito em produção:** nenhum. O `Default` é 250 ms / 3 s, ou seja, 12 leituras, e 3 s ≥ o VCP de 1 s. O que se perde é a garantia, afirmada pela linha e pela D-17(a), de que só a política manda.
  - **Correção testada na cópia:**
    - uma linha de razão alta na tabela, `(1 µs, 100 ms)`, com 100 000 leituras em relógio virtual: (b) em 0,03 s e (e) em 0,04 s;
    - no (e), um 2º cliente por política com `budgets.vcp` maior que a janela (`DdcHiBudgets { vcp: settle.window * 3, ..budgets }`), exigindo `write_budget_of(0x60) == vcp + janela`.

    Resultado: HEAD verde. M1r20, M1r32, M1g20, M1g32, M1cw32, M1gs1 e M1bv ficam VERMELHAS, e as 10 antigas seguem vermelhas.
  - **Para o loop convergir, emendar também o TEXTO de (b)/(e).** Ele deve afirmar só o que uma tabela finita prova: pisos e tetos cujo valor, ou cuja razão janela/passo ou janela/VCP, cai dentro das faixas da tabela, com as faixas escritas. Um piso abaixo do menor valor testado, ou um teto acima do maior, é indetectável por qualquer tabela. Exemplos: M1gw100, piso de 100 ms igual à menor janela, que a asserção `vcp < janela` do (e) prende em ≥ 70 ms; e um piso de passo de 1 ns.
  - Como `worker/tests.rs` está congelado, isso pede emenda do DoD (novo SHA-256) e nova D-XX.

- **W-24 (linha 8, não objetiva: evasão deliberada; D-2026-10-01-5(c)).**
  - A regex de atributos olha linha por linha. Um `#[path = …]` quebrado em 3 linhas (`#[` / `path = "…"` / `]`) sob `#[rustfmt::skip]` escapa dela, e o rustfmt preserva a forma.
  - Na prova composta (M8p3 + M1g + `settle_checks.rs` sem a linha 5 ms/100 ms), o módulo de teste compilado é o arquivo não congelado, e os Verify 1 e 8 dão `OK`.
  - Ninguém escreve isso sem querer, então fica como nota.
  - Endurecimento sugerido, robusto a qualquer forma de atributo: provar pela compilação. Ler o dep-info do binário de teste (`target/debug/deps/ddc_adapters-*.d`, após `cargo test -p ddc-adapters --lib --no-run`) e exigir que o conjunto de `.rs` compilados sob `ddc_hi_backend/` seja exatamente o esperado.

Notas, sem ação no código:
- **M1t (janela contada de antes do write).** Fica verde e não é afirmada pelo DoD. Se o PR quiser travar a origem da janela, um teste com write lento em relógio virtual a pega.
- **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. Um teste futuro que alcance `DdcHiDisplays` falaria com o RTK real nesta máquina. Manter, nas próximas rodadas (reviewer, critic, `/jdi-ship`), a leitura do 5.7 e a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **`.jdi/DECISIONS.md`.** É uma visão gerada e ignorada (`.gitignore:31`). As fontes estão em `.jdi/decisions/` (inclusive `D-2026-10-01-input-switch-autostart-5.md`), e `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter, com 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado e em menos de 0,5 s. (b) faz uma TABELA de 8 políticas extremas em relógio virtual (leituras `== ceil(janela/passo)`, sleeps exatos, `Σ == janela`). (e) chama `write_budget_of` para cada política da tabela. (f) usa o `WorkerClient` real em 3 partes, com no máximo `ceil(janela/passo) + 1` leituras na 1ª. (g) fixa `Default` e `without_backoff()`. (h) cobre a fiação. O CONJUNTO das linhas `INPUT_SETTLE_*` = 2 `const` privadas + 2 usos no `Default`, em `retry.rs`. 3 arquivos de teste congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,41 s. O conjunto listado são os 7 nomes, cada um com `1 passed; 0 failed` em 0,00 s, exceto (f), com 0,32 s (30/30 em repouso; 0,32–0,33 s sob 24 laços). (f) leu 20 de no máximo 21 em 30/30. (h) listado e passa. `git grep` das constantes = `retry.rs:27/32/112/113` (mais o `///` da 22, excluído). SHA-256 `b1509c2a…`, `ec2d2980…` e `12ac10db…` batem. As 10 mutações da família ficam VERMELHAS, 0/3 cada. **Resíduo (W-23): M1r32, M1g20, M1cw32 e M1bv dão `OK` 8/8.** |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,11 s. Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base sai 0; sem `sleep`. Sonda de 1 byte herdada (Verify e `ddc-core` sem diff). |
| 3 | View-model puro + 4 testes nomeados, com o texto inteiro conferido por igualdade com frases LITERAIS (papéis de mantida e pedida) em `en`/`pt-BR`, para o aviso de input e o genérico; `tests/ui` inteira verde; `view-model.test.mjs` congelado por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 0,25 s. `npm run test:unit`: `# tests 165 / # pass 165 / # fail 0`, 88.48%. SHA-256 `7c650133…` bate. M4s, M4g, M4ge, M4u e M3f herdadas (arquivos, templates e Verify sem diff desde `c17b872`). |
| 4 | Aviso visível de ponta a ponta. `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; texto inteiro de cada toast contra a frase literal com papéis; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 18,3 s, com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed; 10 `✓` em `input-notice.spec.mjs`. SHA-256 `92f99e00…`/`ee930894…` batem. M4s, M4g, M4u e M4f herdadas (arquivos sem diff). |
| 5 | Menu nativo: rótulo en/pt-BR; item nas 2 plataformas com marca = estado do SO; `from_id`; fiação LINUX (ksni) e testes de `tray`. Fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 1,93 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha com `0 ignored`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); o chamador de PRODUÇÃO imprime a linha do `crate::report`; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,43 s. Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` imprime 2 vezes a linha `ddc-tray: could not change the start-with-system entry: read-only home`. M6 herdada. |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,52 s. `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` impresso; listagem real idêntica (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: tokens de hardware só nos 3 arquivos de produção; nesses 3, só `#[cfg(test)]` + `mod tests;`, sem atributo com `test` ou `path`; `doctest = false` efetivo e sem bloco `Doc-tests ddc_adapters`; todo teste listado sob `ddc_hi_backend::` num módulo `tests` ancorado; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 0,95 s. Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `cargo metadata`: `["lib"] ddc_adapters doctest=false`, sem `Doc-tests ddc_adapters`. A `--list` sob `ddc_hi_backend::` tem 51 testes, todos em `tests` (4), `worker::tests` (32), `hardware::tests` (10) ou `identity::tests` (5). `capabilities/` só tem `default.json`, sem diff e sem `autostart`. `tauri-plugin-autostart v2.6.0` -> `ddc-tray`. M8p, M8p2 e M8p4 VERMELHAS (`#[path]` fechado na forma de 1 linha); M8m, M8x e M8i VERMELHAS; M8ctl `OK`. **Resíduo não objetivo (W-24): M8p3, `#[` multilinha sob `#[rustfmt::skip]`.** O "nenhum teste fala com monitor real" está no 5.7: leitura + inotify, zero aberturas. |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B; monitor simulado exigido nas duas; marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **3/3 execuções literais** (5,63 / 5,61 / 5,64 s). Execução direta no `bwrap`, com zero eventos de `/dev/i2c-*`. Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor` e `smoke-sni: OK — PID 2525085 registered its tray item, … and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor`, checkmark desmarcado; o 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray`, e o 2º remove; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. PID 24429 intacto. W-16 não apareceu. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase (caminhos fixos + `crates/ddc-adapters/Cargo.toml` UNIDOS aos não-Rust do diff); `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,02 s. M10c, M10n e M10g (VERMELHAS) e M10ctl `TODO(#12)` (`OK`) herdadas: o Verify e os arquivos não-Rust da phase não têm diff desde `c17b872`. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,06 s. Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,01 s. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` (linha 10, "Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18). O último release é `## [0.1.0] - 2026-09-28` (linha 31). Nenhum heading de release novo nesta phase; sem diff desde `c17b872`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330, 333 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento; os smokes em `scripts/` são citados (linha 387); a limitação do `Exec=` com espaço está em "Known limitations of the tray app" (linhas 392/399). Sem diff desde `c17b872`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers pelos gates. Os 13 Verify automáticos dão `OK` no HEAD, rodados literalmente. Os achados do critic anterior estão fechados por mutação:
- os oito sobreviventes da linha 1 (M1g, M1g16, M1gw, M1c, M1s, M1s4, M1d, M1bf150) ficam vermelhos em 3/3, e M1m e M1f seguem vermelhas;
- o `#[path]` de 1 linha fica vermelho no Verify 8;
- os controles (M8ctl) seguem sem falso positivo.

A prova por inotify em `bwrap` deu zero aberturas de `/dev/i2c-*`. O (f) leu 20 de no máximo 21 em 30/30.

**Antes de seguir para o critic, fechar a W-23** (linha 1, objetiva provável). Um limite de leituras de 32 por troca (`MAX_SETTLE_READS`, natural pelo barramento compartilhado), um piso relativo `step.max(window / 20)` ou um teto `window.min(step * 32)` no worker e no cliente passam no `fmt`, no clippy e em todos os testes. Um piso do orçamento atrelado ao VCP (`window.max(budgets.vcp)`) também passa. Todos dão Verify 1 `OK` 8/8, porque a razão janela/passo da tabela para em 20 e o VCP do (e) é fixo abaixo de toda janela. Se o loop seguir para o critic sem essa correção, o reviewer espera BLOCKED pela linha 1, na mesma família.

Correção mínima, já testada na cópia:
- uma linha `(1 µs, 100 ms)` na tabela, com 100 000 leituras, (b) em 0,03 s e (e) em 0,04 s;
- no (e), um 2º cliente por política com `budgets.vcp` maior que a janela, exigindo `write_budget_of(0x60) == vcp + janela`.

Resultado: HEAD verde. M1r20, M1r32, M1g20, M1g32, M1cw32, M1gs1 e M1bv vermelhas, e as antigas seguem vermelhas.
- Para o loop convergir, emendar também o texto de (b)/(e): afirmar pisos e tetos dentro das faixas da tabela (valores, razão janela/passo e razão janela/VCP, escritas), e não "qualquer". Um piso abaixo do menor valor testado não é detectável por nenhuma tabela finita.
- Pede novo SHA-256 de `worker/tests.rs` e nova D-XX.

Outros pontos:
- **W-24** (não objetiva): considerar trocar a regex de atributos da linha 8 por uma prova pela compilação. O dep-info do binário de teste lista os `.rs` compilados sob `ddc_hi_backend/`.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL em `/dev/i2c-*`, manter a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2). Um `22 reads` ou um (f) lento lá aponta para a W-11;
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura; rodada 2, iteração 3).
- Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados por inotify; houve só o evento do controle positivo.
- Linhas 2–7 e 9–13: `hollow=false`.
- Linha 1 oca (objective). Todas as mutações abaixo passam em fmt 0 e clippy 0, deixam a lib em `76 passed` e o Verify 1 imprime `OK`:
  - **W-23 confirmada:** M1r32 (`MAX_SETTLE_READS` = 32), M1g20 (`step.max(window/20)`), M1cw32 (`window.min(step*32)`, no worker e no cliente) e M1bv (`window.max(budgets.vcp)` no orçamento). Causa: a razão janela/passo da tabela vai de 1 a 20, e o VCP do (e) é fixo abaixo de toda janela.
  - **N1:** `settle.window.max(settle.step)`, no worker e no cliente, porque a menor razão da tabela é 1.
  - **N2:** piso no passo só no caminho de produção. N2clk16 é o `Clock::resolution()` de 16 ms em `SystemClock`; N2spawn50 é um piso de 50 ms no `spawn`. O (b) só roda `Worker::new` com `VirtualClock`, e o (f) aceita de 2 a 21 leituras.
  - **N3:** `RESTORE_FACTORY_DEFAULTS` orçado como o input. O (e) só confere 0x10, 0xD6 e 0xE1.
  - **N4:** pós-processamento do orçamento dentro de `WorkerClient::write_vcp`, com `.max(MIN_WRITE_BUDGET)` ou `+ QUEUE_SLACK`. O (e) testa `write_budget_of`, e as margens do (f) absorvem a diferença.
  - **Abaixo do menor valor testado:** M1gs1 (piso de 1 ms no passo) e M1gw100 (piso de 100 ms na janela). Nenhuma tabela finita fecha isso; só o texto.
- Linha 8 oca, NÃO objetiva. W-24 confirmada: um `#[path]` multilinha sob `#[rustfmt::skip]`, posto antes do `#[cfg(test)]`, troca o módulo de teste compilado e dá Verify 1 e Verify 8 `OK`. É evasão deliberada.

Fechamento validado na cópia:
- **Tabela de (b):** ganha `(1 µs, 100 ms)` (razão 10⁵) e `(1 dia, 1 µs)` (passo maior que a janela).
- **(e):** dois clientes por política. O curto tem `vcp = min(70 ms, janela/2)`; o folgado tem `vcp = 1000·janela + 1 s`, com `capabilities` e `enumerate` maiores. Os dois exigem `vcp + janela` para 0x60 e `vcp` para os 255 outros códigos.
- **(f):** ganha o limite de baixo `reads * 2 >= ceil(janela/passo)`.
- **Verify 1:**
  - fixa o corpo de `WorkerClient::write_vcp`;
  - confere, pelo dep-info do binário de teste, o conjunto exato de `.rs` compilados sob `ddc_hi_backend/`, o que também fecha a W-24 na linha 8;
  - passa a declarar as faixas cobertas.
- **Resultado:** com isso, o HEAD fica verde e todas as mutações acima ficam vermelhas.

**Verdict:** BLOCKED
