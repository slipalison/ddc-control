# Phase 10: USB switch follow — Plan  (slug: usb-switch-follow)

## Goal
O monitor segue o switch USB (KVM por software, só Linux, só bandeja): quando os dispositivos aprendidos SAEM desta máquina, ela manda `0x60` para a entrada da outra. Inclui aprender, config persistida, ligar/desligar pelo menu e o texto "em outra entrada" para monitor com EDID e DDC mudo. PC e notebook Linux.

## Locked decisions (from CONTEXT.md)
- -2: `UsbDeviceId`, port `UsbPresence`, `Follower` e `learn` puros em `ddc-core` (só `thiserror`); `usb_sysfs` em `ddc-adapters` com raiz injetada; config, laço e menu em `apps/ddc-tray`.
- -3: polling a 500 ms, debounce de 3 leituras; id `vid:pid[:serial]`, porta fora; hubs (`bDeviceClass` 09) ignorados; dispara quando TODOS saem, uma vez; rearma quando um volta; chegada e estado inicial nunca disparam; relógio injetável.
- -4: `$XDG_CONFIG_HOME/ddc-control/usb-follow.json` (fallback `~/.config`), `version` 1; desconhecida/JSON inválido = não configurado, reportado, nunca sobrescrito sem ação do usuário; escrita atômica; aprender nunca liga.
- -5: menu Linux depois de "Start with system": check "Follow USB switch", "Learn USB switch" (janela de 30 s), checks `0x0F`/`0x10`/`0x11`/`0x12`, linha de resumo; ligar sem config completa não liga e reporta; monitor = alvo dos atalhos no momento do aprender.
- -6: `header.silent` = `{label} (on another input)` / `{label} (em outra entrada)`; `hint.ddc` cobre "outra entrada OU DDC/CI desligado" (textos literais da D-6).
- -7: uma tentativa de `0x60`, fora da thread da UI e fora do laço de poll; erro só em `crate::report`; o follower rearma pela D-3 seja qual for o resultado.
- -8: consentimento = ligar o follow com config completa; `Confirm::Yes` só com `enabled`, só no `monitor_id` e na `target_input` gravados; aprender nunca liga nem escreve.
- -9: fakes em toda ponta (sysfs tempdir, `InMemoryMonitorBackend`, relógio, config tempdir); suíte passa com `/sys` coberto; `DDC_TRAY_USB_ROOT` só com `DDC_TRAY_FAKE=1`.
- -10: Windows, macOS, CLI, hotkeys, udev netlink etc. fora. -11: Verify autocontidos, prefixos exclusivos, `<FREEZE>`, smoke release, linha `ddc-tray: follow: switching <monitor-id> to input 0x<hh>`.
- Herdadas: D-1, D-2, D-4 (enumerate não sonda), D-12 (nenhum teste escreve em monitor real), D-2026-09-30-input-switch-autostart-3 (assentamento de 3 s no adapter).

## Tasks

Specialist de todas: `jdi-doer-ddc-control` (single-stack, glob `**/*`). 1 task = 1 commit, testes primeiro (vistos vermelhos, saída no SUMMARY). Gates por task: `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`; em T-1/T-2 também `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu`; em T-6 `cd apps/ddc-tray && npm run test:unit`. Os nomes de teste e as linhas do smoke do CONTEXT são CONTRATO: usar exatamente. Regras transversais: sem `unsafe`/`env::set_var`; nenhum teste toca o `/sys` real, `/dev/i2c-*` ou escreve em monitor real (D-9, D-12); nenhuma palavra `todo`/`fixme` (nem o pt-BR "todos") em `.rs` ou arquivo não-Rust novo; constantes nomeadas com uma linha WHY; `Cargo.lock` vai com a task que o muda.

### Wave 1

