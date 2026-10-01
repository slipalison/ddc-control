# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 3 do loop. Branch `phase/input-switch-autostart`, HEAD `c5a0deb`, base `origin/main` = merge-base `134b665`. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
> Os 10 `Verify:` do CONTEXT.md e o `Verify:` do TODO do PROJECT.md foram extraídos por programa e rodados LITERALMENTE com `bash --noprofile --norc` (`LC_ALL=C.UTF-8`, `PATH` com `~/.cargo/bin`, `DISPLAY`/`WAYLAND_DISPLAY` do ambiente). A linha 4 rodou com `npm ci` e Playwright inteiro. Saídas em `/tmp/rv3/`.
> Nada da evidência da iteração 2 foi herdado para o código que mudou (`retry.rs`, `worker/tests.rs`, `smoke-autostart-private.sh`). Para o resto, a herança é só por diff: `git diff 49da47c HEAD -- . ':!.jdi'` lista exatamente esses 3 arquivos. Isso cobre `autostart.rs`, `lib.rs`, `menu.rs`, `tray/*`, `view-model.js`, `app.js`, i18n, README e CHANGELOG, que seguem byte a byte como na iteração 2.
> O `ddc-tray` do usuário (PID 4884) ficou rodando e intacto do começo ao fim. A listagem do `~/.config/autostart` real tem o mesmo sha256 antes e depois (`bf0a7045a8fb…`). Nenhum teste, script ou mutação escreveu em monitor real: o input `0x60` e Dangerous só passaram por fakes. `#[ignore]`, `--ignored` e `DDC_HW_TESTS` não foram usados. A mutação "sem `DDC_TRAY_FAKE`" rodou dentro de `bwrap` com os 16 `/dev/i2c-*` sobrepostos por `/dev/null` (conferido no namespace: `1:3`).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` exit 0. |
| Tests | PASS | 412 passed, 0 failed, 9 ignored (hardware, não rodados). Iteração 2: 412 (sem queda). Phase anterior: 386. |
| Coverage | PASS | `TOTAL 5577 832 85.08% 648 109 83.18% 3640 535 85.30% 0 0 -`, coluna Lines = **85.30%** (3640 linhas, 535 perdidas), `COV_EXIT=0`, limiar 80%. `main.rs`/`build.rs` excluídos. UI (`npm run test:unit`): 165 pass, `all files` 88.48%. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes (grep do gate sem saída). |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. |
| Consistency | PASS (com warnings) | D-3..D-18 conformes. W-8, W-9 e W-10 da iteração 2 tratados; avisos novos W-12 a W-14. |
| UI Validation | PASS | `has_frontend: true`. Suíte Playwright do app: 152 passed, 6 skipped (só `screenshots.spec.mjs:24/34/47` em `light` e `dark`, `test.skip` condicionado a `SCREENSHOTS=1`, arquivo fora do diff), 0 failed, 0 flaky. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`). Não é port do core: o core não tem conceito de autostart, e a trait é uma costura interna do adaptador de entrada (D-11). Tem 2 impls (`PluginEntry` e `FakeEntry` de teste), então não é YAGNI. Só nota. `PluginEntry::new` só é chamado no composition root (`lib.rs:122`) e no teste de integração.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`). O `-L` de raízes sem `forbid(unsafe_code)` não listou nenhuma; `[workspace.lints.rust] unsafe_code = "deny"` em `Cargo.toml:24`. Sem `unsafe {` em `ddc-adapters`.
- **5.6** Todos os hits de `unwrap/expect/panic` estão depois do `#[cfg(test)]` (conferido por número de linha em `autostart.rs`, `lib.rs`, `tray.rs`, `status_item.rs`) ou em comentário (`worker.rs:141`). Nenhum em `ddc-core`.
- **5.7** `Dangerous` existe no core. Sem `Confirm::Yes` fora de fronteira humana/teste. `DdcHiMonitorBackend` em testes só em `crates/ddc-adapters/tests/real_monitor.rs` (7 testes `#[ignore]` com `DDC_HW_TESTS`, arquivo fora do diff) e no composition root `lib.rs:79`.
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit`: 1 warning permitido, `yoke-derive 0.8.3` yanked, já no `Cargo.lock` da base. Deps novas da phase (`tauri-plugin-autostart 2.6.0`, `auto-launch 0.5.0`, `dirs 4.0.0`, `dirs-sys 0.3.7`, `redox_users 0.4.6`, `getrandom 0.2.17`, `winreg 0.10.1`) sem advisory.
- **5.11** Sem segredo. Nenhuma linha adicionada fora de `.jdi/` (qualquer extensão) contém `TODO`/`FIXME`/`to-do:`/`unimplemented!`.

### Consistency em detalhe
- **Commits novos desde a iteração 2** (`49da47c..HEAD`): `1ed653b` e `9bed052` (código), `c5a0deb` (`.jdi/`) e `2d86137` (`.jdi/`). Escopo = slug, cabeçalhos <= 69 caracteres, `.jdi/` nunca no mesmo commit que código (D-12). Ver W-14 sobre o tipo de `1ed653b`.
- **Zero byte de diff** contra a base em `crates/ddc-core`, `crates/ddc-cli`, `.github/`, `capabilities/`, `tauri.conf.json`, `src-tauri/src/main.rs` e `scripts/smoke-sni.sh`.
- **D-3 e D-14** Conformes. Passo (250 ms) e janela (3 s) vivem em `RetryPolicies.input_settle` (`retry.rs:111-114`). `write_budget(budgets, policies, code)` é pura e serve ao `WorkerClient::write_vcp` (`worker.rs:535`) e ao teste. `settle_input` lê passo e janela da política (`worker.rs:318-324`), limitados pelo `deadline`. Relógio injetado. Retorno sempre `Ok(())`. Outros códigos inalterados.
- **D-17(a)** Conforme. `INPUT_SETTLE_STEP`/`INPUT_SETTLE_WINDOW` são privadas de `retry.rs`; o `grep -rn INPUT_SETTLE crates apps` acha só `retry.rs:22,27,32,112,113`. Prova negativa: um `use super::retry::INPUT_SETTLE_WINDOW` em `worker.rs` dá `error[E0603]: constant INPUT_SETTLE_WINDOW is private`. Ver W-12 para a lacuna que a privacidade não fecha.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (linha 2).
- **D-5 / D-6** `readBackNotice` e `writeFailureText` puras (`view-model.js`), i18n `en` + `pt-BR`, sem string literal. `app.js` só chama as funções puras.
- **D-8** `tauri-plugin-autostart 2.6.0`. `autostart::register` é a única configuração e `run()` a chama (`lib.rs:113`). `capabilities/` só com `default.json`, permissões idênticas, sem `autostart`.
- **D-9** Estado = `is_enabled()`. Padrão desligado (a parte B do smoke lê o item como checkmark desmarcado num HOME novo). Só o alternar chama `set`.
- **D-10 / D-11** `CheckmarkItem` (ksni) e `CheckMenuItem` (Windows), `MenuAction::Autostart`, rótulo em `Labels`. Módulo `autostart` com `toggled()` puro; falha vai a `crate::report`; fake em memória `#[cfg(test)]`.
- **D-12** Nenhuma linha nova fora dos 3 arquivos do adaptador cita `DDC_HW_TESTS`, `DdcHiMonitorBackend` ou `/dev/i2c` em `.rs` (linha 8). Sem `unsafe` nem `env::set_var`.
- **D-13** `smoke-sni.sh` com 0 linhas de diff.
- **D-15 / D-18** Conformes. D-18 emenda o texto da D-15(d): o wrapper da parte A foi refatorado para `source` o `private-bus.sh`. A saída é igual (banner e linha OK idênticos; `/bin/true` sai 1).
- **D-16 / D-17(d)** Linha 10 existe e não é vazia (tabela de casos abaixo). Lacuna residual em W-13.

### Mutações reproduzidas pelo reviewer (cópia descartável em `/var/tmp`, `git archive HEAD`, `CARGO_TARGET_DIR` próprio; fontes restauradas e conferidas com `cmp`)
**Linha 1** (`cargo test -p ddc-adapters --locked --lib -- ddc_hi_backend::worker::tests`, 2 execuções cada, determinístico):

| Mutação | Resultado |
|---|---|
| baseline | `32 passed; 0 failed` (2/2) |
| M0 `WorkerClient::write_vcp`: `write_budget(...)` -> `self.budgets.vcp` | VERMELHA 2/2: só `input_write_through_the_client_outlives_the_vcp_budget FAILED` |
| M1 `write_budget`: `budgets.vcp + Duration::from_secs(3)` no lugar de `policies.input_settle.window` | VERMELHA 2/2: só `input_write_budget_covers_the_settle_window FAILED` |
| M2 `settle_input`: `Duration::from_secs(3)` no lugar de `settle.window` | VERMELHA 2/2: `..._budget_covers_the_settle_window` e `..._returns_ok_after_the_settle_window_when_the_monitor_keeps_the_old_value` falham nas duas; `..._through_the_client_...` falhou só na 1ª (corrida de tempo real), então só (b) e (e) contam |
| M3 `settle_input`: `Duration::from_millis(250)` no lugar de `settle.step` | VERMELHA 2/2: (b) e `..._through_the_client_...` |
| M4 `INPUT_SETTLE_WINDOW` = 1500 ms | VERMELHA 2/2: `input_write_default_settle_is_250_ms_steps_inside_a_3_s_window FAILED` |
| M7 worker lê `RetryPolicies::default().input_settle` em vez da própria política | VERMELHA 2/2: (b), (e) e (f) |
| M5 `worker.rs` nomeia a constante privada | NÃO COMPILA: `error[E0603]: constant INPUT_SETTLE_WINDOW is private` |
| **M8 cliente chama `write_budget(&self.budgets, &RetryPolicies::default(), code)`** | **VERDE 2/2: `32 passed; 0 failed`. Sobrevive. Ver W-12.** |
| revertidas | `32 passed; 0 failed` (2/2) |

**Linha 9** (binários release compilados na cópia; scripts do HEAD; 3 execuções cada):

| Binário / mutação | Resultado |
|---|---|
| base (cópia sem mutação) | `smoke-autostart-private.sh` OK 3/3, exit 0 |
| A: apagar `fn menu_about_to_show(&mut self) {}` de `tray/status_item.rs` | parte B VERMELHA 3/3, exit 1: `FAIL: the item marked for an entry created by hand did not happen within 5 s` (os dois cliques passam, só a checagem de "acompanha o SO" falha). `smoke-sni-private.sh` com o mesmo binário segue `smoke-sni: OK` |
| R: apagar `let builder = autostart::register(builder);` de `run()` | parte B VERMELHA 3/3, exit 1: `FAIL: exactly one .desktop entry after the first click did not happen within 5 s`, com `could not read/change the start-with-system entry: the autostart plugin is not registered` no stderr. Parte A segue OK |
| B1: `smoke-autostart-private.sh` sem `DDC_TRAY_FAKE=1` (binário real, em `bwrap` com `/dev/i2c-*` mascarados) | exit 1, ANTES de qualquer clique: `FAIL: the app did not say it serves the simulated monitor ('ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched')` |
| B2: `smoke-sni-private.sh` sem `--fake` (mesmo isolamento) | sai 0, mas a linha `smoke-sni: the app serves the simulated monitor` some; o regex da linha 9 do DoD NÃO casa, então a linha 9 falharia |

**Linha 10** (o `Verify:` literal, rodado em repositórios git descartáveis; 16 casos, 15 como esperado):
- VERMELHO, como deveria: ` * TODO: x` interno de `/** */`; `FIXME` interno de `<!-- -->`; `# fixme`; `/* Todo */`; `// TODO: x`; comentário em `src/i18n`; `TODO(#1)` seguido de `TODO` sem issue; `to-do:`; `"""TODO fix"""` em `.py`; `<p>TODO</p>` em `.html`.
- VERDE, como deveria: `TODO(#12)` em JSDoc; `# TODO #7`; "Todos os ajustes" em `src/i18n`; `tests/fixtures`; arquivo limpo.
- **Fora do esperado (W-13):** ` * FIXME later` interno de `/** */` dentro de `src/i18n` sai VERDE.

Negativas dos dois wrappers (parte A e parte B), todas exit 1 e sem a linha `OK —`: `/bin/true`, sem argumento, binário inexistente, binário que dorme 1,5 s e sai 0, `python3` sem `gi` (`FAIL: the stand-in watcher exited before answering`).

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração 2:
- **W-2 (Windows não rodado): PERSISTE, com evidência nova.** `tray/notification_area.rs` não mudou desde a iteração 1. Numa cópia descartável do HEAD, com `windres`/`ar` de stub só em `/tmp`, `cargo clippy -p ddc-tray --all-targets --locked --target x86_64-pc-windows-gnu -- -D warnings` sai 0. O código Windows compila e está limpo de clippy. O comportamento (o menu abre depois de `is_right_click`; o item reflete o estado do SO; a chave `HKCU\...\Run`) só o job `rust-windows` do CI e a validação humana provam. A linha 5 não o afirma (D-15) e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 (leitura antes de assentar) e H2 (sem sinal, auto source volta) seguem NÃO provadas: provar exigiria escrever `0x60` no RTK real, vedado. README, CHANGELOG e as notas dizem "pode". O PR precisa registrar o observado: entrada com sinal, entrada sem sinal, logout/login com autostart ligado.
- **W-4 (comportamento, baixo): PERSISTE.** O worker é único para todos os monitores. Um write de `0x60` que o monitor não confirma o segura por até 3 s, e leituras de outro monitor nesse intervalo estouram `budgets.vcp` (1 s) com `Timeout`. Raro (entrada trocada com dois monitores e atualização concorrente). Código inalterado.
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** O arquivo sai como `DDC Control.desktop` (espaço no nome) e o `Exec=` termina com um espaço; sem aspas (caminho com espaço quebra, README "Known limitations" já diz); `create_dir` não recursivo (exige `~/.config`); ignora `$XDG_CONFIG_HOME`. Falhas são reportadas, sem pânico. Código inalterado.
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para QUALQUER diferença de read-back num slider. Um monitor que arredonda o brilho (pede 76, lê 75) o mostra a cada ajuste. É D-5 à risca. Código inalterado.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js:16` duplica `VcpCode::INPUT_SOURCE` do core (nomeado, cross-language). Se houver um terceiro uso, ler do DTO.
- **W-8 (PLAN defasado): TRATADO.** `PLAN.md` ganhou `## Emendas` (T-1 com `RetryPolicies.input_settle`, 7 testes e `retry.rs`; T-8/D-15 com os 5 scripts em `files_modified`; 10 linhas de DoD). Resta só estilo: o corpo da T-1 ainda cita "5 testes" e a assinatura antiga `write_budget(&DdcHiBudgets, VcpCode)`; a seção de Emendas cobre.
- **W-9 (SUMMARY com "128 testes"): TRATADO.** `grep -n 128 README.md` sem saída; o SUMMARY diz agora que o "128" saiu na iteração 2.
- **W-10 (D-15(d)): TRATADO.** D-18 emenda a redação; comportamento do wrapper verificado igual.
- **W-11 (teste (f) em relógio real): PERSISTE, não flakou.** `input_write_through_the_client_outlives_the_vcp_budget` leva 0,10 s (5 execuções). Também passou 40/40 com 30 laços ocupados (loadavg ~23 em 24 núcleos). A folga é de 60 ms (janela 100 ms, `budgets.vcp` 60 ms); num runner Windows com timer grosso é menor, e o `rust-windows` do CI mostra primeiro.

Novos na iteração 3:
- **W-12 (linha 1, lacuna residual, objetiva, baixo).** A mutação M8, `write_budget(&self.budgets, &RetryPolicies::default(), code)` em `WorkerClient::write_vcp` (`worker.rs:535`), deixa os 32 testes do módulo verdes (2/2). O texto de (e) no CONTEXT diz "um cliente que usasse a janela de `Default` falha aqui", mas isso só vale para um `write_budget` que ignorasse o argumento `policies` (M1, vermelha). Um cliente que passe a política errada ao `write_budget` não é visto: (e) chama `write_budget` direto com a política custom, e em (f) um orçamento MAIOR (3 s + `vcp`) também devolve `Ok(())`. Em produção a política é sempre a `Default`, então o comportamento do usuário não muda. Para fechar: extrair a escolha para um método do cliente (por exemplo `fn write_budget_of(&self, code)`, usado por `write_vcp`) e afirmar `vcp + janela custom` num cliente com política custom, sem dormir; ou estreitar o texto de (e) para "um `write_budget` que ignorasse a política".
- **W-13 (linha 10, lacuna residual, objetiva, baixo).** A 4ª alternativa do regex `C` (`^[^:]*:[0-9]+:[[:space:]]*\*`) só pode casar na 2ª etapa do pipe (saída do `git grep`, com prefixo `arquivo:linha:`). Na 1ª etapa sobre `src/i18n` o `git grep` roda `C` sobre o conteúdo cru, sem prefixo, então ` * TODO`/` * FIXME` interno de bloco `/** */` em `src/i18n` passa (caso reproduzido acima). Hoje nenhum arquivo de tradução tocado (`en.js`, `pt-BR.js`) tem comentário de bloco, e `guard.js` (que tem JSDoc) não está no diff; nenhuma linha adicionada pela phase contém o marcador. Para fechar: trocar a 4ª alternativa por `^[[:space:]]*\*` na parte aplicada ao `git grep` de `src/i18n`.
- **W-14 (tipo de commit, baixo).** `1ed653b` é `test(...)` mas altera produção em `retry.rs` (tira `pub(crate)` das duas constantes). Seria `refactor`/`fix`. Sem efeito funcional; só histórico.
- Nota (sem ação): `.jdi/DECISIONS.md` é uma visão gerada e não rastreada (`jdi-cli render`) e ainda não lista a D-18; a fonte é `.jdi/decisions/D-...-18.md`, presente.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), cada um passa sozinho, não ignorado, < 0,5 s; (b) e (e) com política CUSTOM; (f) pelo `WorkerClient` real; (g) fixa o `Default`; constantes privadas de `retry.rs` | CONTEXT | Auto | PASS | `OK`, exit 0, 1,5 s. Conjunto listado = os 7 nomes; cada um `1 passed; 0 failed`, 0,00 s (o do cliente 0,10 s). Mutações reproduzidas (tabela): M0 (f), M1 (e), M2 (b)+(e), M3 (b), M4 (g), M7 (b)+(e)+(f) ficam VERMELHAS; M5 não compila. **Residual: M8 (cliente com `RetryPolicies::default()`) sobrevive, ver W-12.** |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0, 0,15 s. Dep normal direta = `thiserror`; `git diff --quiet $(merge-base) -- crates/ddc-core` sai 0; sem `sleep` em `crates/ddc-core/src`. |
| 3 | View-model puro + 4 testes nomeados; `tests/ui` inteira verde | CONTEXT | Auto | PASS | `OK`, exit 0, 0,4 s. `# tests 165 / # pass 165 / # fail 0 / # cancelled 0 / # skipped 0 / # todo 0`, e os 4 nomes com `ok`. |
| 4 | Aviso visível de ponta a ponta (Playwright, `input-notice.spec.mjs` com 5 testes x 2 temas, 2 estados de pseudo-locale x 2 temas, suíte inteira verde) | CONTEXT | Auto | PASS | `OK`, exit 0, 43,5 s com `npm ci --ignore-scripts` incluído. Releitura com `--reporter=list`: `152 passed`, `6 skipped` (só `screenshots.spec.mjs`, fora do diff), 0 failed, 0 flaky. 10 `✓` em `input-notice.spec.mjs` (5 x `[light]`/`[dark]`) e 4 `✓` nos estados `rtk after the monitor kept the old input` / `rtk after an input write whose read-back failed`. Sem `disableRules`/`.exclude(`/`.include(`/`.options(`/`setLegacyMode` nem skip/fixme/only novos. |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, fiação LINUX (ksni) e testes `tray`; fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, 3,0 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha `0 ignored`. Código inalterado desde a iteração 2 (diff). Fiação Windows em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro, fake em memória (4 `autostart::tests::*`); `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0, 2,1 s. Os 4 testes listados e passando. Que `run()` chama `register` é provado pela linha 9: mutação R reproduzida (parte B vermelha, parte A verde). |
| 7 | Entrada real no Linux, HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0, 0,8 s. Conjunto = pai + filho; `Exec=.*autostart_entry-` impresso; listagem real do `~/.config/autostart` idêntica antes e depois (`bf0a7045a8fb…`). |
| 8 | Segurança de hardware e superfície (sem tokens de hardware novos fora dos 3 arquivos do adaptador; capabilities idênticas; plugin no lock e ligado ao tray) | CONTEXT | Auto | PASS | `OK`, exit 0, 0,25 s. `capabilities/` só com `default.json`, permissões idênticas às da base, sem `autostart`; `cargo tree -p ddc-tray -i tauri-plugin-autostart` resolve (2.6.0). `worker/tests.rs` (alterado na iteração 3) não adiciona token. |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, sessão D-Bus PRIVADA, com o `ddc-tray` do usuário rodando, `--fake`; parte A (`smoke-sni-private.sh`) + parte B (`smoke-autostart-private.sh`), monitor simulado exigido nas duas, marca segue o SO por mudança à mão | CONTEXT | Auto | PASS | `OK`, exit 0, 6,8 s (release já compilado). Saída real da parte A: `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher`, `smoke-sni: the app serves the simulated monitor`, `smoke-sni: OK — PID 3164773 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. Parte B: `smoke-autostart-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME`, `smoke-autostart: the app serves the simulated monitor`, item checkmark desmarcado, 1º clique grava `.../DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray` e marca, 2º clique remove e desmarca, `the item followed an entry created and removed behind the app's back`, `smoke-autostart: OK — the Start with system item wrote the desktop entry on click and removed it on the next one`. PID 4884 intacto, nenhum watcher/`dbus-run-session`/tempdir sobrando, `~/.config/autostart` real idêntico. Mutações A, R, B1 e B2 reproduzidas (tabela), todas vermelhas; negativas dos dois wrappers saem 1. |
| 10 | Sem `TODO`/`FIXME` sem issue nos arquivos NÃO-Rust da phase (`apps/ddc-tray/src`, `tests` sem `fixtures`, `scripts`), em qualquer ponto da linha, inclusive linhas internas de `/** */` e `<!-- -->`; `src/i18n` só em comentário | CONTEXT | Auto | PASS | `OK`, exit 0, 0,02 s. Não é vazia: 15 de 16 casos descartáveis como esperado, incluindo ` * TODO` interno de `/** */` e `FIXME` interno de `<!-- -->` (VERMELHO). **Residual: ` * FIXME` interno de bloco dentro de `src/i18n` passa, ver W-13** (nenhum caso real hoje). Nenhuma linha adicionada pela phase fora de `.jdi/` tem o marcador. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | Gate 2: `TEST_EXIT=0`, soma 412 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.30% (`3640 535 85.30%`). |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou `Added` (Start with system) e `Fixed` (troca de entrada assenta até 3 s; o popup avisa); último release `## [0.1.0] - 2026-09-28`; nenhum heading de release novo nesta phase. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: as 2 linhas falsas de autostart foram corrigidas, a limitação do `Exec=` sem aspas está documentada, o "128 of them" saiu (`grep 128 README.md` sem saída) e os smokes privados estão citados; falta só a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers: pode seguir para `/jdi-ship` depois de confirmar as 2 linhas Manual.

