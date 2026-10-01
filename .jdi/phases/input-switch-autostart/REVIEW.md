# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 4 do loop. Branch `phase/input-switch-autostart`, HEAD `e076b0e`, base `origin/main` = merge-base `134b665`. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
> Os `Verify:` foram extraídos por programa, com regex sobre as linhas `**Verify:**`: 10 do CONTEXT.md e 3 do PROJECT.md. A extração bate byte a byte com a de uma rodada anterior no mesmo scratchpad. O ambiente de execução:
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8` e `PATH` com `~/.cargo/bin`;
> - `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR` e `DBUS_SESSION_BUS_ADDRESS` do ambiente.
>
> Os 10 do CONTEXT e o de TODO do PROJECT rodaram LITERALMENTE, e o `cargo test` do PROJECT também. A linha 4 rodou com `npm ci` e o Playwright inteiro. A cobertura reusa o Gate 3.
> **Herança da iteração 3, só por diff.** `git diff 612dc00 HEAD -- . ':!.jdi'` lista 4 arquivos: `apps/ddc-tray/tests/e2e/support.mjs`, `ddc_hi_backend/tests.rs`, `worker.rs` e `worker/tests.rs`. Para eles nada foi herdado. Herdados, porque os arquivos não mudaram:
> - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
> - as mutações A, R, B1 e B2 da linha 9 (`status_item.rs`, `lib.rs` e `scripts/` sem diff desde `612dc00`).
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
> **Estado da máquina.** O `ddc-tray` do usuário (PID 24429; a máquina reiniciou desde a iteração 3) ficou rodando e intacto do começo ao fim. A listagem do `~/.config/autostart` real tem o mesmo sha256 antes e depois (`bf0a7045a8fb…`).
> **Monitor real.** Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados. A mutação M8d (um `#[test]` que enumeraria o backend real) só existiu no repositório descartável do Verify da linha 8. Esse Verify não compila nem roda testes, e a mutação foi revertida sem nunca ter sido compilada.
> **Incidente, sem efeito no repositório.** Uma medição de tempo sob carga estourou o limite do shell e deixou vivos, por cerca de 6 min, 30 laços de CPU criados pelo reviewer. Eles foram encerrados por PID, depois de conferir o cmdline de cada um. Nenhum processo do usuário foi tocado. A medição foi refeita com `timeout`.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` exit 0. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados). Iteração 3: 412; o +1 é o teste (h). Phase anterior: 386. |
| Coverage | PASS | `TOTAL 5585 832 85.10% 650 109 83.23% 3642 535 85.31% 0 0 -`: coluna Lines = **85.31%** (3642 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%. `main.rs`/`build.rs` excluídos. Arquivos da phase: `worker.rs` 96.91%, `retry.rs` 96.88%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz leitura de cada teste novo e uma prova comportamental: inotify em `bwrap`, zero aberturas de `/dev/i2c-*`. |
| Consistency | PASS (com warnings) | D-3..D-19 e D-2026-10-01-1 conformes no código. A frase da D-2026-10-01-1(c) não bate com `retry.rs` (W-15). Novos avisos: W-15 a W-17. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped (só `screenshots.spec.mjs:24/34/47` em `light` e `dark`, `test.skip` condicionado a `SCREENSHOTS=1`, arquivo fora do diff), 0 failed, 0 flaky. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduos: linha 8 (W-15) e instabilidade ambiental da linha 9 (W-16). |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`). Não é port do core: o core não tem conceito de autostart, e a trait é uma costura interna do adaptador de entrada (D-11). Tem 2 impls (`PluginEntry` e o `FakeEntry` de teste). Arquivo inalterado desde a iteração 2. Só nota.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O `grep -RL` não lista nenhuma raiz sem `forbid(unsafe_code)`, e `[workspace.lints.rust] unsafe_code = "deny"` está em `Cargo.toml:24`. Não há `unsafe {` em `ddc-adapters`.
- **5.6** Todos os hits de `unwrap`/`expect`/`panic` ficam depois do primeiro `#[cfg(test)]` de cada arquivo ou em comentário:
  - `autostart.rs` 200/213/274 (cfg na linha 172);
  - `lib.rs` 282/359/360 (cfg na 252);
  - `tray.rs:380` (cfg na 207);
  - `status_item.rs` 304/320 (cfg na 233);
  - `worker.rs:141` (comentário);
  - `stop_signals.rs` e `kwin_placement.rs` (sem diff).

  Nenhum hit em `ddc-core`.
