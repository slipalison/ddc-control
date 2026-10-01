# Phase 9: Input switch fix and autostart — Summary  (slug: input-switch-autostart)

**Status:** complete (iteração 1 do loop: T-1..T-8; iterações 2 a 5: correção dos achados do critic)
**Tasks:** 8/8 complete, 0 blocked

Escrito pelo orquestrador a partir do relatório do doer: o harness recusou a escrita de `SUMMARY.md` por um subagente. As saídas brutas do doer ficaram em `/tmp/t/` (`t1-red.txt`, `t2-red.txt`, `t4-red.txt`, `smoke-evidence.txt`, `cov.txt`).

## Executed tasks
- T-1 (`ee33a55`): após write OK em `0x60`, `settle_input` dorme `INPUT_SETTLE_STEP` (250 ms) e faz UMA leitura `isolated` (sem a `RetryPolicy` de 3), até ler o valor, esgotar `INPUT_SETTLE_WINDOW` (3 s contados do write) ou o `deadline`. Leitura com erro, valor errado ou eco de outro código segue o polling; pânico ou "unsupported" encerram. Sempre `Ok(())`. `write_budget(&DdcHiBudgets, VcpCode)` é pura e serve ao `WorkerClient` e ao teste. `ddc-core` e `retry.rs` intactos.
- T-2 (`d5ebff3`): `readBackNotice` e `writeFailureText` puras no view-model, chaves i18n `en` + `pt-BR`.
- T-3 (`c2d1c5a`): módulo `autostart` e `tauri-plugin-autostart` 2.6.0 com `Cargo.lock`; `register(builder)` é a única configuração do plugin e o `run()` passa por ela; `PluginEntry` usa `try_state` (sem pânico se o plugin faltar); `capabilities/` intacto.
- T-4 (`461a659`): `app.js` (`showReadBack`/`writeFailed`), `input-notice.spec.mjs` e 2 estados novos em `pseudo-locale.spec.mjs`.
- T-5 (`1c5e05a`): item "Start with system" nos menus Linux (ksni `CheckmarkItem`, remontado por `menu_about_to_show`) e Windows (`CheckMenuItem`, `set_checked` com o estado real do SO).
- T-6 (`5e94b60`): `tests/autostart_entry.rs`, só Linux, pai + filho com `HOME` em tempdir.
- T-7 (`a3111c2`): README, CHANGELOG `[Unreleased]` e `docs/hardware-validation.md`.
- T-8 (`d55cc60`, status no PLAN `e70ec5f`): `apps/ddc-tray/scripts/smoke-sni-private.sh` + `fake-sni-watcher.py` (D-13). O wrapper se re-executa sob `dbus-run-session --config-file=<tmp>` (config mínimo, sem ativação de serviços: com o padrão, o GTK ativaria `xdg-desktop-portal` e o `xdg-document-portal` montaria FUSE no mesmo `/run/user/1000/doc` da sessão real), sobe o watcher de stand-in (espera até 5 s que responda `ProtocolVersion`), imprime a linha `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher` e roda o `smoke-sni.sh --fake --activate` SEM editá-lo (diff de 0 linhas), propagando o código de saída; trap mata o watcher; uma guarda recusa o passo interno na sessão real (`SMOKE_SNI_OUTER_BUS`).
- Status das tasks T-1..T-7 no PLAN: `36152e9`.

## Blocked tasks
- _(none)_

## Testes vistos falhando antes (T-1, T-2, T-4)
- T-1 (com `write_budget` estubado, sem polling): `input_write_returns_ok_after_the_settle_window_when_the_monitor_keeps_the_old_value FAILED (left: 0ns, right: 3s)`; `..._survives_reads_that_fail_or_lie_while_settling FAILED (left: 1, right: 7)`; `..._returns_once_the_monitor_reads_back_the_value FAILED (left: [Write], right: [Write, Read, Read, Read])`; `input_write_budget_covers_the_settle_window FAILED (left: 1s, right: 4s)`; `writes_to_other_codes_do_not_settle` passa antes do fix por natureza (guarda). `test result: FAILED. 23 passed; 7 failed`.
- T-2: `SyntaxError: ... does not provide an export named 'readBackNotice'`; `not ok 1 - tests/ui/view-model.test.mjs`.
- T-4 (specs novos contra o `app.js` antigo, projeto `light`): `5 failed, 31 passed` — os dois `the toast names the kept and the asked input (en|pt-BR)`, `a failed read after an input write ...`, `a setting other than the input ... generic notice` e o estado `rtk after the monitor kept the old input`.