#### T-1: núcleo puro do follow em `ddc-core`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/usb.rs` (novo), `crates/ddc-core/src/domain/mod.rs`, `crates/ddc-core/src/ports/usb_presence.rs` (novo), `crates/ddc-core/src/ports/mod.rs`, `crates/ddc-core/src/app/usb_follow.rs` (novo), `crates/ddc-core/src/app/usb_follow/tests.rs` (novo), `crates/ddc-core/src/app/usb_learn.rs` (novo), `crates/ddc-core/src/app/mod.rs`
- **Acceptance:**
  - Domínio e port: `UsbDeviceId { vendor: u16, product: u16, serial: Option<String> }` com `Display`/`FromStr` em `vvvv:pppp[:serial]` (hex minúsculo de 4 dígitos, ida e volta, serial vazio = `None`), `Ord` (vai em `BTreeSet`) e `vendor_product()`. Erro próprio `UsbPresenceError` (thiserror). Port `UsbPresence: Send + Sync` com `present() -> Result<BTreeSet<UsbDeviceId>, UsbPresenceError>`, só a trait. Testes livres em `domain::usb::tests`.
  - `app::usb_follow`: `POLL_INTERVAL` (500 ms), `DEBOUNCE_POLLS` (3) e `LEARN_WINDOW` (30 s). `Follow::{On, Off}`, sem bool. `Follower::new(learned)` + `observe(&present, Follow) -> Option<Fire>` em tempo lógico (1 chamada = 1 leitura). Dispara na borda "todos ausentes por `DEBOUNCE_POLLS` leituras", uma vez, e rearma quando um volta. Começa esperando ver um presente. Com `Off`, acompanha as bordas e consome a saída sem disparar. Conjunto vazio nunca dispara. Expõe o estado debounced (ex.: `Waiting`/`Armed`/`Spent`) para o diagnóstico de T-4. `learn(before, after) = before − after`.
  - Exatamente os 8 testes de `app::usb_follow::tests::` da linha 1 do DoD, e nenhum outro com esse prefixo. `app::usb_learn::LearnSession` é pura: snapshot A e `observe(&present)`, que devolve `Watching`; `Learned(set)` quando o mesmo `learn(A, presente)` não vazio dura `DEBOUNCE_POLLS` leituras; ou `Expired` depois de `LEARN_WINDOW / POLL_INTERVAL` leituras. Testes próprios em `app::usb_learn::tests`, outra unidade, fora do prefixo do DoD.
  - `crates/ddc-core/Cargo.toml` intacto (D-2). Nada de `std::fs`, `std::thread`, `Instant` ou `cfg(target_os)`. `///` em todo item `pub`.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-core --locked --lib -- app::usb_follow:: app::usb_learn:: domain::usb::` + DoD Verify linha 1
- **Commit:** `feat(usb-switch-follow): pure USB follower and learning in the core`
- **Status:** pending

### Wave 2 (parallel-eligible)

#### T-2: adapter `usb_sysfs` com raiz injetada
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/usb_sysfs.rs` (novo), `crates/ddc-adapters/src/usb_sysfs/tests.rs` (novo), `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/Cargo.toml` (dev-dep `tempfile`, já no lock), `Cargo.lock`
- **Acceptance:**
  - `SysfsUsbPresence::new(root)` implementa `UsbPresence`. É exportado em `lib.rs` fora da feature `ddc-hi` e sem `cfg` de SO, então os testes rodam também no runner Windows.
  - Para cada entrada da raiz, lê `idVendor`/`idProduct` (hex, com `trim`) e o `serial` opcional (com `trim`; vazio = sem serial). Pula entradas sem `idVendor` (interfaces `1-1:1.0`), com `bDeviceClass` `09` (hubs e root hubs) ou com valor malformado.
  - Raiz ausente (`NotFound`) = `Ok(vazio)`; outro erro na raiz = `Err(UsbPresenceError)`. Nenhum caminho `/sys` no crate: a raiz real fica só na composição (D-2).
  - Exatamente os 5 testes de `usb_sysfs::tests::` da linha 2 do DoD, em tempdir. O último apaga o diretório do dispositivo entre duas chamadas de `present()`.
