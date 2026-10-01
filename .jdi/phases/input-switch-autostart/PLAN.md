# Phase 9: Input switch fix and autostart — Plan  (slug: input-switch-autostart)

## Goal
Corrigir a troca de entrada (`0x60`) que volta para DisplayPort 1 — o app só informa o que o monitor manteve, depois de assentar, e explica uma reversão — e adicionar "iniciar com o sistema" no menu da bandeja do `ddc-tray`, em Linux e Windows.

## Locked decisions (from CONTEXT.md)
- D-2026-09-30-input-switch-autostart-3: assentamento no ADAPTER — após write OK em `0x60`, polling de `read_vcp(0x60)` a cada `INPUT_SETTLE_STEP` (250 ms) até ler o valor ou esgotar `INPUT_SETTLE_WINDOW` (3 s); sempre `Ok(())`; orçamento do write de `0x60` = `budgets.vcp + INPUT_SETTLE_WINDOW`; outros códigos inalterados; `Clock` injetável.
- -4: `ddc-core`, `MonitorBackend` e `MonitorControl` não mudam. -7: CLI fora de escopo (ganha o assentamento pelo adapter).
- -5: toast VISÍVEL via `readBackNotice` pura (input: nomeia mantida e pedida + "pode estar sem sinal"; demais: genérico); i18n `en` + `pt-BR`; estado próprio em `TOAST_STATES`.
- -6: write de input + `timeout`/`transport`/`not_found` → aviso "pode ter comutado"; demais kinds → toast de hoje.
- -8: `tauri-plugin-autostart` 2.x, só pelo Rust, sem capability nova, registrado no `run()` (`lib.rs`).
- -9: estado = entrada do SO (`is_enabled()`), nunca config nossa; padrão DESLIGADO; nada liga sozinho.
- -10: item CHECÁVEL "Start with system" / "Iniciar com o sistema" (ksni e Tauri), `MenuAction::Autostart`, rótulo em `Labels`; sem botão no popup.
- -11: módulo `autostart` (`src-tauri/src/autostart.rs`): trait mínima + impl do plugin + fake em memória; alternar é função pura; falha → `crate::report`.
- -12: nenhum teste escreve input em monitor real; sem `unsafe`/`env::set_var`; commits `type(input-switch-autostart): ...` ≤ 72 chars.

## Tasks

Specialist de todas: `jdi-doer-ddc-control` (single-stack, glob `**/*`). 1 task = 1 commit. Gates por task: `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`; `npm run test:unit` em `apps/ddc-tray` quando tocar `src/` ou `tests/` do popup. Os nomes de teste abaixo são a fonte única (os `Verify:` do DoD dependem deles).

### Wave 1 (parallel-eligible)