## Verify do DoD (bash, `LC_ALL=C.UTF-8`; sob zsh o `$N` da linha 1 não divide palavras)
1. OK — conjunto `input_write_*`/`writes_to_other_codes_*` = os 5 nomes; cada um passa sozinho em `0.00s`.
2. OK — `ddc-core` só com `thiserror`, sem diff e sem `sleep`.
3. OK — `# fail 0`, `# cancelled 0`, `# skipped 0`, `# todo 0`; os 4 testes nomeados `ok`.
4. OK, sem o `npm ci` (para não apagar o `node_modules` da máquina). Playwright inteiro: `152 passed`, `6 skipped` (só `screenshots.spec.mjs`); os 5 testes novos e os 2 estados de pseudo-locale com `✓` em `light` e `dark`.
5. OK — os 3 testes nomeados; `menu:: i18n::` fecha com `0 ignored`.
6. OK — os 4 `autostart::tests::*`.
7. OK — o arquivo lista exatamente pai e filho; o pai imprimiu o `.desktop` cru com `Exec=.../target/debug/deps/autostart_entry-<hash>`; `ls -A ~/.config/autostart` real idêntico antes e depois.
8. OK — nenhuma menção nova a `DDC_HW_TESTS`, `DdcHiMonitorBackend` ou `/dev/i2c` fora dos 3 arquivos do adapter; `capabilities/` só com `default.json`, sem `autostart`; `cargo tree -i tauri-plugin-autostart` resolve.
9. (Iteração 1, antes da T-8) na sessão real NÃO rodou: há um `ddc-tray` do usuário rodando (PID 4884, `~/.local/bin/ddc-tray`) e a single-instance faz o binário do smoke sair (`the app exited before registering ... is another instance already running?`). Passou numa sessão D-Bus privada com um watcher de stand-in: `smoke-sni: OK — PID 2450397 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`. Isso gerou a D-2026-09-30-input-switch-autostart-13 e a task T-8; com ela a linha 9 foi reescrita e passa (abaixo).

## T-8 — linha 9 do DoD, rodada literalmente (bash, `LC_ALL=C.UTF-8`, `cargo build -p ddc-tray --release --locked -q` antes), com o `ddc-tray` do usuário (PID 4884) rodando e sem encerrá-lo
Antes: `user bus: unix:path=/run/user/1000/bus`; o barramento privado era outro. Resultado `OK`, `exit=0`:
```
smoke-sni-private: private session bus, stand-in StatusNotifierWatcher
smoke-sni: started target/release/ddc-tray as PID 2580669 with DDC_TRAY_FAKE=1
smoke-sni: org.kde.StatusNotifierWatcher lists org.kde.StatusNotifierItem-2580669-1, owned by PID 2580669
smoke-sni: the app serves the simulated monitor
smoke-sni: called org.kde.StatusNotifierItem.Activate on org.kde.StatusNotifierItem-2580669-1/StatusNotifierItem
smoke-sni: the app printed 'ddc-tray: popup shown' after Activate
smoke-sni: the popup was still shown 1.5 s later
smoke-sni: OK — PID 2580669 registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked
```
Depois: `pgrep -a ddc-tray` só com o PID 4884; nenhum watcher, `dbus-run-session`, `dbus-daemon` ou portal ficou para trás.
Prova negativa: com `/bin/true` no lugar do `ddc-tray` (e com um script que faz `sleep 1` e sai) o wrapper sai 1 e não imprime `smoke-sni: OK` (`smoke-sni: FAIL: the app exited before registering a tray item with status 0 — is another instance already running?`). Outros caminhos de falha exercitados: sem `dbus-run-session`/`busctl`/`python3` (PATH restrito), `gi` ausente (simulado), watcher mudo (`did not answer ... within 5 s`, em 5,13 s), watcher que morre cedo, argumento ausente/inexistente, passo interno na sessão real.
Ressalvas: `shellcheck` não está instalado (usou `bash -n` e `py_compile`); `register_object` do Gio é deprecado (a troca exige GLib >= 2.84), mantido com filtro de warning pontual e comentário; o stand-in não trata `StatusNotifierItemUnregistered` (o smoke não consulta e cada execução sobe um watcher novo). `cargo test --workspace --locked` passou.