- **Dependencies:** T-1
- **Test:** `cargo test -p ddc-adapters --locked --lib -- usb_sysfs::` + DoD Verify linha 2
- **Commit:** `feat(usb-switch-follow): sysfs USB presence adapter`
- **Status:** pending

#### T-3: config persistida `follow_config`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/follow_config.rs` (novo), `apps/ddc-tray/src-tauri/src/lib.rs` (`pub mod`)
- **Acceptance:**
  - `FollowConfig { enabled, devices: BTreeSet<UsbDeviceId>, monitor_id: Option<MonitorId>, target_input: Option<u8> }`. O DTO serde fica só neste módulo, no formato da D-4: `version: 1`, `target_input` decimal, `monitor_id` e `target_input` podem faltar. `FOLLOW_INPUTS = [0x0F, 0x10, 0x11, 0x12]`.
  - `config_path(xdg_config_home, home)` é pura e devolve, em ordem: `$XDG_CONFIG_HOME/ddc-control/usb-follow.json` se a variável é absoluta e não vazia; `$HOME/.config/ddc-control/usb-follow.json`; ou `None`.
  - `ConfigStore` tem o diretório injetável. `load()` devolve `Missing` (padrão desligado), `Loaded` ou `NotConfigured(motivo)`. `NotConfigured` cobre versão desconhecida, JSON inválido, id inválido e `target_input` fora de `FOLLOW_INPUTS`. `load` nunca escreve. `save` é atômico (temporário no mesmo diretório + `rename`) e cria o diretório.
  - Transições puras: `with_learned(devices, monitor)` grava sempre `enabled: false` (aprender nunca liga; re-aprender com o follow ligado desliga, porque o monitor pode mudar e o consentimento vale para o gravado, D-8); `with_target_input(code)`; `enabled_toggled()` → `Err(Incomplete)` com o que falta (dispositivos, monitor ou entrada, D-5).
  - Exatamente os 5 testes de `follow_config::tests::` da linha 4 do DoD, em tempdir. O teste de versão desconhecida compara os bytes do arquivo antes e depois.
- **Dependencies:** T-1
- **Test:** `cargo test -p ddc-tray --locked --lib -- follow_config::` + DoD Verify linha 4
- **Commit:** `feat(usb-switch-follow): persisted USB follow config`
- **Status:** pending

### Wave 3