- Os achados da segunda rodada do critic (linhas 1, 9, 10; suspeita da 5) estão corrigidos e as mutações que os expunham ficam vermelhas, reproduzidas aqui de forma independente.
- W-8, W-9 e W-10 estão tratados.
- Lacunas residuais, não bloqueantes pelo `Verify:` literal, mas candidatas a achado do critic: W-12 (linha 1, M8) e W-13 (linha 10, bloco interno em `src/i18n`). Se o loop quiser fechá-las, cada uma é uma mudança pequena: um método do cliente para a escolha do orçamento com teste, e uma alternativa de regex.
- No PR, registrar o que foi observado no RTK real (W-3), conferir o job `rust-windows` verde (W-2, e W-11 se o teste do cliente flakar) e olhar o toast genérico de slider em monitor que arredonda (W-6).
- Não herdar esta evidência em iterações seguintes sem novo diff de código.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 3). Linhas 2, 3, 5, 6, 7, 11, 12 e 13: `hollow=false`. Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective, três achados). (i) W-12 CONFIRMADA: `worker.rs:535` `write_budget(&self.budgets, &self.policies, code)` -> `write_budget(&self.budgets, &RetryPolicies::default(), code)` deixa `32 passed; 0 failed` ((e) chama `write_budget` direto, `tests.rs:1195`; (f) usa `no_backoff()` e só afirma `Ok` e `waited >= janela`, `tests.rs:1232-1233`, então um orçamento MAIOR passa). Nuance: a forma literal deixa `WorkerClient.policies` sem leitura e o clippy `-D warnings` a derruba, mas a variante com `.field("policies", &self.policies)` no `Debug` (`worker.rs:511-516`) fica verde nos testes e no clippy. Não muda o comportamento em produção (`new()` passa `Default`); garantia estrutural. (ii) NOVO, o mais grave, MUDA o comportamento do usuário: na fiação de produção, `ddc_hi_backend.rs:95-99`, `RetryPolicies::default()` -> `RetryPolicies { input_settle: InputSettle { step: 250ms, window: Duration::ZERO }, ..RetryPolicies::default() }` deixa os 50 testes `ddc_hi_backend::` verdes: nenhum teste não-ignorado olha a política que `DdcHiMonitorBackend::new()` entrega; a fiação real só é exercida em `crates/ddc-adapters/tests/real_monitor.rs` (`#[ignore]`), em `compose_osd` (`lib.rs:79`) e `cli/main.rs:28`, e os smokes usam `--fake`. O assentamento some no app real e o bug original volta com tudo verde. (iii) O Verify é mais frouxo que a frase: o texto diz "nenhum dorme de verdade mais que 0,2 s", o Verify exige `finished in < 0.5`; `retry.rs:135` `window: from_millis(100)` -> `400` passa ((f) `finished in 0.40s`; (g) só afirma `no_backoff().window < default().window`, `tests.rs:1249`). Estrutural.
- DoD row «4 — aviso visível de ponta a ponta» (objective): contagem em vez de conjunto — o Verify exige `grep -c 'expectAccessible(page)' $S` >= 2 mas o spec tem 5 (`input-notice.spec.mjs:70,81,91,106,127`); apagar 3 das 5 chamadas deixa 2, o `-ge 2` passa e o Playwright segue verde. O texto promete axe nos 5 testes. Menor: o Verify também não checa que o spec importa `test` de `support.mjs` (o coletor de console/pageerror vive nesse `base.extend`). Estrutural.
- DoD row «8 — segurança de hardware e superfície» (objective): o texto diz "nenhuma linha nova ... menciona `DDC_HW_TESTS`, `DdcHiMonitorBackend` ou `/dev/i2c`", o Verify só varre `git diff -U0 ... -- '*.rs'`. Anexar `export DDC_HW_TESTS=1  # /dev/i2c-4 real monitor` a `scripts/smoke-autostart-private.sh` deixa o fragmento do Verify em `ok=1` (reproduzido num clone descartável); scripts `.sh`/`.py`/`.mjs` ficam fora do alcance e as outras linhas não os pegam. Hoje sem ocorrência real. Estrutural.
- DoD row «9 — smokes do binário release» (objective=false, suspeita): "o `smoke-sni.sh` segue SEM edição" está só na parenteses do Verify, sem `git diff --quiet $b -- apps/ddc-tray/scripts/smoke-sni.sh`; a parte A delega todas as asserções a esse script e o Verify só casa linhas que o próprio script imprime, então um `smoke-sni.sh` enfraquecido que imprima as mesmas linhas passaria. Hoje 0 bytes de diff (Gate 6, fora do Verify). As partes B e os wrappers estão sólidos (A, R, B1 e B2 confirmadas; `sni-dbusmenu.py` faz `AboutToShow` e depois `GetLayout`; `FORBIDDEN_STDERR` bate com `report()`, `lib.rs:242`).
- DoD row «10 — TODO/FIXME nos arquivos não-Rust» (objective): W-13 CONFIRMADA e mais larga: em `src/i18n/en.js`, ` * FIXME later` dentro de `/** */` dá OK; também dá OK uma continuação de `/* ... */` SEM asterisco (`   TODO later`), que o conserto sugerido (`^[[:space:]]*\*`) não cobriria. Escopo: `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md` e `src-tauri/Cargo.toml` são não-Rust que a phase toca e nenhum Verify varre (hoje sem marcador). Controles certos: `// see #12 TODO` e `TODO: and see #12` ficam RED; `TODO:#12 TODO #13 FIXME(#14)` fica OK. Estrutural.

Severidade apontada pelo critic: o item (ii) da linha 1 é o único que muda o comportamento do usuário; o resto é garantia estrutural do DoD.

**Verdict:** BLOCKED