#### T-1: assentar o write de `0x60` no worker do adapter (bugfix: teste primeiro)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend.rs` (só doc), `crates/ddc-adapters/src/ddc_hi_backend/retry.rs` (só se precisar de política de 1 tentativa)
- **Acceptance:**
  - Os 5 testes iniciais de `ddc_hi_backend::worker::tests` (7 depois das Emendas, mais o (h) em `ddc_hi_backend::tests`) são escritos antes e vistos falhando (saída vermelha no SUMMARY): `input_write_returns_once_the_monitor_reads_back_the_value`, `input_write_returns_ok_after_the_settle_window_when_the_monitor_keeps_the_old_value`, `input_write_survives_reads_that_fail_or_lie_while_settling`, `writes_to_other_codes_do_not_settle`, `input_write_budget_covers_the_settle_window`, cobrindo as afirmações (a)–(e) do DoD; nenhum outro teste do módulo começa com `input_write_`/`writes_to_other_codes_`; todos em `VirtualClock` via `Worker::new` (nunca `WorkerClient`, que dormiria), cada um < 1,5 s.
  - `INPUT_SETTLE_STEP` e `INPUT_SETTLE_WINDOW` nomeadas com linha WHY; polling só após write OK em `VcpCode::INPUT_SOURCE`; cada poll é UMA tentativa `isolated` (catch_unwind, sem a `RetryPolicy` de 3); `Err`, valor errado (16/17 por 15) ou eco de outro código seguem o polling; pânico ou "unsupported" encerram com `Ok(())`; a janela conta a partir do write e nunca passa do `deadline`; nenhum sleep depois do poll que acertou.
  - Função pura do orçamento (hoje `write_budget(&DdcHiBudgets, &RetryPolicies, VcpCode)`, alcançada só por `WorkerClient::write_budget_of`; ver Emendas) usada por `WorkerClient::write_vcp` E pelo teste (e); `budgets.vcp` para qualquer outro código. Fake estendido (sequência roteirizada de leituras), sem mock por teste. `crates/ddc-core` sem 1 byte mudado; nenhuma linha nova de `worker/tests.rs` cita `DDC_HW_TESTS`, `DdcHiMonitorBackend` ou `/dev/i2c`; `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` verde.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-adapters --locked --lib -- ddc_hi_backend::worker::tests` + DoD Verify linhas 1 e 2
- **Commit:** `fix(input-switch-autostart): settle input writes before reading back`
- **Status:** completed

#### T-2: avisos puros no view-model + i18n (teste primeiro)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src/view-model.js`, `apps/ddc-tray/src/i18n/en.js`, `apps/ddc-tray/src/i18n/pt-BR.js`, `apps/ddc-tray/tests/ui/view-model.test.mjs`
- **Acceptance:**
  - 4 testes com nomes exatos em `tests/ui/view-model.test.mjs`, escritos antes e vistos falhando: `a read-back that differs from the request gives a notice naming both values`, `the input notice says the asked input may have no signal`, `a read-back equal to the request gives no notice`, `a failed read after an input write says the monitor may have switched away`.
  - `readBackNotice` (export, pura) devolve `null` quando lido == pedido após a MESMA normalização de `withReadBack` (byte baixo das listas: ler `0x0111` de um pedido `0x11` não avisa); senão texto com o nome mantido E o pedido — para `0x60` com a frase de "pode estar sem sinal", para outro código o genérico, sem ela. Função pura irmã (ex.: `writeFailureText(error, code, t)`): input + `timeout`/`transport`/`not_found` → aviso "pode ter comutado"; `invalid_value` (testado) e os demais kinds, ou outro código → `errorText` de sempre.
  - Chaves novas em `en` e `pt-BR` (paridade de `i18n-parity.test.mjs`), nenhuma string literal; nomes de entrada entram como parâmetro (dado do core). `node --test --test-reporter=tap tests/ui/*.test.mjs` com `# fail 0`, `# cancelled 0`, `# skipped 0`, `# todo 0` (DoD linha 3).
- **Dependencies:** none
- **Test:** `cd apps/ddc-tray && npm run test:unit` + DoD Verify linha 3
- **Commit:** `fix(input-switch-autostart): pure notices for a kept or unread input`
- **Status:** completed