- **5.7 (segurança de escrita no monitor, delegada pela linha 8).**
  - Greps do gate:
    - `Dangerous` existe no core;
    - não há `Confirm::Yes` fora de fronteira humana ou teste;
    - `DdcHiMonitorBackend` em `crates/*/tests` e `apps` só aparece em `crates/ddc-adapters/tests/real_monitor.rs` (7 testes `#[ignore]` com `DDC_HW_TESTS`, 0 bytes de diff) e no composition root (`lib.rs:29`, `lib.rs:79`).
  - **Leitura dos 28 `#[test]` Rust novos da phase:**
    - **`worker/tests.rs`, 15 testes.** Todos usam `FakeDisplays`/`CrashingDisplays` (`spawn` em `tests.rs:358-359`; `WorkerClient::spawn(FakeDisplays…)` em 1195-1196 e 1225-1231). Nenhum usa `DdcHiDisplays`.
    - **`ddc_hi_backend/tests.rs`, teste (h).** Constrói o backend REAL duas vezes, por `new()` e `new().with_budgets(...)`, mas só lê `client.policies()` e o `Debug`. `WorkerClient::spawn` só cria o canal e a thread (`worker.rs:436-454`). `Worker::new` só inicializa campos (`worker.rs:218-226`). `run` espera jobs (`worker.rs:210-214`). O `Debug` não enfileira nada (`worker.rs:519-525`). Nenhum job é enviado, então `DdcHiDisplays::enumerate` (`hardware.rs:23-29`) nunca roda.
    - **`autostart.rs` e `tests/autostart_entry.rs`.** Usam `tauri::test::mock_builder`/`mock_app`, e o filho roda com `HOME` em tempdir.
    - **`menu.rs`, `i18n.rs`, `tray.rs` e `tray/status_item.rs`.** Funções puras, `mock_app`, `FakeEntry` e `InMemoryMonitorBackend`.
    - **`notification_area.rs`.** Função pura `is_right_click`, compilada só no Windows.
  - **Testes JS.** `view-model.test.mjs` é puro. `input-notice.spec.mjs` e `pseudo-locale.spec.mjs` usam o bridge demo em memória (sem `window.__TAURI__`).
  - **Smokes.** `smoke-sni-private.sh` passa `--fake`. `smoke-autostart-private.sh` usa `DDC_TRAY_FAKE=1` e exige a linha do monitor simulado antes de qualquer clique.
  - **Prova comportamental.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`, com um observador inotify `IN_OPEN` nesses arquivos. Ali rodou `cargo test -p ddc-adapters -p ddc-tray -p ddc-cli -p ddc-core --locked`: 413 passed, `SUITE_EXIT=0` e ZERO eventos. O controle positivo (`cat /dev/i2c-3` no mesmo sandbox) gerou 1 evento.
  - Barreira ambiental extra, não contada como prova: neste usuário os `/dev/i2c-*` são `root 0600`, sem ACL.
  - Lacuna do próprio gate (W-15): o grep 5.7c não varre testes de unidade em `src/` (o (h) mora em `src/ddc_hi_backend/tests.rs`). A leitura manual e o inotify o cobriram.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit`: 1 warning permitido, `yoke-derive 0.8.3` yanked, que já estava no lock da base. As deps novas da phase não têm advisory: `tauri-plugin-autostart 2.6.0`, `auto-launch`, `dirs`, `dirs-sys`, `getrandom`, `redox_users`, `winreg`. `Cargo.lock` sem diff desde a iteração 3.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração 3** (`612dc00..HEAD`), 9 ao todo:
  - código: `1ed13b0` test, `53f18e9` refactor, `4f17fb0` test, `1b847ff` test, `4c149fc` test (helper de teste `support.mjs`), `9a02a69` test (move helper de teste);
  - `.jdi/`: `da95bda`, `433852c` e `e076b0e`.

  Escopo = slug, cabeçalhos com 64 a 71 caracteres, `.jdi/` nunca no mesmo commit que código (D-12). Os tipos batem com a natureza: os `test` só tocam código de teste (`#[cfg(test)]`/`tests.rs`/`support.mjs`), e o `refactor` toca produção e teste sem mudar comportamento. Só `e076b0e` traz a linha `Claude-Session:` (o SUMMARY registra); o squash-merge torna isso irrelevante.
