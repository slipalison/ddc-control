# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Rodada 2, iteração 1 do loop (6ª iteração absoluta, depois do auto-reset 1/3).
> - Branch `phase/input-switch-autostart`, HEAD `846dbc1`, base `origin/main` = merge-base `134b665`.
> - O conteúdo foi produzido pelo reviewer e gravado pelo orquestrador, porque o harness nega escrita de `.md` ao subagente.
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT (as 2 linhas Manual não têm comando).
> - Prefixos sha256 dos comandos: ctx1 `cc94c90671ee`, ctx2 `81b3ba163edc`, ctx3 `8f648d4f4fe0`, ctx4 `05ff88d61fb8`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `b89b1c611b3b`, ctx9 `ae65d59b93ea`, ctx10 `3ce6ae8234d5`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - As linhas 2, 5, 6, 7, 9 e 10 batem byte a byte com a extração do CONTEXT de `a91d193`.
> - As linhas 1, 3, 4 e 8 diferem, como esperado: são as emendadas em `f9e869e`/`846dbc1` (D-2026-10-01-input-switch-autostart-3).
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus` do ambiente.
>
> Os 10 do CONTEXT e os Verify de `cargo test` e de TODO do PROJECT rodaram LITERALMENTE:
> - a linha 4 rodou com `npm ci` e o Playwright inteiro;
> - a linha 9 rodou 3 vezes;
> - a cobertura reusa o Gate 3.
>
> **Herança da iteração 5, só por diff.** `git diff a91d193 HEAD -- . ':!.jdi'` lista 4 arquivos: `input-notice.spec.mjs`, `view-model.test.mjs`, `crates/ddc-adapters/Cargo.toml` e `worker/tests.rs`. Para eles nada foi herdado, nem o tempo do (f) nem as sondas da linha 10, que foram refeitas. Herdados, porque os arquivos não mudaram desde `a91d193`:
> - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
> - as mutações A, R, B1 e B2 da linha 9 (`status_item.rs`, `lib.rs` e `scripts/` sem diff);
> - a M6 da linha 6 e a sonda de 1 byte da linha 2 (arquivos e Verify sem diff).
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
>
> **Estado da máquina.**
> - O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim.
> - A listagem do `~/.config/autostart` real tem os mesmos sha256 antes e depois: `ls -A` `bf0a7045a8fb…` e `ls -la` `f69da4fd3ea5…`.
> - No fim não sobrou nenhum `dbus-run-session`, watcher, servidor HTTP do Playwright, tempdir `/tmp/smoke-*` ou laço de CPU.
> - As cópias `/var/tmp/rv6` e `/var/tmp/rv6b` foram apagadas.
>
> **Monitor real.**
> - Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados.
> - As mutações mexeram só em templates i18n, em constantes, no cálculo da janela de `settle_input` sobre `FakeDisplays`, no `Cargo.toml` ou plantaram corpos vazios (`fn planted() {}`).
> - Nenhuma mutação criou teste que fale com o backend real.
> - Todo `cargo` das mutações rodou em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados (ver 5.7).
>
> **Ambiente.** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. A prova por inotify em `bwrap` rodou ANTES de qualquer execução nativa da suíte.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0, e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` também. Única nota: o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados), igual no `bwrap` e no nativo. Igual à iteração 5; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed`. A saída não tem mais o bloco `Doc-tests ddc_adapters`: só `ddc_cli`, `ddc_core` e `ddc_tray`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. Rodou dentro do `bwrap`, com o observador ativo. Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. `main.rs` sem diff na phase. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0 no repositório e também A FRIO, na cópia descartável com `target/` próprio. O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a leitura de cada teste alterado e a prova por inotify em `bwrap`: zero aberturas de `/dev/i2c-*` na suíte, na cobertura e nas mutações, com controle positivo. |
| Consistency | PASS (com warnings) | D-2026-10-01-3 (a), (b) e (c) conformes no código, assim como D-3, D-4, D-12, D-14, D-17(a), D-19 e D-2026-10-01-1/-2. 5 commits desde `a91d193`, com tipo, escopo e tamanho corretos. W-8 fechado. A W-20 é regressão de força de teste introduzida em `952dac1`, não violação de D-XX no código. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped, 0 failed, 0 flaky. Os 6 skipped são só `screenshots.spec.mjs:24/34/47` em `light`/`dark` (`test.skip` condicionado a `SCREENSHOTS=1`, arquivo sem diff). `input-notice.spec.mjs` tem 10 `✓`. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduos: **W-20** (linha 1, objetivo provável: piso no `Default` passa em 15 de 20 rodadas) e W-21 (linha 8, não objetivo: `#[test]` gerado por macro). |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`). É costura interna do adaptador de entrada (D-11), não port do core. Arquivo sem diff desde a iteração 2. Só nota.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O `grep -RL` não lista nenhuma raiz sem `forbid(unsafe_code)`, e `[workspace.lints.rust] unsafe_code = "deny"` está em `Cargo.toml:24`. Não há `unsafe {` em `ddc-adapters`.
- **5.6** Todos os hits de `unwrap`/`expect`/`panic` ficam depois do primeiro `#[cfg(test)]` de cada arquivo ou em comentário:
  - `autostart.rs` 200/213/274 (cfg na linha 172);
  - `lib.rs` 282/359/360 (cfg na 252);
  - `tray.rs:380` (cfg na 207);
  - `status_item.rs` 304/320 (cfg na 233);
  - `worker.rs:141` (comentário);
  - `stop_signals.rs` 105/108 (cfg na 82) e `kwin_placement.rs` 246/256/260/267 (cfg na 195).

  Nenhum hit em `ddc-core`. Os lints `unwrap_used`/`expect_used`/`panic` = `warn` seguem no workspace, e `clippy.toml` só libera em testes.