#### T-3: módulo `autostart` sobre o plugin + registro no composition root
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/Cargo.toml`, `Cargo.lock`, `apps/ddc-tray/src-tauri/src/autostart.rs` (novo), `apps/ddc-tray/src-tauri/src/lib.rs`
- **Acceptance:**
  - `tauri-plugin-autostart` 2.x (2.6.0 pede `tauri ^2.12`) em `[dependencies]` do `ddc-tray`; `tauri` com feature `test` em `[dev-dependencies]`; `Cargo.lock` no mesmo commit; `cargo tree -p ddc-tray --locked -i tauri-plugin-autostart` resolve; `capabilities/` intacto (só `default.json`, mesmas permissões, nenhum `autostart`).
  - `autostart.rs` (`pub mod autostart`): trait mínima (ex.: `AutostartEntry`) sem parâmetro booleano (enum de estado); impl de produção `pub` sobre `app.autolaunch()` (`enable`/`disable`/`is_enabled`), construível de `&AppHandle<R>` genérico em `R: Runtime` (T-6 a usa no runtime mock); `register(builder: tauri::Builder<R>) -> tauri::Builder<R>` é a ÚNICA configuração do plugin (sem args: o SO inicia como um start qualquer) e `run()` passa por ela; alternar = função pura sobre a trait que devolve o erro ao chamador, que o manda a `crate::report` (reporter injetável no teste); fake em memória `#[cfg(test)]`.
  - Passam `autostart::tests::toggle_enables_a_disabled_entry`, `autostart::tests::toggle_disables_an_enabled_entry`, `autostart::tests::a_failed_toggle_is_reported_and_leaves_the_os_state_as_it_was`, `autostart::tests::the_app_builder_registers_the_autostart_plugin` (DoD linha 6). O último passa `tauri::test::mock_builder()` por `register`, monta com `mock_context(noop_assets())` e afirma `try_state::<AutoLaunchManager>()` presente — sem single-instance no builder de teste e sem tocar `~/.config/autostart`. `#![forbid(unsafe_code)]` mantido, sem `env::set_var`; nada chama `enable` fora do alternar.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-tray --locked --lib -- autostart::` + DoD Verify linhas 6 e 8
- **Commit:** `feat(input-switch-autostart): autostart entry over the Tauri plugin`
- **Status:** completed

### Wave 2 (parallel-eligible)

#### T-4: popup mostra os avisos + specs Playwright
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src/app.js`, `apps/ddc-tray/tests/e2e/input-notice.spec.mjs` (novo), `apps/ddc-tray/tests/e2e/pseudo-locale.spec.mjs`, `apps/ddc-tray/tests/e2e/confirm.spec.mjs` (só se uma asserção existente conflitar com o toast novo)
- **Acceptance:**
  - `showReadBack` chama `showToast` com o `readBackNotice` quando não-nulo (mantém `flag` + `announce`; lido == pedido → só `hideToast`); `writeFailed` segue com UMA menção a `showToast`, texto da função pura de T-2; após falha o chip de input segue na entrada anterior.
  - `pseudo-locale.spec.mjs`: `STATES` ganha `rtk after the monitor kept the old input` e `rtk after an input write whose read-back failed`; `TOAST_STATES` ganha `{ site: 'app.js › showReadBack', state: 'rtk after the monitor kept the old input' }`; `tests/ui/toast-states.test.mjs` verde sem edição.
  - `input-notice.spec.mjs` com os 5 testes (nomes exatos): `the toast names the kept and the asked input (en)`, `the toast names the kept and the asked input (pt-BR)`, `an input read back as asked shows no notice`, `a failed read after an input write says the monitor may have switched away`, `a setting other than the input that the monitor did not apply shows the generic notice`; usa `test` e `expectAccessible(page)` de `support.mjs` (1 por teste, provado pela anotação `axe` no relatório JSON; ver Emendas); patch da linha `entry.reading.current = value;` do demo como em `confirm.spec.mjs` (DisplayPort-1 mantido ao pedir DisplayPort-2) e `fail=write` para a falha; toast VISÍVEL com os dois nomes. Nenhum `disableRules`/`.exclude(`/`.include(`/`.options(`/`setLegacyMode`/`test.skip|fixme|only|fail` em `tests/e2e/*.mjs`; suíte Playwright INTEIRA verde em `light` e `dark` (DoD linha 4).
- **Dependencies:** T-2
- **Test:** iterar `npx playwright test tests/e2e/input-notice.spec.mjs tests/e2e/pseudo-locale.spec.mjs`; suíte inteira ao fechar + DoD Verify linhas 3 e 4
- **Commit:** `fix(input-switch-autostart): show the read-back and input notices`
- **Status:** completed