- **Zero byte de diff** contra a base em:
  - `crates/ddc-core`, `crates/ddc-cli` e `.github/`;
  - `capabilities/`, `tauri.conf.json`, `src-tauri/src/main.rs` e `commands.rs`;
  - `scripts/smoke-sni.sh`;
  - `crates/ddc-adapters/tests/real_monitor.rs`, `tests/e2e/confirm.spec.mjs` e `tests/ui/toast-states.test.mjs`.
- **D-3, D-14, D-17(a)** Conformes:
  - passo de 250 ms e janela de 3 s em `RetryPolicies.input_settle`;
  - `INPUT_SETTLE_STEP`/`INPUT_SETTLE_WINDOW` privadas de `retry.rs` (`grep INPUT_SETTLE` acha só `retry.rs:22,27,32,112,113`);
  - `settle_input` lê só `self.policies.input_settle` (`worker.rs:312-325`), limitada pelo `deadline`;
  - sempre `Ok(())`, e os outros códigos ficam inalterados.
- **D-19(a)/(b)** Conformes:
  - teste (h) presente;
  - `write_budget` virou privada (`fn`, `worker.rs:188`);
  - o único caminho de produção até ela é `WorkerClient::write_budget_of(&self, code)` (`worker.rs:462-464`), chamado por `write_vcp` (`worker.rs:543`);
  - (g) fixa 250 ms/3 s e 5 ms/100 ms por literais.
- **D-2026-10-01-1(a)/(b)** Conformes: (f) ganhou o limite de cima (`tests.rs:1250-1254`), e `expectAccessible` anota `axe` depois do `expect(blocking…).toEqual([])` (`support.mjs:143-144`).
- **D-2026-10-01-1(c)** Código conforme. `production_backends()` está em `ddc_hi_backend/tests.rs`, e a única linha `cfg(test)` nova nos 3 arquivos é a do acessor `policies()`. **A frase da decisão, porém, não é exata:** "o único código de teste que a phase acrescenta aos 3 arquivos é o acessor". A phase também acrescentou 4 linhas só de teste em `retry.rs:133-136`, dentro do bloco `#[cfg(test)] impl RetryPolicies` (`without_backoff()` com 5 ms/100 ms, D-14), que já existia. Ver W-15.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código desde a iteração 3 nos arquivos que os implementam. Conformidade reconfirmada pelas linhas 3 a 10 rodadas de novo.
- **PLAN x commits** Todos os arquivos do diff estão no PLAN (`support.mjs` e `ddc_hi_backend/tests.rs` via `## Emendas`). Todas as tasks `completed` têm teste. Ver W-8 e W-17 para o texto defasado.

### Achados do critic da iteração 3, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| Linha 1 (i), W-12: cliente que passa `RetryPolicies::default()` ao orçamento | FECHADO | MC VERMELHA ((e) e (f)) e MA VERMELHA ((f)) |
| Linha 1 (ii): fiação de produção sem teste | FECHADO | MB VERMELHA ((h)) |
| Linha 1 (iii): Verify (0,5 s) mais frouxo que o texto (0,2 s) | FECHADO | O texto diz 0,5 s; MD (`without_backoff` 100 -> 400 ms) VERMELHA ((g)) |
| Linha 4: contagem `-ge 2` em vez de conjunto; import não checado | FECHADO | M4a: `grep -c` segue 5 e 10 testes verdes, mas o conjunto `axe` cai para 8 e o Verify sai sem `OK`; M4c sem `OK` |
| Linha 8: só `*.rs` | FECHADO | M8a (`export DDC_HW_TESTS=1  # /dev/i2c-4` em `.sh`) VERMELHA. **Resíduo novo: W-15** |
| Linha 9: `smoke-sni.sh` sem checagem de diff | FECHADO | Editar `smoke-sni.sh` no repositório descartável dá `ok=0` no fragmento do Verify |
| Linha 10, W-13 e alcance | FECHADO | 6 casos plantados VERMELHOS, 2 legítimos `OK` (tabela abaixo) |

