# Phase 9: Review  (slug: input-switch-autostart)

**Verdict:** APPROVED_PENDING_MANUAL

> Iteração 2 do loop. Branch `phase/input-switch-autostart`, HEAD `c728e72`, base `origin/main` = merge-base `134b665`. Escrito pelo orquestrador a partir do resultado integral do reviewer, porque o harness nega escrita de `.md` ao subagente.
> Os 10 `Verify:` do CONTEXT.md e o `Verify:` do TODO do PROJECT.md foram extraídos por programa e rodados LITERALMENTE com `bash --noprofile --norc` (`LC_ALL=C.UTF-8`, `PATH` com `~/.cargo/bin`, `DISPLAY`/`WAYLAND_DISPLAY` do ambiente). Saídas em `/tmp/rv2/`.
> Nada da evidência da iteração 1 foi herdado para o código que mudou (`retry.rs`, `worker.rs`, `worker/tests.rs`, scripts, README). Única herança declarada: W-2 (Windows), porque `tray/notification_area.rs` não mudou na iteração 2 (`git diff 3fc7e11 HEAD` sem esse arquivo).
> O `ddc-tray` do usuário (PID 4884) ficou rodando e intacto do começo ao fim. Nenhum teste, script ou mutação escreveu em monitor real: o input `0x60` e Dangerous só passaram por fakes, e `#[ignore]`/`DDC_HW_TESTS` não foram usados.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` exit 0. |
| Tests | PASS | 412 passed, 0 failed, 9 ignored (hardware, não rodados). Iteração 1 tinha 410: +2 (os dois testes novos da D-14). Phase anterior: 386. |
| Coverage | PASS | `TOTAL 5577 832 85.08% 648 109 83.18% 3640 535 85.30% 0 0 -`, coluna Lines = **85.30%** (3640 linhas, 535 perdidas), exit 0, limiar 80%. `main.rs`/`build.rs` excluídos. UI (`npm run test:unit`): 1843/2083 = 88.48%. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` novo no diff (`git diff -U0` filtrado) nem fora de testes. |
| Hexagonal/Safety/Hygiene | PASS (com notas) | 5.1 a 5.11, ver abaixo. |
| Consistency | PASS (com warnings) | D-3..D-16 conformes; PLAN/SUMMARY defasados em pontos de bookkeeping (W-8, W-9, W-10). |
| UI Validation | PASS | `has_frontend: true` no PROJECT.md. Suíte Playwright do app: 152 passed, 6 skipped (só `screenshots.spec.mjs`, fora do diff), 0 failed, 0 flaky. Helpers compartilhados: zero console/pageerror e zero axe critical/serious. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto PASS (10 CONTEXT + 3 PROJECT), 2 manuais herdados pendentes. |

### Gate 5 em detalhe
- **5.1** `ddc-core [dependencies]` = só `thiserror`. Sem saída.
- **5.2 / 5.3a / 5.4a / 5.4b / 5.8** Sem saída.
- **5.3b** Um hit: `apps/ddc-tray/src-tauri/src/autostart.rs:55` (`pub trait AutostartEntry`).
  - Não é port do core: o core não tem conceito de autostart, e a trait é uma costura interna do adaptador de entrada (D-11).
  - Tem 2 impls (`PluginEntry` e `FakeEntry` de teste), então não é YAGNI. Só nota.
  - `PluginEntry::new` só é chamado em `lib.rs` (composition root) e no teste de integração.
- **5.5** `unsafe` só aparece em 2 comentários (`lib.rs:159`, `tests/autostart_entry.rs:6`).
  - O grep `-L` de crate roots sem `forbid(unsafe_code)` não listou nenhum dos 3; `[workspace.lints.rust] unsafe_code = "deny"` em `Cargo.toml:24`.
  - Sem `set_var`/`remove_var` em código. Sem `unsafe {` em `ddc-adapters`.
