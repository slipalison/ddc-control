# Phase 10: USB switch follow — Summary  (slug: usb-switch-follow)

**Status:** complete (iteração 1 do loop)

Escrito pelo orquestrador a partir do relatório do doer: o harness nega escrita de `.md` ao subagente.

**Tasks:** 8/8 complete, 0 blocked

## Executed tasks
- T-1: `fd684fc` feat — `UsbDeviceId` (`vvvv:pppp[:serial]`), port `UsbPresence`, `Follower` (dispara uma vez quando TODOS os aprendidos ficam 3 leituras ausentes, rearma quando um volta, nunca na chegada nem no estado inicial), `learn` e `LearnSession` (janela de 30 s). Core só com `thiserror`; sem `std::fs`, `std::thread`, `Instant` ou `cfg`.
- T-2: `fb00966` feat — `SysfsUsbPresence` com raiz injetada: pula interfaces, hubs (`bDeviceClass` 09) e ids malformados; raiz ausente = vazio; outro erro = `Err`. Dev-dep `tempfile`, já no lock.
- T-3: `7890d6b` feat — `follow_config`: JSON `version` 1 em `$XDG_CONFIG_HOME/ddc-control/usb-follow.json` (fallback `~/.config`). Ausente = desligado; versão desconhecida, JSON inválido ou valor inválido = `NotConfigured`, sem escrever nada. Gravação atômica (temporário + `rename`). Aprender sempre grava `enabled: false`; ligar exige dispositivos, monitor e entrada.
- T-4: `475d9a2` feat — `follow.rs`: `FollowLoop` (lê a cada 500 ms numa thread `usb-follow`; no disparo imprime `ddc-tray: follow: switching <id> to input 0x<hh>` e despacha UMA `set_feature(0x60, Confirm::Yes)` numa thread própria; erro só no `report`, sem retry). `FollowState`/`SharedFollow` grava cada mudança antes de valer. Relógio, executor e saída são injetáveis.
- (extra) `e43fbe4` fix — imports sem uso no Windows no teste só-unix do `usb_sysfs` (o clippy do `rust-windows` reprovaria).
- T-5: `84e08cd` feat — menu Linux: check "Follow USB switch", "Learn USB switch", 4 checks de entrada e a linha "Learned: …"; i18n en/pt-BR. Cliques fora da thread do menu; aprender usa o alvo dos atalhos. `run()`/`setup` carrega a config, registra `SharedFollow` e sobe o laço; `usb_root` pura. O menu do Windows não muda.
- T-6: `63c2854` feat — `header.silent` = `{label} (on another input)` / `{label} (em outra entrada)` e `hint.ddc` com os textos literais da D-6; `pickerOptions` e `messageTip` no view-model, usados pelo `app.js`.
- T-7: `68703c1` docs — `docs/usb-switch-follow.md`, `docs.rs` (2 testes), README, CHANGELOG (e uma linha do `docs/hardware-validation.md`, ver ressalvas).
- T-8: `531d3c4` test — `scripts/smoke-follow-private.sh`.
- (reforços pós-mutação) `4625553` test (debounce de 3 leituras fixado nos testes do core), `d49544e` test (linha de troca exigida sem `DDC_TRAY_DEBUG`), `df69902` test (o hub do switch sai junto no smoke).

## Testes vistos vermelhos antes do código
- T-1 `app::usb_follow::` (stub `observe → None`, `learn → ∅`): `test result: FAILED. 1 passed; 7 failed` (o que passou é `all_absent_at_start_does_not_fire`, trivial com o stub).
- T-1 `app::usb_learn::`: `FAILED. 2 passed; 4 failed`.
- T-1 `domain::usb::` foi escrito junto com o código. Vermelho mostrado depois por mutação (hex base 10 e serial vazio mantido): `FAILED. 4 passed; 2 failed`.
- T-2 `usb_sysfs::` (stub vazio): `FAILED. 1 passed; 4 failed`.
- T-3 `follow_config::` (stubs): `FAILED. 1 passed; 4 failed`.
- T-4 `follow::tests::` (`tick` vazio): `FAILED. 0 passed; 5 failed`.
- T-5 i18n: `left: "0x0F" right: "DisplayPort 1"`, `FAILED. 8 passed; 1 failed`. Menu: `FAILED`, 5 falhas (2 listas Linux exatas, ids únicos 8→14 e 2 testes novos). `follow_ids_round_trip_through_from_id` e `follow_items_are_not_in_the_windows_menu` passaram de primeira: o enum e o `from_id` tinham de existir para compilar, e o do Windows é um teste de guarda.
- T-6 `node --test`: `not ok … view-model.test.mjs` (importa funções que ainda não existiam) + `not ok … pseudo-locale wraps…`, `# fail 2`.
- T-7 `docs::` (guia vazio): 2 falhas, com `left` listando todos os literais ausentes.
- T-8 provas negativas: com `/bin/true` → `FAIL: the app exited before registering a tray item with status 0`, saída 1; com `gi` quebrado no `PYTHONPATH` → `FAIL: python3 cannot load the gi module with Gio 2.0 (package python3-gobject)`, saída 1.
- Escritos depois do código, sem vermelho próprio: os testes livres de `tray::tests`, `status_item::tests`, `switch_tests`, `follow_config::rule_tests`, `follow::learning_tests` e `usb_sysfs::robustness_tests`.

