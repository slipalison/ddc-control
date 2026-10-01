# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Rodada 2, iteração 2 do loop (7ª iteração absoluta, depois do auto-reset 1/3).
> - Branch `phase/input-switch-autostart`, HEAD `a2d0d3c`, base `origin/main` = merge-base `134b665`.
> - O conteúdo foi produzido pelo reviewer e gravado pelo orquestrador, porque o harness nega escrita de `.md` ao subagente.
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT. As 2 linhas Manual não têm comando.
> - Prefixos sha256 dos comandos: ctx1 `d541d1854033`, ctx2 `81b3ba163edc`, ctx3 `8f648d4f4fe0`, ctx4 `05ff88d61fb8`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `edc707e58a76`, ctx9 `ae65d59b93ea`, ctx10 `24da480a271e`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - As linhas 2, 3, 4, 5, 6, 7 e 9 batem byte a byte com a extração de `846dbc1`.
> - As linhas 1, 8 e 10 diferem, como esperado. São as emendadas em `1a8c59d`, com o SHA-256 de `worker/tests.rs` recongelado em `a2d0d3c` (D-2026-10-01-input-switch-autostart-4).
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`, tirados do ambiente.
>
> Os 10 do CONTEXT e os Verify de `cargo test` e de TODO do PROJECT rodaram LITERALMENTE:
> - a linha 4 rodou com `npm ci` e o Playwright inteiro;
> - a linha 9 rodou 3 vezes;
> - a cobertura reusa o Gate 3.
>
> **Herança, só por diff.** `git diff 99cb379 HEAD -- . ':!.jdi'` lista só `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs` (+19 −7, `75fca0b`). Para esse arquivo nada foi herdado: o tempo do (f), a contagem de leituras e as mutações da linha 1 foram refeitos. Herdados, porque os arquivos e os Verify não têm diff desde `99cb379`:
> - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
> - as mutações A, R, B1 e B2 da linha 9;
> - a M6 da linha 6;
> - a sonda de 1 byte da linha 2;
> - M4s, M4g, M4ge, M4u, M3f e M4f das linhas 3 e 4.
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
>
> **Estado da máquina.**
> - O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim.
> - A listagem do `~/.config/autostart` real tem os mesmos sha256 antes e depois: `ls -A` `bf0a7045a8fb…` e `ls -la` `f69da4fd3ea5…`.
> - No fim não sobrou nenhum `dbus-run-session`, watcher, servidor HTTP do Playwright, tempdir `/tmp/smoke-*`, laço de CPU ou observador inotify.
> - `/var/tmp/rv7` foi apagado: cópia, `target/`, arquivos que cobriam os `/dev/i2c-*` e logs.
>
> **Monitor real.**
> - Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados.
> - As mutações mexeram só no cálculo da janela e do passo de `settle_input`, sempre sobre `FakeDisplays`, plantaram corpos vazios (`fn planted() {}`, `assert_eq!(0x60, 0x60)`), comentários `TODO` ou um `eprintln!` de contagem. Uma sonda de teste em relógio virtual usou `FakeDisplays`.
> - Nenhuma mutação criou teste que fale com o backend real.
> - Todo `cargo` das mutações rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados (ver 5.7).
>
> **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. A prova por inotify em `bwrap` rodou ANTES de qualquer execução nativa da suíte.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0, e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` também. Única nota: o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados), igual no `bwrap` e no nativo. Os 9 são os 7 de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs`, todos `needs the dev monitor attached; run with DDC_HW_TESTS=1`. Igual à iteração anterior; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed`. Blocos `Doc-tests`: só `ddc_cli`, `ddc_core` e `ddc_tray`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. Rodou dentro do `bwrap`, com o observador ativo. Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0 no repositório e também A FRIO, na cópia descartável com `target/` próprio (365 crates, 33 s, em `bwrap`). O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a leitura do teste alterado e a prova por inotify em `bwrap`: zero aberturas de `/dev/i2c-*` na suíte, na cobertura, nas 80 execuções do (f) e nas mutações, com controle positivo. |
| Consistency | PASS (com warnings) | D-2026-10-01-4 (a), (b) e (c) estão conformes no código e nos Verify. 3 commits desde `99cb379`, com tipo, escopo e tamanho corretos. A W-22 é falta de força de teste, não violação de D-XX no código. Ela mostra que a premissa da D-2026-10-01-4(a) ("qualquer piso acima da janela injetada excede o limite") só vale com o passo da política. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped, 0 failed. Os 6 skipped são só `screenshots.spec.mjs:24/34/47` em `light`/`dark` (`test.skip` condicionado a `SCREENSHOTS=1`). `input-notice.spec.mjs` tem 10 `✓`. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduo novo: **W-22** (linha 1, objetiva provável: um piso no PASSO passa sempre; um piso na janela junto com um piso no passo passa em 8 de 10). |

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
  - **Leitura do único teste alterado, `worker/tests.rs` (`75fca0b`).** Na 1ª parte de (f) (`tests.rs:1245`) entram só três coisas:
    - `let most = …` (1273);
    - `let reads = calls.len() - 1;` (1280);
    - `assert!(reads <= most, …)` (1281-1287).

    O monitor segue o mesmo: `FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)])` (1249) e os dois displays presos em `Behaviour::Block(gate)` (1254, 1257), via `spawn(&source, budgets(vcp))`, que dá um `WorkerClient<FakeDisplays>` (358). O arquivo cita `DdcHi` só em `DdcHiBudgets`, e `/dev/i2c-*` só num comentário (950). Não há `DdcHiDisplays` nem `DdcHiMonitorBackend`.
  - **Demais testes Rust e JS.** Sem diff desde `99cb379`, e cobertos pela prova abaixo.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`. Ali dentro, `stat` mostra os 16 como "arquivo comum vazio". Um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) ficou ligado nesses arquivos das 10:46:28 às 10:58:56.
    - Controle positivo: `cat /dev/i2c-3` no sandbox gerou 1 evento (`mask=0x20`).
    - `cargo test --workspace --locked` no sandbox: `SUITE_EXIT=0`, 413 passed, 0 failed, 9 ignored, ZERO eventos.
    - `cargo llvm-cov`, as 80 execuções do (f) no HEAD e todas as mutações e sondas das linhas 1 e 8, tudo no sandbox: ZERO eventos.
    - No log inteiro há 1 evento, o do controle.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` 0.22.2 (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, warning permitido que já estava no lock da base. `Cargo.lock` sem diff desde `99cb379`.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração anterior** (`99cb379..HEAD`), 3 ao todo:
  - `1a8c59d` docs (69 caracteres): a decisão `D-2026-10-01-input-switch-autostart-4.md` e o CONTEXT;
  - `75fca0b` test (69 caracteres): só `worker/tests.rs`;
  - `a2d0d3c` docs (66 caracteres): CONTEXT (SHA-256 recongelado), PLAN e SUMMARY.

  Escopo = slug, `.jdi/` nunca no mesmo commit que código (D-12), e os 3 trazem `Claude-Session:`.
- **Zero byte de diff desde `99cb379`** fora de `worker/tests.rs`, incluindo `Cargo.lock`, `README.md` e `CHANGELOG.md`. Código de produção não mudou nesta iteração. Contra a base seguem com 0 bytes:
  - `crates/ddc-core`;
  - `scripts/smoke-sni.sh`;
  - `real_monitor.rs`;
  - `capabilities/`.
- **D-2026-10-01-4.** Conforme:
  - (a) A 1ª parte de (f) tem o limite `reads <= ceil(janela / passo)` (`tests.rs:1273/1280-1287`), e `worker/tests.rs` bate com `124ba97c…`.
  - (b) O Verify 10 varre a união dos caminhos fixos (mais `crates/ddc-adapters/Cargo.toml`) com os não-Rust do diff da phase.
  - (c) O Verify 8 ancora pela `--list` todo teste sob `ddc_hi_backend::` nos módulos de teste. Os testes de `ddc_hi_backend::` hoje: 32 em `worker::tests`, 10 em `hardware::tests`, 5 em `identity::tests` e 4 em `tests`.
  - **Mas a premissa de (a), "qualquer piso acima da janela injetada (100 ms) excede o limite", só vale com o passo da política. Ver W-22.**
- **D-17(a) ("cliente e worker devem ler a janela e o passo SÓ da `RetryPolicies`").** Conforme no código: `settle_input` lê só `self.policies.input_settle` (`worker.rs:313`), e o `sleep` usa `settle.step` (`worker.rs:320`). O que falta é TESTE: nenhum teste determinístico usa um passo menor que um piso plausível (W-22).
- **D-3 / D-14 / D-19 / D-2026-10-01-1/-2/-3** Sem diff de código. `write_budget_of` (`worker.rs:464`) segue sendo o único caminho, e `write_vcp` a chama (`worker.rs:539`). O conjunto das linhas `INPUT_SETTLE_*` são as 4 esperadas em `retry.rs:27/32/112/113`, mais o comentário `///` da 22.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo.
- **PLAN x commits.**
  - Os 33 arquivos do diff da phase seguem em `## Files modified (all tasks)`.
  - A entrada "Rodada 2, iteração 2 (D-2026-10-01-input-switch-autostart-4)" foi acrescentada (`PLAN.md:137`).
  - Resíduo cosmético que segue: `PLAN.md:151` ainda diz "emendados por D-13, D-14, D-15, D-16 e D-17".
- **SUMMARY.** A seção "Rodada 2, iteração 2" bate com o reproduzido aqui:
  - M1f e M1m 10/10 FAILED, com 31–32 leituras ou `Err(Timeout)`;
  - 413/0/9 e 85,25%.

  O SUMMARY diz "18 a 20 leituras, com 20 em 7 de 10"; aqui deu 20 em 40 de 40. Os dois confirmam a margem zero.

### Achados do critic da iteração anterior, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| Linha 1 (W-20): M1f, piso no `Default`, e M1m, `MIN_SETTLE_WINDOW` de 1 s, davam Verify 1 `OK` em 15–16 de 20 | FECHADO na forma original | M1f: lib FAILED 3/3, (f) FAILED 10/10 (9× `32 reads of the input, more than the 20` em `tests.rs:1281`; 1× `Err(Timeout)` em `tests.rs:1275`), Verify 1 sem `OK` 3/3. M1m: lib FAILED 3/3, (f) FAILED 10/10 (9× 32 e 1× 31 leituras), Verify 1 sem `OK` 3/3. **Resíduo novo: W-22** |
| Linha 8 (W-21): `#[test]` gerado por macro (`planted!(test)`), e macro de tabela definido em `lib.rs` e chamado sem `cfg` em `worker.rs` | FECHADO | M8m: `ddc_hi_backend::planted ... ok` roda (`77 passed`), `fmt` 0 e clippy 0, mas Verify 8 sem `OK`. M8x: `ddc_hi_backend::worker::the_input_source_is_vcp_0x60 ... ok`, mesmo resultado, Verify 8 sem `OK`. M8i (nova): `mod extra { #[cfg(test)] mod tests { … } }` em `identity.rs`, fora dos 3 arquivos, Verify 8 sem `OK`. Controle M8ctl: um `#[test]` legítimo em `identity/tests.rs` dá Verify 8 `OK` (sem falso positivo) |
| Linha 10: `crates/ddc-adapters/Cargo.toml` fora do pathspec | FECHADO | M10c (a mutação do critic, commitada): Verify 10 exit 1; o Verify 13, só `*.rs`, segue `OK`, como deveria. M10n: arquivo novo `crates/ddc-adapters/NOTES.txt` com `fixme:`, Verify 10 exit 1, porque entra pela lista do diff. M10g: `// TODO later` em `.github/workflows/ci.yml`, Verify 10 exit 1. Controle M10ctl (`# TODO(#12): …`): `OK` |

### Mutações reproduzidas pelo reviewer
**Montagem.** Todas rodaram num repositório git descartável, `/var/tmp/rv7/repo`, já apagado, com `CARGO_TARGET_DIR=/var/tmp/rv7/target` próprio:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1.

A cópia é fiel ao repositório real:
- árvores idênticas: base `0c8815c7…` e HEAD `b6dacadd…`;
- `diff --stat` idêntico: 33 arquivos, `2649 insertions(+), 87 deletions(-)`.

**Execução.**
- Mutações aplicadas por substituição exata, com 1 ocorrência conferida, e formatadas com `cargo fmt`.
- Reversão: `git checkout -- .` para as das linhas 1 e 8; `reset --hard` + `clean` para os commits da linha 10. `git status` ficou limpo depois de cada uma.
- Todo `cargo` rodou em `bwrap` com os 16 `/dev/i2c-*` cobertos e observados.
- Baselines na cópia, todas `OK`: linha 1 (5,13 s, a frio), linha 8 (3,76 s) e linha 10 (0,03 s).

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | M1f `settle_input`: `let window = settle.window.max(RetryPolicies::default().input_settle.window);` (piso no `Default`, formatado pelo rustfmt) | `fmt` 0, clippy 0. VERMELHA: lib `75 passed; 1 failed` 3/3; (f) 10/10 FAILED (9× 32 leituras, 1× `Err(Timeout)`); Verify 1 sem `OK` 3/3 |
| 1 | M1m `const MIN_SETTLE_WINDOW: Duration = Duration::from_secs(1);` + `settle.window.max(MIN_SETTLE_WINDOW)` em `worker.rs` | `fmt` 0, clippy 0. VERMELHA: lib FAILED 3/3; (f) 10/10 FAILED (31–32 leituras); Verify 1 sem `OK` 3/3 |
| 1 | **M1g (sonda)**: `const MIN_READ_GAP: Duration = Duration::from_millis(50);` ("DDC/CI pede 50 ms entre transações") + `self.clock.sleep(left.min(settle.step.max(MIN_READ_GAP)));` em `worker.rs:320` | **VERDE: `fmt` 0, clippy 0, lib `76 passed` 3/3, (f) ok 10/10, Verify 1 `OK` 5/5.** Ver W-22 |
| 1 | **M1gw (sonda)**: M1m + M1g, ou seja, piso de 1 s na janela e de 50 ms no passo | **Verify 1 `OK` em 8 de 10.** (f) ok em 17 de 20; as 3 falhas são `Err(Timeout)` em `tests.rs:1275`, pela corrida no `deadline`. Lib FAILED 2/3. Com o passo de 50 ms, são 3–4 leituras, bem abaixo do limite de 20. Ver W-22 |
| 1 | Correção proposta para a W-22, só na cópia: teste em relógio virtual, espelho de (b), sob `no_backoff()` (5 ms / 100 ms): `slept == janela`, todo sleep `== passo` e leituras `== janela/passo` | HEAD ok 3/3. M1f FAILED 3/3 (220 leituras contra 20), M1m FAILED 3/3 (200 contra 20), M1g FAILED 3/3 (2 contra 20), M1gw FAILED 3/3 (tempo virtual 1 s contra 100 ms) |
| 8 | M8m `macro_rules! planted { ($m:meta) => { #[$m] fn planted() {} }; }` + `planted!(test);` em `ddc_hi_backend.rs` | `fmt` 0, clippy 0, o teste roda (`77 passed`). VERMELHA: Verify 8 exit 1 (na iteração anterior dava `OK`) |
| 8 | M8x (critic) `vcp_table!` com `#[test]` literal definido em `lib.rs` e chamado sem `cfg` em `worker.rs` | `fmt` 0, clippy 0, `…worker::the_input_source_is_vcp_0x60 ... ok`. VERMELHA: Verify 8 exit 1 |
| 8 | M8i `mod extra { #[cfg(test)] mod tests { #[test] fn planted() {} } }` em `identity.rs` | `fmt` 0, clippy 0, `…identity::extra::tests::planted ... ok`. VERMELHA: Verify 8 exit 1 |
| 8 | M8ctl (controle) `#[test] fn planted() {}` no fim de `identity/tests.rs` | Verify 8 `OK`, como deveria |
| 10 | M10c (critic) `# TODO: turn doctests back on once the adapter has examples` acima de `doctest = false` (commitada) | VERMELHA: Verify 10 exit 1 |
| 10 | M10n arquivo novo `crates/ddc-adapters/NOTES.txt` com `fixme: the step floor` (commitado) | VERMELHA: Verify 10 exit 1 |
| 10 | M10g `// TODO later` no fim de `.github/workflows/ci.yml` (commitada) | VERMELHA: Verify 10 exit 1 |
| 10 | M10ctl `# TODO(#12): …` no `Cargo.toml` | `OK`, como deveria |

**Limite de leituras do (f) no HEAD (margem zero).**
- 40 execuções isoladas (`cargo test -p ddc-adapters --locked --lib -- --exact …input_write_through_the_client_outlives_the_vcp_budget`), em `bwrap`, na cópia com a árvore do HEAD: 40/40 `ok`, todas com `finished in 0.32s`.
- Mais 40 execuções numa cópia instrumentada, que só ganhou um `eprintln!` da contagem antes do `assert!`: 40/40 `ok`, e as 40 com `rv-reads=20 most=20`.
- **Nenhuma rodada passou do limite, mas todas ficaram EXATAMENTE nele.**
- Por que isso é determinístico no Linux:
  - não há leitura antes do primeiro `sleep`;
  - a leitura j só ocorre se `(j−1)·passo < janela`, porque o `nanosleep` e o `Instant` medem pelo mesmo `CLOCK_MONOTONIC`;
  - logo j ≤ 20.
- No Windows, a premissa depende de o timer não voltar antes do pedido, medido pelo `Instant`. O doer já registrou essa ressalva (ver W-11).

**Tempo de (f).** O arquivo mudou, então o tempo foi medido de novo:
- em repouso: 0,32 s em 40/40;
- com 24 laços de CPU (cada um sob `timeout 75`, conferidos encerrados): 0,32–0,33 s em 10/10, todas passando.

A folga até o limite de 0,5 s do Verify é de 0,17 s.

**Linha 9:** 3/3 execuções literais `OK` (5,62 / 5,61 / 5,58 s), mais 1 execução direta dos dois scripts com saída 0. A W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração anterior:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff desde a iteração 1. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. Código do assentamento inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O README documenta.
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final: `Exec=…/autostart_entry-c0e56ec32fbab8ff $` no `cat -A`.
  - `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN): FECHADO, com resíduo cosmético.** `PLAN.md:151` ainda diz "emendados por D-13 … D-17".
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou, agora com dois limites.**
  - Tempo: 0,32 s em repouso (40/40) e 0,32–0,33 s sob carga (10/10). A folga até os 0,5 s do Verify é de 0,17 s.
  - Leituras: margem ZERO (20 de 20 em 40/40). No Linux o limite é determinístico. No `windows-latest`, a ressalva do doer é plausível: o timer de alta resolução da std trunca o pedido em intervalos de 100 ns, e o `Instant` usa outro relógio. Um `21 reads` lá apontaria para essa premissa, não para o worker.
  - O `rust-windows` mostra primeiro.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 3/3 literais + 1 direta OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-20 (linha 1): FECHADO na forma original.** M1f e M1m ficam VERMELHAS em todas as rodadas, pelo limite de leituras ou pelo `Timeout`. A forma composta, com piso no passo, reabre a lacuna como W-22.
- **W-21 (linha 8): FECHADO.** M8m, M8x e M8i ficam VERMELHAS pela âncora da `--list`, e o controle M8ctl segue `OK`.
- **Achado do critic na linha 10: FECHADO.** M10c, M10n e M10g ficam VERMELHAS, e o controle `TODO(#12)` segue `OK`.
- **W-9, W-10: TRATADOS** (iteração 3). **W-12, W-13: FECHADOS** (iteração 4). **W-15, W-17, W-18, W-19: FECHADOS** (iterações 5 e anterior).

Novo nesta iteração:
- **W-22 (linha 1, lacuna objetiva provável; D-17(a), D-2026-10-01-4(a), D-14).**

  **O que a linha afirma:**
  - "o passo e a janela do assentamento vivem na `RetryPolicies`";
  - "só a política manda";
  - no (f), com limite determinístico, "um worker que impusesse um piso à janela da política, com qualquer nome, falha aqui".

  **Os testes não seguram o passo por baixo:**
  - (a) e (c) usam o passo do `Default` (250 ms);
  - (b) usa 500 ms;
  - só o (f) usa 5 ms, e ele exige apenas `calls.len() > 2` e `reads <= 20`.

  Um piso no passo de até ~45 ms passa por tudo.
  - **M1g**, 1 constante e 1 linha de produção, já formatadas pelo rustfmt:
    ```rust
    /// DDC/CI wants at least 50 ms between two transactions on the bus.
    const MIN_READ_GAP: Duration = Duration::from_millis(50);
    // worker.rs:320
    self.clock.sleep(left.min(settle.step.max(MIN_READ_GAP)));
    ```
    `fmt` 0, clippy `-D warnings` 0, lib `76 passed` 3/3, (f) 10/10, **Verify 1 `OK` 5/5, determinístico.** É a versão para o passo da M1m que o critic achou para a janela. A desculpa é natural: a regra de 50 ms do DDC/CI.
  - **M1gw**, M1m + M1g (piso de 1 s na janela e de 50 ms no passo): **Verify 1 `OK` em 8 de 10.**
    - Com passo de 50 ms o worker lê 3–4 vezes até o `deadline` do cliente, abaixo do limite de 20.
    - Só a corrida do `Err(Timeout)` (`tests.rs:1275`) o pega às vezes.
    - Isso desmente a frase nova do (f) e a premissa da D-2026-10-01-4(a), "qualquer piso acima da janela injetada (100 ms) excede o limite".
  - **Efeito em produção:** nenhum, porque o `Default` é 250 ms / 3 s. O que se perde é a garantia, afirmada pela linha e pela D-17(a), de que só a política manda.
  - **Correção testada na cópia.** Um teste em RELÓGIO VIRTUAL, espelho de (b), sob uma política abaixo do `Default` e de qualquer piso plausível (`no_backoff()`, 5 ms / 100 ms). Ele exige:
    - `slept == janela`;
    - todo sleep `== passo`;
    - leituras `== ceil(janela/passo)`.

    Resultado: HEAD 3/3 verde; M1f, M1m, M1g e M1gw 3/3 VERMELHAS. É determinístico e não custa tempo real.
    - O conjunto exato de 7 nomes `input_write_*`/`writes_to_other_codes_*` do Verify 1 pede cuidado. O caminho mais simples é fazer (b) rodar as duas políticas custom (500 ms / 4 s e 5 ms / 100 ms). As alternativas são um nome fora desses prefixos ou emendar o conjunto.
    - Como `worker/tests.rs` está congelado, isso pede emenda do DoD (novo SHA-256), nova D-XX e a correção do texto do (f).

Notas, sem ação no código:
- **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. Um teste futuro que alcance `DdcHiDisplays` falaria com o RTK real nesta máquina. Manter, nas próximas rodadas (reviewer, critic, `/jdi-ship`), a leitura do 5.7 e a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **`.jdi/DECISIONS.md`.** É uma visão gerada e ignorada (`.gitignore:31`). As fontes estão em `.jdi/decisions/` (inclusive `D-2026-10-01-input-switch-autostart-4.md`), e `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter, com 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado e em menos de 0,5 s. (b)/(e) com política CUSTOM de janela MAIOR que a do `Default`; (e) via `write_budget_of`; (f) pelo `WorkerClient` real em 3 partes, com no máximo `ceil(janela/passo)` leituras na 1ª; (g) fixa `Default` e `without_backoff()`; (h) fiação. O CONJUNTO das linhas `INPUT_SETTLE_*` = 2 `const` privadas + 2 usos no `Default`, em `retry.rs`. 3 arquivos de teste congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,36 s. O conjunto listado são os 7 nomes, cada um com `1 passed; 0 failed` em 0,00 s, exceto (f), com 0,32 s (40/40 em repouso; 0,32–0,33 s sob 24 laços). (f) leu 20 de no máximo 20 em 40/40: nenhuma rodada passou do limite, margem zero. (h) listado e passa. `git grep` das constantes = `retry.rs:27/32/112/113` (mais o `///` da 22, excluído). SHA-256 `124ba97c…`, `ec2d2980…` e `12ac10db…` batem. M1f e M1m VERMELHAS em 10/10 no (f) e 3/3 no Verify. **Resíduo (W-22): M1g, piso de 50 ms no passo, dá `OK` 5/5; M1gw, piso na janela + piso no passo, dá `OK` em 8/10.** |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,10 s. Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base sai 0; sem `sleep`. Sonda de 1 byte herdada (Verify e `ddc-core` sem diff). |
| 3 | View-model puro + 4 testes nomeados, com o texto inteiro conferido por igualdade com frases LITERAIS (papéis de mantida e pedida) em `en`/`pt-BR`, para o aviso de input e o genérico; `tests/ui` inteira verde; `view-model.test.mjs` congelado por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 0,24 s. `npm run test:unit`: `# tests 165 / # pass 165 / # fail 0`, 88.48%. SHA-256 `7c650133…` bate. M4s, M4g, M4ge, M4u e M3f herdadas (arquivos, templates e Verify sem diff desde `99cb379`). |
| 4 | Aviso visível de ponta a ponta. `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; texto inteiro de cada toast contra a frase literal com papéis; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 20,1 s, com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed; 10 `✓` em `input-notice.spec.mjs`. SHA-256 `92f99e00…`/`ee930894…` batem. M4s, M4g, M4u e M4f herdadas (arquivos sem diff). |
| 5 | Menu nativo: rótulo en/pt-BR; item nas 2 plataformas com marca = estado do SO; `from_id`; fiação LINUX (ksni) e testes de `tray`. Fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 1,86 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha com `0 ignored`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); o chamador de PRODUÇÃO imprime a linha do `crate::report`; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,39 s. Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` imprime 2 vezes a linha `ddc-tray: could not change the start-with-system entry: read-only home`. M6 herdada. |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,50 s. `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` impresso; listagem real idêntica (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: tokens de hardware só nos 3 arquivos de produção; nesses 3, só `#[cfg(test)]` + `mod tests;`; `doctest = false` efetivo e sem bloco `Doc-tests ddc_adapters`; todo teste listado sob `ddc_hi_backend::` num módulo `tests` ancorado; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 0,94 s. Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `cargo metadata`: `["lib"] ddc_adapters doctest=false`, sem `Doc-tests ddc_adapters`. A `--list` sob `ddc_hi_backend::` tem 51 testes, todos em `tests`, `worker::tests`, `hardware::tests` ou `identity::tests`. `capabilities/` só tem `default.json`, sem diff e sem `autostart`. `tauri-plugin-autostart v2.6.0` -> `ddc-tray`. M8m, M8x e M8i VERMELHAS, M8ctl `OK` (W-21 fechada). O "nenhum teste fala com monitor real" está no 5.7: leitura + inotify, zero aberturas. |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B; monitor simulado exigido nas duas; marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **3/3 execuções literais** (5,62 / 5,61 / 5,58 s). Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor` e `smoke-sni: OK — PID 2021740 registered its tray item, … and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor`, checkmark desmarcado; o 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray`, e o 2º remove; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. PID 24429 intacto. W-16 não apareceu. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase (caminhos fixos + `crates/ddc-adapters/Cargo.toml` UNIDOS aos não-Rust do diff); `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,06 s. M10c (a do critic), M10n (arquivo novo) e M10g (`.github/workflows/ci.yml`) VERMELHAS; M10ctl `TODO(#12)` `OK`. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,05 s. Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,01 s. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` (linha 10, "Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18). O último release é `## [0.1.0] - 2026-09-28` (linha 31). Nenhum heading de release novo nesta phase; sem diff desde `99cb379`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330, 333 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento (250 ms, até 3 s); os smokes em `scripts/` são citados (linha 387); a limitação do `Exec=` com espaço está em "Known limitations of the tray app" (linhas 392/399). Sem diff desde `99cb379`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers pelos gates. Os 13 Verify automáticos dão `OK` no HEAD, rodados literalmente. Os três achados do critic anterior estão fechados por mutação, com controles sem falso positivo:
- linha 1: M1f e M1m vermelhas em todas as rodadas;
- linha 8: M8m, M8x e M8i;
- linha 10: M10c, M10n e M10g.

A prova por inotify em `bwrap` deu zero aberturas de `/dev/i2c-*`. O (f) no HEAD nunca passou do limite (40/40 com 20 de 20 leituras), mas a margem é zero.

**Antes de seguir para o critic, fechar a W-22** (linha 1, objetiva provável). Um piso de 50 ms no PASSO, natural pela regra de 50 ms do DDC/CI, passa no `fmt`, no clippy e em todos os testes, e dá Verify 1 `OK` sempre. Junto com um piso na janela, dá `OK` em 8 de 10, o que desmente a frase nova do (f) e a premissa da D-2026-10-01-4(a). Se o loop seguir para o critic sem essa correção, o reviewer espera BLOCKED pela linha 1.

Correção mínima, já testada na cópia: um teste em relógio virtual, espelho de (b), sob a política `no_backoff()` (5 ms / 100 ms), que exige `slept == janela`, todo sleep `== passo` e leituras `== ceil(janela/passo)`. HEAD verde, M1f, M1m, M1g e M1gw vermelhas em 3/3.
- O jeito mais simples de respeitar o conjunto exato de 7 nomes é fazer (b) rodar as duas políticas custom.
- Pede novo SHA-256 de `worker/tests.rs`, nova D-XX e ajuste do texto do (f): o limite de leituras só pega piso na janela com o passo da política.

Outros pontos:
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL em `/dev/i2c-*`, manter a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2). Um `21 reads` ou um (f) lento lá aponta para a W-11;
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura; rodada 2, iteração 2). Todo `cargo` rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados por inotify, e não houve evento fora do controle. Linhas 2 a 13: `hollow=false`. Só a linha 1 está oca (objective), na família "só a política manda". Todas as mutações abaixo passam no fmt e no clippy `-D warnings` e deixam a lib em `76 passed`:

- **Piso no PASSO (W-22).** M1g (`settle.step.max(MIN_READ_GAP)` com 50 ms), M1g90 (90 ms) e M1g16 (`settle.step.max(settle.window / 16)`) dão Verify 1 `OK` em 5/5. O (f) só exige `calls.len() > 2`.
- **Piso na janela + piso no passo.** M1gw dá Verify 1 `OK` em 14/20. Isso desmente a frase do (f) e a premissa da D-2026-10-01-4(a), "qualquer piso acima da janela injetada excede o limite".
- **Teto na JANELA**, no worker e no cliente. M1c (`settle.window.min(MAX_SETTLE_WINDOW)` com 5 s, em `settle_input` e `write_budget`) dá 5/5, porque a maior janela dos testes é 4 s.
- **Teto no PASSO.** M1s (`settle.step.min(1 s)`) e M1s4 (`settle.step.min(settle.window / 4)`) dão 5/5, porque o maior passo dos testes é 500 ms.
- **Passar da janela.** M1d (`sleep(settle.step)` no lugar de `sleep(left.min(settle.step))`) dá 5/5, porque toda janela de teste é múltiplo do passo. Isso quebra o "sem ultrapassar" de (b).
- **Piso no orçamento do CLIENTE.** M1bf150 (`window.max(150 ms)` em `write_budget`) dá 10/10. (e) só testa a janela de 4 s, e (f) tolera até `vcp + janela + 1 s`.
- **Não objetivo (evasão deliberada).** Um `#[path = "worker/settle_checks.rs"]` antes de `#[cfg(test)] mod tests;` troca o módulo de teste compilado por um arquivo não congelado. A regex do Verify 8 não casa `#[path`.
- Mortas (Verify 1 0/5): leitura extra antes do 1º sleep, `if left < step`, sem `.min(deadline)` e `end = deadline`. Não afirmado: janela contada desde antes do write.

Fechamento proposto: (b) itera uma TABELA de políticas em relógio virtual e exige, para cada uma, `Ok`, `leituras == ceil(janela/passo)`, `sleeps == [passo; n−1] ++ [janela − passo·(n−1)]` e `Σ == janela`. A tabela cobre 5 ms/100 ms, 500 ms/4,25 s, 2 s/10 s, 10 min/20 min, 300 ms/1 s e `passo == janela`. (e) exige `write_budget_of(0x60) == vcp + janela` para cada política. Numa sonda feita só na cópia, o HEAD passa e as 8 sobreviventes falham.

**Verdict:** BLOCKED