#### T-5: item "Start with system" nos menus Linux e Windows
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/menu.rs`, `apps/ddc-tray/src-tauri/src/i18n.rs`, `apps/ddc-tray/src-tauri/src/tray.rs`, `apps/ddc-tray/src-tauri/src/tray/status_item.rs`, `apps/ddc-tray/src-tauri/src/tray/notification_area.rs`, `apps/ddc-tray/src-tauri/src/lib.rs`
- **Acceptance:**
  - `Labels.autostart` = "Start with system" (en) / "Iniciar com o sistema" (pt-BR); `MenuAction::Autostart` com id estável `autostart`; entrada checável cuja marca vem do estado do SO passado a `menu_entries`; ordem travada: Windows = Open panel · sep · Start with system · Quit; Linux = Open panel · sep · 5 brilhos · sep · Start with system · Quit.
  - Novos: `i18n::tests::the_autostart_label_is_set_in_every_locale_and_differs_between_them`, `menu::tests::the_autostart_item_is_checked_exactly_when_the_os_entry_exists_on_both_platforms` (ligado e desligado × Linux e Windows), `menu::tests::the_autostart_action_maps_back_from_its_id`; atualizados: listas exatas de `menu.rs`, ids únicos 7 → 8, `english_labels_are_exact`, `brazilian_portuguese_labels_are_exact`, `no_label_is_empty_in_any_locale`; `cargo test -p ddc-tray --locked --lib -- menu:: i18n::` com `0 ignored` (DoD linha 5).
  - Impl de produção de T-3 construída só no setup do `run()` e entregue ao tray (estado gerenciado); ksni (`CheckmarkItem`) e Windows (`CheckMenuItem`) marcados por `is_enabled()` a cada montagem; clicar chama o alternar de T-3, falha → `crate::report`, e o item volta ao estado real do SO (menu remontado ou `set_checked`); `is_enabled()` com erro → desmarcado + `report`, sem pânico; montar o menu só LÊ. Smoke: `cargo build -p ddc-tray --release --locked -q && bash apps/ddc-tray/scripts/smoke-sni.sh --fake --activate target/release/ddc-tray` verde (DoD linha 9).
- **Dependencies:** T-3
- **Test:** `cargo test -p ddc-tray --locked --lib -- menu:: i18n::` + smoke SNI + DoD Verify linhas 5 e 9
- **Commit:** `feat(input-switch-autostart): start-with-system item in the tray menu`
- **Status:** completed

#### T-6: teste de integração do `.desktop` em `HOME` sandboxed (processo filho)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/tests/autostart_entry.rs` (novo), `apps/ddc-tray/src-tauri/Cargo.toml` (dev-dep `tempfile`, já no lock como transitiva), `Cargo.lock`
- **Acceptance:**
  - Arquivo `#![cfg(target_os = "linux")]` com EXATAMENTE 2 testes: pai `enabling_then_disabling_in_a_sandboxed_home_writes_then_removes_the_desktop_entry` e filho `child_enables_or_disables_the_entry_under_the_sandboxed_home`.
  - Pai: tempdir; relança `current_exe()` com `--exact <filho> --nocapture`, `HOME=<tempdir>`, marcador de sandbox e ação via `Command::env`, `env_remove("APPIMAGE")` e `env_remove("XDG_CONFIG_HOME")`; após habilitar lê `<HOME>/.config/autostart/*.desktop` (exatamente 1), IMPRIME o conteúdo cru (linha `Exec=` na coluna 0), afirma `Exec=` no binário de teste e `is_enabled()` verdadeiro relatado pelo filho; após desabilitar, arquivo removido e `is_enabled()` falso.
  - Filho: sem marcador → retorna sem efeito (suíte normal); com marcador e `HOME` fora do tempdir → `std::process::abort()`; usa a impl de produção de T-3 via `register(tauri::test::mock_builder())` + `mock_context(noop_assets())`. Sem `unsafe`/`env::set_var`; o `~/.config/autostart` real idêntico antes e depois (DoD linha 7).
- **Dependencies:** T-3
- **Test:** `cargo test -p ddc-tray --locked --test autostart_entry -- --exact enabling_then_disabling_in_a_sandboxed_home_writes_then_removes_the_desktop_entry --nocapture` + DoD Verify linhas 7 e 8
- **Commit:** `test(input-switch-autostart): desktop entry in a sandboxed HOME`
- **Status:** completed

