# Phase 10: USB switch follow — Context  (slug: usb-switch-follow)

## Goal
O monitor segue o switch USB: quando os dispositivos aprendidos saem desta máquina, ela manda `0x60` para a entrada de destino (KVM por software, só Linux, só bandeja).

## Locked decisions
- D-2026-10-02-usb-switch-follow-2: lógica pura (`Follower`, `learn`, `UsbPresence`) em `ddc-core`; `usb_sysfs` em `ddc-adapters` com raiz injetada; config, laço e menu em `apps/ddc-tray`.
- D-2026-10-02-usb-switch-follow-3: polling de sysfs a 500 ms, debounce de 3 leituras; id `vid:pid[:serial]`; hubs ignorados; dispara quando TODOS os aprendidos saem, uma vez, rearma ao voltar um; chegada e estado inicial nunca disparam.
- D-2026-10-02-usb-switch-follow-4: config `$XDG_CONFIG_HOME/ddc-control/usb-follow.json` (fallback `~/.config`), JSON com `version` 1; versão desconhecida = não configurado.
- D-2026-10-02-usb-switch-follow-5: menu Linux: check "Follow USB switch", "Learn USB switch" (janela de 30 s), entrada de destino (`0x0F`/`0x10`/`0x11`/`0x12`), linha de resumo; monitor = alvo do popup no momento do aprender.
- D-2026-10-02-usb-switch-follow-6: texto do mudo: `{label} (on another input)` / `{label} (em outra entrada)` e a dica cobre "outra entrada OU DDC/CI desligado".
- D-2026-10-02-usb-switch-follow-7: DDC mudo = uma tentativa, erro só em `crate::report`, sem pânico nem retry.
- D-2026-10-02-usb-switch-follow-8: consentimento Dangerous dado ao ligar o follow com config completa; só o monitor e a entrada gravados.
- D-2026-10-02-usb-switch-follow-9: fakes em toda ponta (sysfs tempdir, `InMemoryMonitorBackend`, relógio, config tempdir); suíte passa com `/sys` coberto; `DDC_TRAY_USB_ROOT` só com `DDC_TRAY_FAKE=1`.
- D-2026-10-02-usb-switch-follow-10: Windows, macOS, CLI, hotkeys etc. fora (ver `.jdi/todos/2026-10-02-usb-switch-follow.md`).
- Herdadas: D-1, D-2, D-4 (enumerate não sonda), D-12 (nenhum teste escreve em monitor real), D-2026-09-30-input-switch-autostart-3 (assentamento no adapter).

## Canonical refs
- Card do `/jdi-issue` de 2026-10-02 (sem tracker); registro da phase: D-2026-10-02-usb-switch-follow-1
- `apps/ddc-tray/src-tauri/src/autostart.rs` (padrão trait + fake + report), `menu.rs`, `tray/status_item.rs`
- `apps/ddc-tray/src/app.js`, `src/i18n/{en,pt-BR}.js`; `apps/ddc-tray/scripts/private-bus.sh`

## Out of scope
- Ver D-10 e `.jdi/todos/2026-10-02-usb-switch-follow.md`.

## Definition of Done
Baseline do `.jdi/PROJECT.md` herdado: testes do workspace, cobertura >= 80% e TODO/FIXME em `.rs`. fmt e clippy `-D warnings` são gates do reviewer. Modo `dod=auto_only`. Cada `Verify:` é autocontido e roda num `bash` limpo (D-11). O fragmento `for t in ...` no fim de cada linha de comportamento é o CONGELAMENTO do código revisado: árvores git, blobs e working tree limpo (ver Notes). O orquestrador o atualiza depois de cada passada do doer.