### Mutações reproduzidas pelo reviewer
Mutações aplicadas por substituição exata (1 ocorrência conferida), fontes restauradas e conferidas com `cmp` ou `git checkout` + `status` limpo. Linhas 1 e 4: cópia `git archive HEAD` em `/var/tmp/rv4/head`, `CARGO_TARGET_DIR` próprio. Linhas 8, 9 e 10: repositório git descartável `/var/tmp/rv4/git8`, montado assim:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1;
- `diff --stat` idêntico ao do repositório real.

**Linha 1** (Verify literal + `cargo test -p ddc-adapters --locked --lib`):

| Mutação | Resultado |
|---|---|
| baseline da cópia | `76 passed; 0 failed` |
| MA `write_vcp`: `self.write_budget_of(code)` -> `write_budget(&self.budgets, &RetryPolicies::default(), code)` (`write_budget_of` segue vivo) | VERMELHA: só (f) FAILED, `75 passed; 1 failed … finished in 3.16s`. Verify sem `OK` |
| MB `DdcHiMonitorBackend::new()`: `RetryPolicies::default()` -> `RetryPolicies { input_settle: InputSettle { step: 250ms, window: ZERO }, ..default() }` | VERMELHA: só (h) FAILED. Verify sem `OK` |
| MC `write_budget_of`: `&self.policies` -> `&RetryPolicies::default()` | VERMELHA: (e) e (f) FAILED. Verify sem `OK` |
| MD `without_backoff`: janela 100 ms -> 400 ms | VERMELHA: só (g) FAILED. Verify sem `OK` |

Tempo de (f) sem mutação: 0,26 s em 10/10 execuções em repouso. Com 24 laços de CPU (loadavg ~28), ficou entre 0,26 e 0,27 s em 20/20, todas passando. A folga até o limite de 0,5 s do Verify é de cerca de 0,23 s.

**Linha 4** (Verify literal com `npm ci` e Playwright inteiro, na cópia):

| Mutação | Resultado |
|---|---|
| baseline da cópia | `OK` |
| M4a `// await expectAccessible(page);` em `an input read back as asked shows no notice` | `grep -c 'expectAccessible(page)'` segue 5; o spec dá `10 passed`; o conjunto `axe` tem 8 entradas. Verify sem `OK` |
| M4c o spec importa `test` de `@playwright/test` (os demais helpers de `support.mjs`) | Verify sem `OK` |

**Linha 8** (Verify literal no repositório descartável):

| Mutação | Resultado |
|---|---|
| baseline | `OK` |
| M8a anexar `export DDC_HW_TESTS=1  # /dev/i2c-4 real monitor` a `scripts/smoke-autostart-private.sh` | VERMELHA (sem `OK`) |
| M8b anexar `#[cfg(test)] fn helper() {}` a `ddc_hi_backend.rs` | VERMELHA (sem `OK`) |
| **M8c** função nova com `"DDC_HW_TESTS=1 /dev/i2c-4 DdcHiMonitorBackend::new()"` DENTRO do bloco `#[cfg(test)] impl RetryPolicies` que já existia em `retry.rs` | **VERDE (`OK`). Sobrevive. Ver W-15** |
| **M8d** anexar a `ddc_hi_backend.rs` um `#[test] fn talks_to_the_monitor() { let _ = DdcHiMonitorBackend::new().unwrap().enumerate(); }` solto (nunca compilado nem rodado) | **VERDE (`OK`). Sobrevive: a regex `cfg\(.*\btest\b` não vê `#[test]`, e o 5.7c não varre `src/`. Ver W-15** |

**Linha 9:**
- Fragmento `git diff --quiet "$b" -- apps/ddc-tray/scripts/smoke-sni.sh` com `smoke-sni.sh` editado dá `ok=0`.
- Herdadas da iteração 3, com arquivos sem diff: A (sem `menu_about_to_show`), R (sem `autostart::register` em `run()`), B1 (parte B sem `DDC_TRAY_FAKE=1`, em `bwrap`) e B2 (parte A sem `--fake`). Todas ficam vermelhas.
- Estabilidade, sobre binários release e com o `smoke-sni-private.sh` do HEAD:

| Binário | Parte A | Parte B |
|---|---|---|
| HEAD (`target/release/ddc-tray`) | 16 OK / 4 FAIL em 20 (3/3 dentro do Verify literal; 4 falhas em 17 execuções diretas), sempre `FAIL: the popup was hidden within 1.5 s of 'ddc-tray: popup shown'` depois de `ddc-tray: popup focused` | 5/5 OK |
| BASE `134b665` (release compilado em `/var/tmp`) | 7 OK / 1 FAIL em 8 (intercalado com o HEAD), mesma mensagem | n/a |

**Linha 10** (Verify literal no repositório descartável):
- VERMELHO, como deveria: ` * FIXME later` dentro de `/** */` em `en.js`; `   TODO later` sem asterisco dentro de `/* */` em `pt-BR.js`; o mesmo em `guard.js`; `TODO:` em `README.md`; `# FIXME` em `src-tauri/Cargo.toml`; `fixme` em `docs/hardware-validation.md`.
- VERDE, como deveria: `'Todos os ajustes'` em `pt-BR.js`; `// TODO(#12)` em `app.js`.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração 3:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff desde a iteração 1. A evidência da iteração 3 fica herdada: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam: menu no clique direito, item refletindo o SO, chave `HKCU\...\Run`. A linha 5 não o afirma (D-15) e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado. O PR registra o observado: entrada com sinal, entrada sem sinal, logout/login com autostart ligado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` que o monitor não confirma segura o worker por até 3 s. Leituras de outro monitor nesse intervalo estouram `budgets.vcp` com `Timeout`. Código do assentamento inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.**
  - O arquivo se chama `DDC Control.desktop`, com espaço.
  - O `Exec=` sai sem aspas e com espaço final.
  - `create_dir` não é recursivo.
  - `$XDG_CONFIG_HOME` é ignorado.

  As falhas são reportadas, sem pânico, e o README documenta.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider. Um monitor que arredonda o valor o mostraria a cada ajuste.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN defasado, estilo): PERSISTE, menor.** `## Emendas` cobre a iteração 4. O corpo das tasks segue com texto antigo:
  - T-1 fala em "5 testes" e em `write_budget(&DdcHiBudgets, VcpCode)`;
  - T-4 fala em "≥ 2 chamadas, idealmente 1 por teste", e a linha 4 agora exige as 5 por execução.
- **W-9, W-10: TRATADOS** (iteração 3).
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.** A 2ª parte nova levou o teste de 0,10 para 0,26 s. Sob 24 laços de CPU ficou entre 0,26 e 0,27 s em 20/20, com folga de cerca de 0,23 s até o limite de 0,5 s do Verify. Num runner Windows com timer grosso a folga é menor, e o `rust-windows` mostra primeiro.
- **W-12: FECHADO.** As mutações MC e MA ficam vermelhas.
- **W-13: FECHADO.** `en.js`/`pt-BR.js` não podem ter `/*`, e os casos plantados ficam vermelhos.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.