#### T-8: smoke do tray numa sessão D-Bus privada (DoD linha 9, D-13)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/scripts/smoke-sni-private.sh` (novo), `apps/ddc-tray/scripts/fake-sni-watcher.py` (novo)
- **Acceptance:**
  - `smoke-sni-private.sh <ddc-tray>` sobe `dbus-run-session` com o `fake-sni-watcher.py` (Gio, `org.kde.StatusNotifierWatcher`: `RegisterStatusNotifierItem`/`RegisterStatusNotifierHost`, `RegisteredStatusNotifierItems`, `IsStatusNotifierHostRegistered`, `ProtocolVersion`, sinal `StatusNotifierItemRegistered`), espera o watcher responder e roda `smoke-sni.sh --fake --activate <ddc-tray>` SEM editar esse script; imprime a linha exata `smoke-sni-private: private session bus, stand-in StatusNotifierWatcher` antes de rodar; mata o watcher ao sair; `set -euo pipefail`; sai ≠ 0 com mensagem clara se `dbus-run-session`, `busctl`, `python3` ou `python3-gobject` (`gi`) faltar, ou se o watcher não responder em 5 s.
  - A saída final do smoke é `smoke-sni: OK — PID <n> registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked`, EXECUTADA com um `ddc-tray` do usuário rodando na sessão real (prova de que a linha não depende do estado ambiente) e com `DBUS_SESSION_BUS_ADDRESS` do usuário diferente do privado.
  - Prova negativa registrada no SUMMARY: com o binário trocado por um que sai em 1 s (`/bin/true`), o wrapper sai ≠ 0 e NÃO imprime a linha `smoke-sni: OK`.
  - Nenhuma linha nova cita `/dev/i2c`, `DdcHiMonitorBackend` ou escreve em monitor real (`--fake`).
- **Dependencies:** none
- **Test:** DoD Verify linha 9
- **Commit:** `test(input-switch-autostart): smoke the tray on a private bus`
- **Status:** completed

### Wave 3