## Gates
- `cargo fmt --all --check` e `clippy --workspace --all-targets --locked -- -D warnings`: limpos.
- `cargo test --workspace --locked`: 410 passed, 0 failed, 9 ignored (os `#[ignore]` de hardware).
- Cross-check Linux de `ddc-core`, `ddc-adapters`, `ddc-cli`: verde.
- Cobertura Rust real (`--fail-under-lines 80`, ignore `main|build.rs`): 85,25% de linhas, exit 0. UI com `node --test`: 165 passed, 88,48%.

## Iteração 2 — correção dos achados do critic (D-14, D-15, D-16)
Commits: `1077707` (D-14), `45829cf` (D-15), `3451169` (W-1 do reviewer). Mensagens no log da branch.

**D-14 / linha 1 (`1077707`).** `RetryPolicies` ganhou `input_settle: InputSettle { step, window }` (`Default` 250 ms / 3 s; `without_backoff()` de teste = 5 ms / 100 ms); `write_budget(budgets, policies, code)` recebe a política; `WorkerClient` guarda as `policies` e `settle_input` lê passo e janela dela. Testes novos, nomes da linha 1: `input_write_through_the_client_outlives_the_vcp_budget` (cliente REAL, `budgets.vcp` 60 ms, janela 100 ms, monitor que mantém o valor antigo: `Ok(())` e `elapsed >= janela`) e `input_write_default_settle_is_250_ms_steps_inside_a_3_s_window` (fixa o `Default` por literais). O conjunto `input_write_*`/`writes_to_other_codes_*` agora tem 7 nomes.
- Mutação 1 (`WorkerClient::write_vcp` com `self.budgets.vcp` no lugar de `write_budget(...)`), vermelha: `panicked at .../worker/tests.rs:1172:5: left: Err(Timeout) right: Ok(())` e, na corrida do cliente, `60.068921ms < 100ms` (`FAILED. 0 passed; 1 failed`). Revertida byte a byte.
- Mutação 2 (`INPUT_SETTLE_WINDOW` = 1500 ms), vermelha: `input_write_default_settle_is_250_ms_steps_inside_a_3_s_window FAILED (left: 1.5s, right: 3s)`; os outros 6 seguem verdes. Revertida.
- Estabilidade: 15 execuções da lib `ddc-adapters` (75 testes) e 10 do conjunto `input_write*` sob 8 processos de CPU, todas verdes; o teste do cliente leva ~0,10 s.