- **5.6** Varredura dos trechos de produção (antes do `#[cfg(test)]`) de `worker.rs`, `retry.rs`, `autostart.rs`, `tray.rs`, `menu.rs`, `status_item.rs`, `notification_area.rs`: nenhum `unwrap`/`expect`/`panic!`/`todo!`. Nenhum em `ddc-core`.
- **5.7** `Dangerous` existe no core. Sem `Confirm::Yes` fora de fronteira humana/teste.
  - `DdcHiMonitorBackend` em testes só em `crates/ddc-adapters/tests/real_monitor.rs`, onde os 7 testes têm `#[ignore = "... DDC_HW_TESTS=1"]`. O arquivo não está no diff.
  - No resto, só o composition root `lib.rs:79`.
  - Varredura de TODAS as linhas novas (qualquer extensão): nenhuma cita `DDC_HW_TESTS`/`DdcHiMonitorBackend`/`/dev/i2c`, exceto uma frase descritiva do composition root no README (`README.md:420`).
- **5.9** Nenhum `#[tauri::command]` não-async.
- **5.10** `cargo audit`: 1 warning permitido, `yoke-derive 0.8.3` yanked, já no `Cargo.lock` da base (não introduzido aqui). Pacotes novos no lock, sem advisory: `tauri-plugin-autostart 2.6.0`, `auto-launch 0.5.0`, `dirs 4.0.0`, `dirs-sys 0.3.7`, `redox_users 0.4.6`, `getrandom 0.2.17`, `winreg 0.10.1`.
- **5.11** Sem segredo. Sem TODO/FIXME sem issue em `*.rs`. Nos não-Rust, a linha 10 do DoD cobre (ver abaixo).

### Consistency em detalhe
- **Commits**
  - A iteração 2 tem 3 commits de código: `1077707` (fix, `worker.rs`, `retry.rs`, `worker/tests.rs`), `45829cf` (test, 4 scripts) e `3451169` (docs, README). Escopo = slug, tipo coerente.
  - Todos os cabeçalhos da branch <= 72 caracteres (maior: 69). `.jdi/` nunca no mesmo commit que código (D-12).
  - Zero byte de diff em `crates/ddc-core`, `crates/ddc-cli`, `.github/`, `capabilities/`, `tauri.conf.json`, `src-tauri/src/main.rs` e `scripts/smoke-sni.sh`.
- **Arquivos do PLAN**
  - Todos aparecem no log, menos `confirm.spec.mjs`, que era condicional e não mudou.
  - Os 5 scripts de `apps/ddc-tray/scripts/` não constam no `files_modified` do PLAN, que só lista 2 (W-8).
- **D-3 e D-14** Conformes.
  - Passo (250 ms) e janela (3 s) agora são o `Default` de `RetryPolicies.input_settle` (`retry.rs:21-29,100-110`), com a linha WHY preservada.
  - `write_budget(budgets, policies, code)` é pura e usada por `WorkerClient::write_vcp` e pelo teste.
  - `settle_input` lê passo e janela da política, limitados pelo `deadline`. Relógio injetado. Retorno sempre `Ok(())`. Sem mudança para outros códigos.
