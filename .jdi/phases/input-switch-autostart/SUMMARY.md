# Phase 9: Input switch fix and autostart — Summary  (slug: input-switch-autostart)

**Status:** complete (iteração 1 do loop: T-1..T-8)
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

## Ressalvas
- Código só-Windows (`tray/notification_area.rs`) não compilou aqui (`cargo check --target x86_64-pc-windows-{gnu,msvc}` para em `tauri-winres`: falta `windres`/`llvm-rc`); assinaturas conferidas no `tauri-2.12.0` e no `tray-icon-0.25.1`. Só o `windows-latest` do CI prova.
- `auto-launch 0.5.0`: o `Exec=` sai sem aspas (caminho com espaço quebra a entrada no Linux — documentado em "Known limitations" do README); o `enable` usa `create_dir` não recursivo (exige `~/.config`; o teste o cria); o arquivo se chama `package_info().name`.
- H1 e H2 seguem NÃO provadas (exigiriam escrever o input no RTK real); README, CHANGELOG e as notas dizem "pode".
- Decisões do doer, dentro do escopo: comparação do input pelo byte baixo (como `withReadBack`; `0x0111` lido de um pedido `0x11` assenta); 4 testes extras em `worker/tests.rs` (prazo do chamador; pânico ou recusa durante o assentamento; byte baixo; `a_failed_input_write_is_not_followed_by_reads`); o toast dos avisos herda "Tentar de novo"; o `announce.readBack` curto segue sendo a última fala ao leitor de tela; o README ainda diz "128 testes" do tray no Windows (não recontado).

## Files modified
- `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`
- `apps/ddc-tray/src/{app.js,view-model.js}`, `apps/ddc-tray/src/i18n/{en,pt-BR}.js`, `apps/ddc-tray/tests/ui/view-model.test.mjs`
- `apps/ddc-tray/tests/e2e/{input-notice,pseudo-locale}.spec.mjs`
- `apps/ddc-tray/scripts/{smoke-sni-private.sh,fake-sni-watcher.py}`
- `apps/ddc-tray/src-tauri/Cargo.toml`, `Cargo.lock`, `apps/ddc-tray/src-tauri/src/{autostart,lib,menu,i18n,tray}.rs`, `apps/ddc-tray/src-tauri/src/tray/{status_item,notification_area}.rs`, `apps/ddc-tray/src-tauri/tests/autostart_entry.rs`
- `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`

## Tests
- Total: 410 passed (workspace), 0 failed, 9 ignored; Playwright 152 passed; UI `node --test` 165 passed.
- Coverage: 85,25% (Rust, `cargo llvm-cov` real), 88,48% (UI).