### Auto-verifiable
- [ ] Core: o `Follower` e o `learn` seguem a D-3 com relógio fake. Os testes exigem quatro comportamentos:
  - dispara uma vez quando TODOS os aprendidos saem;
  - não dispara na chegada, desligado, em saída parcial, em ausência mais curta que o debounce, nem com tudo ausente no início;
  - só volta a disparar depois que um dispositivo voltou;
  - `learn` = presentes antes − presentes depois.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-core app::usb_follow::tests:: fires_once_when_all_learned_devices_left arrival_never_fires disabled_never_fires partial_leave_does_not_fire absence_shorter_than_debounce_does_not_fire fires_again_only_after_a_device_returned all_absent_at_start_does_not_fire learn_is_present_before_minus_present_after || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** hardware real; tempo real (o relógio é fake); outras plataformas.
      **Source:** CONTEXT
- [ ] Adapter: o `usb_sysfs` lê, numa raiz de sysfs fake (tempdir):
  - a identidade `vid:pid[:serial]` (D-3);
  - um dispositivo sem serial como `vid:pid`;
  - pula hubs (`bDeviceClass` 09);
  - raiz ausente = conjunto vazio;
  - dispositivo removido some entre duas leituras.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-adapters usb_sysfs::tests:: reads_vid_pid_and_serial_from_fake_root device_without_serial_is_vid_pid_only skips_hubs missing_root_is_empty_set unplugged_device_disappears_between_two_reads || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** o formato do `/sys` real de cada kernel; hardware real.
      **Source:** CONTEXT
- [ ] Laço do follow no `ddc-tray`, com sysfs fake, `InMemoryMonitorBackend` e relógio fake:
  - depois da saída dos aprendidos, o log de escritas do fake é EXATAMENTE `[(RTK-RTK-QHD-HDR-01010101, 0x60, 0x10)]`, e o stderr tem a linha `ddc-tray: follow: switching RTK-RTK-QHD-HDR-01010101 to input 0x10`;
  - na chegada e com o follow desligado, o log é `[]`;
  - uma escrita que falha (DDC mudo) vai para o `report` e o laço continua lendo (D-7);
  - só o monitor configurado e só o código `0x60`;
  - o `Stderr` REAL (escrita injetável) imprime a linha de troca com o diagnóstico (`DDC_TRAY_DEBUG`) DESLIGADO (D-13b).
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-tray follow::tests:: leaving_learned_devices_writes_0x60_target_exactly_once_on_configured_monitor arrival_writes_nothing disabled_writes_nothing silent_ddc_write_is_reported_and_loop_keeps_polling writes_only_configured_monitor_and_code_0x60 || ok=0; sc ddc-tray follow::stderr_tests:: announce_prints_without_the_diagnostics_switch || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** que o monitor real aceita a troca; o DDC mudo real; a fiação no `run()`, que a linha 8 prova.
      **Source:** CONTEXT
- [ ] Config (D-4):
  - vai e volta por um tempdir;
  - o caminho usa `XDG_CONFIG_HOME` e depois `~/.config`;
  - arquivo ausente = desligado;
  - versão desconhecida = não configurado, e o arquivo não é sobrescrito;
  - o aprender nunca liga `enabled`.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-tray follow_config::tests:: round_trips_through_a_tempdir path_uses_xdg_config_home_then_home_config missing_file_is_disabled_default unknown_version_is_not_configured_and_not_overwritten learn_never_sets_enabled || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** permissões de arquivo, disco cheio, outras plataformas.
      **Source:** CONTEXT
- [ ] Menu e rótulos (D-5).
  - O check "Follow USB switch" fica marcado exatamente quando `enabled`.
  - Os itens do follow aparecem no menu Linux e não no Windows.
  - Os ids fazem ida e volta por `from_id`.
  - O teste de i18n afirma os LITERAIS: `Follow USB switch`/`Seguir o switch USB`, `Learn USB switch`/`Aprender o switch USB`, `DisplayPort 1`, `DisplayPort 2`.
  - Os testes antigos de `menu::` e `i18n::` continuam passando.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-tray menu::tests::follow_ check_is_marked_exactly_when_enabled items_are_in_the_linux_menu items_are_not_in_the_windows_menu ids_round_trip_through_from_id || ok=0; sc ddc-tray i18n::tests::follow_ labels_are_literal_in_en_and_pt_br || ok=0; o=$(cargo test --locked -p ddc-tray --lib -- menu:: i18n:: 2>&1); printf '%s\n' "$o" | grep -qE 'test result: ok\. [0-9]+ passed; 0 failed; 0 ignored' || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** o desenho do menu num desktop real (KDE/GNOME).
      **Source:** CONTEXT