- **D-4 / D-7** `ddc-core`, `MonitorBackend`, `MonitorControl` e CLI sem diff (DoD 2).
- **D-5 / D-6** `readBackNotice` e `writeFailureText` puras (`view-model.js`), i18n `en` + `pt-BR`, sem string literal em JSX. `app.js` só chama as funções puras. Specs nos dois temas.
- **D-8** `tauri-plugin-autostart 2.6.0`, `autostart::register` é a única configuração e `run()` a chama (`lib.rs:113`). Sem capability nova (`capabilities/` só com `default.json`, permissões idênticas).
- **D-9** Estado = `is_enabled()`. Padrão desligado (o smoke lê o item como checkmark DESMARCADO num HOME novo). Só o alternar chama `set`.
- **D-10** `CheckmarkItem` (ksni) e `CheckMenuItem` (Windows), `MenuAction::Autostart`, rótulo em `Labels`, sem botão no popup.
- **D-11** Módulo `autostart`, `toggled()` puro, falha vai para `crate::report`, fake em memória `#[cfg(test)]`.
- **D-12** Nenhum teste escreve input em monitor real. Sem `unsafe`/`env::set_var`. Commits <= 72.
- **D-13** `smoke-sni.sh` com 0 linhas de diff.
- **D-15** Conforme no essencial: smoke novo, linha 5 nomeia os testes Linux, linha 5 não afirma a fiação Windows. Desvio de redação: o wrapper da parte A foi refatorado (W-10).
- **D-16** Linha 10 existe e não é vazia (ver linha 10 do checklist).

### Mutações reproduzidas pelo reviewer
Numa cópia descartável em `/tmp` (`git archive HEAD`, `CARGO_TARGET_DIR` próprio, já removida). Nada no repositório mudou. Isto confirma de forma independente a evidência do doer.

| Linha | Mutação | Resultado |
|---|---|---|
| 1 | `WorkerClient::write_vcp`: `write_budget(...)` -> `self.budgets.vcp` | VERMELHA: `input_write_through_the_client_outlives_the_vcp_budget FAILED` (`worker/tests.rs:1173`); `8 passed; 1 failed`. |
| 1 | `INPUT_SETTLE_WINDOW` = 1500 ms | VERMELHA: `input_write_default_settle_is_250_ms_steps_inside_a_3_s_window FAILED (left: 1.5s, right: 3s)`; `8 passed; 1 failed`. |
| 1 | revertidas | `9 passed; 0 failed` (as duas). |
| 5 | `status_item.rs:169` `checked,` -> `checked: false,` | VERMELHA: `the_menu_marks_start_with_system_by_the_os_entry_each_time_it_is_mounted FAILED`. |
| 5 | `status_item.rs:143` `menu()` sem ler o SO (`Autostart::Disabled`) | VERMELHA: o mesmo teste. |
| 5 | `tray.rs:98` braço `MenuAction::Autostart => {}` | VERMELHA: `a_click_on_the_start_with_system_item_flips_the_os_entry FAILED` (5,01 s). |
| 5 | revertidas | `23 passed; 0 failed`. |
| 9 | apagar `let builder = autostart::register(builder);` de `run()`, release | `smoke-autostart-private.sh` VERMELHO: `FAIL: exactly one .desktop entry after the first click did not happen within 5 s`, com `could not read/change the start-with-system entry: the autostart plugin is not registered` no stderr. O `smoke-sni-private.sh` com o MESMO binário segue `OK`, como o critic diagnosticou. |

## Blockers
- nenhum.