**D-15 / linhas 5, 6, 9 (`45829cf`).** `scripts/private-bus.sh` (lib para `source`: config do bus sem ativação de serviços, `run_on_private_bus`, `start_watcher`/`stop_watcher`, `require_tools`); `smoke-sni-private.sh` passou a usá-la com saída idêntica à anterior (comparada com PID normalizado nos casos OK, `/bin/true`, sem argumento e binário inexistente; `smoke-sni.sh` com 0 linhas de diff); `scripts/sni-dbusmenu.py` (Gio: `item`, `state`, `click` sobre `com.canonical.dbusmenu`); `scripts/smoke-autostart-private.sh` (sessão privada + watcher de stand-in + `HOME` em tempdir com `.config` + `DDC_TRAY_FAKE=1`, `LC_ALL`/`LANG=en_US.UTF-8`). Saída real (PID 2749669), com o `ddc-tray` do usuário (PID 4884) rodando e intacto:
```
smoke-autostart-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME
smoke-autostart: started target/release/ddc-tray as PID 2749669 with DDC_TRAY_FAKE=1 and HOME=/tmp/smoke-autostart-home.wNHqRY
smoke-autostart: org.kde.StatusNotifierWatcher lists org.kde.StatusNotifierItem-2749669-1/StatusNotifierItem, owned by PID 2749669
smoke-autostart: 'Start with system' is an unmarked checkmark and the autostart directory is empty
smoke-autostart: /tmp/smoke-autostart-home.wNHqRY/.config/autostart/DDC Control.desktop has Exec=/home/slipalison/repos/ddc-control/target/release/ddc-tray
smoke-autostart: the first click wrote the entry and the item is marked
smoke-autostart: the second click removed the entry and the item is unmarked
smoke-autostart: OK — the Start with system item wrote the desktop entry on click and removed it on the next one
```
Seis execuções seguidas saíram 0, sem tempdir sobrando. Saídas ≠ 0: binário que sai cedo (`/bin/true`: `FAIL: the app exited before registering a tray item with status 0`), sem argumento, dois argumentos, binário inexistente, `python3-gobject` ausente (`FAIL: python3 cannot load the gi module with Gio 2.0`), `busctl` ausente.
- Mutação (apagar `let builder = autostart::register(builder);` de `run()`, rebuild release), vermelha no smoke NOVO: `smoke-autostart-private: FAIL: exactly one .desktop entry after the first click did not happen within 5 s`, com `ddc-tray: could not read the start-with-system entry: the autostart plugin is not registered` e `could not change ...`, exit 1; o `smoke-sni-private.sh` com o mesmo binário mutado seguia dando `smoke-sni: OK` (confirma o diagnóstico do critic). `lib.rs` revertido, sem diff.
- Linha 5 Linux: `status_item.rs` `checked,` -> `checked: false,` faz `tray::status_item::tests::the_menu_marks_start_with_system_by_the_os_entry_each_time_it_is_mounted` falhar (`left: [("Start with system", false)] right: [("Start with system", true)]`). Revertido. Os 6 nomes da linha 5 existem com esses caminhos em `--list`.
- Ressalva do plugin: o arquivo gravado se chama `DDC Control.desktop` (com espaço) e o `Exec=` termina com um espaço (o `auto-launch` reserva lugar para args); o smoke compara o valor de `Exec=` sem o espaço final, exigindo uma única linha `Exec=`.

**D-16 / linha 10.** Rodada literalmente: `OK`; os arquivos novos não trazem marcador.
**W-1 (`3451169`).** O "128 of them" saiu do README (não verificável daqui: falta o linker mingw) e o README passou a citar os smokes em `scripts/`.

**Verify do DoD (iteração 2), bash `--noprofile --norc`, `LC_ALL=C.UTF-8`:** linhas 1 a 10 `OK` (linha 9 ~5 s com release compilado; linha 4 21 s). Gates: fmt e clippy `-D warnings` limpos; cross-check Linux verde; `cargo test --workspace --locked` 412 passed, 0 failed, 9 ignored; `cargo llvm-cov` TOTAL linhas 85,30% (3640 linhas, 535 perdidas). Nenhum teste/script escreveu em monitor real; `~/.config/autostart` real com o mesmo hash antes e depois.

## Iteração 3 — segunda rodada do critic (D-17)
Commits só de código: `1ed653b` `test(input-switch-autostart): settle tests read only the policy` (linha 1) e `9bed052` `test(input-switch-autostart): smoke proves fake and OS-fed mark` (linha 9). A linha 10 não precisou de commit.

**Linha 1 (D-17a).** `INPUT_SETTLE_STEP`/`INPUT_SETTLE_WINDOW` perderam o `pub(crate)` e só o `Default` de `RetryPolicies` as usa; `worker.rs` nunca as importou (só lê `policies.input_settle`; sem diff) e `worker/tests.rs` deixou de importá-las. (b) e (e) rodam com uma política CUSTOM (passo 100 ms, janela 1 s, ambos ≠ `Default`; `custom_settle_policies()` afirma isso): (b) afirma `settle.step * reads == settle.window`, tempo virtual == janela custom e todos os sleeps == passo custom; (e) afirma `budgets.vcp + janela custom` para `0x60` e `budgets.vcp` para o resto, com `budgets(400 ms)` para que `vcp < janela` no braço "cortado". (g) segue fixando o `Default` por literais (250 ms / 3 s). Os 7 nomes não mudaram.
Mutações provisórias em `worker.rs` (scripts em `/tmp/it3/mut.sh`), cada uma 5x, determinísticas, revertidas byte a byte (`cmp`, `sha256sum -c`, `git diff` limpo):