## Mutações (produção; cada uma revertida por `git checkout`, com `git diff --quiet` conferido)
Verify rodado como na seção de gates. "vermelho" = sem `OK` e saída 1.
| Linha | Mutação | Resultado real |
|---|---|---|
| 1 | follower dispara na chegada | vermelho; 5 testes `app::usb_follow::tests` FAILED |
| 1 | `DEBOUNCE_POLLS = 1` | **antes de `4625553`: `OK` (furo)**; depois: vermelho (`absence_shorter_than_debounce_does_not_fire`, `fires_once…` FAILED) |
| 1 | `DEBOUNCE_POLLS = 4` / `POLL_INTERVAL = 2 s` | vermelho (3 e 1 testes FAILED) |
| 2 | não pula hubs | vermelho; `skips_hubs` FAILED |
| 2 | ignora o serial / raiz ausente vira erro / sem `trim` | vermelho (3, 1 e 5 FAILED) |
| 3 | retenta a escrita uma vez no erro | vermelho; `silent_ddc_write_is_reported_and_loop_keeps_polling` FAILED |
| 3 | ignora `enabled` | vermelho; `disabled_writes_nothing` FAILED |
| 3 | escreve no primeiro monitor listado | vermelho; `writes_only_configured_monitor_and_code_0x60` FAILED |
| 3 | linha de troca só como diagnóstico | **antes de `d49544e`: `OK` (furo)**; depois: vermelho (`leaving_…exactly_once…` e `silent_ddc…` FAILED) |
| 4 | aceita versão desconhecida | vermelho; `unknown_version_is_not_configured_and_not_overwritten` FAILED |
| 4 | aprender mantém `enabled` / `load` sobrescreve o inválido / HOME antes do XDG / ausente = ligado | vermelho (1 FAILED em cada) |
| 5 | check marcado por "tem dispositivos" / itens também no Windows / `from_id` aceita qualquer código | vermelho (1 FAILED em cada) |
| 5 | pt-BR "Seguir switch USB" / "DisplayPort-2" | vermelho (2 e 1 FAILED) |
| 6 | texto antigo do mudo em `en.js` (`(no DDC/CI)`) | vermelho |
| 6 | dica antiga em `pt-BR.js` / `pickerOptions` sem marcar o mudo | vermelho |
| 6 | `app.js` sem `pickerOptions` | **`OK`**; só o Playwright cai (`fallback.spec`, `pseudo-locale.spec`). Ver ressalva 2 |
| 7 | sem a frase do aprender / sem `modules-load.d` | vermelho (1 FAILED em cada) |
| 8 | `run()` sem `follow::spawn` | vermelho: `FAIL: 'ddc-tray: follow: learning started' after the click on 'Learn USB switch' did not happen within 10 s` |
| 8 | clique no Follow sem efeito | vermelho: `FAIL: 'Follow USB switch' marked after the click did not happen within 10 s` |
| 8 | laço ignora `enabled` | vermelho: `FAIL: the app printed '…switching RTK-RTK-QHD-HDR-01010101 to input 0x10' 2 times, not 1, after the devices left with the follow off` |
| 8 | adapter não pula hubs | **antes de `df69902`: `OK` (furo)**; depois: `FAIL: a settings file with the keyboard and the mouse learned did not happen within 10 s` |
| 9 | adapter ignora a raiz injetada e lê `/sys/bus/usb/devices` (rodado só dentro do bwrap da linha 9) | vermelho; 6 testes `usb_sysfs` FAILED (`77 passed; 6 failed`) |
| 10 | `# TODO:` no smoke / `todo:` em comentário do `pt-BR.js` / `(FIXME)` no guia | vermelho nos três |