#### T-4: laço do follow fora da thread da UI
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/follow.rs` (novo), `apps/ddc-tray/src-tauri/src/lib.rs` (`pub mod`)
- **Acceptance:**
  - Estado compartilhado `SharedFollow`: config em memória, `ConfigStore` e pedido de aprender com o `MonitorId`. Cada mudança é gravada antes de valer em memória; o erro volta ao chamador e T-5 o reporta.
  - `FollowLoop` sobre `UsbPresence` + `SharedOsd`. A cada `tick()`, lê a presença e recria o `Follower` quando o conjunto aprendido muda. Com um pedido de aprender, tira o snapshot A da `LearnSession` e diz em `diagnose` `follow: learning started`. Durante a janela, o follower conta como `Off` (aprender nunca escreve, D-8). No fim, grava `with_learned` com o monitor do pedido; `Expired` → `report`, nada gravado.
  - No `Fire`, só com `enabled` e config completa no mesmo snapshot: emite `switch_line(id, input)` = `ddc-tray: follow: switching <id> to input 0x<hh>` (`{:02x}`) pelo sink de anúncio (produção: `eprintln!`, sempre, sem `DDC_TRAY_DEBUG`). Depois despacha UMA `set_feature(monitor_id, VcpCode::INPUT_SOURCE, alvo, Confirm::Yes)` por executor injetado (produção: thread própria, e o poll segue durante o assentamento de 3 s; teste: em linha). `Confirm::Yes` só existe nesse ramo; erro → `report`, sem retry (D-7).
  - As bordas do follower vão a `diagnose` (`follow: learned devices present` / `follow: learned devices absent`). Erro de leitura do sysfs → `report` uma vez por sequência de falhas, e o laço segue.
  - `run(clock, keep_going)` = `tick` + `clock.sleep(POLL_INTERVAL)`. `Clock` é injetável e o fake não dorme. `spawn(...)` sobe uma `std::thread` nomeada (`usb-follow`), nunca na thread da UI.
  - Exatamente os 5 testes de `follow::tests::` da linha 3 do DoD, via `run` com relógio fake, `SysfsUsbPresence` numa raiz fake, `InMemoryMonitorBackend` e config em tempdir. O log de escritas é `[WriteVcp(RTK, 0x60, 0x10)]` e o anúncio é a linha do DoD; chegada e follow desligado dão `[]`. `with_vcp_failure(INPUT_SOURCE, …)` vai a `report`, há mais leituras depois do erro e novo disparo após um retorno. Com 2 monitores com `0x60`, toda escrita é `(configurado, 0x60, alvo)`.
- **Dependencies:** T-1, T-2, T-3
- **Test:** `cargo test -p ddc-tray --locked --lib -- follow::` + DoD Verify linha 3
- **Commit:** `feat(usb-switch-follow): follow loop off the UI thread`
- **Status:** pending

### Wave 4 (parallel-eligible)

#### T-5: itens do follow no menu Linux + fiação no `run()`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src-tauri/src/menu.rs`, `apps/ddc-tray/src-tauri/src/i18n.rs`, `apps/ddc-tray/src-tauri/src/tray.rs`, `apps/ddc-tray/src-tauri/src/tray/status_item.rs`, `apps/ddc-tray/src-tauri/src/tray/notification_area.rs`, `apps/ddc-tray/src-tauri/src/lib.rs`
- **Acceptance:**
  - Rótulos: `Labels.follow` = "Follow USB switch"/"Seguir o switch USB"; `Labels.learn` = "Learn USB switch"/"Aprender o switch USB"; resumo `Learned: <vendor:product, …>`/`Aprendidos: …`, ou `Learned: none`/`Aprendidos: nenhum`; `input_label` = `DisplayPort 1`, `DisplayPort 2`, `HDMI 1`, `HDMI 2`.
  - Ações e entradas: `MenuAction::{Follow, Learn, FollowInput(u8)}`, ids `follow-usb`, `learn-usb`, `follow-input-0f`…; `from_id` recusa código fora de `FOLLOW_INPUTS`. `MenuEntry::Note` é uma linha desabilitada.
  - Ordem no Linux: Open panel · sep · 5 brilhos · sep · Start with system · sep · Follow USB switch (check = `enabled`) · Learn USB switch · 4 checks de entrada (marcado o gravado) · resumo · sep · Quit. Windows inalterado.
  - Testes novos: `menu::tests::follow_check_is_marked_exactly_when_enabled`, `menu::tests::follow_items_are_in_the_linux_menu`, `menu::tests::follow_items_are_not_in_the_windows_menu`, `menu::tests::follow_ids_round_trip_through_from_id` e `i18n::tests::follow_labels_are_literal_in_en_and_pt_br` (literais da linha 5). Nenhum teste antigo começa com `follow_`.
  - Testes atualizados: as listas exatas do Linux, ids únicos 8 → 14, `an_id_the_menu_never_shows_maps_to_nothing`, os 3 de rótulos (exatos e não vazios) e o de checkmarks de `status_item`. `menu::`/`i18n::` com `0 failed; 0 ignored`.
  - `tray.rs`: todo clique roda fora da thread do menu e termina em `platform::follow_changed`. Follow alterna numa thread de bloqueio; config incompleta → `report` com o motivo, e o follow fica desligado. Learn pega `panel::shortcut_target` via `on_blocking_thread` e faz o pedido de aprender. FollowInput grava a entrada. Sem `SharedFollow`, o menu lê desligado/nenhum e o clique só reporta.
  - Testes livres em `tray::tests` cobrem config incompleta, config completa (grava `enabled: true`), entrada, monitor do aprender e aprender de ponta a ponta pelo laço numa raiz fake.
  - `lib.rs`: `usb_root` é pura e lê `DDC_TRAY_USB_ROOT` só com `DDC_TRAY_FAKE=1`, senão `/sys/bus/usb/devices` (teste em `switch_tests`). Só no Linux, o `setup` carrega a config (`NotConfigured` → `report`, arquivo intocado), registra `SharedFollow` antes de `tray::install` e sobe `follow::spawn` com `SysfsUsbPresence`, o core do `AppState` e o relógio real.
  - `notification_area.rs` só ganha os braços exaustivos e um `follow_changed` vazio; a prova é o runner windows-latest.