| Mutação | Resultado (5/5) |
|---|---|
| M1: `budgets.vcp + Duration::from_secs(3)` em `write_budget` | `input_write_budget_covers_the_settle_window FAILED` (`tests.rs:1199`: `left: 3.4s`, `right: 1.4s`); os outros 6 passam |
| M2: `Duration::from_secs(3)` no lugar de `settle.window` em `settle_input` | (b) FAILED (`left: 2s`, `right: 1s`) e (e) FAILED (`left: 1.4s`, `right: 1s`) em 5/5; (f) falhou junto em 2 das 5 (`Err(Timeout)` contra `Ok(())`), então só (b) e (e) contam |
| M3: `left.min(Duration::from_millis(250))` no lugar de `left.min(settle.step)` | (b) FAILED (`left: 400ms`, `right: 1s`) e (f) FAILED (`tests.rs:1235`) em 5/5 |

**Linha 9 (D-17b e c).** `smoke-autostart-private.sh`: `require_simulated_monitor` roda logo depois de `find_item` e antes de qualquer clique, exige a `SIMULATED_LINE` (a de `smoke-sni.sh`) no stderr e imprime `smoke-autostart: the app serves the simulated monitor`; `check_the_item_follows_the_os`, depois do 2º clique, cria o `.desktop` à mão no `HOME` de tempdir (com o nome que o 1º clique gravou, `DDC Control.desktop`), relê o menu (`AboutToShow` + `GetLayout`) e exige MARCADO, remove o arquivo e exige DESMARCADO, e imprime `smoke-autostart: the item followed an entry created and removed behind the app's back` antes do OK. `smoke-sni-private.sh` já passa `--fake` e a linha 9 do DoD casa `smoke-sni: the app serves the simulated monitor`. Saída real (release recém-compilado):
```
smoke-autostart-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME
smoke-autostart: the app serves the simulated monitor
smoke-autostart: 'Start with system' is an unmarked checkmark and the autostart directory is empty
smoke-autostart: the first click wrote the entry and the item is marked
smoke-autostart: the second click removed the entry and the item is unmarked
smoke-autostart: the item followed an entry created and removed behind the app's back
smoke-autostart: OK — the Start with system item wrote the desktop entry on click and removed it on the next one
```
- Mutação A (apagar `fn menu_about_to_show(&mut self) {}` de `tray/status_item.rs`, rebuild release): o smoke novo falha 3/3 — `smoke-autostart-private: FAIL: the item marked for an entry created by hand did not happen within 5 s` (exit 1); o `smoke-sni-private.sh` com o mesmo binário segue `OK`. Revertido byte a byte, release recompilado.
- Mutação B (tirar `DDC_TRAY_FAKE=1` do app, em cópias em `/tmp/it3/mutB/`, DENTRO de `bwrap` com os 16 `/dev/i2c-*` sobrepostos por `/dev/null`, conferido no namespace e fora dele — nenhum monitor real alcançável): o script ANTIGO saía 0 com `OK` (o buraco era real); o NOVO sai 1 3/3, antes de qualquer clique: `FAIL: the app did not say it serves the simulated monitor ('ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched')`. `smoke-sni-private.sh` sem `--fake` (no mesmo isolamento) sai 0, mas a linha `smoke-sni: the app serves the simulated monitor` some e o regex da linha 9 não casa.

**Linha 10 (D-17d).** `OK` (0,0 s); o comando literal em repositórios descartáveis acerta 11 de 11 casos (vermelho: ` * TODO: x` em `/** */`, `FIXME` interno de `<!-- -->`, `# fixme`, `/* Todo */`, `// TODO:`, comentário em `src/i18n`, `TODO(#1)` seguido de `TODO` sem issue; OK: `TODO(#12)` em JSDoc, `# TODO #7`, "Todos os ajustes" em `src/i18n`, arquivo limpo).

**Verify do DoD (iteração 3), bash `--noprofile --norc`, `LC_ALL=C.UTF-8`, ambiente gráfico mantido:** linhas 1 a 10 `OK` (1: 1,9 s; 4: 22,2 s; 9: 5,8 s) e a linha do PROJECT.md (TODO em `*.rs`) `OK`. Gates por commit: fmt, clippy `-D warnings`, `cargo test --workspace --locked` (412 passed, 0 failed, 9 ignored) e cross-check Linux, todos exit 0. `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex "(main|build)\.rs$"`: `COV_EXIT=0`, `TOTAL ... 3640 535 85.30%` (Lines 85,30%). A suíte não escreveu em monitor real; o `ddc-tray` do usuário (PID 4884) seguiu rodando e o `~/.config/autostart` real ficou idêntico.
Ressalvas: o teste (f) segue no relógio do sistema (W-11) — a detecção determinística de M2 vem de (b) e (e), em relógio virtual; a mutação B só rodou com `/dev/i2c-*` mascarado; o app com o backend real poderia enumerar monitores antes do clique (por isso o isolamento).