#### T-7: README, CHANGELOG e nota de validação de hardware
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`
- **Acceptance:**
  - README: as 2 linhas falsas ("No autostart, profiles or global hotkeys yet", "Profiles, global hotkeys and autostart (phase `profiles-hotkeys`)") corrigidas — autostart existe (item "Start with system"), perfis/hotkeys seguem em `profiles-hotkeys`; a seção do tray cita o item e o aviso de entrada mantida.
  - CHANGELOG `[Unreleased]`: `Added` (Start with system, Linux e Windows) e `Fixed` (troca de entrada assenta até 3 s; o popup avisa quando o monitor mantém outra entrada ou pode ter comutado).
  - `docs/hardware-validation.md`: o autostart só mexe em `~/.config/autostart` (seguro, desfeito pelo menu); trocar a entrada segue em "Never do through the popup".
- **Dependencies:** T-1, T-4, T-5, T-6
- **Test:** sem teste novo; gates do repo seguem verdes
- **Commit:** `docs(input-switch-autostart): autostart and input settling notes`
- **Status:** completed

## Execution
- Total tasks: 8 (+ trabalho de correção do loop, sem task nova: ver Emendas) · Waves: 3 · Estimated parallel speedup: ~2,3x (o doer pode rodar T-1..T-7 em sequência)
- DoD → task: linhas 1–2 → T-1; 3 → T-2 (T-4 mantém verde); 4 → T-4; 5 e 9 → T-5; 6 → T-3; 7 → T-6; 8 → T-1, T-3, T-6.
- Deferred to PR review (hardware, logout/login, Windows Inicialização, redação) não vira task.

## Emendas (trabalho de correção do loop, sem task nova)
As iterações 2 e 3 do loop corrigiram achados do DoD Critic, não tasks do plano; seus commits têm o escopo da phase e estão no SUMMARY. O que mudou em relação às tasks acima:
- T-1: o passo e a janela do assentamento moram em `RetryPolicies.input_settle` (`retry.rs` entrou em `files_modified`; as constantes são privadas de `retry.rs`); a linha 1 do DoD tem 7 testes, não 5 (D-14, D-17).
- T-8 e D-15: além dos 2 scripts de T-8, `apps/ddc-tray/scripts/private-bus.sh` (lib para `source`), `smoke-autostart-private.sh` e `sni-dbusmenu.py` entram em `files_modified`; `smoke-sni-private.sh` passou a usar `private-bus.sh` com saída idêntica (D-18).
- DoD: 10 linhas Auto no CONTEXT (a 10ª é D-16, TODO/FIXME em arquivos não-Rust).
- Iteração 4 (D-2026-09-30-input-switch-autostart-19, D-2026-10-01-input-switch-autostart-1): T-1 ganha `WorkerClient::write_budget_of` e o teste (h) da fiação em `ddc_hi_backend/tests.rs` (que entra em `files_modified`, com `production_backends()`); (f) ganha o limite de cima do orçamento; T-4 toca `apps/ddc-tray/tests/e2e/support.mjs` (anotação `axe` em `expectAccessible`).
- Iteração 5 (D-2026-10-01-input-switch-autostart-2): T-1 move `without_backoff()` para `ddc_hi_backend/retry/tests.rs` (novo, em `files_modified`), tira o acessor de teste de `worker.rs` (`WorkerClient::policies` vira `pub(super)`) e (f) ganha a 3ª parte (outro código no orçamento simples); T-4 confere o toast por literais; os rows 1 e 4 congelam os arquivos de teste por SHA-256.
- Rodada 2, iteração 1 (D-2026-10-01-input-switch-autostart-3): T-1 usa uma política custom com janela maior que o `Default` e `crates/ddc-adapters/Cargo.toml` entra em `files_modified` (`[lib] doctest = false`); T-2 e T-4 conferem as frases literais inteiras; `tests/ui/view-model.test.mjs` passa a ser congelado na linha 3.
- Rodada 2, iteração 2 (D-2026-10-01-input-switch-autostart-4): T-1 ganha o limite de leituras `ceil(janela / passo)` em (f); os Verify das linhas 8 (`--list` ancorado) e 10 (arquivos não-Rust do diff) mudam sem código.

## Files modified (all tasks)
- `crates/ddc-adapters/Cargo.toml`, `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{worker,retry,tests}.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{worker,retry}/tests.rs`
- `apps/ddc-tray/src/{app.js,view-model.js}`, `apps/ddc-tray/src/i18n/{en,pt-BR}.js`, `apps/ddc-tray/tests/ui/view-model.test.mjs`
- `apps/ddc-tray/tests/e2e/{input-notice,pseudo-locale,confirm}.spec.mjs`, `apps/ddc-tray/tests/e2e/support.mjs`
- `apps/ddc-tray/src-tauri/Cargo.toml`, `Cargo.lock`, `apps/ddc-tray/src-tauri/src/{autostart,lib,menu,i18n,tray}.rs`, `apps/ddc-tray/src-tauri/src/tray/{status_item,notification_area}.rs`, `apps/ddc-tray/src-tauri/tests/autostart_entry.rs`
- `apps/ddc-tray/scripts/{smoke-sni-private.sh,fake-sni-watcher.py,private-bus.sh,smoke-autostart-private.sh,sni-dbusmenu.py}`, `crates/ddc-adapters/src/ddc_hi_backend/retry.rs` (correções do loop)
- `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`

## Test requirements
- Rust: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; cross-check Linux em T-1.
- UI: `cd apps/ddc-tray && npm run test:unit` e `npx playwright test` (light + dark).
- Minimum coverage: 80% linhas (`cargo llvm-cov --workspace --locked --fail-under-lines 80`) + piso de UI do pipeline.
- Os 10 `Verify:` do DoD do CONTEXT.md (emendados por D-13, D-14, D-15, D-16 e D-17), executados pelo reviewer.

## Risks
- O plugin exige `AppHandle`: os testes (T-3, T-6) precisam do runtime de teste do Tauri (`tauri` feature `test` em dev-dep), o que amplia o tempo de build de teste. A Tauri CLI lê toda entrada `tauri` do `Cargo.toml` (D-2026-09-28-release-packaging-4): conferir que a dev-dep não muda features nem dependências dos pacotes deb/rpm.
- Nenhum builder de teste leva o single-instance: com um `ddc-tray` aberto na máquina, ele encerraria o processo de teste.
- A suíte Playwright inteira é lenta (todas as specs × 2 temas): iterar no spec novo + pseudo-locale e rodar a inteira uma vez ao fechar T-4.
- A dependência nova pede rede no `cargo`: rodar `cargo add`/`cargo update -p tauri-plugin-autostart` primeiro (grava o `Cargo.lock`); `--locked` só depois. Se a resolução mexer no patch de `tauri`, vai no mesmo commit de T-3.
- O código só-Windows (`notification_area.rs`, chave `HKCU\...\Run`) só é provado pela compilação e pelos testes no runner windows-latest do CI.
- O smoke SNI (DoD linha 9) precisa de sessão com host StatusNotifierItem; sem ela, o doer registra no SUMMARY e a evidência fica com o reviewer.
