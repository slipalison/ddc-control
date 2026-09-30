# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 1 do loop. Branch `phase/input-switch-autostart`, HEAD `949af41`, base `origin/main` = `134b665`. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
> Os 9 `Verify:` do CONTEXT.md foram extraídos por programa e rodados LITERALMENTE com `bash --noprofile --norc` (`LC_ALL=C.UTF-8`, `PATH` com `~/.cargo/bin`). Saídas em `/tmp/dod/`.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` exit 0. |
| Tests | PASS | 410 passed, 0 failed, 9 ignored (7 + 2 de hardware, não rodados). A phase anterior tinha 386 (+24). |
| Coverage | PASS | `TOTAL ... 3626 / 535 missed / 85.25%` (linha Lines, copiada do `cargo llvm-cov` real, exit 0). `main.rs` e `build.rs` excluídos; `main.rs` do tray não está no diff. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]` novo: os 2 `allow(clippy::panic)` de `worker/tests.rs:281,288` já existiam e têm `// reason:`. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. |
| Consistency | PASS | Plano e D-3..D-13 conformes, ver abaixo. |
| UI Validation | PASS | `has_frontend: true` no PROJECT.md. Gate 7 = suíte Playwright do app: 152 passed, 6 skipped, exit 0. |
| DoD | PASS_PENDING_MANUAL | 12/12 auto PASS (9 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`).
  - Não é port do core: o core não tem conceito de autostart, e a trait é uma costura interna do adaptador de entrada (D-11).
  - Tem 2 impls (`PluginEntry` de produção e `FakeEntry` de teste), então não é YAGNI. Só nota.
  - `PluginEntry::new` só é chamado em `lib.rs:122` (composition root) e no teste de integração.
  - Nada chama `enable()` fora de `PluginEntry::set`.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O forbid existe em `ddc-core/src/lib.rs:8`, `ddc-cli/src/main.rs:5` e `ddc-tray/src-tauri/src/lib.rs:8`, além de `[workspace.lints.rust] unsafe_code = "deny"`. Sem `set_var`/`remove_var` em código.
- **5.6** Todos os hits `unwrap/expect/panic` estão depois de `#[cfg(test)]` ou em comentário. Conferido arquivo a arquivo: `lib.rs`, `stop_signals.rs`, `tray.rs`, `kwin_placement.rs`, `status_item.rs`, `autostart.rs`. Nenhum em `ddc-core`.
- **5.7** `Dangerous` existe no core. Sem `Confirm::Yes` fora de fronteira humana. `DdcHiMonitorBackend` em testes só em `crates/ddc-adapters/tests/real_monitor.rs` (`#[ignore]` e `DDC_HW_TESTS`, arquivo fora do diff) e no composition root `lib.rs:79`.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit`: 1 warning permitido, `yoke-derive 0.8.3` yanked, já no lock da base (não introduzido aqui).
  - Deps novas: `tauri-plugin-autostart 2.6.0`, `auto-launch 0.5.0`, `dirs 4.0.0`, `dirs-sys 0.3.7`, `redox_users 0.4.6`, `getrandom 0.2.17`, `winreg 0.10.1`. Nenhuma tem advisory.
  - `tauri` com feature `test` como dev-dep não afeta os pacotes: o `tauri-cli-2.12.0` só lê `[dependencies]`/`[build-dependencies]`/target-deps (`interface/rust/manifest.rs`; dev-dependencies só aparecem na migração v1).
- **5.11** Sem segredo. Sem TODO/FIXME sem issue em `.rs`, nem nas linhas novas de js/mjs/sh/py.

### Consistency em detalhe
- **Commits**
  - As 8 tasks viraram 8 commits, com os títulos do PLAN (`ee33a55`, `d5ebff3`, `c2d1c5a`, `461a659`, `1c5e05a`, `5e94b60`, `a3111c2`, `d55cc60`).
  - Escopo = slug, tipos coerentes, todos os cabeçalhos <= 72 caracteres (maior: 69).
  - `.jdi/` nunca no mesmo commit que código (D-12).
  - Os 2 `chore(jdi)` e os `docs(...)` só de `.jdi/` são setup e bookkeeping do orquestrador.
- **Arquivos**
  - Todo `files_modified` do PLAN aparece no log.
  - `retry.rs` e `confirm.spec.mjs` eram condicionais e não mudaram.
  - `crates/ddc-core`, `crates/ddc-cli`, `.github/`, `capabilities/`, `tauri.conf.json` e `main.rs` do tray: 0 bytes de diff.
- **Tasks**
  - T-1..T-8 `completed`.
  - Testes: T-1 (9 no `worker/tests.rs`), T-2, T-3, T-4, T-5, T-6. T-8 tem o Verify 9 e a prova negativa, reexecutados abaixo. T-7 é doc pura, sem teste por plano.
- **D-3** Conforme.
  - `INPUT_SETTLE_STEP` (250 ms) e `INPUT_SETTLE_WINDOW` (3 s) têm linha WHY.
  - O polling só ocorre após write OK em `INPUT_SOURCE`, com UMA tentativa `isolated` por poll.
  - `Err`, valor errado ou eco de outro código seguem; pânico ou unsupported encerram; retorno sempre `Ok(())`.
  - A janela é limitada pelo `deadline`. `write_budget` é pura e usada por `WorkerClient::write_vcp` e pelo teste. O relógio é injetado (`self.clock`).
  - Nuance: a janela conta do fim do `transact` do write, não do início. Desvio desprezível, sem efeito no DoD.
- **D-4 / D-7** Core, `MonitorBackend`, `MonitorControl` e CLI sem diff. DoD 2 prova.
- **D-5 / D-6** `readBackNotice` e `writeFailureText` puras, i18n `en` + `pt-BR`, sem string literal. Specs Playwright nos dois temas.
- **D-8** `tauri-plugin-autostart 2.6.0`. `register()` é a única configuração do plugin e o `run()` passa por ela (`lib.rs:113`). Sem capability nova.
- **D-9** O estado vem de `is_enabled()`. O padrão é desligado, e só o alternar chama `set(Enabled)`.
- **D-10** `CheckmarkItem` (ksni) e `CheckMenuItem` (Windows), `MenuAction::Autostart` com id `autostart`, rótulo em `Labels`, sem botão no popup.
- **D-11** Módulo `autostart`, toggle puro, falha vai para `crate::report`, fake em memória `#[cfg(test)]`.
- **D-12** Nenhum teste ou linha nova fora dos 3 arquivos do adaptador cita `DDC_HW_TESTS`, `DdcHiMonitorBackend` ou `/dev/i2c` em `.rs`. Sem `allow(unsafe_code)` nem `env::set_var`.
- **D-13** O wrapper roda o `smoke-sni.sh` SEM editá-lo (diff de 0 linhas contra a base).

## Blockers
- nenhum.

## Warnings
- **W-1 (docs, baixo):** `README.md:397` diz que o tray roda "128 of them" testes no Windows. A contagem não foi refeita depois desta phase, que adicionou testes no tray: 152 na lib em Linux, e cerca de 12 novos valem também no Windows (autostart, i18n, menu, `tray.rs`, `notification_area`). Corrigir a contagem ou tirar o número.
- **W-2 (Windows não rodado):** o código só-Windows (`tray/notification_area.rs`: `CheckMenuItem`, `set_checked` no clique direito) nunca rodou num desktop Windows.
  - Evidência nova do reviewer: com `windres`/`ar` de stub só em `/tmp/stub` (nada no repo mudou), `cargo clippy -p ddc-tray --all-targets --locked --target x86_64-pc-windows-gnu -- -D warnings` sai 0. O código compila e está limpo de clippy.
  - O comportamento (o menu abre depois de `is_right_click`; o item reflete o estado do SO) só o `windows-latest` do CI e a validação humana provam.
- **W-3 (hardware, deferido ao PR):** H1 (leitura antes de assentar) e H2 (sem sinal, auto source volta) seguem NÃO provadas; provar exigiria escrever `0x60` no RTK real. A correção é testada só por fake em relógio virtual. O PR precisa registrar a observação no RTK (entrada com sinal, entrada sem sinal, logout/login).
- **W-4 (comportamento, baixo):** o worker é único para todos os monitores. Um write de `0x60` que o monitor não confirma segura o worker por até 3 s, e leituras de outro monitor nesse intervalo estouram o `budgets.vcp` (1 s) com `Timeout`. D-3 aceita os "~3 s a mais" só para a própria chamada, não para os outros monitores. Raro (entrada trocada com dois monitores e atualização concorrente).
- **W-5 (plugin, baixo):** limites do `auto-launch 0.5.0`. Falhas são reportadas, sem pânico.
  - O `Exec=` sai sem aspas (caminho com espaço quebra; README "Known limitations" já diz).
  - O `enable` usa `create_dir` não recursivo (exige `~/.config`).
  - Ignora `$XDG_CONFIG_HOME`: sempre grava `~/.config/autostart`.
- **W-6 (UX, observar no PR):** a notice genérica aparece para QUALQUER diferença de read-back num slider. Um monitor que arredonda o brilho (pede 76, lê 75) mostra o toast a cada ajuste. É D-5 à risca, e o flag + announce já existiam, mas pode incomodar.
- **W-7 (nota de estilo):** `INPUT_CODE = 0x60` em `view-model.js:16` duplica `VcpCode::INPUT_SOURCE` do core. O JS já usa `key === 'input'` em outros pontos, mas `writeFailureText` só recebe o `code`. Aceitável (nomeado, cross-language); se houver um terceiro uso, ler do DTO.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 5 testes nomeados (conjunto exato), passam sozinhos, relógio virtual < 1,5 s | CONTEXT | Auto | PASS | `OK`, exit 0. O conjunto `input_write_*`/`writes_to_other_codes_*` é exatamente os 5 nomes; cada um passou com `1 passed; 0 failed` em tempo de relógio virtual. As asserções (a) a (e) foram lidas no `worker/tests.rs` e cobrem o que o CONTEXT exige (sleeps só múltiplos de `INPUT_SETTLE_STEP`, `Ok(())` sempre, leituras `Err`/16/17, brilho sem leitura extra nem sleep, `write_budget` compartilhado). |
| 2 | `ddc-core` intacto, só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0. Dep normal direta = `thiserror`; `git diff --quiet $(merge-base) -- crates/ddc-core` sai 0; sem `sleep` em `crates/ddc-core/src`. |
| 3 | View-model puro + 4 testes nomeados; `tests/ui` verde | CONTEXT | Auto | PASS | `OK`, exit 0. `# tests 165 / # pass 165 / # fail 0 / # cancelled 0 / # skipped 0 / # todo 0`; os 4 testes nomeados com `ok`. |
| 4 | Aviso visível de ponta a ponta (Playwright, `input-notice.spec.mjs` com 5 testes, 2 estados de pseudo-locale, suíte inteira verde) | CONTEXT | Auto | PASS | `OK`, exit 0, em 22 s com `npm ci --ignore-scripts` incluído. Releitura com `--reporter=list`: `152 passed`, `6 skipped` (só `screenshots.spec.mjs:24/34/47` em `light` e `dark`, arquivo fora do diff), 0 failed, 0 flaky. Os 5 testes novos e os 2 estados `rtk after the monitor kept the old input` / `rtk after an input write whose read-back failed` com `✓` em `[light]` e `[dark]`. Sem `disableRules`/`.exclude(`/`.include(`/`.options(`/`setLegacyMode` nem `skip`/`fixme`/`only` novos. |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, testes atualizados | CONTEXT | Auto | PASS | `OK`, exit 0. Os 3 testes nomeados existem e passam; `menu:: i18n::` fecha `ok. N passed; 0 failed; 0 ignored`. |
| 6 | Alternar e registro com fake em memória (4 testes `autostart::tests::*`) | CONTEXT | Auto | PASS | `OK`, exit 0. Os 4 testes listados e passando, incluindo `the_app_builder_registers_the_autostart_plugin` (`try_state::<AutoLaunchManager>()` presente via `register(mock_builder())`). |
| 7 | Entrada real no Linux em HOME sandboxed (processo filho, pai + filho exatos) | CONTEXT | Auto | PASS | `OK`, exit 0. O pai, rodado com `--nocapture`, imprimiu o `.desktop` cru: `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff`. O hash do `ls -A ~/.config/autostart` real ficou idêntico antes e depois (`755bb58...`). O filho aborta se `HOME` não estiver sob o tempdir; sem `unsafe`/`set_var`. |
| 8 | Segurança de hardware e superfície (sem `DDC_HW_TESTS`/`DdcHiMonitorBackend`/`/dev/i2c` novos fora do adaptador; capabilities idênticas; plugin no lock e ligado ao tray) | CONTEXT | Auto | PASS | `OK`, exit 0. `capabilities/` só com `default.json`, permissões idênticas à base, sem `autostart`; `cargo tree -p ddc-tray -i tauri-plugin-autostart` resolve (2.6.0). |
| 9 | Release com o plugin sobe na bandeja, sessão D-Bus PRIVADA, com o `ddc-tray` do usuário rodando | CONTEXT | Auto | PASS | `OK`, exit 0, 22 s (build release + smoke). Saída real: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher` e `smoke-sni: OK — PID 2636030 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`, com `started ... with DDC_TRAY_FAKE=1`, `Activate` chamado e `popup still shown 1.5 s later`. PID 4884 (usuário) intacto antes e depois; `smoke-sni.sh` com 0 linhas de diff. Prova negativa: `smoke-sni-private.sh /bin/true` saiu 1 com `FAIL: the app exited before registering a tray item ...` e 0 ocorrências de `smoke-sni: OK`. |
| 10 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Gate 2: exit 0, 410 passed. |
| 11 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: TOTAL Lines 85.25%. |
| 12 | Sem `TODO`/`FIXME` sem issue | PROJECT | Auto | PASS | `OK`, exit 0 (Verify literal do PROJECT). |
| 13 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) ganhou `Added` (Start with system) e `Fixed` (troca de entrada assenta até 3 s); último release `## [0.1.0]`; nenhum heading de release novo nesta phase. |
| 14 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: as 2 linhas falsas de autostart foram corrigidas, e o item, o aviso e a limitação do `Exec=` foram documentados; ver W-1 ("128 of them" desatualizado em `README.md:397`). |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers: pode seguir para `/jdi-ship` depois de confirmar as 2 linhas Manual. Antes do PR:
- corrigir a contagem de testes do README (W-1);
- registrar no PR o que foi visto no RTK real (W-3): entrada com sinal, entrada sem sinal e logout/login com a entrada de autostart ligada;
- se possível, olhar o toast genérico de slider em monitor que arredonda (W-6).
As linhas 1 a 9 do DoD foram reexecutadas pelo reviewer; não herdar esta evidência para iterações seguintes sem novo diff de código.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 1). Linhas 2, 3, 4, 7, 8, 10 e 11: `hollow=false`. Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective): nenhum dos 5 testes passa pelo `WorkerClient::write_vcp`; o (e) chama `write_budget` direto (`worker/tests.rs:1123,1132`) e o fake só roda `Worker::write_vcp` com um deadline montado pelo teste. Mutação: `crates/ddc-adapters/src/ddc_hi_backend/worker.rs:538` `write_budget(&self.budgets, code)` -> `self.budgets.vcp`. O cliente volta a desistir em 1 s com `Timeout` no meio do assentamento e o Verify ainda imprime OK (o clippy pegaria o `dead_code` de `write_budget`, o Verify da linha não). O CONTEXT exige a função "usada pelo cliente E pelo teste" e só a metade do teste é provada. Suspeita sem linha própria: uma `INPUT_SETTLE_WINDOW` reduzida para 1,5 s deixa os 5 testes verdes (eles importam a constante); nenhum teste fixa o valor de D-3.
- DoD row «5 — menu nativo» (objective): o Verify só roda os 3 testes nomeados e o filtro `menu:: i18n::`, todos sobre a função pura `menu_entries`. Windows: `tray/notification_area.rs:112` (`*checked` -> `false`) ou remover `autostart_changed(app)` (linha 135) passa, pois o arquivo é `cfg(not(target_os = "linux"))` (`tray.rs:21`) e não compila nem é testado no Linux (W-2). Linux: `tray/status_item.rs:169` `checked,` -> `checked: false` também passa esta linha, porque o teste `status_item::tests::the_menu_marks_start_with_system_...` não casa com `menu::`/`i18n::`; só o Gate 2 o pegaria. "Marcado exatamente quando a entrada do SO existe nas duas plataformas" fica provado só no modelo puro.
- DoD row «6 — alternar e registro» (objective): `the_app_builder_registers_the_autostart_plugin` (`autostart.rs:270-277`) testa `register(mock_builder())` isolado e nenhum teste executa `run()`; o único uso em produção é `lib.rs:113`. Mutação: apagar essa linha, sem warning. Os 4 `autostart::tests::*` e o `tests/autostart_entry.rs` continuam passando (o último chama `register` ele mesmo). O CONTEXT pede "o builder que o `run()` usa deixa o plugin registrado" e esse elo não é provado.
- DoD row «9 — smoke do binário release» (objective): o smoke não afirma nada sobre o plugin nem sobre o menu. Sem o registro, `PluginEntry::with_manager` usa `try_state` (`autostart.rs:135`), então não há pânico; o app só imprime `could not read the start-with-system entry: the autostart plugin is not registered` (`lib.rs:241-243`). O `smoke-sni.sh` só procura `panicked` e linhas exatas e ninguém consulta o dbusmenu: o item é registrado, o painel abre, o binário segue vivo e `smoke-sni: OK` sai igual. Mesma causa da linha 6.
- DoD row «12 — sem TODO/FIXME sem issue (baseline do PROJECT.md)» (objective, gravidade baixa): o Verify varre só `--include='*.rs'`; `// TODO: x` em `apps/ddc-tray/src/app.js`, nos `.mjs`, em `smoke-sni-private.sh` ou em `fake-sni-watcher.py` passa, e esta phase adicionou quase só JS/sh/py fora de `.rs`. O reviewer compensou com varredura manual das linhas novas (estado atual limpo), mas a linha sozinha não prova a ausência fora de Rust.

**Verdict:** BLOCKED