## Iteração 4 — terceira rodada do critic e pré-crítica (D-19, D-2026-10-01-1)
Retomada pelo `/jdi-issue` em 2026-10-01 ("continue o desenvolvimento"). Base `134b665`, HEAD `9a02a69`. Os 3 primeiros commits corrigem os achados da 3ª rodada do critic (REVIEW.md da iteração 3). Os 3 últimos fecham as lacunas que a D-2026-10-01-input-switch-autostart-1 previu para a 4ª rodada, nos rows 1, 4 e 8, emendados em `433852c`. Escrito pelo orquestrador a partir do relatório do doer.

**Commits.**
- `1ed13b0` test: teste (h), `ddc_hi_backend::tests::the_real_backend_hands_the_default_retry_policies_to_its_client`. O `DdcHiMonitorBackend::new()`, com e sem `with_budgets`, entrega `RetryPolicies::default()` ao seu cliente. É a fiação de produção que a 3ª rodada achou descoberta.
- `53f18e9` refactor: `WorkerClient::write_budget_of(&self, code)`, chamado por `write_vcp`. O teste (e) o chama num cliente com política custom, sem dormir.
- `4f17fb0` test: (g) fixa por literais também os 5 ms / 100 ms de `RetryPolicies::without_backoff()`.
- `da95bda` e `433852c` docs: DoD apertado, na 3ª rodada do critic e na pré-crítica.
- `1b847ff` test (row 1 (f)): `input_write_through_the_client_outlives_the_vcp_budget` ganha uma 2ª parte, sem mudar o nome. Um 2º `WorkerClient` real, com as mesmas `budgets(vcp)` e `no_backoff()`, escreve `0x60` num display `Behaviour::Block(gate)`. O resultado tem de ser `Err(DdcError::Timeout)` com `elapsed >= vcp + window` e `elapsed < vcp + window + 1 s`. O gate só abre depois da medição.
- `4c149fc` test (row 4): `expectAccessible` (`support.mjs`) anota o teste com `axe` DEPOIS do `expect(blocking, ...).toEqual([])`.
- `9a02a69` test (row 8): `production_backends()` sai de `ddc_hi_backend.rs` e vai para `ddc_hi_backend/tests.rs`, com o mesmo corpo. As únicas linhas `cfg(test)` que a phase acrescenta aos 3 arquivos de produção são as do acessor `policies()` de `worker.rs`.

**Mutações (cada uma revertida e conferida byte a byte).**
- Row 1 (f): `write_vcp` com `let budget = write_budget(&self.budgets, &RetryPolicies::default(), code);` e `write_budget_of` mantido vivo. Fica VERMELHO: `panicked at .../worker/tests.rs:1251:5: 3.060067524s >= 160ms + 1 s`, `FAILED. 0 passed; 1 failed ... finished in 3.16s`. O módulo dá `50 passed; 1 failed` e o Verify do row 1 sai sem `OK`. Revertida, o teste roda em `finished in 0.26s` (3 execuções).
- Row 4: `await expectAccessible(page);` comentado em `an input read back as asked shows no notice`. O `grep -c` antigo continuava em 5. O Verify literal sai sem `OK`: o jq dá `a has 8 entries, e has 10`, sem `light`/`dark › an input read back as asked shows no notice`, embora os 10 testes passem. No Playwright 1.63.0 a anotação de runtime aparece em `tests[].annotations` e em `results[].annotations`, e cada projeto vira um `spec` com 1 teste. O jq trata os dois casos.
- Row 8: antes do move o Verify dava `exit=1`; depois, `OK`. Mutação da fiação (`window: Duration::ZERO` em `DdcHiMonitorBackend::new()`): VERMELHA, `left: ... InputSettle { step: 250ms, window: 0ns }` contra `right: ... window: 3s`. Prova negativa: um `#[cfg(test)] fn helper() {}` em `ddc_hi_backend.rs` faz o row 8 dar `exit=1`.

