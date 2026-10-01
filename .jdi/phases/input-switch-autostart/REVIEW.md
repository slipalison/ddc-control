# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 5 do loop. Branch `phase/input-switch-autostart`, HEAD `46d06c4`, base `origin/main` = merge-base `134b665`. O conteúdo foi produzido pelo reviewer e gravado pelo orquestrador, porque o harness nega escrita de `.md` ao subagente.
>
> **Extração dos Verify.** Os `Verify:` foram extraídos por programa, com a regex `^\s*\*\*Verify:\*\*\s*`([^`]+)`` sobre o CONTEXT.md e o PROJECT.md: 10 do CONTEXT e 3 do PROJECT (as 2 linhas Manual não têm comando).
> - Prefixos sha256 dos comandos: ctx1 `c8a6801796b6`, ctx2 `81b3ba163edc`, ctx3 `175d15c5dc40`, ctx4 `4c5c7c2b8b53`, ctx5 `da4ccacf136b`, ctx6 `e41e73076d3f`, ctx7 `a1e6cea1a756`, ctx8 `7071e4216c66`, ctx9 `ae65d59b93ea`, ctx10 `3ce6ae8234d5`, prj1 `8c60d6ffe7ad`, prj3 `b9c553f9a577`.
> - As linhas 2, 3, 5, 7, 9 e 10 batem byte a byte com a extração da rodada anterior. As linhas 1, 4, 6 e 8 diferem, como esperado: são as emendadas em `9d20392`/`46d06c4`.
>
> **Ambiente de execução:**
> - `env -i` com `bash --noprofile --norc`;
> - `LC_ALL=C.UTF-8`, `HOME` e `PATH=~/.cargo/bin:/usr/local/bin:/usr/bin:/bin`;
> - `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_RUNTIME_DIR=/run/user/1000` e `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus` do ambiente.
>
> Os 10 do CONTEXT e os Verify de `cargo test` e de TODO do PROJECT rodaram LITERALMENTE. A linha 4 rodou com `npm ci` e o Playwright inteiro. A cobertura reusa o Gate 3.
>
> **Herança da iteração 4, só por diff.** `git diff d18829d HEAD -- . ':!.jdi'` lista 6 arquivos: `input-notice.spec.mjs`, `retry.rs`, `retry/tests.rs` (novo), `ddc_hi_backend/tests.rs`, `worker.rs` e `worker/tests.rs`. Para eles nada foi herdado. Herdados, porque os arquivos não mudaram desde `d18829d`:
> - W-2: clippy `-D warnings` do `notification_area.rs` para `x86_64-pc-windows-gnu`;
> - as mutações A, R, B1 e B2 da linha 9 (`status_item.rs`, `lib.rs` e `scripts/` sem diff);
> - os 6 casos vermelhos e os 2 verdes plantados na linha 10, cujo Verify não mudou. Duas sondas novas foram refeitas no spec alterado.
>
> Fora isso, todos os gates e todos os Verify rodaram de novo.
>
> **Estado da máquina.** O `ddc-tray` do usuário (PID 24429) ficou rodando e intacto do começo ao fim. A listagem do `~/.config/autostart` real tem os mesmos sha256 antes e depois: `ls -A` `bf0a7045a8fb…` e `ls -la` `f69da4fd3ea5…`. No fim não sobrou nenhum `dbus-run-session`, watcher, servidor na 1420, tempdir `/tmp/smoke-*` ou laço de CPU. A cópia `/var/tmp/rv5` foi apagada.
>
> **Monitor real.** Nada escreveu em monitor real, e `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados. As mutações da linha 8 plantaram só corpos vazios (`fn planted() {}`) e nunca foram compiladas: o Verify 8 não compila nada. Nenhuma mutação criou teste que fale com o backend real.
>
> **Mudança de ambiente desde a iteração 4.** `/dev/i2c-1..5` e `/dev/i2c-9..15` agora têm ACL `user:slipalison:rw-` (criada às 07:31 de hoje). Na iteração 4 eram `root 0600` sem ACL. A barreira ambiental não existe mais, então a prova por inotify em sandbox rodou ANTES das execuções nativas (ver 5.7).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` sai 0, e `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` também. Única nota: o aviso future-incompat de `nom v3.2.1`, que já estava na base. |
| Tests | PASS | 413 passed, 0 failed, 9 ignored (hardware, não rodados). Igual à iteração 4; a phase anterior tinha 386. A lib `ddc-adapters` dá `76 passed`. |
| Coverage | PASS | `TOTAL 5573 832 85.07% 647 109 83.15% 3626 535 85.25% 0 0`: coluna Lines = **85.25%** (3626 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%, `main.rs`/`build.rs` excluídos. A iteração 4 tinha 3642 linhas; as −16 são o código só de teste que saiu de `retry.rs`/`worker.rs`. Arquivos da phase: `worker.rs` 96.88%, `retry.rs` 96.08%, `autostart.rs` 96.00%, `menu.rs` 98.94%, `i18n.rs` 100%. `main.rs` sem diff na phase. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` sai 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` sai 0 no repositório e também A FRIO, na cópia descartável com `target/` próprio. O grep de `#[allow(` fora de testes não tem saída. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. O 5.7 traz a leitura de cada teste novo ou alterado e a prova por inotify em `bwrap`: zero aberturas de `/dev/i2c-*`, com controle positivo. |
| Consistency | PASS (com warnings) | D-3, D-4, D-12, D-14, D-17(a), D-19, D-2026-10-01-1 e -2 conformes no código. 5 commits desde `d18829d`, com tipo, escopo e tamanho corretos. W-15 e W-17 fechados; W-8 persiste. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped (só `screenshots.spec.mjs:24/34/47` em `light`/`dark`, `test.skip` condicionado a `SCREENSHOTS=1`, arquivo fora do diff), 0 failed, 0 flaky. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. Resíduos: W-18 (linha 8, sintático, o Gate 4 pega) e W-19 (linhas 3/4, papéis dos nomes, objetivo). |

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
  - `stop_signals.rs` 105/108 (cfg na 82) e `kwin_placement.rs` 246/256/260/267 (cfg na 195), ambos sem diff.

  Nenhum hit em `ddc-core`. Os lints `unwrap_used`/`expect_used`/`panic` = `warn` seguem no workspace, e `clippy.toml` só libera em testes.
- **5.7 (segurança de escrita no monitor, delegada pela linha 8).**
  - Greps do gate:
    - `Dangerous` existe no core (`error.rs:33`, `capabilities/tests.rs`);
    - não há `Confirm::Yes` fora de fronteira humana ou teste;
    - `DdcHiMonitorBackend` em `crates/*/tests` e `apps` só aparece em `crates/ddc-adapters/tests/real_monitor.rs` e no composition root (`lib.rs:29`, `lib.rs:79`). `real_monitor.rs` tem 7 testes, todos `#[ignore = "…DDC_HW_TESTS=1"]`, e 0 bytes de diff contra a base.
  - Varredura extra em `src/`, que o 5.7c não cobre: `DdcHiDisplays`/`DdcHiMonitorBackend`/`ddc_hi::Display::enumerate` só aparecem nos 3 arquivos de produção, em `hardware.rs` (sem diff), em `lib.rs` do adapter (re-export), em `ddc-cli/src/main.rs` e `ddc-tray/src/lib.rs` (composition roots) e em `ddc_hi_backend/tests.rs`.
  - **Leitura dos testes novos ou alterados nesta iteração:**
    - **`worker/tests.rs`, 3ª parte de (f)** (`tests.rs:1237-1271`): `FakeDisplays::with([FakeDisplay::new("a", Behaviour::Block(gate.clone()))])` e `spawn(&stuck_other, budgets(vcp))`. O `spawn` de `tests.rs:358-359` é `WorkerClient::spawn(source.clone(), budgets, no_backoff())` sobre `FakeDisplays`. O arquivo não cita `DdcHiDisplays` nem `DdcHiMonitorBackend` (só `DdcHiBudgets`).
    - **`ddc_hi_backend/tests.rs`, teste (h)** (`tests.rs:61`): agora lê o CAMPO `backend.client.policies` e o `Debug`. `WorkerClient::spawn` só cria o canal e a thread (`worker.rs:438-456`), e `Worker::new` só inicializa campos (`worker.rs:218-226`). `run` espera jobs (`worker.rs:210-214`), e o `Debug` imprime só os `budgets` (`worker.rs:515-521`). Nenhum job é enviado, então `DdcHiDisplays::enumerate` (`hardware.rs:20-29`) nunca roda.
    - **`retry/tests.rs`.** Nenhum `#[test]`, só o helper `without_backoff()` (`retry/tests.rs:12`).
    - **`input-notice.spec.mjs`.** Só asserções novas (`expectToastNaming`, `spec:57`), sobre o bridge demo em memória (sem `window.__TAURI__`).
    - **Demais testes Rust e JS.** Sem diff desde a iteração 4 (leitura daquela iteração), e cobertos pela prova abaixo.
  - **Prova comportamental, ANTES das execuções nativas.** O reviewer pôs 16 arquivos comuns sobre `/dev/i2c-0..15` dentro de `bwrap --dev-bind / /`, com um observador inotify (`IN_OPEN|IN_ACCESS|IN_MODIFY`, Python + ctypes) nesses arquivos.
    - Controle positivo: `cat /dev/i2c-3` no mesmo sandbox gerou 1 evento (`mask=0x20`).
    - Ali rodou `cargo test --workspace --locked`: `SUITE_EXIT=0`, 413 passed, 0 failed, 9 ignored e ZERO eventos.
  - Sem barreira ambiental: ver a nota de ambiente nos Warnings.
  - Lacuna do próprio gate, que segue: o grep 5.7c não varre testes de unidade em `src/`. A leitura manual e o inotify a cobriram. Fora do escopo da phase.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit` 0.22.2 (1278 advisories, 531 crates): só `yoke-derive 0.8.3` yanked, warning permitido que já estava no lock da base. `Cargo.lock` sem diff desde `d18829d`; as deps novas da phase seguem sem advisory.
- **5.11** Sem segredo. Sem `TODO`/`FIXME` sem issue em `*.rs`.

### Consistency em detalhe
- **Commits desde a iteração 4** (`d18829d..HEAD`), 5 ao todo:
  - código:
    - `ecf5ef0` test: só `worker/tests.rs`;
    - `4393927` test: só `input-notice.spec.mjs`;
    - `ed8114b` refactor: `retry.rs`, `retry/tests.rs` (novo), `ddc_hi_backend/tests.rs` e `worker.rs`. Move código de teste e muda a visibilidade de um campo, sem mudar comportamento: as mesmas 413 passam;
  - `.jdi/`: `9d20392` e `46d06c4`.

  Escopo = slug, cabeçalhos com 63 a 71 caracteres, `.jdi/` nunca no mesmo commit que código (D-12), e os 5 trazem a linha `Claude-Session:`.
- **Zero byte de diff** contra a base, reconfirmado em:
  - `crates/ddc-core`, `crates/ddc-cli` e `.github/`;
  - `capabilities/`, `tauri.conf.json`, `src-tauri/src/main.rs` e `commands.rs`;
  - `scripts/smoke-sni.sh`;
  - `crates/ddc-adapters/tests/real_monitor.rs`, `tests/e2e/confirm.spec.mjs` e `tests/ui/toast-states.test.mjs`.

  `apps/ddc-tray/src-tauri/src`, `apps/ddc-tray/src` e `apps/ddc-tray/scripts` não têm diff desde `d18829d`.
- **D-3 ("outros códigos inalterados")** Conforme. `write_budget` (`worker.rs:188`) segue `budgets.vcp` para todo código que não é `0x60`. Agora isso está provado pela 3ª parte de (f): a mutação M1 fica VERMELHA.
- **D-14 / D-17(a) / D-19** Conformes:
  - `INPUT_SETTLE_STEP`/`INPUT_SETTLE_WINDOW` são privadas de `retry.rs` (`git grep` acha só `retry.rs:22,27,32,112,113`);
  - `settle_input` lê só `self.policies.input_settle` (`worker.rs:312`);
  - `write_budget` é privada; `write_budget_of` (`worker.rs:464`) é o único caminho e `write_vcp` a chama (`worker.rs:539`);
  - `without_backoff()` mantém 5 ms/100 ms em `retry/tests.rs`, e (g) os fixa por literais.
- **D-2026-10-01-2** Conforme:
  - (a) (f) ganhou a 3ª parte (`tests.rs:1237-1271`);
  - (b) `expectToastNaming` confere literais em `spec:75/88/115/137`;
  - (c) linha 6 provada pelo stderr, sem código novo;
  - (d) nos 3 arquivos de produção, a regex casa só `#[cfg(test)]` + `mod tests;` (`ddc_hi_backend.rs:136-137`, `worker.rs:552-553`, `retry.rs:179-180`). O acessor `policies()` saiu, e `WorkerClient::policies` passou a `pub(super)` (`worker.rs:432`), visível só dentro de `ddc_hi_backend`. `DdcHiDisplays` entrou nos tokens proibidos do Verify 8.

  Trade-off aceito pela decisão: o campo fica legível também pelo código de produção de `ddc_hi_backend.rs`, mas hoje só o teste (h) o lê. Sem ação.
- **D-2026-10-01-1(c)** A frase "o único código de teste … é o acessor" foi substituída pela D-2026-10-01-2(d) e pelo texto novo da linha 8. O estado atual bate com a decisão vigente, o que fecha a parte de texto da W-15.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6 / D-8 / D-9 / D-10 / D-11 / D-13 / D-15 / D-16 / D-18** Sem diff de código nos arquivos que os implementam desde a iteração 4. Conformidade reconfirmada pelas linhas 3 a 10, rodadas de novo. Sobre a D-5 ("nomeia mantida e pedida"), o código do HEAD está correto, mas nenhuma linha Auto fixa os papéis: ver W-19.
- **PLAN x commits** Todos os arquivos do diff estão no PLAN (`retry/tests.rs` via `## Emendas`, iteração 5), e todas as tasks `completed` têm teste. O `## Files modified (all tasks)` segue sem `ddc_hi_backend/tests.rs`, `retry/tests.rs` e `support.mjs` (W-8).
- **SUMMARY** O `## Files modified` lista exatamente os 32 arquivos do diff da phase (W-17 fechado).

### Achados do critic da iteração 4, reavaliados
| Achado | Estado | Prova nesta iteração |
|---|---|---|
| Linha 1, M1: `write_vcp` com `self.write_budget_of(VcpCode::INPUT_SOURCE)` (brilho preso esperaria `vcp + 3 s`) | FECHADO | M1 VERMELHA: só (f) FAILED em `worker/tests.rs:1267`, `75 passed; 1 failed`, Verify 1 sem `OK`. Harness congelado: M1 com o assert enfraquecido (`budget * 2`) dá `76 passed`, mas o Verify 1 sai sem `OK` pelo SHA-256 de `worker/tests.rs` |
| Linha 4, M4: `notice.inputKept` sem `{kept}` (tautologia do template) | FECHADO | M4 VERMELHA: (en) e (pt-BR) ✘ nos 2 temas (4 falhas), `Expected substring: "DisplayPort-1"` contra `Received string: "The monitor is still on, not HDMI-1. …"`. Verify 4 sem `OK`. **Resíduo novo: W-19** |
| Linha 6, M6: `let _ = autostart::toggle(&**entry);` em `flip_autostart` (erro engolido) | FECHADO | Com M6, `a_refused_flip_leaves_the_entry_as_it_was` segue verde (lib `152 passed`), mas a linha `ddc-tray: could not change the start-with-system entry: read-only home` some do stderr, e o Verify 6 sai sem `OK`. No HEAD ela sai 2 vezes, uma por direção do laço, via `flip_autostart` -> `toggle_or_report` -> `crate::report` (`lib.rs:241-243`) |
| Linha 8 (W-15): código de teste dentro de `cfg(test)` que já existia em `retry.rs`; `#[test]` solto | FECHADO | `retry.rs` não tem mais código de teste, e o acessor saiu de `worker.rs`. M8x (`#[test] fn planted() {}` solto em `ddc_hi_backend.rs`), M8y (`#[cfg_attr(test, derive(Clone))]` em `WorkerClient`) e M8t (`// DdcHiDisplays …` em `hardware/tests.rs`) ficam VERMELHAS. **Resíduo novo: W-18** |

### Mutações reproduzidas pelo reviewer
Todas rodaram num repositório git descartável, `/var/tmp/rv5/repo`, já apagado, com `target/` próprio, montado assim:
- commit 1 = `git archive 134b665`;
- commit 2 = `git archive HEAD`;
- `origin/main` -> commit 1;
- `diff --stat` idêntico ao do repositório real.

As mutações foram aplicadas por substituição exata, com 1 ocorrência conferida, e revertidas com `git checkout -- .` e `git status` limpo. Baselines da cópia: linha 1 `OK` (4,9 s), linha 4 `OK` (18,6 s), linha 6 `OK` (41,6 s, com compilação), linha 8 `OK` (0,2 s).

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | M1 `write_vcp`: `self.write_budget_of(code)` -> `self.write_budget_of(VcpCode::INPUT_SOURCE)` | VERMELHA: só (f) FAILED (`tests.rs:1267`). Verify sem `OK` |
| 1 | M1 + assert da 3ª parte enfraquecido para `other_gave_up < budget * 2` | Lib `76 passed; 0 failed`, mas `sha256sum: worker/tests.rs: FALHOU`. Verify sem `OK` |
| 4 | M4 `still on {kept}, not {asked}` -> `still on, not {asked}`, e o mesmo em pt-BR | VERMELHA: 4 ✘. Verify sem `OK` |
| 4 | **M4s (sonda)** 1ª frase com papéis trocados, `still on {asked}, not {kept}` / `continua em {asked}, não em {kept}` | **VERDE: Verify 3 `OK` e Verify 4 `OK`. Sobrevive. Ver W-19** |
| 4 | **M4g (sonda)** genérico pt-BR `manteve {asked} em vez de {kept}` | **VERDE: Verify 3 `OK` e Verify 4 `OK`. Sobrevive. Ver W-19.** Em `en`, a linha 3 pega por igualdade literal |
| 6 | M6 `autostart::toggle_or_report(&**entry, report);` -> `let _ = autostart::toggle(&**entry);` | VERMELHA: teste verde, linha do stderr ausente. Verify sem `OK` |
| 8 | M8x `#[test]` + `fn planted() {}` em `ddc_hi_backend.rs` | VERMELHA (sem `OK`) |
| 8 | M8y `#[cfg_attr(test, derive(Clone))]` em `WorkerClient` (`worker.rs`) | VERMELHA (sem `OK`) |
| 8 | M8t `// DdcHiDisplays is the real source` em `hardware/tests.rs` | VERMELHA (sem `OK`) |
| 8 | **M8z (sonda)** `#[ test ]` com espaços + `fn planted() {}` em `ddc_hi_backend.rs` | **VERDE (`OK`)**, mas `cargo fmt --all --check` sai 1. Ver W-18 |
| 8 | **M8w (sonda)** `#[cfg(` / `test` / `)]` em 3 linhas + `fn planted() {}` em `retry.rs` | **VERDE (`OK`)**, mas `cargo fmt --all --check` sai 1. Ver W-18 |
| 2 | 1 byte (`\n`) em `crates/ddc-core/src/lib.rs` | VERMELHA (sem `OK`) |
| 10 | `// TODO later` no fim de `input-notice.spec.mjs` | VERMELHA (sem `OK`) |
| 10 | `// TODO(#12) later` no mesmo lugar | `OK`, como deveria |

No HEAD, a busca por atributos com espaço depois de `#[` ou por `cfg(` em fim de linha em `crates`/`apps` não tem saída.

**Tempo de (f).** 0,32 s em 10/10 em repouso. Com 24 laços de CPU (cada um sob `timeout 75`), ficou entre 0,32 e 0,33 s em 20/20, todas passando. A folga até o limite de 0,5 s do Verify é de 0,18 s; era 0,23 s antes da 3ª parte. A 3ª parte mede cerca de 60 ms contra o teto de 160 ms. Os laços foram conferidos encerrados.

**Linha 9:** 3/3 execuções literais `OK` (5,70 / 5,60 / 5,61 s), mais 1 execução direta dos dois scripts com saída 0. A instabilidade W-16 não apareceu.

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração 4:
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` não tem diff desde a iteração 1. Fica herdada a evidência: clippy `-D warnings` para `x86_64-pc-windows-gnu`, com stubs, sai 0. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 e H2 seguem NÃO provadas, porque provar exigiria escrever `0x60` no RTK real, o que é vedado.
- **W-4 (worker único, baixo): PERSISTE.** Um write de `0x60` não confirmado segura o worker por até 3 s. Código do assentamento inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O arquivo se chama `DDC Control.desktop`, com espaço. O `Exec=` sai sem aspas e com espaço final (ver a linha 7: `Exec=…/autostart_entry-c0e56ec32fbab8ff `). `create_dir` não é recursivo, e `$XDG_CONFIG_HOME` é ignorado. O README documenta.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para qualquer diferença de read-back num slider.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js` duplica `VcpCode::INPUT_SOURCE`, entre linguagens.
- **W-8 (PLAN defasado, estilo): PERSISTE, menor.** `## Emendas` cobre a iteração 5, mas:
  - `## Files modified (all tasks)` não lista `ddc_hi_backend/tests.rs`, `retry/tests.rs` e `apps/ddc-tray/tests/e2e/support.mjs`;
  - T-1 fala em "5 testes";
  - T-4 fala em "≥ 2 chamadas".
- **W-9, W-10: TRATADOS** (iteração 3). **W-12, W-13: FECHADOS** (iteração 4).
- **W-11 (teste (f) no relógio real): PERSISTE, não flakou.** A 3ª parte levou o teste de 0,26 para 0,32 s. Sob 24 laços de CPU, ficou entre 0,32 e 0,33 s em 20/20. A folga é de 0,18 s até os 0,5 s do Verify, e de cerca de 100 ms no teto da 3ª parte. Num runner Windows com timer grosso a folga é menor, e o `rust-windows` mostra primeiro.
- **W-14 (tipo de commit `1ed653b`): PERSISTE, só histórico.** Some no squash-merge.
- **W-15: FECHADO.** `retry.rs` e `worker.rs` não têm mais código de teste embutido. M8x, M8y e M8t ficam vermelhas, e o texto da linha 8 agora é verdadeiro no HEAD. O resíduo sintático virou a W-18.
- **W-16 (linha 9, instabilidade ambiental): PERSISTE como risco, não reproduzida.** 3/3 literais + 1 direta OK. Se uma nova execução falhar com `the popup was hidden within 1.5 s of 'ddc-tray: popup shown'`, rodar de novo: é perda de foco no KWin real, já presente na base.
- **W-17: FECHADO.** O `## Files modified` do SUMMARY lista os 32 arquivos do diff da phase.

Novos na iteração 5:
- **W-18 (linha 8, resíduo sintático, baixo).** A regex de atributos de teste do Verify 8 (`#\[[[:alnum:]_:]*test\b` e `cfg(...)` na mesma linha) não vê duas formas válidas em Rust:
  - `#[ test ]`, com espaço depois de `#[` (M8z);
  - um `#[cfg(` quebrado em várias linhas (M8w).

  Nas duas sondas o Verify 8 dá `OK`, mas `cargo fmt --all --check` sai 1, então o Gate 4 (fmt é gate do reviewer, CONTEXT § DoD) as rejeita. No HEAD não há nenhuma das duas formas. Opcional: normalizar espaços antes da regex, ou rodar `rustfmt --check` nos 3 arquivos dentro do próprio Verify.
- **W-19 (linhas 3 e 4, lacuna objetiva, baixo para o usuário, provável achado do critic; D-5).** As linhas exigem "os dois nomes" por literais, mas nenhuma fixa QUAL é o mantido e qual é o pedido. As duas sondas abaixo deixam os Verify 3 e 4 `OK`:
  - **M4s** troca os papéis na 1ª frase do aviso de input, em `en.js:58` e `pt-BR.js:58`. O toast passaria a dizer "The monitor is still on HDMI-1, not DisplayPort-1", ou seja, que o monitor está na entrada em que NÃO está.
  - **M4g** troca os papéis no aviso genérico pt-BR (`pt-BR.js:56`, "manteve 76% em vez de 75%").

  Por que passam:
  - `expectToast` compara com o MESMO template sob teste;
  - `expectToastNaming` usa `toContainText` por nome, sem ordem;
  - na linha 3, só `en` do genérico tem igualdade literal (`'Brightness: the monitor kept 75% instead of 80%.'`), e o input só fixa `DisplayPort-2 may have no signal` / `pode estar sem sinal`.

  O HEAD está correto: `still on {kept}, not {asked}` e `continua em {kept}, não em {asked}`. Fechar exige asserção da frase literal com papéis. Por exemplo, no spec `toContainText('still on DisplayPort-1, not HDMI-1')`, `toContainText('continua em DisplayPort-1, não em HDMI-1')` e `toContainText('manteve 75% em vez de 76%')`, e na linha 3 o equivalente em pt-BR. Como o spec está congelado por SHA-256 na linha 4, isso pede emenda do DoD com nova D-XX.
- **Nota (ambiente, sem ação no código).** `/dev/i2c-1..5` e `/dev/i2c-9..15` passaram a ter ACL `user:slipalison:rw-` (07:31 de hoje); na iteração 4 eram `root 0600` sem ACL. Um teste futuro que alcance `DdcHiDisplays` falaria com o RTK real nesta máquina. Recomendação para as próximas rodadas (reviewer, critic, `/jdi-ship`): manter a leitura do 5.7 e rodar a prova por inotify em `bwrap` ANTES de qualquer execução nativa da suíte.
- **Nota (sem ação).** `.jdi/DECISIONS.md` é uma visão gerada, ignorada (`.gitignore:31`), e está defasada: não contém as D-2026-10-01-*. As fontes `D-2026-10-01-input-switch-autostart-1.md` e `-2.md` estão em `.jdi/decisions/`. `npx -y jdi-cli render` regenera a visão.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um sozinho, não ignorado, < 0,5 s; (b)/(e) com política CUSTOM; (e) via `write_budget_of`; (f) pelo `WorkerClient` real com limite de baixo e de cima para `0x60` e 3ª parte para outro código; (g) fixa `Default` e `without_backoff()`; (h) fiação; 3 arquivos de teste congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,38 s. O conjunto listado são os 7 nomes, cada um com `1 passed; 0 failed` em 0,00 s, exceto (f), com 0,32 s (10/10 em repouso; 20/20 entre 0,32 e 0,33 s sob 24 laços de CPU). (h) está listado e passa. `worker/tests.rs` `fadb5aaa…`, `ddc_hi_backend/tests.rs` `ec2d2980…` e `retry/tests.rs` `12ac10db…` batem. Mutações: M1 VERMELHA ((f), `tests.rs:1267`); M1 com o teste enfraquecido dá 76 verdes e Verify sem `OK` pelo SHA-256. |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,11 s. Dep normal direta = `thiserror`; `git diff --quiet` contra o merge-base sai 0; sem `sleep`. Sonda: 1 byte em `ddc-core/src/lib.rs` deixa o Verify sem `OK`. |
| 3 | View-model puro + 4 testes nomeados; `tests/ui` inteira verde | CONTEXT | Auto | PASS | `OK`, exit 0, 0,25 s. `# tests 165 / # pass 165 / # fail 0` (`npm run test:unit`, 88.48%). **Resíduo (W-19): M4s/M4g (papéis trocados) passam.** |
| 4 | Aviso visível de ponta a ponta: `input-notice.spec.mjs` com 5 testes que importam `test` de `support.mjs`; nomes conferidos por LITERAIS; anotação `axe` em exatamente 5 × 2 aprovados; 2 estados de pseudo-locale × 2 temas; suíte inteira verde; spec e `support.mjs` congelados por SHA-256 | CONTEXT | Auto | PASS | `OK`, exit 0, 19,3 s, com `npm ci --ignore-scripts` incluído. Releitura (Gate 7): `152 passed`, `6 skipped` (só `screenshots.spec.mjs`), 0 failed, 0 flaky; 10 `✓` em `input-notice.spec.mjs` e 4 `✓` nos 2 estados de pseudo-locale. Conjunto `axe` = as 10 entradas (`a = e`, pelo `OK`). SHA-256 `bdb21385…`/`ee930894…` batem. M4 (critic) VERMELHA: 4 ✘. **Resíduo (W-19): os papéis dos nomes não são fixados.** |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, fiação LINUX (ksni) e testes de `tray`; fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 1,90 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha com `0 ignored`. `src-tauri/src` sem diff desde `d18829d`. Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro com fake em memória (4 `autostart::tests::*`); chamador de PRODUÇÃO (`tray::flip_autostart`) imprime a linha do `crate::report` no stderr; `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 1,42 s. Os 4 testes listados passam. `a_refused_flip_leaves_the_entry_as_it_was --exact --nocapture` passa e imprime 2 vezes `ddc-tray: could not change the start-with-system entry: read-only home`. M6 (critic) VERMELHA: o teste segue verde, a linha some e o Verify sai sem `OK`. Que `run()` registra o plugin é provado pela parte B da linha 9 (mutação R herdada; `lib.rs` sem diff). |
| 7 | Entrada real no Linux com HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,50 s. Conjunto = pai + filho; `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` impresso; listagem real idêntica (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície: `DDC_HW_TESTS`/`/dev/i2c`/`DdcHiDisplays` em não-`.md` de `apps`/`crates` só nos 3 arquivos de produção; `DdcHiMonitorBackend` só neles e em `ddc_hi_backend/tests.rs`; nos 3, cada atributo de teste = `#[cfg(test)]` + `mod tests;`; capabilities idênticas; plugin no lock e ligado ao tray | CONTEXT | Auto | PASS | `OK`, exit 0, 0,20 s. Nos 3 arquivos, só `#[cfg(test)]`/`mod tests;` (`ddc_hi_backend.rs:136`, `worker.rs:552`, `retry.rs:179`). `capabilities/` só tem `default.json`, com permissões iguais às da base e sem `autostart`; `cargo tree -i tauri-plugin-autostart` resolve. M8x, M8y e M8t VERMELHAS. **Resíduo (W-18): `#[ test ]` e `#[cfg(` multilinha passam aqui, mas o `fmt --check` (Gate 4) os rejeita.** O "nenhum teste fala com monitor real", delegado ao 5.7, foi conferido por leitura e por inotify (zero aberturas). |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com `--fake`; `smoke-sni.sh` sem diff; partes A e B, monitor simulado exigido nas duas, marca segue o SO; `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, **3/3 execuções literais** (5,70 / 5,60 / 5,61 s). Parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor` e `smoke-sni: OK — PID 840566 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. Parte B: banner `… sandboxed HOME`, `the app serves the simulated monitor`, checkmark desmarcado; o 1º clique grava `…/DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray` e marca; o 2º remove e desmarca; `the item followed an entry created and removed behind the app's back`; `smoke-autostart: OK — …`. `smoke-sni.sh` com 0 bytes de diff. PID 24429 intacto, nada sobrando. W-16 não apareceu. |
| 10 | Sem `TODO`/`FIXME` sem issue nos não-Rust da phase; `en.js`/`pt-BR.js` só em comentário e sem `/*` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,01 s. Sondas no spec alterado: `// TODO later` VERMELHO e `// TODO(#12) later` `OK`. Os 6 casos vermelhos e os 2 verdes da iteração 4 ficam herdados, porque o Verify não mudou. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Verify literal -> `OK`, exit 0, 1,09 s. Soma: 413 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.25% (`3626 535 85.25%`), `COV_EXIT=0`. |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0, 0,01 s. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) tem `### Added` ("Start with system" no menu da bandeja, Linux e Windows) e `### Fixed` (linha 18); o último release é `## [0.1.0] - 2026-09-28` (linha 31); nenhum heading de release novo nesta phase; sem diff desde `d18829d`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o menu lista **Start with system** (linhas 330 e 334); o parágrafo "Input switch" (linha 412) descreve o assentamento; os smokes em `scripts/` são citados (linha 387); a limitação do `Exec=` com espaço está em "Known limitations of the tray app" (linha 399); sem diff desde `d18829d`; falta a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rodar `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers. Depois de confirmar as 2 linhas Manual, a phase pode seguir para `/jdi-ship`.

- **Achados do critic da iteração 4: todos fechados.** As mutações que os expunham foram reproduzidas aqui de forma independente e ficam vermelhas:
  - linha 1: M1, e o congelamento por SHA-256 pega o harness enfraquecido;
  - linha 4: M4;
  - linha 6: M6;
  - linha 8: M8x, M8y e M8t.
- **Candidato forte a achado da próxima rodada do critic: W-19** (linhas 3 e 4, objetivo). Com os papéis de mantida e pedida trocados no template, o toast diz o contrário do que o monitor fez, e os Verify 3 e 4 dão `OK` (M4s, M4g). Se o loop for seguir com critic, fechar antes:
  - uma asserção da frase literal com papéis no spec, para `en`, `pt-BR` e o genérico;
  - e/ou em `view-model.test.mjs`.

  Isso pede emenda do DoD (novo SHA-256 do spec) e nova D-XX.
- **W-18 é opcional.** O Gate 4 já cobre, por `fmt --check`.
- **W-16:** se uma nova execução da linha 9 falhar com `the popup was hidden within 1.5 s`, rodar de novo.
- **Ambiente:** com a ACL nova em `/dev/i2c-*`, rodar a prova por inotify em `bwrap` antes de qualquer execução nativa da suíte, nas próximas rodadas.
- **No PR:**
  - registrar o que foi observado no RTK real (W-3);
  - conferir o job `rust-windows` verde (W-2, e W-11 se o teste (f) flakar);
  - olhar o toast genérico de slider num monitor que arredonda (W-6).
- **Evidência:** não herdar esta evidência em iterações seguintes sem um novo diff.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 5; todo `cargo` em `bwrap` com os 16 `/dev/i2c-*` cobertos e observados por inotify, zero eventos). Linhas 2, 5, 6, 7, 9, 10, 11, 12 e 13: `hollow=false`. Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective). O texto afirma: "as constantes do passo e da janela são PRIVADAS de `retry.rs` e só alimentam o `Default`: `worker.rs` não as nomeia, então só a política manda". O Verify não confere nada disso. Mutação M1k, 3 linhas de produção:
  - `retry.rs:32` passa a `pub(super) const INPUT_SETTLE_WINDOW`;
  - `worker.rs` importa a constante;
  - `worker.rs:314` passa a `(self.clock.now() + settle.window.min(INPUT_SETTLE_WINDOW)).min(deadline)`.

  Resultado: Verify 1 `OK`, lib `76 passed`, fmt e clippy `-D warnings` saem 0. A mutação quebra a letra (a constante deixa de ser privada e `worker.rs` a nomeia) e o efeito: uma política com janela maior que 3 s seria cortada em silêncio. Sem efeito em produção, porque o `Default` é 3 s e nenhum teste usa janela maior que 1 s. É estrutural.
- DoD row «3 — view-model puro» (hollow, NÃO objetiva). W-19 reproduzida:
  - M4s inverte os papéis no template do input (`still on {asked}, not {kept}` / `continua em {asked}, não em {kept}`);
  - M4g inverte os papéis no genérico pt-BR (`manteve {asked} em vez de {kept}`).

  Com as duas, o Verify 3 dá `OK`. A letra da linha exige só que os dois nomes apareçam. O papel vem da D-5 e do sentido para o usuário: o toast diria que o monitor está na entrada em que NÃO está.
- DoD row «4 — aviso visível de ponta a ponta» (hollow, NÃO objetiva). M4s e M4g dão Verify 4 `OK`, com o Playwright completo. `expectToast` compara com o mesmo template, e `expectToastNaming` faz `toContainText` por nome, sem ordem. Mesma avaliação da linha 3.
- DoD row «8 — segurança de hardware e superfície» (objective). DOCTEST: o rustdoc roda os exemplos ```` ``` ```` de comentários `///` como testes, inclusive em itens privados, e o `ddc-adapters` não tem `doctest = false`. Mutação M8d: um exemplo de 5 linhas na doc de `impl Default for DdcHiBudgets` (`ddc_hi_backend.rs:53`). Resultado:
  - Verify 8 `OK`;
  - fmt e clippy saem 0;
  - `cargo test -p ddc-adapters --doc` roda `ddc_hi_backend::DdcHiBudgets::default (line 56) ... ok`.

  Há, portanto, código de teste executado por `cargo test --workspace` num arquivo excluído da varredura de tokens, contra a frase "NÃO têm código de teste embutido". A mesma classe vale para `#[cfg(doctest)]`, porque `\btest\b` não casa `doctest`. W-18 confirmada: `#[ test ]` e `#[cfg(` multilinha dão `OK` e o fmt os rejeita. Mas o próprio rustfmt GERA um `#[cfg(all(` multilinha estável quando o predicado passa de 100 colunas, e com ele o Verify 8 dá `OK` e o fmt sai 0. Fica não objetivo (predicado artificial; um helper sem uso cai no `dead_code`).

**Verdict:** BLOCKED