- **5.7 (segurança de escrita no monitor, delegada pela linha 8).**
  - Greps do gate:
    - `Dangerous` existe no core (`error.rs:33`, `capabilities/tests.rs`);
    - não há `Confirm::Yes` fora de fronteira humana ou teste;
    - `DdcHiMonitorBackend` em `crates/*/tests` e `apps` só aparece em `crates/ddc-adapters/tests/real_monitor.rs` e no composition root (`lib.rs:29`, `lib.rs:79`).
    - `real_monitor.rs` tem 7 testes, todos `#[ignore = "…DDC_HW_TESTS=1"]`, e 0 bytes de diff contra a base.
  - **Leitura dos testes alterados nesta iteração:**
    - **`worker/tests.rs` (`952dac1`).**
      - `custom_settle_policies()` (`tests.rs:1017-1030`, passo 500 ms e janela 4 s, `assert!(custom.window > defaults.input_settle.window)` na 1025) e `keeps_old_input_through()` (`tests.rs:1092`) só montam valores.
      - (b) roda por `write_then_read_with` (`tests.rs:1046-1059`): `FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)])` e `Worker` em relógio virtual (`on_bus_with`).
      - (e) cria também um `WorkerClient::spawn(source, budgets, policies)` sobre `FakeDisplays` (`tests.rs:1211-1212`) e só chama `write_budget_of`, que é puro (`worker.rs:464-465`).
      - O arquivo cita `DdcHi` só em `DdcHiBudgets`; não há `DdcHiDisplays` nem `DdcHiMonitorBackend`.
    - **`view-model.test.mjs`** (`6daa9cb`). Testes `node --test` puros, com os tradutores reais (`translator('en')`/`translator('pt-BR')`, linhas 31-32).
    - **`input-notice.spec.mjs`** (`6daa9cb`). Só entraram asserções `toHaveText` com frases literais (linhas 79, 95, 124, 149), sobre o bridge demo em memória. O `webServer` é `python3 -m http.server --directory src`, sem `window.__TAURI__`.
    - **`crates/ddc-adapters/Cargo.toml`** (`66b2665`). Só `[lib] doctest = false`.
    - **Demais testes Rust e JS.** Sem diff desde `a91d193`, e cobertos pela prova abaixo.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`. Ali dentro, `stat` mostra os 16 como "arquivo comum vazio". Um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) ficou ligado nesses arquivos da prova até o fim das mutações.
    - Controle positivo: `cat /dev/i2c-3` no sandbox gerou 1 evento (`mask=0x20`).
    - `cargo test --workspace --locked` no sandbox: `SUITE_EXIT=0`, 413 passed, 0 failed, 9 ignored, ZERO eventos.
    - `cargo llvm-cov` e cerca de 12 minutos de mutações, todos no sandbox: ZERO eventos.
    - No log inteiro há 1 evento, o do controle.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` 0.22.2 (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, warning permitido que já estava no lock da base. `Cargo.lock` sem diff desde `a91d193` (o `66b2665` não o muda). As deps novas da phase seguem sem advisory.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração 5** (`a91d193..HEAD`), 5 ao todo:
  - código:
    - `952dac1` test: só `worker/tests.rs`;
    - `66b2665` build: só `crates/ddc-adapters/Cargo.toml`;
    - `6daa9cb` test: `input-notice.spec.mjs` e `view-model.test.mjs`;
  - `.jdi/`: `f9e869e` e `846dbc1`.

  Escopo = slug, cabeçalhos com 64 a 71 caracteres, `.jdi/` nunca no mesmo commit que código (D-12), e os 5 trazem a linha `Claude-Session:`. O tipo `build` para `[lib] doctest = false` está correto.
- **Zero byte de diff** contra a base, reconfirmado em:
  - `crates/ddc-core`, `crates/ddc-cli` e `.github/`;
  - `capabilities/`, `tauri.conf.json`, `src-tauri/src/main.rs` e `commands.rs`;
  - `scripts/smoke-sni.sh`;
  - `crates/ddc-adapters/tests/real_monitor.rs`, `tests/e2e/confirm.spec.mjs` e `tests/ui/toast-states.test.mjs`.

  Não têm diff desde `a91d193`: `apps/ddc-tray/src-tauri/src`, `apps/ddc-tray/src`, `apps/ddc-tray/scripts` e `Cargo.lock`. Também não têm diff `worker.rs`, `retry.rs` e `ddc_hi_backend.rs`; código de produção não mudou nesta iteração.
- **D-2026-10-01-3** Conforme no código:
  - (a) a política custom de (b)/(e) é 500 ms/4 s, maior que o `Default` (`tests.rs:1025`). O conjunto das linhas não-comentário com `INPUT_SETTLE_*` é exatamente as 2 declarações `const` privadas e os 2 usos no `Default`, todos em `retry.rs`. **Mas a troca de 1 s por 4 s, em vez da soma das duas, abriu a W-20.**
  - (b) `[lib] doctest = false` (`crates/ddc-adapters/Cargo.toml:11-12`). `cargo metadata` dá `["lib"] ddc_adapters doctest=false`. A regex nova dos 3 arquivos casa só `#[cfg(test)]` + `mod tests;` (`ddc_hi_backend.rs:136-137`, `worker.rs:552-553`, `retry.rs:179-180`).
  - (c) frases literais inteiras no view-model (`view-model.test.mjs:263-268`, `assert.equal` em cada um dos 4 testes) e no spec (`toHaveText` nas linhas 79/95/124/149). `view-model.test.mjs` está congelado na linha 3.
- **D-3 ("outros códigos inalterados")** Conforme. `write_budget` (`worker.rs:188`) segue `budgets.vcp` para todo código que não é `0x60`, e a 3ª parte de (f) segue no harness congelado.
- **D-14 / D-17(a) / D-19** Conformes no código:
  - `settle_input` lê só `self.policies.input_settle` (`worker.rs:313`);
  - `write_budget_of` (`worker.rs:464`) é o único caminho, e `write_vcp` a chama (`worker.rs:539`);
  - `without_backoff()` mantém 5 ms/100 ms em `retry/tests.rs`, e (g) os fixa por literais.

  O que falta é TESTE: nenhum teste determinístico confere que o worker respeita uma janela de política MENOR que a do `Default` (W-20).
- **D-2026-10-01-2** Conforme e sem diff de código: `WorkerClient::policies` `pub(super)` (`worker.rs:432`), sem código de teste nos 3 arquivos de produção.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5** Agora fixado: os papéis de mantida e pedida são conferidos por frases literais nas linhas 3 e 4 (W-19 fechada).
- **D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código desde a iteração 5. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo.
- **PLAN x commits** Todos os 33 arquivos do diff da phase estão no PLAN. Todas as tasks `completed` têm teste. **W-8 fechado:**
  - `## Files modified (all tasks)` agora lista `crates/ddc-adapters/Cargo.toml`, `ddc_hi_backend/tests.rs`, `retry/tests.rs` e `support.mjs`;
  - T-1 fala em "5 testes iniciais (7 depois das Emendas, mais o (h))";
  - T-4 fala em "1 por teste, provado pela anotação `axe`".

  Resíduo cosmético: `## Test requirements` ainda diz "emendados por D-13, D-14, D-15, D-16 e D-17".
- **SUMMARY** O `## Files modified` lista exatamente os 33 arquivos do diff da phase. As mutações relatadas pelo doer (MW `74 passed; 2 failed`; M4s `not ok 147/148` e 4 ✘; M4g 2 ✘) batem com as reproduzidas aqui (M1d, M4s, M4g).

### Achados do critic da iteração 5, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| Linha 1, M1k: `INPUT_SETTLE_WINDOW` `pub(super)` + `use` em `worker.rs` + `settle.window.min(INPUT_SETTLE_WINDOW)` | FECHADO | M1k VERMELHA: (b) em `tests.rs:1154` e (e) em `tests.rs:1220`, `74 passed; 2 failed`, e o conjunto das constantes não bate. M1p (só `pub(super) const`, sem uso) dá `76 passed`, mas o Verify 1 sai sem `OK` pelo conjunto. **Resíduo novo: W-20** |
| Linha 3 (W-19): papéis trocados nos templates passavam | FECHADO | M4s, M4g e M4ge VERMELHAS (`not ok 147`/`148`). M4u (texto do "pode ter comutado") VERMELHA (`not ok 150`). M3f (comentário no `view-model.test.mjs`): `165 pass`, mas Verify 3 sem `OK` pelo SHA-256 |
| Linha 4 (W-19): idem, ponta a ponta | FECHADO | M4s 4 ✘ (`Expected: "The monitor is still on DisplayPort-1, not HDMI-1. …"` contra `Received: "The monitor is still on HDMI-1, not DisplayPort-1. …"`, e o mesmo em pt-BR). M4g 2 ✘ e M4u 2 ✘. M4f (comentário no spec): Verify 4 sem `OK` pelo SHA-256 |
| Linha 8, M8d: doctest em arquivo de produção rodava na suíte; e `#[cfg(all(` multilinha gerado pelo rustfmt | FECHADO | `doctest=false` efetivo (`cargo metadata`). Nenhum bloco `Doc-tests ddc_adapters` no `cargo test -p ddc-adapters` nem no `--workspace` do HEAD. M8n (sem `[lib] doctest = false`) e M8t (`doctest = true`) VERMELHAS. M8a (`#[cfg(all(test, …))]` formatado pelo rustfmt em várias linhas, `fmt` 0) e M8c (`#[cfg(doctest)]`) VERMELHAS. **Resíduo novo: W-21** |

### Mutações reproduzidas pelo reviewer
**Montagem.** Todas rodaram num repositório git descartável, `/var/tmp/rv6/repo`, já apagado, com `CARGO_TARGET_DIR=/var/tmp/rv6/target` próprio:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1;
- `diff --stat` idêntico ao do repositório real (33 arquivos, `2637 insertions(+), 87 deletions(-)`).

**Execução.**
- Mutações aplicadas por substituição exata, com 1 ocorrência conferida, e revertidas com `git checkout -- .` e `git status` limpo.
- Todo `cargo` (Verify 1, Verify 8, lib, fmt, clippy) rodou em `bwrap` com os 16 `/dev/i2c-*` cobertos e observados.
- Os Verify 3 e 4, que não usam `cargo`, rodaram fora do sandbox.
- Baselines da cópia, todas `OK`: linha 1 (4,9 s, a frio), linha 8 (1,6 s), linha 3 (0,39 s), linha 4 (23,6 s, com `npm ci`).
- Na cópia, `fmt` e clippy `-D warnings` a frio saem 0.

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | M1d `settle_input`: `settle.window` -> `settle.window.min(RetryPolicies::default().input_settle.window)` (trava no `Default` sem nomear a constante) | VERMELHA: (b) `tests.rs:1154` e (e) `tests.rs:1220`, `74 passed; 2 failed`. Verify sem `OK` |
| 1 | M1k (critic): `pub(super) const INPUT_SETTLE_WINDOW` + `use` em `worker.rs` + `.min(INPUT_SETTLE_WINDOW)` | VERMELHA: mesmas 2 falhas, mais o conjunto das constantes. Verify sem `OK` |
| 1 | M1p: só `pub(super) const INPUT_SETTLE_WINDOW` | Lib `76 passed`, Verify sem `OK` pelo conjunto das constantes |
| 1 | M1t: M1d + `worker/tests.rs` de `a91d193` (janela custom de 1 s) | Lib `76 passed`, `sha256sum: WARNING: 1 computed checksum did NOT match`. Verify sem `OK` |
| 1 | **M1f (sonda)**: `let window = settle.window.max(RetryPolicies::default().input_settle.window);`, formatada pelo rustfmt (piso no `Default`) | **`fmt` 0, clippy `-D warnings` 0, lib `76 passed` em 3/3, (b) e (e) verdes. Verify 1 `OK` em 15 de 20 rodadas.** Só o (f) a pega, por corrida (`Err(Timeout)` em `tests.rs:1270`; (f) isolado verde em 7/10). Contra os testes de `a91d193`: VERMELHA em 3/3, em (b) e (e). Ver W-20 |
| 1 | **M1h (sonda)**: `settle.window` -> `Duration::from_secs(4)`, o valor exato da janela custom | Lib `76 passed`, Verify `OK`; (f) falha em 3/15 pela mesma corrida. Sobreajustada (copia o número do teste). Ver W-20 |
| 1 | Correção proposta para a W-20, só na cópia: limite de cima de leituras na 1ª parte de (f) | HEAD 10/10 verde. M1f 10/10 VERMELHA (`32 reads`). M1h 10/10 VERMELHA |
| 3 | M4s `still on {kept}, not {asked}` -> `still on {asked}, not {kept}`, e o mesmo em pt-BR | VERMELHA: `not ok 147`, `not ok 148`. Verify sem `OK` |
| 3 | M4g genérico pt-BR `manteve {asked} em vez de {kept}` | VERMELHA: 147 e 148. Verify sem `OK` |
| 3 | M4ge genérico en `kept {asked} instead of {kept}` | VERMELHA: 147 e 148. Verify sem `OK` |
| 3 | M4u pt-BR `que este computador não alcança` -> `que este PC não alcança` | VERMELHA: `not ok 150`. Verify sem `OK` |
| 3 | M3f `// weakened` no fim de `view-model.test.mjs` | `165 pass`, mas SHA-256 `FAILED`. Verify sem `OK` |
| 4 | M4s (acima) | VERMELHA: 4 ✘ (`(en)`/`(pt-BR)` × `light`/`dark`), `Expected`/`Received` literais. Verify sem `OK` |
| 4 | M4g (acima) | VERMELHA: 2 ✘ (`Expected: "Brilho: o monitor manteve 75% em vez de 76%."` contra `Received: "… 76% em vez de 75%."`). Verify sem `OK` |
| 4 | M4u (acima) | VERMELHA: 2 ✘ na leitura que falha. Verify sem `OK` |
| 4 | M4f `// weakened` no fim de `input-notice.spec.mjs` | SHA-256 `FAILED`. Verify sem `OK` |
| 8 | M8n: remove `[lib]` + `doctest = false` | VERMELHA (sem `OK`) |
| 8 | M8t: `doctest = true` | VERMELHA (sem `OK`) |
| 8 | M8a: `#[cfg(all(test, target_os = "linux", target_pointer_width = "64", target_endian = "little", feature = "ddc-hi"))]` + `fn planted() {}`, depois de `cargo fmt` (multilinha, `fmt` 0) | VERMELHA (sem `OK`) |
| 8 | M8s: `#[ test ]` + `fn planted() {}` | VERMELHA (sem `OK`) |
| 8 | M8c: `#[cfg(doctest)]` + `fn planted() {}` | VERMELHA (sem `OK`) |
| 8 | **M8m (sonda)**: `macro_rules! planted { ($m:meta) => { #[$m] fn planted() {} }; }` + `planted!(test);` em `ddc_hi_backend.rs` | **VERDE (`OK`). `fmt` 0, clippy 0, e `ddc_hi_backend::planted ... ok` roda na suíte (`77 passed`).** Ver W-21 |
| 8 | M8l (sonda): `#[` / `test` / `]` em 3 linhas + `fn planted() {}` | VERDE (`OK`), mas `cargo fmt --all --check` sai 1 (Gate 4) |
| 10 | `// TODO later` e `  // fixme: x` no fim de `view-model.test.mjs` e de `input-notice.spec.mjs` (cópia `/var/tmp/rv6b`, já apagada) | VERMELHAS (sem `OK`) nos 2 arquivos |
| 10 | `// TODO(#12) later` nos mesmos lugares | `OK`, como deveria |

**Tempo de (f).** `worker/tests.rs` mudou, então foi medido de novo:
- em repouso: 0,32 s em 10/10;
- com 24 laços de CPU (cada um sob `timeout 75`, conferidos encerrados): entre 0,32 e 0,33 s em 10/10, todas passando.

A folga até o limite de 0,5 s do Verify é de 0,17 s.

**Linha 9:** 3/3 execuções literais `OK` (5,64 / 6,09 / 6,04 s), mais 1 execução direta dos dois scripts com saída 0. A instabilidade W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração 5:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff desde a iteração 1. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. Código do assentamento inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.**
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final (linha 7: `Exec=…/autostart_entry-c0e56ec32fbab8ff `).
  - `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado.

  O README documenta.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN defasado): FECHADO.** `## Files modified (all tasks)`, T-1 e T-4 foram atualizados. Resta 1 linha cosmética em `## Test requirements` ("emendados por D-13 … D-17").
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.** 0,32 s em repouso (10/10) e 0,32–0,33 s sob 24 laços de CPU (10/10); folga de 0,17 s até os 0,5 s do Verify. Num runner Windows com timer grosso a folga é menor, e o `rust-windows` mostra primeiro.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 3/3 literais + 1 direta OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-18 (linha 8, sintático): FECHADO.** `#[ test ]` (M8s) e o `#[cfg(all(` multilinha, inclusive o que o rustfmt gera (M8a), ficam vermelhos. Resta só a M8l (`#[`/`test`/`]` em 3 linhas), que o `fmt --check` (Gate 4) rejeita.
- **W-19 (linhas 3 e 4, papéis): FECHADO.** M4s, M4g, M4ge e M4u ficam vermelhas nas duas linhas, e os harnesses estão congelados por SHA-256 (M3f e M4f ficam vermelhas).
- **W-9, W-10: TRATADOS** (iteração 3). **W-12, W-13: FECHADOS** (iteração 4). **W-15, W-17: FECHADOS** (iteração 5; W-17 reconfirmado com 33 arquivos).

Novos nesta iteração:
- **W-20 (linha 1, lacuna objetiva provável, regressão do `952dac1`; D-14, D-17(a), D-2026-10-01-3(a)).**

  O que a linha afirma: "só a política manda", e que (b) pega "um worker que lesse a janela de outro lugar que não a política".

  O que mudou: o `952dac1` TROCOU a janela custom de (b)/(e), de 1 s (menor que o `Default`) para 4 s (maior), em vez de acrescentar a segunda. Desde então, nenhum teste determinístico confere que o worker respeita uma janela de política MENOR que a do `Default`.
  - **M1f**, 1 linha de produção já formatada pelo rustfmt: um piso no `Default` em `settle_input` (`worker.rs:313-314`).
    ```rust
    let window = settle
        .window
        .max(RetryPolicies::default().input_settle.window);
    let end = (self.clock.now() + window).min(deadline);
    ```
    - `fmt` 0, clippy `-D warnings` 0 no workspace, lib `76 passed` em 3/3; (b) e (e) passam.
    - Só o (f) a pega, e por corrida: com janela de 100 ms na política, o worker vai até o `deadline` do cliente (160 ms) e às vezes responde depois do `recv_timeout` (`Err(Timeout)` em `tests.rs:1270`). O (f) só exige `Ok` e `waited >= janela`, sem limite de cima.
    - **Verify 1 `OK` em 15 de 20 rodadas.**
    - Contra o `worker/tests.rs` de `a91d193` (janela de 1 s), a mesma mutação fica VERMELHA em 3/3, em (b) e (e).
  - **M1h**, sobreajustada: 4 s fixos no worker, o número exato da janela custom. Lib `76 passed`, Verify 1 `OK`; (f) falha em 3/15 pela mesma corrida. Sozinha não é objetiva, porque copia o número do teste, mas mostra a mesma causa.
  - Efeito em produção: nenhum, porque o `Default` é 3 s. O que se perde é a garantia, afirmada pela linha, de que só a política manda.
  - **Correção testada na cópia.** Na 1ª parte de (f), um limite de cima no número de leituras. É determinístico, porque `sleep` nunca volta antes do pedido, então são no máximo `ceil(janela/passo)` leituras:
    ```rust
    let most = settle.window.as_nanos().div_ceil(settle.step.as_nanos());
    assert!(calls.len() - 1 <= usize::try_from(most).unwrap(), "{} reads", calls.len() - 1);
    ```
    Resultado: HEAD 10/10 verde, M1f 10/10 VERMELHA (`32 reads`), M1h 10/10 VERMELHA. Alternativa: rodar (b) também com a janela de 1 s, além da de 4 s.
  - Como `worker/tests.rs` está congelado por SHA-256 na linha 1, isso pede emenda do DoD com nova D-XX.
- **W-21 (linha 8, resíduo não objetivo, baixo).** Um `#[test]` gerado por macro escapa da regex dos 3 arquivos:
  ```rust
  macro_rules! planted {
      ($m:meta) => {
          #[$m]
          fn planted() {}
      };
  }
  planted!(test);
  ```
  - Em `ddc_hi_backend.rs` (M8m): `fmt` 0, clippy 0, e `ddc_hi_backend::planted ... ok` roda na suíte (`77 passed`). O Verify 8 dá `OK`.
  - Exige metaprogramação deliberada para esconder o atributo. Nenhuma forma natural passa: M8a, M8s e M8c ficam vermelhas, e a M8l o `fmt` rejeita.
  - Fechamento barato e comportamental, se o loop quiser: exigir pelo `cargo test -p ddc-adapters --locked --lib -- --list` que todo teste sob `ddc_hi_backend`, `ddc_hi_backend::worker` e `ddc_hi_backend::retry` esteja num submódulo `::tests::`.
  - Hoje os módulos de teste da lib são `caching::tests`, `ddc_hi_backend::tests`, `ddc_hi_backend::hardware::tests`, `ddc_hi_backend::identity::tests`, `ddc_hi_backend::worker::tests` e `in_memory::tests`.
- **Nota (ambiente, sem ação no código).** `/dev/i2c-1..5` e `/dev/i2c-9..15` seguem com ACL `user:slipalison:rw-`. Um teste futuro que alcance `DdcHiDisplays` falaria com o RTK real nesta máquina. Manter, nas próximas rodadas (reviewer, critic, `/jdi-ship`), a leitura do 5.7 e a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **Nota (sem ação).** `.jdi/DECISIONS.md` é uma visão gerada e ignorada (`.gitignore:31`), e está defasada: não contém nenhuma D-2026-10-01-*. As fontes estão em `.jdi/decisions/`, e `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado, < 0,5 s; (b)/(e) com política CUSTOM de janela MAIOR que a do `Default`; (e) via `write_budget_of`; (f) pelo `WorkerClient` real em 3 partes; (g) fixa `Default` e `without_backoff()`; (h) fiação; o CONJUNTO das linhas que citam `INPUT_SETTLE_*` = 2 `const` privadas + 2 usos no `Default`, em `retry.rs`; 3 arquivos de teste congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,41 s. O conjunto listado são os 7 nomes, cada um com `1 passed; 0 failed` em 0,00 s, exceto (f), com 0,32 s (10/10 em repouso; 10/10 entre 0,32 e 0,33 s sob 24 laços de CPU). (h) listado e passa. `git grep` das constantes = as 4 linhas esperadas (mais 1 comentário `///`, excluído). `worker/tests.rs` `a91c4582…`, `ddc_hi_backend/tests.rs` `ec2d2980…` e `retry/tests.rs` `12ac10db…` batem. Mutações: M1d e M1k VERMELHAS ((b) `tests.rs:1154`, (e) `tests.rs:1220`). M1p pega pelo conjunto, e M1t pelo SHA-256. **Resíduo (W-20): M1f, piso no `Default`, dá Verify `OK` em 15 de 20 rodadas.** |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,11 s. Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base sai 0; sem `sleep`. Sonda de 1 byte herdada (Verify e `ddc-core` sem diff). |
| 3 | View-model puro + 4 testes nomeados, texto inteiro conferido por igualdade com frases LITERAIS (papéis de mantida e pedida) em `en`/`pt-BR`, para o aviso de input e o genérico; `tests/ui` inteira verde; `view-model.test.mjs` congelado por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 0,25 s. `# tests 165 / # pass 165 / # fail 0` (`npm run test:unit`, 88.48%). SHA-256 `7c650133…` bate. M4s, M4g e M4ge VERMELHAS (`not ok 147/148`), M4u VERMELHA (`not ok 150`); M3f VERMELHA pelo SHA-256. W-19 fechada. |
| 4 | Aviso visível de ponta a ponta: `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; nomes por LITERAIS e texto inteiro de cada toast (input en/pt-BR, genérico, leitura que falha) contra a frase literal com papéis; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 22,8 s, com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed, 0 flaky. São 10 `✓` em `input-notice.spec.mjs`, e o `OK` confere também os 4 `✓` dos 2 estados de pseudo-locale e o conjunto `axe` = as 10 entradas. SHA-256 `92f99e00…`/`ee930894…` batem. M4s (4 ✘), M4g (2 ✘) e M4u (2 ✘) VERMELHAS com `Expected`/`Received` literais; M4f VERMELHA pelo SHA-256. W-19 fechada. |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, fiação LINUX (ksni) e testes de `tray`; fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 1,89 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha com `0 ignored`. `src-tauri/src` sem diff desde `a91d193`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); o chamador de PRODUÇÃO (`tray::flip_autostart`) imprime a linha do `crate::report` no stderr; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,40 s. Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` passa e imprime 2 vezes `ddc-tray: could not change the start-with-system entry: read-only home`. M6 herdada (Verify, `tray.rs`, `autostart.rs` e `lib.rs` sem diff). |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,50 s. Conjunto = pai + filho; `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` impresso; listagem real idêntica (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: `DDC_HW_TESTS`/`/dev/i2c`/`DdcHiDisplays` em não-`.md` de `apps`/`crates` só nos 3 arquivos de produção; `DdcHiMonitorBackend` só neles e em `ddc_hi_backend/tests.rs`; nos 3, toda linha com `cfg` e todo atributo com `test` = só `#[cfg(test)]` + `mod tests;`; `doctest = false` efetivo e sem bloco `Doc-tests ddc_adapters`; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 0,83 s. Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `cargo metadata`: `["lib"] ddc_adapters doctest=false`. `cargo test -p ddc-adapters` sem `Doc-tests ddc_adapters`, e no `--workspace` também (só `ddc_cli`/`ddc_core`/`ddc_tray`). `capabilities/` só tem `default.json`, sem diff contra a base e sem `autostart`. `tauri-plugin-autostart v2.6.0` -> `ddc-tray`. M8n, M8t, M8a (formatada, `fmt` 0), M8s e M8c VERMELHAS. **Resíduo (W-21, não objetivo): `#[test]` gerado por macro dá `OK`. A M8l o Gate 4 rejeita.** O "nenhum teste fala com monitor real", delegado ao 5.7, foi conferido por leitura e por inotify (zero aberturas). |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B, monitor simulado exigido nas duas, marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **3/3 execuções literais** (5,64 / 6,09 / 6,04 s). Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor` e `smoke-sni: OK — PID 1426467 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor`, checkmark desmarcado; o 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray` e marca; o 2º remove e desmarca; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. `smoke-sni.sh` com 0 bytes de diff. PID 24429 intacto, nada sobrando. Mutações A/R/B1/B2 herdadas (arquivos sem diff). W-16 não apareceu. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase; `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,01 s. Sondas refeitas nos 2 arquivos alterados (`view-model.test.mjs`, `input-notice.spec.mjs`): `// TODO later` e `  // fixme: x` VERMELHOS, `// TODO(#12) later` `OK`. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,05 s. Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,01 s. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` (linha 10, "Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18). O último release é `## [0.1.0] - 2026-09-28` (linha 31). Nenhum heading de release novo nesta phase; sem diff desde `a91d193`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330, 333 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento (250 ms, até 3 s); os smokes em `scripts/` são citados (linha 387); a limitação do `Exec=` com espaço está em "Known limitations of the tray app" (linhas 392/399). Sem diff desde `a91d193`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers pelos gates. Todos os 13 Verify automáticos dão `OK` no HEAD, e os achados do critic da iteração 5 estão fechados:
- linha 1: M1k, com o conjunto das constantes e o SHA-256 também pegando M1p e M1t;
- linhas 3 e 4: M4s, M4g, M4ge e M4u, com os harnesses congelados;
- linha 8: doctest e `cfg(all(` multilinha.

**Antes de seguir para o critic, fechar a W-20** (linha 1, objetiva provável). Um piso no `Default` em `settle_input`, de 1 linha, passa no `fmt`, no clippy e em todos os testes determinísticos, e dá Verify 1 `OK` em 15 de 20 rodadas. É regressão do `952dac1`, que trocou a janela custom de 1 s pela de 4 s em vez de manter as duas. Se o loop seguir com critic sem essa correção, o reviewer espera BLOCKED pela linha 1. Correção mínima, já testada na cópia:
- um limite de cima de leituras na 1ª parte de (f) (`calls.len() - 1 <= ceil(janela/passo)`): HEAD 10/10 verde, M1f e M1h 10/10 vermelhas;
- e/ou (b) rodado também com uma janela menor que a do `Default`;
- ou as duas coisas.

Como `worker/tests.rs` está congelado, isso pede emenda do DoD (novo SHA-256) e nova D-XX.

Outros pontos:
- **W-21 é opcional:** exige metaprogramação deliberada. Se for fechar, a checagem pela `--list` (testes só em submódulos `::tests::`) é comportamental e pega qualquer forma.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL em `/dev/i2c-*`, manter a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2, e W-11 se o teste (f) flakar);
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura; rodada 2, iteração 1; todo `cargo` em `bwrap` com os 16 `/dev/i2c-*` cobertos e observados por inotify: zero eventos fora do controle). Linhas 2, 3, 4, 5, 6, 7, 9, 11, 12 e 13: `hollow=false`. Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective). W-20 confirmada e ampliada:
  - M1f, o piso no `Default` em `settle_input`, dá Verify 1 `OK` em 16 de 20 rodadas;
  - M1m, um piso "defensivo" com outro nome (`const MIN_SETTLE_WINDOW: Duration = Duration::from_secs(1);` + `settle.window.max(MIN_SETTLE_WINDOW)` em `worker.rs`), passa em fmt e clippy, dá lib `76 passed` e Verify 1 `OK` em 16 de 20 rodadas. O conjunto das linhas `INPUT_SETTLE_*` não a vê.

  As duas quebram "só a política manda" e a frase de (b). Causa: (b) e (e) só usam janela MAIOR que o `Default`, e a 1ª parte de (f) não tem limite de cima. Correção testada: limite de leituras na 1ª parte de (f) (`calls.len()-1 <= ceil(janela/passo)`); com ele, o HEAD dá 10/10 ok e a M1m 10/10 FAILED. Uma segunda janela de 1 s em (b) NÃO pegaria a M1m.