Novos na iteração 4:
- **W-15 (linha 8, lacuna residual, objetiva, baixo; D-2026-10-01-1(c)).** O texto da linha 8 diz: "o ÚNICO código de teste que a phase acrescenta a esses 3 arquivos é o acessor `#[cfg(test)] pub(crate) fn policies(&self)` de `worker.rs` (nenhum helper de teste se esconde da varredura nos arquivos excluídos …)". Isso não procede em dois pontos:
  - (i) **Hoje:** a phase acrescentou 4 linhas só de teste em `crates/ddc-adapters/src/ddc_hi_backend/retry.rs:133-136`, dentro do bloco `#[cfg(test)] impl RetryPolicies` (linha 119), que já existia. São o `input_settle: InputSettle { step: 5 ms, window: 100 ms }` de `without_backoff()`, da D-14. O conteúdo é inofensivo, sem tokens de hardware, mas a frase fica literalmente falsa.
  - (ii) **Alcance do Verify:** ele só confere as linhas `cfg(...test...)` NOVAS e a linha seguinte de cada uma. Por isso não vê código acrescentado dentro de um bloco `cfg(test)` que já existia (M8c sobrevive) nem um `#[test]` solto (M8d sobrevive). No caso do M8d, um teste novo em `ddc_hi_backend.rs` que enumera o backend real passaria a linha 8. Também passaria o grep 5.7c, que só olha `crates/*/tests` e `apps`. Só a leitura manual pegaria.

  No HEAD não há `#[test]` solto nos 3 arquivos, e a leitura do 5.7 e a prova por inotify confirmam que nenhum teste fala com monitor. O comportamento do usuário não muda. É garantia estrutural do DoD, e o critic deve marcar a linha como oca e objetiva. Duas formas de fechar:
  - (a) Mover `without_backoff()` para um arquivo de teste (por exemplo `retry/tests.rs` ou `worker/tests.rs`), para que os 3 arquivos de produção não tenham código de teste inline além do acessor, e incluir `#\[(tokio::)?test\b` na regex de atributos do Verify.
  - (b) Estreitar o texto para "nenhum `cfg(test)` nem `#[test]` NOVO além do acessor; dentro do bloco `#[cfg(test)]` que já existia em `retry.rs`, as linhas acrescentadas são exatamente as 4 de `without_backoff()`", e conferir esse conjunto exato no Verify.