**Gates (HEAD `9a02a69`).**
- `cargo fmt --all --check`, `clippy --workspace --all-targets --locked -- -D warnings` e o cross-check Linux saem exit 0.
- `cargo test --workspace --locked`: 413 passed, 0 failed, 9 ignored (eram 412; o +1 é o teste (h)).
- `npm run test:unit`: 165 pass, cobertura 88,48%. Playwright: 152 passed, 6 skipped (`screenshots.spec.mjs`).
- `cargo llvm-cov --workspace --locked --fail-under-lines 80 --ignore-filename-regex '(main|build)\.rs$' --summary-only`: exit 0, `TOTAL ... 3642 535 85.31%`.

**Verify do DoD.** Os 10 rodaram literalmente (`env -i`, `bash --noprofile --norc`, `LC_ALL=C.UTF-8`, ambiente gráfico e D-Bus do usuário) e todos saíram `OK`. Tempos: row 1 2 s, row 4 18 s, row 9 22 s.
Nenhum teste, script ou mutação escreveu em monitor real, e não foram usados `--ignored` nem `DDC_HW_TESTS`. O `ddc-tray` do usuário seguiu rodando e o `~/.config/autostart` real ficou com o mesmo sha256.
Ressalvas: o limite de baixo de (f) depende de `recv_timeout` nunca expirar antes do prazo, o que a std garante; a folga de tempo é 0,26 s contra o limite de 0,5 s. Os 3 últimos commits saíram sem a linha `Claude-Session:` da atribuição, que chegou depois.

## Iteração 5 — quarta rodada do critic (D-2026-10-01-2)
HEAD do código: `ed8114b`. Os rows 1, 4 e 8 ganharam código novo e cada um fica vermelho sob a sua mutação. O row 6 não pedia código: o Verify novo prova pelo stderr que o chamador de produção relata a falha, e o orquestrador conferiu que a mutação M6 o deixa sem `OK`. O orquestrador escreveu esta seção a partir do relatório do doer.

**Commits.** Nenhum commit mistura `.jdi/`.
- `ecf5ef0` test (row 1 (f)): o teste ganha uma 3ª parte. Um `WorkerClient` real, com as mesmas `budgets(vcp)` e `no_backoff()`, escreve BRIGHTNESS num display `Behaviour::Block(gate)`. O resultado tem de ser `Err(DdcError::Timeout)` com `elapsed >= vcp` e `elapsed < vcp + janela`. Leva `finished in 0.32s` (4/4).
- `4393927` test (row 4): o helper `expectToastNaming` confere por literais o texto de `#toast-text`, sem largar a igualdade com o template:
  - `(en)`/`(pt-BR)`: `DisplayPort-1` e `HDMI-1`;
  - aviso genérico: `Brilho`, `75%` e `76%`;
  - leitura que falha: `troca de entrada` e `pode ter comutado`.
- `ed8114b` refactor (row 8):
  - `without_backoff()` sai de `retry.rs` e vai para o novo `ddc_hi_backend/retry/tests.rs`, com os mesmos literais;
  - o acessor `#[cfg(test)] policies()` sai de `worker.rs`, e o campo `WorkerClient::policies` passa a `pub(super)`;
  - o teste (h) lê `backend.client.policies`;
  - nos 3 arquivos de produção, a regex do row 8 casa só `#[cfg(test)]` + `mod tests;`.

**Mutações (todas revertidas byte a byte).**
- M1 (`write_vcp` com `self.write_budget_of(VcpCode::INPUT_SOURCE)`): VERMELHA. Saída: `panicked at .../worker/tests.rs:1267:5: 160.083193ms >= 60ms + 100ms`, `FAILED. 0 passed; 1 failed`.
- M4 (`notice.inputKept` sem `{kept}` nas duas traduções): o Verify 4 sai sem `OK`, e o spec dá `4 failed, 6 passed`, com `(en)`/`(pt-BR)` vermelhos nos dois temas: `Expected substring: "DisplayPort-1"` contra `Received string: "The monitor is still on, not HDMI-1. ..."`.
- M8a (`#[test] fn x() {}` solto em `ddc_hi_backend.rs`), M8b (`#[cfg(test)] impl RetryPolicies { fn x() }` em `retry.rs`) e M8c (`if cfg!(test) {}` em `write_budget_of`): o Verify 8 sai sem `OK` nas três.
- M8d (fiação com `window: Duration::ZERO` em `DdcHiMonitorBackend::new()`): o teste (h) fica VERMELHO (`left: ... window: 0ns` contra `right: ... window: 3s`), e o Verify 1 sai sem `OK`.