- DoD row «8 — segurança de hardware e superfície» (hollow, NÃO objetiva). W-21 confirmada. Além do `planted!(test)` do reviewer, há uma variante mais forte: um macro de tabela com `#[test]` literal definido em `lib.rs` (fora dos 3 arquivos) e chamado sem `cfg` em `worker.rs`. Ele roda `ddc_hi_backend::worker::the_input_source_is_vcp_0x60 ... ok`, com fmt 0, clippy 0 e Verify 8 `OK`. Exige uma chamada de teste sem `cfg` em escopo de produção, o que nenhuma convenção produz. Fechamento: uma checagem ANCORADA pela `--list`, em que todo teste sob `ddc_hi_backend::` case com `^ddc_hi_backend::((worker|retry|hardware|identity)::)?tests::`.
- DoD row «10 — TODO/FIXME nos arquivos não-Rust» (objective). Achado novo. A phase passou a tocar `crates/ddc-adapters/Cargo.toml` (`66b2665`), mas o pathspec do Verify 10 não inclui esse arquivo. Mutação, commitada num repositório descartável: `# TODO: turn doctests back on once ...` acima de `doctest = false`. Com ela, o Verify 10 e o Verify 13 dão `OK`. Controle: a mesma linha em `apps/ddc-tray/src-tauri/Cargo.toml` deixa o Verify 10 com exit 1. Correção: incluir o manifesto ou derivar a lista do diff da phase.

**Verdict:** BLOCKED