## Warnings
Reavaliação dos da iteração 1:
- **W-1 (README "128 of them"): RESOLVIDO.** `grep -n 128 README.md` sem saída; `README.md:397` agora só diz "runs its tests there (`tray/notification_area.rs`'s click tests included)" (commit `3451169`). Os dois smokes privados estão citados em `scripts/`.
- **W-2 (Windows não rodado): PERSISTE.** `tray/notification_area.rs` (`CheckMenuItem`, `set_checked` no clique direito) não compila nem roda no Linux e não mudou na iteração 2. A evidência da iteração 1 (clippy `-D warnings` sai 0 com `windres`/`ar` de stub) continua válida para este arquivo, mas não foi repetida agora. O comportamento só o job `rust-windows` do CI e a validação humana provam. A linha 5 do DoD já não o afirma (D-15), e o `## Deferred to PR review` o registra.
- **W-3 (hardware, deferido ao PR): PERSISTE.** H1 (leitura antes de assentar) e H2 (sem sinal, auto source volta) seguem NÃO provadas: provar exigiria escrever `0x60` no RTK real, vedado. README, CHANGELOG e as notas dizem "pode". O PR precisa registrar o observado: entrada com sinal, entrada sem sinal, logout/login com autostart ligado.
- **W-4 (comportamento, baixo): PERSISTE.** O worker é único para todos os monitores. Um write de `0x60` que o monitor não confirma o segura por até 3 s, e leituras de outro monitor nesse intervalo estouram `budgets.vcp` (1 s) com `Timeout`. D-14 só moveu os valores para a política, sem mudar o comportamento. Raro (entrada trocada com dois monitores e atualização concorrente).
- **W-5 (limites do `auto-launch 0.5.0`): PERSISTE.** Falhas são reportadas, sem pânico.
  - O arquivo sai como `DDC Control.desktop` (com espaço; o teste de integração, com o mock, gera `test.desktop`) e o `Exec=` termina com um espaço. O smoke compara o `Exec=` sem o espaço final, exigindo uma única linha `Exec=`. O nome com espaço é aceito pelo XDG e o ciclo ligar/desligar funcionou no smoke.
  - O `Exec=` sai sem aspas: caminho com espaço quebra (README "Known limitations" já diz).
  - O `enable` usa `create_dir` não recursivo (exige `~/.config`) e ignora `$XDG_CONFIG_HOME` (sempre `~/.config/autostart`).
- **W-6 (UX, observar no PR): PERSISTE.** O toast genérico aparece para QUALQUER diferença de read-back num slider. Um monitor que arredonda o brilho (pede 76, lê 75) o mostra a cada ajuste. É D-5 à risca.
- **W-7 (estilo): PERSISTE.** `INPUT_CODE = 0x60` em `view-model.js:16` duplica `VcpCode::INPUT_SOURCE` do core (nomeado, cross-language). Se houver um terceiro uso, ler do DTO.

Novos na iteração 2:
- **W-8 (plano, bookkeeping, baixo).** `PLAN.md` não foi emendado para D-14/-15/-16.
  - T-1 ainda diz "5 testes" e "retry.rs só se precisar", e o Execution fala em "9 Verify", enquanto o CONTEXT emendado tem 7 testes na linha 1 e 10 linhas de DoD.
  - `private-bus.sh`, `smoke-autostart-private.sh` e `sni-dbusmenu.py` não constam em nenhum `files_modified`.
  - O trabalho tem commits com o escopo da phase e está documentado em "Iteração 2" do SUMMARY, mas não virou tarefa T-9/T-10.
- **W-9 (SUMMARY defasado, baixo).** `SUMMARY.md` `## Ressalvas`, último item, ainda diz que "o README ainda diz '128 testes' do tray no Windows (não recontado)", o que é falso desde `3451169`. Remover.
- **W-10 (D-15, redação).** D-15(d) diz que "o wrapper da parte A fica como está", mas `smoke-sni-private.sh` foi refatorado para `source` o `private-bus.sh` (109 linhas alteradas).
  - Comportamento verificado igual: banner e linha `smoke-sni: OK` iguais; com `/bin/true` sai 1 com `FAIL: the app exited before registering ...`; o `smoke-sni.sh` segue com 0 linhas de diff.
  - A refatoração é justificada por DRY (evita copiar ~80 linhas de bus/watcher). Só registrar, ou emendar o texto da D-15.
- **W-11 (teste com relógio real, baixo).** `input_write_through_the_client_outlives_the_vcp_budget` roda no relógio do sistema (janela de 100 ms, orçamento de 160 ms, folga de 60 ms).
  - 60/60 execuções passaram em Linux com carga média ~41 em 24 núcleos.
  - Num runner Windows com timer grosso a folga é menor; se flakar, o `rust-windows` do CI mostra primeiro.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Assentamento no adapter: 7 testes nomeados (conjunto exato `input_write_*`/`writes_to_other_codes_*`), passam sozinhos, não ignorados, rápidos; política de assentamento RÁPIDA injetada no cliente | CONTEXT | Auto | PASS | `OK`, exit 0, ~1,1 s no total. O conjunto listado é exatamente os 7 nomes, cada um `1 passed; 0 failed` e `< 0,5 s`. Mutações reproduzidas (tabela acima): orçamento do cliente = `budgets.vcp` faz (f) falhar (`Timeout`); janela de 1,5 s faz (g) falhar (`left: 1.5s, right: 3s`). (a)-(e) só mudaram na aridade de `write_budget` e na origem das constantes (diff lido). |