## Gates (estado final, HEAD `df69902`)
- `cargo fmt --all --check`: limpo.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: saída 0. Só aparece o aviso preexistente de future-incompat do `nom v3.2.1`.
- Clippy para Windows (`--target x86_64-pc-windows-gnu`, `ddc-tray`/`ddc-adapters`/`ddc-core`, com `windres` falso que só cria arquivos vazios; check não linka): saída 0. Foi ele que achou os problemas corrigidos em `e43fbe4` e o `dead_code` de `follow_config` resolvido no T-5.
- Cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu`: saída 0.
- `cargo test --workspace --locked` (em bwrap, i2c coberto): **484 passed, 0 failed, 9 ignored** (linha de base: 413). Por binário: ddc-adapters lib 83 (antes 76), caching_backend 15, real_monitor 0+7 ignored, ddc-cli lib 56, cli 32, ddc-core lib 64 (antes 44), monitor_control 36, ddc-tray lib 196 (antes 152), autostart_entry 2, rtk_qhd_hdr 0+2 ignored.
- `cargo llvm-cov --workspace --locked --fail-under-lines 80 --ignore-filename-regex '(main|build)\.rs$' --summary-only`: saída 0.
  `TOTAL  8756  949  89.16%  894  119  86.69%  5556  594  89.31%`
  Arquivos novos: `usb_follow.rs`, `usb_learn.rs`, `domain/usb.rs` (linhas), `usb_sysfs.rs`, `docs.rs` e `i18n.rs` em 100%; `follow.rs` 98,31%; `follow_config.rs` 98,46%; `menu.rs` 99,45%.
- `cd apps/ddc-tray && npm run test:unit`: `# tests 166 # pass 166 # fail 0 # cancelled 0 # skipped 0 # todo 0`; JS `all files 88.65`.
- `npx playwright test`: `152 passed`, `6 skipped`. Os pulados são as capturas, presas a `SCREENSHOTS=1`, como já era; e2e sem mudança.
- Smokes antigos, sem edição (`git diff origin/main` vazio): `smoke-sni-private.sh` → `OK — PID … registered its tray item, … never panicked`; `smoke-autostart-private.sh` → `OK — the Start with system item wrote the desktop entry…`.

## Os 10 Verify (extraídos por programa, regex `\*\*Verify:\*\* \x60([^\x60]*)\x60`, `<FREEZE>`→`:;`)
Rodados com `env -i` + `bash --noprofile --norc`, `LC_ALL=C.UTF-8`, `HOME`, `PATH` com `~/.cargo/bin` e `DISPLAY`/`WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`/`DBUS_SESSION_BUS_ADDRESS` do ambiente. As linhas 1–5, 7 e 8 rodaram dentro de bwrap com os 16 `/dev/i2c-*` cobertos; a 9 roda o próprio bwrap; a 6 e a 10 rodaram direto.
- 1: `OK` (exit 0) · 2: `OK` · 3: `OK` · 4: `OK` · 5: `OK` · 6: `OK` · 7: `OK` · 8: `OK` · 9: `OK` · 10: `OK`
- Saída da linha 8 (smoke):
  smoke-follow-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME, fake USB root / smoke-follow: the app serves the simulated monitor / smoke-follow: 'Follow USB switch' is an unmarked checkmark / smoke-follow: learning saw exactly the keyboard and the mouse leave, not the hub / smoke-follow: the config holds the 2 learned devices, the simulated monitor and input 0x10, with enabled false / smoke-follow: after the click 'Follow USB switch' is marked and the config says enabled / smoke-follow: leaving switched the simulated monitor to input 0x10 exactly once / smoke-follow: arriving switched nothing / smoke-follow: with follow off, leaving switched nothing / smoke-follow: OK — the tray followed the fake USB switch end to end
- Ids de árvore do HEAD para o `<FREEZE>`: crates/ddc-core=bfa4632b0d34338434ec85b8c88ed93444cf0edd crates/ddc-adapters=8f16c6d52d1d56826573ed9131466d77c1fcf5e9 apps/ddc-tray/src-tauri=8c5ca2eba0a323efa97c1e07d812d3f47262d8e7 apps/ddc-tray/src=4f59e791e4b11c1f5d7324d97de60531a3c6fdf4 apps/ddc-tray/tests=c5a66d2e97cccdbfafaa4e8f26aea5025dba325f apps/ddc-tray/scripts=356be58bc4afcc2354d48f29f967945a07df8e04 docs=6cadbee168c983cd2140dfe9235fc290b7f83b16 Cargo.toml=a32052f8e329beace662ee6f4cc1d0a33d13eca4 Cargo.lock=4359f59c978411b55f9093f12aa1dcaa45280fe8