**Gates (HEAD `ed8114b`).**
- `cargo fmt --all --check`, `clippy --workspace --all-targets --locked -- -D warnings` e o cross-check Linux saem exit 0.
- `cargo test --workspace --locked`: 413 passed, 0 failed, 9 ignored.
- `npm run test:unit`: 165 pass.
- `cargo llvm-cov ... --summary-only`: exit 0, linhas 85,25% (3626 linhas, 535 sem cobertura).
- Os 10 Verify rodaram literalmente e todos saíram `OK`; o row 9 passou de primeira.
- Nada escreveu em monitor real, o `ddc-tray` do usuário seguiu rodando e o `~/.config/autostart` real ficou idêntico.

**Harness congelado (orquestrador).** Os rows 1 e 4 passam a conferir o SHA-256 destes arquivos:
- `worker/tests.rs` `fadb5aaa…`;
- `ddc_hi_backend/tests.rs` `ec2d2980…`;
- `retry/tests.rs` `12ac10db…`;
- `input-notice.spec.mjs` `bdb21385…`;
- `support.mjs` `ee930894…`.

## Ressalvas
- Código só-Windows (`tray/notification_area.rs`) não compilou aqui (`cargo check --target x86_64-pc-windows-{gnu,msvc}` para em `tauri-winres`: falta `windres`/`llvm-rc`); assinaturas conferidas no `tauri-2.12.0` e no `tray-icon-0.25.1`. Só o `windows-latest` do CI prova.
- `auto-launch 0.5.0`: o `Exec=` sai sem aspas (caminho com espaço quebra a entrada no Linux — documentado em "Known limitations" do README); o `enable` usa `create_dir` não recursivo (exige `~/.config`; o teste o cria); o arquivo se chama `package_info().name`.
- H1 e H2 seguem NÃO provadas (exigiriam escrever o input no RTK real); README, CHANGELOG e as notas dizem "pode".
- Decisões do doer, dentro do escopo: comparação do input pelo byte baixo (como `withReadBack`; `0x0111` lido de um pedido `0x11` assenta); 4 testes extras em `worker/tests.rs` (prazo do chamador; pânico ou recusa durante o assentamento; byte baixo; `a_failed_input_write_is_not_followed_by_reads`); o toast dos avisos herda "Tentar de novo"; o `announce.readBack` curto segue sendo a última fala ao leitor de tela; o "128 testes" do README saiu na iteração 2 (`3451169`).

## Files modified
- `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/retry/tests.rs`
- `apps/ddc-tray/src/{app.js,view-model.js}`, `apps/ddc-tray/src/i18n/{en,pt-BR}.js`, `apps/ddc-tray/tests/ui/view-model.test.mjs`
- `apps/ddc-tray/tests/e2e/{input-notice,pseudo-locale}.spec.mjs`, `apps/ddc-tray/tests/e2e/support.mjs`
- `apps/ddc-tray/scripts/{smoke-sni-private.sh,fake-sni-watcher.py,private-bus.sh,smoke-autostart-private.sh,sni-dbusmenu.py}`
- `crates/ddc-adapters/src/ddc_hi_backend/retry.rs`
- `apps/ddc-tray/src-tauri/Cargo.toml`, `Cargo.lock`, `apps/ddc-tray/src-tauri/src/{autostart,lib,menu,i18n,tray}.rs`, `apps/ddc-tray/src-tauri/src/tray/{status_item,notification_area}.rs`, `apps/ddc-tray/src-tauri/tests/autostart_entry.rs`
- `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`

## Tests
- Total: 413 passed (workspace), 0 failed, 9 ignored; Playwright 152 passed; UI `node --test` 165 passed.
- Coverage: 85,25% (Rust, `cargo llvm-cov` real), 88,48% (UI).