- **W-16 (linha 9, instabilidade ambiental, não é regressão).** A parte A (`smoke-sni-private.sh` -> `smoke-sni.sh` sem edição) falhou em 4 de 20 execuções com o binário do HEAD, e em 1 de 8 com o binário da BASE `134b665` nas mesmas condições. A causa é sempre a mesma: `popup focused` seguido de `popup hidden` em menos de 1,5 s. O popup perde o foco no KWin/Wayland REAL do usuário: a sessão D-Bus privada isola o barramento, não o compositor. `smoke-sni.sh` e o tratamento de foco (`lib.rs:210-235`, `popup.rs`) não mudaram na phase. O Verify literal passou 3/3, e a parte B passou 5/5. Risco: uma nova execução da linha 9 (critic ou `/jdi-ship`) pode dar vermelho espúrio enquanto o usuário usa o desktop. Se falhar com exatamente essa mensagem, rodar de novo. Mexer no `smoke-sni.sh` exigiria nova D-XX (D-13).
- **W-17 (SUMMARY, estilo).** O `## Files modified` do SUMMARY.md omite `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/tests.rs` e `apps/ddc-tray/tests/e2e/support.mjs`. O corpo da "Iteração 4" e o PLAN os citam.
- Nota (sem ação): `.jdi/DECISIONS.md` é uma visão gerada. As fontes `D-2026-09-30-input-switch-autostart-19.md` e `D-2026-10-01-input-switch-autostart-1.md` estão presentes em `.jdi/decisions/`.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado, < 0,5 s; (b), (e) com política CUSTOM; (e) via `WorkerClient::write_budget_of`; (f) pelo `WorkerClient` real com limite de baixo e de cima; (g) fixa `Default` e `without_backoff()`; (h) fiação de produção | CONTEXT | Auto | PASS | `OK`, exit 0, 1,2 s. Conjunto listado = os 7 nomes; cada um `1 passed; 0 failed`, 0,00 s, exceto (f) 0,26 s. (h) listado e passando. Mutações reproduzidas: MA (f), MB (h), MC (e)+(f) e MD (g) ficam VERMELHAS, com Verify sem `OK`. W-12 fechada. (f) sob carga: entre 0,26 e 0,27 s, 20/20. |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,1 s. Dep normal direta = `thiserror`; `git diff --quiet $(merge-base) -- crates/ddc-core` sai 0; sem `sleep` em `crates/ddc-core/src`. |
| 3 | View-model puro + 4 testes nomeados; `tests/ui` inteira verde | CONTEXT | Auto | PASS | `OK`, exit 0, 0,24 s. `# tests 165 / # pass 165 / # fail 0 / # cancelled 0 / # skipped 0 / # todo 0`, e os 4 nomes com `ok`. |
| 4 | Aviso visível de ponta a ponta: `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; anotação `axe` em exatamente 5 x 2 resultados aprovados; 2 estados de pseudo-locale x 2 temas; suíte inteira verde | CONTEXT | Auto | PASS | `OK`, exit 0, 18,1 s com `npm ci --ignore-scripts` incluído. Releitura com `--reporter=list,json`: `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed, 0 flaky. 10 `✓` em `input-notice.spec.mjs` e 4 `✓` nos 2 estados de pseudo-locale. O jq dá exatamente as 10 entradas `light/dark › <5 nomes>` (106 resultados da suíte têm `axe`). M4a e M4c reproduzidas: Verify sem `OK`. |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, fiação LINUX (ksni) e testes de `tray`; fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 1,8 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha com `0 ignored`. Código sem diff desde a iteração 2. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,1 s. Os 4 testes listados e passando. Que `run()` chama `register` é provado pela parte B da linha 9 (mutação R herdada; `lib.rs` sem diff desde a iteração 3). |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,47 s. Conjunto = pai + filho; `Exec=.*autostart_entry-` impresso; listagem real idêntica antes e depois (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: tokens de hardware em qualquer não-`.md` de `apps`/`crates` fora dos 3 arquivos do adaptador; `DdcHiMonitorBackend` só neles e em `ddc_hi_backend/tests.rs`; único `cfg(test)` novo nos 3 = acessor `policies()`; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 0,19 s. `capabilities/` só com `default.json`, permissões iguais às da base, sem `autostart`; `cargo tree -p ddc-tray -i tauri-plugin-autostart` resolve (2.6.0). M8a e M8b VERMELHAS. **Resíduo (W-15): M8c e M8d sobrevivem, e a frase "o ÚNICO código de teste … é o acessor" é falsa por `retry.rs:133-136`** (só de teste, inofensivo). O "nenhum teste fala com monitor real", delegado ao 5.7, foi conferido por leitura e por inotify (zero aberturas). |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, sessão D-Bus PRIVADA, `--fake`; `smoke-sni.sh` sem diff; parte A + parte B, monitor simulado exigido nas duas, marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 5,6 s (release já atualizado no HEAD), **3/3 execuções literais**. Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor`, `smoke-sni: OK — PID 179963 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. Parte B: banner `… sandboxed HOME`, `smoke-autostart: the app serves the simulated monitor`, checkmark desmarcado, 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray` e marca, 2º clique remove e desmarca, `the item followed an entry created and removed behind the app's back`, `smoke-autostart: OK — …`. `smoke-sni.sh` com 0 bytes de diff (mutação: `ok=0`). PID 24429 intacto; nenhum watcher, `dbus-run-session`, tempdir ou servidor na 1420 sobrando. **Instabilidade ambiental da parte A (W-16): 4/20 no HEAD e 1/8 na BASE, não é regressão.** |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase (`src` sem os 2 de tradução, `tests` sem `fixtures`, `scripts`, `src-tauri/Cargo.toml`, `README.md`, `CHANGELOG.md`, `docs`); `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,01 s. Não é vazia: 6 casos plantados VERMELHOS (bloco com e sem asterisco em `en.js`, `pt-BR.js` e `guard.js`, README, Cargo.toml, docs) e 2 legítimos `OK` ("Todos os ajustes", `TODO(#12)`). W-13 fechada. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0. Gate 2: soma de 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.31% (`3642 535 85.31%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `Added` ("Start with system" em Linux e Windows) e `Fixed` (troca de entrada assenta até 3 s; o popup avisa); o último release é `## [0.1.0] - 2026-09-28` (linha 31); nenhum heading de release novo nesta phase. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system**; o parágrafo "Input switch" descreve o polling de 250 ms/3 s; `autostart.rs` e os wrappers `*-private.sh` são citados; a limitação do `Exec=` sem aspas está documentada (iteração 3); falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers. Depois de confirmar as 2 linhas Manual, a phase pode seguir para `/jdi-ship`.

- **Achados do critic da iteração 3: todos fechados.** As mutações que os expunham foram reproduzidas aqui de forma independente e ficam vermelhas, nos rows 1 (MA, MB, MC, MD), 4 (M4a, M4c), 8 (M8a, M8b), 9 (`smoke-sni.sh`) e 10 (6 casos).
- **Candidato a achado da próxima rodada do critic: W-15** (linha 8, objetivo). A frase "o ÚNICO código de teste … é o acessor" é falsa por `retry.rs:133-136`, e um `#[test]` solto ou um helper dentro do `cfg(test)` que já existia escapa do Verify (M8c, M8d). Para evitar outra volta do loop, corrigir antes: mover `without_backoff()` para um arquivo de teste e acrescentar `#[test]` à regex, ou estreitar o texto e conferir o conjunto exato das 4 linhas. Fora do escopo da phase: o grep 5.7c do reviewer não cobre testes de unidade em `src/`.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo. É perda de foco no desktop real, já presente na base.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2, e W-11 se o teste (f) flakar);
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 4). Linhas 2, 3, 5, 7, 9, 10, 11, 12 e 13: `hollow=false`. Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective). Mutação M1 em `worker.rs:543`, `WorkerClient::write_vcp`: `self.write_budget_of(code)` vira `self.write_budget_of(VcpCode::INPUT_SOURCE)`. Com ela, o Verify literal da linha 1 dá `OK`, `cargo test --workspace` dá 413 passed e o clippy `-D warnings` sai 0. Efeito medido numa sonda descartável (display preso num `Gate`, write de BRIGHTNESS): o `Timeout` chega em 160 ms (`vcp` + janela), contra cerca de 60 ms no HEAD. Isso quebra (e) ("`budgets.vcp` para qualquer outro código") e a D-3 ("outros códigos inalterados"): em produção, um write de brilho preso esperaria `budgets.vcp + 3 s`. Causa: (e) prova o MÉTODO, (f) só escreve `0x60`, e nenhum teste mede o orçamento de `write_vcp` para outro código. Correção sugerida: uma 3ª parte em (f) em que um write de outro código, pelo `WorkerClient` real e com o display preso, devolve `Timeout` antes de `budgets.vcp + janela`.
- DoD row «4 — aviso visível de ponta a ponta» (objective). Mutação M4: `notice.inputKept` perde o `{kept}` em `en.js:58` e em `pt-BR.js:58`. O Verify literal da linha 4 dá `OK` (17,6 s), mas o toast deixa de nomear a entrada MANTIDA. Causa: `input-notice.spec.mjs:64` e `:76` calculam o texto esperado com o MESMO template que está sob teste (tautologia). As únicas literais conferidas são "may have no signal" e "pode estar sem sinal". A linha 3 pega a mesma mutação (`not ok 12`). Correção: conferir os nomes por literais (`DisplayPort-1`, `HDMI-1`) em `#toast-text`.
- DoD row «6 — alternar e registro» (objective). Mutação M6 em `tray.rs:127`, `flip_autostart`: `autostart::toggle_or_report(&**entry, report);` vira `let _ = autostart::toggle(&**entry);` (erro engolido). Com ela, o Verify literal dá `OK`, `cargo test --workspace` dá 413 passed e o clippy sai 0. `tray::tests::a_refused_flip_leaves_the_entry_as_it_was -- --nocapture` imprime duas vezes `ddc-tray: could not change the start-with-system entry: read-only home` no HEAD e NADA sob M6. Isso quebra "a falha é devolvida ao chamador, que a manda para `crate::report`" (D-11): nenhum teste olha o que o chamador de produção faz com a falha, e a linha 9 só exige a AUSÊNCIA de `could not change` no caminho de sucesso.
- DoD row «8 — segurança de hardware e superfície» (objective). W-15 confirmada com evidência própria. (i) Segundo o `git blame`, `retry.rs:121-123` e `:133-136` vêm de `1077707`, da phase. Ficam dentro do `#[cfg(test)] impl RetryPolicies` que já existia, então são código só de teste além do acessor, e o Verify imprime OK. (ii) Num repositório descartável (commit 1 = `134b665`, commit 2 = HEAD, `diff --stat` idêntico): M8c' (função nova dentro do mesmo bloco `cfg(test)` de `retry.rs`) dá OK, e M8d' (um `#[test]` solto em `ddc_hi_backend.rs`) dá OK. O controle M8b sai 1. Por construção, um `#[test]` solto nesses 3 arquivos que construísse o backend real passaria, porque a regex `cfg\(.*\btest\b` não vê `#[test]`. Nota fora do texto: `DdcHiDisplays`, a fonte de displays real, não é token proibido.

**Verdict:** BLOCKED