## Files modified
- crates/ddc-core/src/{domain/mod.rs, domain/usb.rs, ports/mod.rs, ports/usb_presence.rs, app/mod.rs, app/usb_follow.rs, app/usb_follow/tests.rs, app/usb_learn.rs}
- crates/ddc-adapters/{Cargo.toml, src/lib.rs, src/usb_sysfs.rs, src/usb_sysfs/tests.rs}, Cargo.lock
- apps/ddc-tray/src-tauri/src/{follow.rs, follow_config.rs, docs.rs, lib.rs, menu.rs, i18n.rs, tray.rs, tray/status_item.rs, tray/notification_area.rs}
- apps/ddc-tray/src/{view-model.js, app.js, i18n/en.js, i18n/pt-BR.js}, apps/ddc-tray/tests/ui/{view-model,i18n-parity}.test.mjs
- apps/ddc-tray/scripts/smoke-follow-private.sh
- docs/usb-switch-follow.md, docs/hardware-validation.md (1 linha), README.md, CHANGELOG.md

## Tests
- Total: 484 Rust (+71) + 166 JS unit + 152 Playwright
- Passing: todos (9 Rust ignorados de hardware; 6 Playwright de capturas, preexistentes)
- Coverage: 89.31% linhas (`cargo llvm-cov --summary-only`, linha TOTAL acima)

## Ressalvas
1. **`usb_root` com `DDC_TRAY_FAKE=1` e sem `DDC_TRAY_USB_ROOT` → nenhuma raiz, e o laço não sobe.** O plano diz "senão `/sys/bus/usb/devices`". Li isso como "fora da simulação", porque senão o smoke-sni e o smoke-autostart (que rodam com FAKE=1 e sem raiz USB) passariam a ler o `/sys` real como raiz do follow, o que o orquestrador veda. Nesse caso o `SharedFollow` é registrado (o menu funciona) e a linha `follow: no USB tree in simulation…` aparece via `diagnose`. Teste: `switch_tests::the_follow_reads_the_tree_of_ddc_tray_usb_root_only_in_simulation`.
2. **Linha 6 do DoD tem um furo.** Com `app.js` sem usar `pickerOptions`, o Verify 6 continua `OK`; só o Playwright (`fallback.spec`, `pseudo-locale.spec`) cai. O "NÃO afirmado" da linha cita o Playwright, mas a frase "O popup mostra…" promete mais do que o Verify prova. Proposta (decisão do orquestrador): acrescentar ao Verify 6, antes do `cd ../..`: `p=$(npx playwright test tests/e2e/fallback.spec.mjs 2>&1) || ok=0; printf '%s\n' "$p" | grep -qE '^ +[0-9]+ passed' || ok=0; printf '%s\n' "$p" | grep -qE '[0-9]+ (failed|flaky|skipped)' && ok=0;`. A alternativa é trocar o texto para "o view-model do popup mostra…".
3. **Três furos achados pelas mutações, fechados em testes sem mudar nomes:**
   - debounce de 1 leitura passava a linha 1 → `4625553` (contagens literais e 500 ms × 3 = 1,5 s);
   - linha de troca rebaixada a diagnóstico passava a linha 3 → `d49544e` (o `Recorder` separa as linhas impressas sempre);
   - adapter que não pula hubs passava o smoke → `df69902` (o hub do switch sai junto, um root hub fica).
4. **Proposta de texto para a linha 8.** O smoke agora remove "teclado, mouse e o hub do switch" (como num switch real), além do que diz o texto "remove teclado e mouse da raiz". Sugestão: "clica 'Learn USB switch' e remove da raiz teclado, mouse e o hub do switch (um root hub fica), e o aprender registra os dois, sem o hub". As 10 linhas impressas não mudam.
5. **Arquivos e commits fora do plano:**
   - `docs/hardware-validation.md` (só "(no DDC/CI)" → "(on another input)", no commit da T-7);
   - o `notification_area.rs` usa `follow_config(app)` em vez de `FollowConfig::default()` (fecha um `dead_code` do Windows);
   - commits extras: `e43fbe4` (fix) e os 3 de reforço.
   - Testes livres fora dos prefixos do DoD ficam em módulos próprios (`usb_sysfs::robustness_tests`, `follow_config::rule_tests`, `follow::learning_tests`), então os conjuntos exatos seguem exatos.
   - `follow.rs` tem cerca de 1200 linhas, com testes inline, porque o plano só listava esse arquivo; o hook avisou do tamanho da T-4 e da T-5 (não bloqueia).