- [ ] O popup mostra o monitor com EDID e DDC mudo como "em outra entrada" (D-6, D-12b, D-13a):
  - o teste afirma as FRASES LITERAIS do seletor (`{label} (on another input)` / `{label} (em outra entrada)`) e da dica nos dois idiomas;
  - a suíte `tests/ui` inteira continua verde, incluindo paridade de i18n e nenhuma string pt-BR em `app.js`;
  - o popup RENDERIZA o rótulo do monitor mudo com o literal `LG TV SSCR2 (em outra entrada)`, conferido pelo teste e2e `a silent monitor reads on another input in the picker` (`tests/e2e/fallback.spec.mjs`) em `light` e `dark`;
  - a suíte Playwright INTEIRA passa sem `failed` nem `flaky` (D-12b, D-13a).
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; cd apps/ddc-tray || exit 1; npm ci --ignore-scripts --no-audit --no-fund --silent || ok=0; o=$(node --test --test-reporter=tap tests/ui/*.test.mjs 2>&1) || ok=0; for s in '# fail 0' '# cancelled 0' '# skipped 0' '# todo 0'; do printf '%s\n' "$o" | grep -qx "$s" || ok=0; done; printf '%s\n' "$o" | grep -qxE ' *ok [0-9]+ - a monitor with a readable EDID and no DDC answer reads on another input in en and pt-BR' || ok=0; p=$(npx playwright test --reporter=list 2>&1) || ok=0; printf '%s\n' "$p" | grep -qE '^ +[0-9]+ passed' || ok=0; ! printf '%s\n' "$p" | grep -qE '[0-9]+ (failed|flaky)' || ok=0; for th in light dark; do printf '%s\n' "$p" | grep -F "[$th] › tests/e2e/fallback.spec.mjs" | grep -F '› a silent monitor reads on another input in the picker' | grep -q '✓' || ok=0; done; cd ../..; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** o mudo real do RTK; que o `app.js` chama as funções novas do view-model (o rótulo renderizado é o que se afirma).
      **Source:** CONTEXT
- [ ] A documentação de instalação nas duas máquinas tem o conteúdo exigido. O teste lê `docs/usb-switch-follow.md` e exige os literais:
  - `i2c-dev`, `/dev/i2c-`, `modules-load.d`;
  - `XDG_CONFIG_HOME`, `usb-follow.json`;
  - `PC`, `notebook`;
  - `0x0F`, `0x10`;
  - o consentimento da escrita Dangerous.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; sc(){ p=$1; q=$2; shift 2; w=$(printf "$q%s\n" "$@" | sort); g=$(cargo test --locked -p "$p" --lib -- --list 2>/dev/null | sed -n 's/: test$//p' | grep "^$q" | sort); [ -n "$g" ] && [ "$w" = "$g" ] || return 1; for n in "$@"; do cargo test --locked -p "$p" --lib -- --exact "$q$n" 2>&1 | grep -q 'test result: ok\. 1 passed; 0 failed; 0 ignored' || return 1; done; }; sc ddc-tray docs::tests:: usb_follow_doc_names_i2c_setup_for_both_machines usb_follow_doc_states_the_consent_of_the_dangerous_write || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** que as instruções funcionam numa máquina limpa, nem que o texto é claro (humano, ver Deferred).
      **Source:** CONTEXT
- [ ] Smoke de ponta a ponta do binário RELEASE. Roda em sessão D-Bus PRIVADA, com o watcher de stand-in, `HOME` e `XDG_CONFIG_HOME` em tempdir, `DDC_TRAY_FAKE=1`, `DDC_TRAY_USB_ROOT` numa raiz USB fake (teclado, mouse e o hub do switch) e locale inglês. O script `apps/ddc-tray/scripts/smoke-follow-private.sh` faz o roteiro pelo `com.canonical.dbusmenu`:
  - lê o check desmarcado;
  - clica "Learn USB switch" e remove da raiz teclado, mouse e o hub do switch (um root hub fica), e o aprender registra os dois, sem o hub (D-12c);
  - devolve os dispositivos;
  - escolhe "DisplayPort 2";
  - confere a config gravada com `enabled` falso;
  - clica o check, que fica marcado e grava `enabled` verdadeiro;
  - remove os dispositivos e confere EXATAMENTE uma linha de troca para `0x10` no stderr;
  - devolve os dispositivos e confere que nada troca;
  - desliga o check, remove os dispositivos e confere que nada troca.

  O stderr não pode ter `panicked`. O `~/.config/autostart` e o `~/.config/ddc-control` reais ficam idênticos em nomes e conteúdo (sha256 de cada arquivo, D-13c). Esta linha prova que o `run()` liga o laço e que o menu configura o follow.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; A="$HOME/.config/autostart"; R="$HOME/.config/ddc-control"; snap(){ [ -d "$1" ] || { echo absent; return 0; }; (cd "$1" && find . | sort && find . -type f -print0 | sort -z | xargs -0 -r sha256sum); }; ba=$(snap "$A"); br=$(snap "$R"); cargo build -p ddc-tray --release --locked -q || ok=0; o=$(bash apps/ddc-tray/scripts/smoke-follow-private.sh target/release/ddc-tray 2>&1) || ok=0; for l in 'smoke-follow-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME, fake USB root' 'smoke-follow: the app serves the simulated monitor' 'smoke-follow: '\''Follow USB switch'\'' is an unmarked checkmark' 'smoke-follow: learning saw exactly the keyboard and the mouse leave, not the hub' 'smoke-follow: the config holds the 2 learned devices, the simulated monitor and input 0x10, with enabled false' 'smoke-follow: after the click '\''Follow USB switch'\'' is marked and the config says enabled' 'smoke-follow: leaving switched the simulated monitor to input 0x10 exactly once' 'smoke-follow: arriving switched nothing' 'smoke-follow: with follow off, leaving switched nothing' 'smoke-follow: OK — the tray followed the fake USB switch end to end'; do printf '%s\n' "$o" | grep -qxF -- "$l" || ok=0; done; [ "$(snap "$A")" = "$ba" ] && [ "$(snap "$R")" = "$br" ] || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** hardware, USB e monitor reais; KDE e GNOME reais (o watcher é de stand-in).
      **Source:** CONTEXT
- [ ] A suíte dos 3 crates passa sem o `/sys` real e sem nós `/dev/i2c-*` (D-9). O próprio sandbox confere que eles não existem lá dentro.
      **Verify:** `export LC_ALL=C.UTF-8; ok=1; bwrap --bind / / --dev /dev --tmpfs /sys sh -c '[ ! -e /sys/bus/usb ] && ! ls /dev | grep -q "^i2c-"' || ok=0; bwrap --bind / / --dev /dev --tmpfs /sys cargo test --locked -p ddc-core -p ddc-adapters -p ddc-tray >/dev/null 2>&1 || ok=0; for t in crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0; [ $ok = 1 ] && echo OK`
      **NÃO afirmado:** que nenhum teste TENTA tocar `/sys` ou `/dev/i2c`; prova só que eles não são necessários. `/proc`, rede e outros caminhos.
      **Source:** CONTEXT
- [ ] Sem `TODO`/`FIXME` sem referência de issue nos arquivos NÃO-Rust da phase: caminhos fixos UNIDOS aos não-Rust do diff, e `en.js`/`pt-BR.js` só em comentário e sem `/*`.
      **Verify:** `export LC_ALL=C.UTF-8; STRIP='s/\b(to[ -]?dos?|fixmes?)\b[[:space:]]*[(:]?[[:space:]]*\(?#[0-9]+\)?//Ig'; W='\b(todo|fixme)s?\b|\bto[ -]dos?\b[[:space:]]*[:(]|\bunimplemented!\('; C='(//|#|<!--).*\b(todo|fixme)s?\b'; ok=1; b=$(git merge-base HEAD origin/main); mapfile -t F < <(git diff --name-only --diff-filter=d "$b" -- . ':!*.rs' ':!.jdi' ':!Cargo.lock' ':!apps/ddc-tray/tests/fixtures' ':!apps/ddc-tray/src/i18n/en.js' ':!apps/ddc-tray/src/i18n/pt-BR.js'); [ ${#F[@]} -gt 0 ] || ok=0; ! { git grep -InEi "$W" -- apps/ddc-tray/src apps/ddc-tray/tests apps/ddc-tray/scripts apps/ddc-tray/src-tauri/Cargo.toml crates/ddc-adapters/Cargo.toml crates/ddc-core/Cargo.toml README.md CHANGELOG.md docs "${F[@]}" ':!apps/ddc-tray/tests/fixtures' ':!apps/ddc-tray/src/i18n/en.js' ':!apps/ddc-tray/src/i18n/pt-BR.js'; git grep -InEi "$C" -- apps/ddc-tray/src/i18n/en.js apps/ddc-tray/src/i18n/pt-BR.js; } | sed -E "$STRIP" | grep -Eiq "$W|$C" || ok=0; ! git grep -qF '/*' -- apps/ddc-tray/src/i18n/en.js apps/ddc-tray/src/i18n/pt-BR.js || ok=0; [ $ok = 1 ] && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- Teste real PC + notebook Linux + switch USB real: apertar o botão e o monitor trocar junto nos dois sentidos (PC deixa -> DP2; notebook deixa -> DP1).
- Aprender com o switch real identifica teclado e mouse e não inclui o hub do switch.
- Leitura humana de `docs/usb-switch-follow.md`: as permissões i2c e a config servem para as duas máquinas.
- Aparência do texto "em outra entrada" e dos itens de menu no KDE/GNOME reais.

## Notes
- **`<FREEZE>`:** o orquestrador o troca, depois de cada passada do doer, por este fragmento com os ids reais de `git rev-parse HEAD:<caminho>`: `for t in crates/ddc-core=<id> crates/ddc-adapters=<id> apps/ddc-tray/src-tauri=<id> apps/ddc-tray/src=<id> apps/ddc-tray/tests=<id> apps/ddc-tray/scripts=<id> docs=<id> Cargo.toml=<id> Cargo.lock=<id>; do [ "$(git rev-parse "HEAD:${t%%=*}")" = "${t#*=}" ] || ok=0; done; git diff --quiet HEAD -- crates apps docs Cargo.toml Cargo.lock || ok=0; [ -z "$(git ls-files --others --exclude-standard -- crates apps docs)" ] || ok=0;`
  - Os ids dependem só do conteúdo e continuam valendo depois do squash-merge.
  - Os testes ficam dentro dessas árvores, então ficam congelados também.
- **Contrato:** os nomes de teste e as linhas do smoke são o contrato do planner. Mudar qualquer um exige editar este CONTEXT.
- **Smoke:** `smoke-follow-private.sh` segue o padrão de `smoke-autostart-private.sh`, `private-bus.sh` e `sni-dbusmenu.py`.
  - Sai ≠ 0 em qualquer condição violada, inclusive sem `python3-gobject` e com binário que sai cedo.
  - O `smoke-sni.sh` e o `smoke-autostart-private.sh` não mudam.
- **D-11:** ajustes do orquestrador antes do plano.