| 2 | `ddc-core` intacto (0 byte desde a base), só `thiserror`, sem `sleep` | CONTEXT | Auto | PASS | `OK`, exit 0. Dep normal direta = `thiserror`; `git diff --quiet $(merge-base) -- crates/ddc-core` sai 0; sem `sleep` em `crates/ddc-core/src`. |
| 3 | View-model puro + 4 testes nomeados; `tests/ui` inteira verde | CONTEXT | Auto | PASS | `OK`, exit 0. `# tests 165 / # pass 165 / # fail 0 / # cancelled 0 / # skipped 0 / # todo 0`, e os 4 nomes com `ok`. |
| 4 | Aviso visível de ponta a ponta (Playwright, `input-notice.spec.mjs` com 5 testes x 2 temas, 2 estados de pseudo-locale x 2 temas, suíte inteira verde) | CONTEXT | Auto | PASS | `OK`, exit 0, 37 s com `npm ci --ignore-scripts` incluído. Releitura com `--reporter=list`: `152 passed`, `6 skipped` (só `screenshots.spec.mjs`, fora do diff), 0 failed, 0 flaky. Os 5 testes novos e os estados `rtk after the monitor kept the old input` / `rtk after an input write whose read-back failed` com `✓` em `[light]` e `[dark]`. Sem `disableRules`/`.exclude(`/`.include(`/`.options(`/`setLegacyMode` nem skip/fixme/only novos. |
| 5 | Menu nativo: rótulo en/pt-BR, item nas 2 plataformas com marca = estado do SO, `from_id`, fiação LINUX (ksni) e testes `tray`; fiação Windows NÃO afirmada | CONTEXT | Auto | PASS | `OK`, exit 0, ~2 s. Os 6 nomes existem em `--list` e passam; `menu:: i18n::` fecha `0 ignored`. Mutações reproduzidas: `checked: false`, `menu()` sem ler o SO e braço `Autostart => {}` ficam VERMELHOS nos testes nomeados (tabela). A fiação Windows fica em `## Deferred to PR review` (W-2). |
| 6 | Alternar e registro, fake em memória (4 `autostart::tests::*`); `run()` provado pelo smoke da linha 9 | CONTEXT | Auto | PASS | `OK`, exit 0. Os 4 testes listados e passando. Que `run()` chama `register` é provado pela linha 9 (mutação A reproduzida). |
| 7 | Entrada real no Linux, HOME em tempdir (processo filho), conjunto exato pai + filho, `Exec=` impresso, `~/.config/autostart` real idêntico | CONTEXT | Auto | PASS | `OK`, exit 0. O pai, com `--nocapture`, imprimiu o `.desktop` cru: `Exec=/home/slipalison/repos/ddc-control/target/debug/deps/autostart_entry-c0e56ec32fbab8ff ` (espaço final, W-5). Listagem do `~/.config/autostart` real com o mesmo sha256 antes e depois (`bf0a7045...`). |
| 8 | Segurança de hardware e superfície (sem tokens de hardware novos fora dos 3 arquivos do adaptador; capabilities idênticas; plugin no lock e ligado ao tray) | CONTEXT | Auto | PASS | `OK`, exit 0. `capabilities/` só com `default.json`, permissões idênticas às da base, sem `autostart`; `cargo tree -p ddc-tray -i tauri-plugin-autostart` resolve (2.6.0). |
| 9 | Release sobe na bandeja e o autostart funciona de ponta a ponta, em sessão D-Bus PRIVADA, com o `ddc-tray` do usuário rodando, `--fake`; parte A (`smoke-sni-private.sh`) + parte B (`smoke-autostart-private.sh`) | CONTEXT | Auto | PASS | `OK`, exit 0, 6,4 s (release já compilado). Parte B: `smoke-autostart-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME`, item "Start with system" checkmark desmarcado, 1º clique grava `.../DDC Control.desktop` com `Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray` e marca o item, 2º clique remove e desmarca, `smoke-autostart: OK — the Start with system item wrote the desktop entry on click and removed it on the next one`. Parte A: `smoke-sni: OK — PID 2857541 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. PID 4884 intacto, nenhum watcher/`dbus-run-session`/tempdir sobrando, `~/.config/autostart` real idêntico. Negativos: `/bin/true`, sem argumento e binário inexistente saem 1 nos dois wrappers; mutação A (sem `register`) fica VERMELHA na parte B e verde na A. |
| 10 | Sem `TODO`/`FIXME` sem issue nos arquivos NÃO-Rust da phase (`apps/ddc-tray/src`, `tests` sem `fixtures`, `scripts`), em comentário `//`, `#`, `/* */`, `<!-- -->` | CONTEXT | Auto | PASS | `OK`, exit 0. Não é vazia: o regex + STRIP marca 8 de 8 casos sintéticos (`// TODO: x`, `# fixme later`, `/* Todo */`, `<!-- FIXME -->`, etc.) e libera os 3 com `#N`; 42 arquivos rastreados varridos; zero hits antes do STRIP. |
| 11 | `cargo test --workspace` exit 0 | PROJECT | Auto | PASS | `cargo test --workspace --locked && echo OK` -> `OK`; soma 412 passed, 0 failed, 9 ignored (mesma saída do Gate 2). |
| 12 | Cobertura >= 80% das linhas | PROJECT | Auto | PASS | Gate 3: coluna Lines do TOTAL = 85.30% (`3640 535 85.30%`). |
| 13 | Sem `TODO`/`FIXME` sem issue (`*.rs`, `src/ crates/ apps/`) | PROJECT | Auto | PASS | Verify literal do PROJECT -> `OK`, exit 0. |
| 14 | CHANGELOG.md atualizado por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (linha 8) ganhou `Added` (Start with system) e `Fixed` (troca de entrada assenta até 3 s; o popup avisa); último release `## [0.1.0] - 2026-09-28` (linha 31); nenhum heading de release novo nesta phase. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: as 2 linhas falsas de autostart foram corrigidas (`README.md:330,333,334`), a limitação do `Exec=` sem aspas está em `:399`, o "128 of them" saiu (W-1 resolvido) e os smokes privados estão citados em `:387,:422`; falta só a leitura humana no diff do PR. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod input-switch-autostart` para confirmar as 2 linhas Manual herdadas (CHANGELOG/README). Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
Sem blockers: pode seguir para `/jdi-ship` depois de confirmar as 2 linhas Manual.

- Os 5 achados do critic da iteração 1 (linhas 1, 5, 6, 9, 12) estão corrigidos e as mutações que os expunham ficam vermelhas nos testes e smokes novos, reproduzidas aqui de forma independente.
- Antes do PR, corrigir o bookkeeping: W-9 (remover o "128 testes" do SUMMARY) e W-8 (emendar PLAN, ou registrar que a iteração 2 é das D-14/-15/-16).
- Decidir W-10: aceitar a refatoração do wrapper da parte A ou emendar a redação da D-15.
- No PR, registrar o que foi observado no RTK real (W-3), conferir o job `rust-windows` verde (W-2, e W-11 se o teste do cliente flakar) e olhar o toast genérico de slider em monitor que arredonda (W-6).
- Não herdar esta evidência em iterações seguintes sem novo diff de código.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 2). Linhas 2, 3, 4, 6, 7, 8, 11, 12 e 13: `hollow=false` (a mutação "orçamento do cliente = `budgets.vcp`" deixa (f) vermelho em 60/60 execuções, e o loop de (f) é real: `worker.rs:200` e `448` usam `SystemClock`). Linhas ocas:

- DoD row «1 — assentamento no adapter» (objective): falta provar que a política INJETADA manda na janela. Mutação M1, `worker.rs:194`: `budgets.vcp + policies.input_settle.window` vira `budgets.vcp + INPUT_SETTLE_WINDOW` (o código de antes da D-14): numa cópia em /tmp os 7 nomes do Verify passaram, 7/7, de forma determinística — (e) só chama `write_budget` com `RetryPolicies::default()` (`tests.rs:1131`) e em (f) um orçamento maior ainda passa. Mutação M2, `worker.rs:318`: `settle.window` vira `INPUT_SETTLE_WINDOW` no worker: os 7 passaram na primeira rodada e (f) sozinho passou em 26 de 40 execuções (o worker responde em `deadline+δ`, na corrida com o `recv_timeout` do cliente). Só a mutação no passo fica vermelha. Com o `Default` de produção o comportamento do usuário não muda; a lacuna é a garantia estrutural da D-14 de que cliente e worker leem a mesma janela.
- DoD row «9 — smokes do binário release» (objective): a linha afirma "sem escrita em monitor real (`--fake`)", mas nenhum Verify nem asserção do script exige o fake. Parte B: apagar `DDC_TRAY_FAKE=1` de `smoke-autostart-private.sh:155` não muda nenhuma checagem (banner, OK, `check_stderr`, `Exec=`, listagens); o script nunca exige o `SIMULATED_LINE` do app. Parte A: apagar `--fake` de `smoke-sni-private.sh:48` também passa: o `smoke-sni.sh` só checa o `SIMULATED_LINE` com `--fake` (`:280-283`), a linha final de OK (`:296`) é idêntica com ou sem fake, e o regex do Verify não casa a linha "the app serves the simulated monitor". Sem fake o app usa `DdcHiMonitorBackend` real (na parte A o popup abre e a UI lê o monitor real). Mutação não executada para não tocar o hardware; a conclusão vem da leitura dos scripts e do regex. Nenhuma escrita acontece (só o clique em "Start with system"), mas o cinto de segurança declarado fica sem prova.
- DoD row «10 — TODO/FIXME nos arquivos não-Rust» (objective): o regex exige o abridor de comentário na MESMA linha do marcador. ` * TODO: x` dentro de `/** ... */` ou `  FIXME: x` entre `<!--` e `-->` não casa. Reproduzido o pipeline do Verify literal num repositório descartável em /tmp, com `apps/ddc-tray/src/app.js` (JSDoc com ` * TODO: debounce this`) e `index.html` (`<!-- FIXME -->` em 3 linhas): saída `OK`. Os 8 casos sintéticos do reviewer eram todos de uma linha só.
- DoD row «5 — menu nativo» (objective=false, suspeita): apagar `fn menu_about_to_show(&mut self) {}` (`status_item.rs`, ~157) deixa os 6 testes nomeados verdes (eles chamam `item.menu()` direto, nenhum passa por `AboutToShow`). No ksni 0.3.6 o impl padrão marca `NO_ABOUT_TO_SHOW` e o menu deixa de ser remontado ao abrir (`service.rs:515-526`), então a marca fica velha se a entrada mudar fora do app. README:334 e CHANGELOG:12 prometem "lido a cada vez que o menu abre". O smoke da linha 9 também não pega, porque o `refresh` do clique mantém o cache fresco no fluxo ligar/desligar. Não marcado `objective`: o texto da linha 5 diz "a cada montagem do menu", não "ao abrir".

**Verdict:** BLOCKED