6. **Incidente no script de mutação.** Um `git checkout` dele reverteu uma vez o reforço do `Recorder` ainda não commitado. Reapliquei, commitei (`d49544e`) e repeti a mutação: mesmo vermelho. Nada mais foi perdido; working tree limpo.
7. **Windows não executado.** `notification_area.rs` e os testes compilam e passam no clippy só com `windres` falso; a prova de execução fica para o `windows-latest` do CI. O teste da raiz-arquivo é `#[cfg(unix)]`, e o nome da interface sem `:` no Windows usa `cfg!`.
8. **`Stderr` (3 delegações de uma linha) não é exercitado** nem pelos testes unitários nem pelo smoke, que roda com `DDC_TRAY_DEBUG=1`.
9. **Uso real.** Depois de uma troca bem-sucedida, o monitor deixa de responder a esta máquina, então a releitura do core falha e o stderr provavelmente terá `could not confirm the switch of …`. O guia explica isso; confirmação humana no teste PC + notebook (Deferred).
10. **Hardware (D-12).** Todo `cargo test` rodou em bwrap com os 16 `/dev/i2c-*` cobertos; o smoke usou `DDC_TRAY_FAKE=1` em barramento privado; a mutação da linha 9 só rodou dentro do bwrap com tmpfs em `/sys`. Nada de `--ignored` nem `DDC_HW_TESTS`. O `ddc-tray` do usuário (PID 33102) continua rodando; a listagem de `~/.config/autostart` é idêntica e `~/.config/ddc-control` não existia antes nem depois.

## Iteração 2 — critic e warnings da iteração 1 (D-13)
Escrito pelo orquestrador a partir do relatório do doer. HEAD do código: `02fdd12`, 6 commits, nenhum com `.jdi/`.

**Commits:**
- `53a6084` test: linha 6 (D-13a). Teste e2e `a silent monitor reads on another input in the picker` (`fallback.spec.mjs`), que confere o literal `LG TV SSCR2 (em outra entrada)` na opção e no botão do seletor, sem `t(...)`.
- `154824a` refactor: linha 3 (D-13b, W3). O `Stderr` real escreve num sink injetável (`io::stderr()` em produção). O novo teste `follow::stderr_tests::announce_prints_without_the_diagnostics_switch` trava o diagnóstico desligado e exige a linha exata no sink.
- `aac028a` test: W1. `follow::learning_tests::a_staggered_leave_while_learning_writes_nothing`.
- `2bcafb2` test: W2. `follow_config::rule_tests::a_save_that_cannot_write_its_temporary_file_keeps_the_old_bytes`.
- `6965ef7` refactor: W4. `i18n::input_label` passa a vir de `value_name(VcpCode::INPUT_SOURCE, code)`, com hífen trocado por espaço.
- `02fdd12` refactor: W6. `spawn_until` privado, e o teste "named thread" para e junta a thread. O `spawn` público só delega.

**Mutações** (todas revertidas):

| Mutação | Saída vermelha |
|---|---|
| `header.silent` pt-BR `(sem DDC/CI)` | e2e ✘ em light e dark (`Expected "LG TV SSCR2 (em outra entrada)"`) e unit `not ok 159` |
| `app.js` sem marcar o mudo no seletor | e2e ✘ nos dois temas (`Received "LG TV SSCR2"`) |
| `announce` só pelo `diagnose` | `stderr_tests` FAILED (`left: ""`), também com `DDC_TRAY_DEBUG=1` |
| Sem a guarda do aprender | `a_staggered_leave…` FAILED (`[WriteVcp(RTK, 0x60, 0x10)]`) |
| `save` direto no destino | `a_save_that_cannot_write…` FAILED |

**Gates:**
- fmt, clippy `-D warnings`, clippy Windows (com `windres` falso) e cross-check Linux saem 0.
- `cargo test --workspace --locked`: 487 passed, 0 failed, 9 ignored.
- `llvm-cov`: linhas 89,54%.
- `test:unit` 166/166; Playwright 154 passed, 6 skipped.

**Verify:**
- As linhas 1–9 falham no literal só pelos ids antigos de `apps/ddc-tray/src-tauri` e `apps/ddc-tray/tests`. Dão `OK` com o congelamento trocado por `:;` e com os ids novos.
- A linha 10 dá `OK`.
- O orquestrador recongelou.

**Ressalvas:**
- `the_system_clock_waits_for_real` ainda dorme 5 ms.
- `demo-data.js` e os ids de teste repetidos ficaram como estavam (resto do W4).
- O `spawn` público, que só delega, não é exercido por teste (3 linhas).