- **Dependencies:** T-4
- **Test:** `cargo test -p ddc-tray --locked --lib` + DoD Verify linha 5; `smoke-sni-private.sh` e `smoke-autostart-private.sh` seguem verdes, sem edição
- **Commit:** `feat(usb-switch-follow): follow items in the Linux tray menu`
- **Status:** pending

#### T-6: popup "em outra entrada" (D-6)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/src/view-model.js`, `apps/ddc-tray/src/app.js`, `apps/ddc-tray/src/i18n/en.js`, `apps/ddc-tray/src/i18n/pt-BR.js`, `apps/ddc-tray/tests/ui/view-model.test.mjs`, `apps/ddc-tray/tests/ui/i18n-parity.test.mjs` (+ `tests/ui/i18n-html.test.mjs` só se o inventário de sites de texto mudar de arquivo, sem afrouxar)
- **Acceptance:**
  - Os valores da D-6, palavra por palavra, entram nas chaves de sempre (`header.silent`, `hint.ddc`), em en e pt-BR. Funções puras novas em `view-model.js` (ex.: `pickerOptions(picker, t)`, `messageTip(state, mute, t)`) são usadas por `paintPicker`/`paintMessage` de `app.js`, sem lógica duplicada nem string literal.
  - Teste top-level `a monitor with a readable EDID and no DDC answer reads on another input in en and pt-BR` em `view-model.test.mjs`. Um monitor com rótulo de EDID em `silentIds` vira `RTK QHD HDR (on another input)` / `RTK QHD HDR (em outra entrada)`, com a dica literal inteira nos dois idiomas. O esperado do pseudo-locale de `header.silent` em `i18n-parity.test.mjs` é atualizado.
  - `node --test --test-reporter=tap tests/ui/*.test.mjs` dá `# fail 0`, `# cancelled 0`, `# skipped 0` e `# todo 0`. `npx playwright test` inteiro verde (light e dark), sem `test.skip`/`fixme`.
- **Dependencies:** none (fica na wave 4 pela ordem do orquestrador; arquivos disjuntos de T-5)
- **Test:** `cd apps/ddc-tray && npm run test:unit` + DoD Verify linha 6
- **Commit:** `feat(usb-switch-follow): a silent monitor reads on another input`
- **Status:** pending

### Wave 5 (parallel-eligible)

#### T-7: documentação de instalação + README + CHANGELOG
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `docs/usb-switch-follow.md` (novo), `apps/ddc-tray/src-tauri/src/docs.rs` (novo), `apps/ddc-tray/src-tauri/src/lib.rs` (`#[cfg(test)] mod docs;`), `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - `docs.rs` lê o doc com `include_str!("../../../../docs/usb-switch-follow.md")` e tem exatamente 2 testes, vistos vermelhos antes do doc. `docs::tests::usb_follow_doc_names_i2c_setup_for_both_machines` exige `i2c-dev`, `/dev/i2c-`, `modules-load.d`, `XDG_CONFIG_HOME`, `usb-follow.json`, `PC`, `notebook`, `0x0F` e `0x10`. `docs::tests::usb_follow_doc_states_the_consent_of_the_dangerous_write` exige `Follow USB switch`, `consent`, `Dangerous`, `0x60` e a frase `Learning never turns follow on and never writes to the monitor.`
  - Doc em inglês. Cobre `i2c-dev` por `modules-load.d` e o acesso a `/dev/i2c-*` nas DUAS máquinas (link para `docs/linux-ddc-setup.md`); o aprender (30 s, hubs ignorados); a entrada de destino de cada máquina (PC deixa → `0x10`, notebook deixa → `0x0F`); a config; o que ligar o follow consente (só o monitor e a entrada gravados, D-8); DDC mudo = 1 tentativa (D-7); os limites (D-10).
  - README (seção do tray e limitações) e CHANGELOG `[Unreleased]` (`Added`: follow; `Changed`: texto do monitor mudo).
- **Dependencies:** T-5, T-6
- **Test:** `cargo test -p ddc-tray --locked --lib -- docs::` + DoD Verify linhas 7 e 10
- **Commit:** `docs(usb-switch-follow): set up the USB follow on both machines`
- **Status:** pending

#### T-8: smoke de ponta a ponta em barramento privado
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `apps/ddc-tray/scripts/smoke-follow-private.sh` (novo), `apps/ddc-tray/scripts/sni-dbusmenu.py` (só se precisar de um subcomando novo; os existentes não mudam)
- **Acceptance:**
  - Segue o padrão de `smoke-autostart-private.sh`: `set -euo pipefail`, `source private-bus.sh` com `PRIVATE_BUS_TAG=smoke-follow-private`, sandbox em `mktemp -d` com `HOME` e `XDG_CONFIG_HOME=$sandbox/.config`. A raiz USB fake fica no sandbox: teclado e mouse (um com `serial`) e o hub com `bDeviceClass 09`. Roda com `DDC_TRAY_FAKE=1`, `DDC_TRAY_DEBUG=1`, `DDC_TRAY_USB_ROOT` e locale inglês. App e watcher sempre param na saída, e o sandbox é removido.
  - Imprime exatamente as 10 linhas da linha 8 do DoD, na ordem do roteiro. Só remove os dispositivos depois de `ddc-tray: follow: learning started`. Lê a config com `python3`/`json`: o conjunto exato dos 2 ids sem o hub, `monitor_id` `RTK-RTK-QHD-HDR-01010101`, `target_input` 16 e `enabled`.
  - Cada remoção e cada devolução espera a borda de diagnóstico (`ddc-tray: follow: learned devices absent`/`present`), como prova de vida do laço. Os passos "nada troca" só passam depois de uma janela de silêncio nomeada (> debounce + 1 poll) em que `ddc-tray: follow: switching RTK-RTK-QHD-HDR-01010101 to input 0x10` ainda aparece 1 vez.
  - Sai ≠ 0 se qualquer condição falha, se falta `python3-gobject`, se o binário sai cedo ou se o stderr tem `panicked` ou um `could not` do follow. Prova negativa no SUMMARY: com `/bin/true`, sai ≠ 0 sem a linha OK. `~/.config/autostart` e `~/.config/ddc-control` reais ficam idênticos; `smoke-sni.sh` e `smoke-autostart-private.sh` ficam sem diff.
- **Dependencies:** T-5
- **Test:** DoD Verify linha 8 (`cargo build -p ddc-tray --release --locked -q && bash apps/ddc-tray/scripts/smoke-follow-private.sh target/release/ddc-tray`)
- **Commit:** `test(usb-switch-follow): smoke the USB follow on a private bus`
- **Status:** pending

## Execution
- Total tasks: 8 · Waves: 5 · Estimated parallel speedup: 1,6x (o doer roda T-1..T-8 em sequência).
- Ordem do orquestrador mantida (core → adapter e config → laço e menu → popup, docs e smoke). Laço e menu viraram waves 3 e 4 porque o menu usa o `SharedFollow` do laço e a fiação do `run()` precisa dos dois.
- DoD → task: linha 1 → T-1; 2 → T-2; 3 → T-4; 4 → T-3; 5 → T-5; 6 → T-6; 7 → T-7; 8 → T-8, sobre a fiação de T-5; 9 (bwrap) → T-1 a T-5 e T-7; 10 (TODO não-Rust) → T-6, T-7 e T-8.
- Deferred to PR review (hardware real, switch real, leitura humana do doc, KDE/GNOME) não vira task.

## Files modified (all tasks)
- `crates/ddc-core/src/domain/{mod,usb}.rs`, `crates/ddc-core/src/ports/{mod,usb_presence}.rs`, `crates/ddc-core/src/app/{mod,usb_follow,usb_learn}.rs`, `crates/ddc-core/src/app/usb_follow/tests.rs`
- `crates/ddc-adapters/src/{lib,usb_sysfs}.rs`, `crates/ddc-adapters/src/usb_sysfs/tests.rs`, `crates/ddc-adapters/Cargo.toml`, `Cargo.lock`
- `apps/ddc-tray/src-tauri/src/{follow,follow_config,docs,lib,menu,i18n,tray}.rs`, `apps/ddc-tray/src-tauri/src/tray/{status_item,notification_area}.rs`
- `apps/ddc-tray/src/{view-model,app}.js`, `apps/ddc-tray/src/i18n/{en,pt-BR}.js`, `apps/ddc-tray/tests/ui/{view-model,i18n-parity}.test.mjs` (+ `i18n-html.test.mjs` se preciso)
- `apps/ddc-tray/scripts/smoke-follow-private.sh`, `apps/ddc-tray/scripts/sni-dbusmenu.py` (se preciso)
- `docs/usb-switch-follow.md`, `README.md`, `CHANGELOG.md`

## Test requirements
- Rust: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; cross-check Linux em T-1/T-2.
- UI: `cd apps/ddc-tray && npm run test:unit` e `npx playwright test` (light + dark).
- Minimum coverage: 80% linhas (`cargo llvm-cov --workspace --locked --fail-under-lines 80`) + piso de UI do pipeline.
- Os 10 `Verify:` do DoD do CONTEXT.md, com `<FREEZE>` preenchido pelo orquestrador, rodados pelo reviewer.

## Risks
- Corrida no aprender: o snapshot A é tirado no primeiro `tick` depois do clique (≤ 500 ms + enumerate). O smoke espera `learning started` antes de remover; no uso real, o humano leva mais que isso para apertar o switch.
- Dois dispositivos iguais sem `serial` viram um id só (`BTreeSet`), e um receptor sem fio aprende como um dispositivo. Os dois casos são aceitáveis para "TODOS saíram"; o doc deve dizer isso.
- No adapter real, a escrita de `0x60` bloqueia até 3 s (assentamento). Por isso ela vai para uma thread própria; um pânico ali não derruba o laço, mas aparece como `panicked` no stderr.
- `notification_area.rs` (só Windows) só é provado pela compilação e pelos testes do runner windows-latest do CI.
- O smoke depende de tempo (debounce de 1,5 s). As janelas são nomeadas e esperam bordas de diagnóstico, nunca só `sleep`.
- O formato do `/sys` real varia por kernel (não afirmado). O adapter pula entradas malformadas sem falhar o conjunto.

## Emendas (iteração 1, D-2026-10-02-usb-switch-follow-12)
- **T-5, `usb_root`:** com `DDC_TRAY_FAKE=1` e sem `DDC_TRAY_USB_ROOT`, não há raiz e o laço não sobe; o `/sys` real só é usado fora da simulação (D-12a).
- **Arquivos fora da lista original:**
  - `docs/hardware-validation.md` (rótulo "(on another input)", T-7);
  - `apps/ddc-tray/src-tauri/src/tray/notification_area.rs` (usa `follow_config(app)`, T-5);
  - módulos de teste livres fora dos prefixos do DoD (`usb_sysfs::robustness_tests`, `follow_config::rule_tests`, `follow::learning_tests`, `switch_tests`).
- **Commits extras:** `e43fbe4` (clippy do Windows) e os reforços de teste `4625553`, `d49544e` e `df69902`, que nasceram de furos achados por mutação.
- **DoD:**
  - a linha 6 exige também a suíte Playwright inteira (D-12b);
  - a linha 8 descreve a saída do hub junto com teclado e mouse (D-12c);
  - o congelamento foi preenchido com os ids de `df69902`.
