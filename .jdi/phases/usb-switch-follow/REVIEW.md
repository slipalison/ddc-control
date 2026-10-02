# Phase 10: Review  (slug: usb-switch-follow)

**Verdict:** APPROVED_PENDING_MANUAL

Revisado no HEAD `43a831f` (código `04ba296..df69902`), com base `origin/main` = `420a3a1`. Ambiente: Rust 1.98.1, Linux (Fedora, kernel 7.2), bash.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` saiu 0. O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` saiu 0; aqui o host é Linux, então o check é nativo. Só aparece o aviso antigo de future-incompat do `nom v3.2.1`. |
| Tests | PASS | 484 passed, 0 failed, 9 ignored (hardware: `real_monitor` 7 e `rtk_qhd_hdr` 2). Rodei primeiro dentro do bwrap da prova de hardware e depois nativo, com o mesmo resultado. Por binário: ddc-adapters lib 83, caching_backend 15, ddc-cli lib 56, cli 32, ddc-core lib 64, monitor_control 36, ddc-tray lib 196, autostart_entry 2. A base anterior era 413; os +71 batem com os 71 `#[test]` novos do diff. |
| Coverage | PASS | 89.31% de linhas, limite 80%, linha TOTAL (`main.rs`/`build.rs` excluídos): `TOTAL 8756 949 89.16% 894 119 86.69% 5556 594 89.31%`, saída 0. Arquivos novos: `usb_follow.rs`, `usb_learn.rs`, `domain/usb.rs`, `usb_sysfs.rs`, `i18n.rs` e `docs.rs` a 100%; `follow.rs` 98.37%; `follow_config.rs` 98.46%; `menu.rs` 99.45%. Em `lib.rs` (53.64%), o que falta é o `start_follow` de composição; a lógica pura (`usb_root`, `follow_settings`) tem teste. |
| Lint | PASS | `cargo fmt --all --check` saiu 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` saiu 0. O clippy para `x86_64-pc-windows-gnu` (ddc-tray, ddc-adapters, ddc-core e ddc-cli), com o `windres` falso do doer, saiu 0. Nenhum `#[allow(...)]` novo. |
| Hexagonal/Safety/Hygiene | PASS (com WARN) | Todos os itens de 5.1 a 5.11 passaram; detalhes abaixo. Os WARN vêm de lacunas de teste achadas por mutação e de DRY. |
| Consistency | PASS (com WARN) | Todos os arquivos do plano aparecem no log; os extras estão registrados nas Emendas (D-12). Os 12 commits de código usam `<tipo>(usb-switch-follow):`. Nenhuma D-XX violada. O WARN é o `**Status:** pending` das tasks no PLAN. |
| UI Validation | PASS | `npm ci --ignore-scripts` + `npx playwright test` inteiro: 152 passed, 6 skipped, 0 failed, 0 flaky, em light e dark. Os 6 pulados são o `screenshots.spec`, preso a `SCREENSHOTS=1`, que já era assim. Inclui `fallback.spec` e `pseudo-locale.spec` lendo o rótulo do monitor mudo. |
| DoD | PASS_PENDING_MANUAL | 13/13 auto (10 do CONTEXT e 3 do PROJECT), 2 manual pendentes (CHANGELOG e README do PROJECT). |

### Gate 5 em detalhe
- **5.1 (D-2), PASS:** sem saída. O `crates/ddc-core/Cargo.toml` não mudou na phase; só tem `thiserror`.
- **5.2, PASS:** sem saída. Os arquivos novos do core também não têm `Instant`, `std::thread`, `std::env`, `SystemTime` nem `serde`.
- **5.3, PASS:**
  - Não há `impl MonitorBackend` nem `impl UsbPresence` no core.
  - O port `UsbPresence` fica em `ddc_core::ports`.
  - Traits `pub` fora do core: `FollowOutput`, `Executor` e `Clock`, em `follow.rs:156/186/208`. São costuras internas do laço do tray (saída no stderr, executor de thread, espera). O core não precisa delas, porque roda em tempo lógico. Não são ports.
- **5.4, PASS:**
  - `SysfsUsbPresence::new` fora da raiz de composição só aparece em `#[cfg(test)]` (`tray.rs:572/598`, `follow.rs:734`), sempre sobre tempdir.
  - Em produção, ele só é construído no `lib.rs::start_follow`, com a raiz de `usb_root()`.
  - Nenhum adapter importa outro.
- **5.5, PASS:**
  - Nenhum `unsafe`; os acertos são só comentários (`lib.rs:242`, e `tests/autostart_entry.rs:6`, que já existia).
  - O workspace tem `unsafe_code = "deny"`, e todos os crates usam `[lints] workspace = true`.
- **5.6, PASS:** todos os acertos estão em módulos `#[cfg(test)]`. O clippy `-D warnings` com `unwrap_used`/`expect_used` passou.
- **5.7, PASS, com leitura manual:**
  - A grep acha `Confirm::Yes` em `apps/ddc-tray/src-tauri/src/follow.rs:390`, fora da lista CLI/commands do gate. A D-2026-10-02-usb-switch-follow-8 autoriza isso: o consentimento é ligar o follow com a config completa.
  - Caminho único até lá: `write_input` ← `switch` ← `tick`. Só acontece quando há `fired` e `target = config.active_target()` (que exige `enabled` e config completa), sem aprendizado em curso.
  - A escrita passa por `SharedOsd = Arc<dyn MonitorControl>`, ou seja, pelo `authorize_write` do core. Não há retry, e o erro vai para o `report`.
  - Mutações que ligam o consentimento mesmo com `enabled` falso foram reprovadas nas linhas 3 e 8.
  - Li CADA um dos 71 testes novos. Todos usam fakes: `InMemoryMonitorBackend` via `osd_with`/`rtk_monitor`, `SysfsUsbPresence` sobre `TempDir`, `ScriptedPresence`, `ConfigStore` em tempdir e `mock_app`.
  - O backend real em testes só aparece em `crates/ddc-adapters/tests/real_monitor.rs`: 7 testes `#[ignore]` presos a `DDC_HW_TESTS`, que já existiam.
- **5.8, PASS:** sem saída. O crate `ddc-adapters` não tem nenhum caminho `/sys` no código; aparece só em comentário. A raiz real vive só em `lib.rs:70` (`REAL_USB_ROOT`).
- **5.9, PASS:** a phase não cria nenhum `#[tauri::command]`.
  - O laço roda numa `std::thread` nomeada `usb-follow`.
  - Cada escrita roda numa thread `usb-follow-switch`.
  - Os cliques do menu vão para `spawn_blocking`/`async_runtime::spawn`.
  - O "Learn" usa `on_blocking_thread`.
  - Nada roda na thread da UI.
- **5.10, PASS:** `cargo audit` (0.22.2, 1280 advisories) achou 0 vulnerabilidades. O único aviso (yank do `yoke-derive 0.8.3`) já existia em `420a3a1` (W7). A phase só acrescenta o dev-dep `tempfile`, que já estava no lock.
- **5.11, PASS:** nenhum segredo e nenhum TODO/FIXME em `.rs`.
- **DRY, KISS, YAGNI e clean-code:**
  - DRY: o W4.
  - KISS e YAGNI: nada especulativo; os traits de costura têm 2 implementações cada (produção e fake).
  - clean-code: o `let _ = fs::remove_file(&temporary)` em `follow_config.rs:257` tem comentário de motivo ("best effort"). Aceito.

### Prova de hardware (D-12), antes de qualquer execução nativa
- Como não há `inotifywait`, usei um observador inotify próprio, em Python com ctypes.
- O `bwrap --bind / / --dev /dev` cobre os **16** `/dev/i2c-*` com arquivos-isca observados de fora. Também cobre `/sys/bus/usb/devices` com uma árvore-isca observada (o dispositivo `9-9`).
- Os 12 nós reais legíveis também foram observados. Os nós `i2c-0`, `-6`, `-7` e `-8` são 0600 de root, inacessíveis também para o usuário.
- **Controle positivo DENTRO do sandbox:** o sandbox viu 16 entradas `i2c-`, todas arquivos comuns e não dispositivos de caractere. Abri cada uma em leitura e escrita e o observador registrou 16/16 `OPEN`. A leitura da raiz USB-isca registrou `DIR OPEN` e `9-9/idVendor OPEN`.
- **Suíte inteira** (`cargo test --workspace --locked`) no mesmo sandbox: 484 passed. **Zero eventos** entre os marcadores `CONTROL_DONE` e `SUITE_DONE`, nas iscas, nos nós reais e na raiz USB-isca.
- O observador ficou ligado durante todas as linhas do DoD e todas as mutações. Saldo: 0 eventos nas iscas i2c e 0 nos nós reais.
- A raiz USB-isca só foi lida na mutação `m8-real-usb-root-in-simulation`, com 396 eventos. Isso prova que a sonda funciona: nenhum teste e nenhuma linha do DoD lê o `/sys/bus/usb/devices` real como raiz do follow.
- Por código: todo `SysfsUsbPresence::new` de teste usa uma raiz em `TempDir`. O `usb_root()` só devolve `DDC_TRAY_USB_ROOT` com `DDC_TRAY_FAKE=1`, e sem essa variável não há raiz (D-12a, teste `switch_tests::the_follow_reads_the_tree_of_ddc_tray_usb_root_only_in_simulation`).

### Mutações (cópia descartável)
- **Ambiente:**
  - repositório em `/var/tmp`, montado com `git archive`: commit 1 = conteúdo de `420a3a1`, com `origin/main` nele; commit 2 = conteúdo de `43a831f`. As árvores são idênticas às reais;
  - `CARGO_TARGET_DIR` próprio, semeado por reflink do btrfs;
  - todo `cargo` rodou em bwrap, com as iscas ou com o bwrap da própria linha 9.
- **Método:** cada mutação recebe um commit e é **recongelada**, como o orquestrador faz depois de cada passada do doer. Depois, o Verify roda literal, num bash limpo, com diagnóstico dos testes que caem; com isso, erro de compilação nunca conta como reprovação.
- **Congelamento:** alteração sem commit dá NO-OK; commit sem recongelar dá NO-OK; alteração inócua recongelada dá OK.

| Linha | Mutações reprovadas (o que caiu) | Sobreviventes |
|---|---|---|
| 1 | dispara na chegada (5 testes); saída parcial conta como saída (3); sem rearme (3); estado inicial `Armed` (`all_absent_at_start…`, `arrival…`); `DEBOUNCE_POLLS=2` (2); `Off` dispara (`disabled_never_fires`); `learn` invertido (`learn_is_present…`). 7/7 | — |
| 2 | hubs mantidos (`skips_hubs`); serial descartado (3); raiz ausente vira erro (`missing_root_is_empty_set`). 3/3 | — |
| 3 | retry de 1 escrita (`silent_ddc…`); consentimento ignora `enabled` (`disabled_writes_nothing`); erro sem `report` (`silent_ddc…`, `a_switch_without_a_core…`); laço para depois do disparo (5). 4/4 | guarda "durante o aprender conta como Off" removida (W1); linha de troca condicionada a `DDC_TRAY_DEBUG` (W3) |
| 4 | versão desconhecida aceita; `load` sobrescreve o inválido; aprender mantém `enabled`; `XDG_CONFIG_HOME` relativo aceito. 4/4 | `save` não atômico (W2) |
| 5 | check marcado por "tem dispositivos"; rótulo pt-BR "Seguir switch USB"; itens também no Windows; `from_id` aceita qualquer código. 4/4 | — |
| 6 | `app.js` sem `pickerOptions` (Playwright: `fallback.spec` ×3 e `pseudo-locale.spec` ×1, em light e dark; o furo antigo está fechado); `en.js` com `(no DDC/CI)` (`not ok 159`, o teste literal). 2/2 | — |
| 7 | sem `modules-load.d`; sem a frase do aprender; `notebook`→`laptop`. 3/3 | — |
| 8 | `run()` sem `follow::spawn` (`learning started` não veio); chegada dispara (2 trocas depois da volta); raiz USB real na simulação (o aprender não grava; 396 leituras da isca); `panicked` no stderr; consentimento ignora `enabled` (2 trocas com o follow desligado); sem rearme (3 trocas). 6/6. O script sai 1 com `/bin/true`, sem `python3-gobject` (gi quebrado no `PYTHONPATH`), com caminho não executável e sem argumento. | linha de troca condicionada a `DDC_TRAY_DEBUG` (W3; o smoke roda com `DDC_TRAY_DEBUG=1`) |
| 9 | teste que exige `/sys/bus/usb/devices`; teste que exige nós `i2c-` em `/dev`. Os dois dão FAILED só dentro do bwrap. 2/2 | — |
| 10 | `# TODO:` no smoke; `// todo:` em comentário do `pt-BR.js`; `(FIXME)` no guia; `to-do:` no README. 4/4. Controles negativos continuam OK: "Todos" num valor pt-BR e `TODO(#12)`. | — |
| PROJECT 1 | janela do aprender com erro de um (`expires_when_nothing…`) | — |
| PROJECT 2 | +800 linhas mortas: TOTAL Lines 77.82% (< 80%) | — |
| PROJECT 3 | `// TODO:` em `follow.rs`. O controle `TODO(#7)` dá OK. | — |

## Blockers
- Nenhum.

## Warnings
- **W1 (5.7 / D-2026-10-02-usb-switch-follow-8 — "aprender nunca escreve" sem teste):**
  - A guarda `config.active_target().filter(|_| self.learning.is_none())`, em `apps/ddc-tray/src-tauri/src/follow.rs:274`, não é protegida por nenhum teste. Removê-la passa nos 484 testes.
  - O teste `follow::learning_tests::learning_while_the_follow_is_on_writes_nothing_and_turns_it_off` (`follow.rs:968`) é oco quanto a ela. No cenário dele, o aprender conclui na mesma leitura do debounce e desliga o follow antes.
  - Uma sonda temporária, só na cópia descartável, confirma que a mutação não é equivalente. Cenário: aprendido = só o teclado, follow ligado, teclado sai uma leitura antes do mouse durante o aprender. Com a guarda, o log é `[]`; sem ela, aparece `WriteVcp(RTK-RTK-QHD-HDR-01010101, 0x60, 0x10)` durante o aprender.
  - O código está correto. Falta o teste: acrescentar esse cenário de saída escalonada em `follow::learning_tests`, fora dos prefixos do DoD.
- **W2 (D-2026-10-02-usb-switch-follow-4 — escrita atômica sem teste):**
  - O `ConfigStore::save` faz temporário + `sync_all` + `rename` (`follow_config.rs:252-254`), mas trocar por escrita direta no arquivo passa nos 484 testes.
  - Sugestão: um teste em `follow_config::rule_tests` que ocupa o nome do temporário (um diretório em `.usb-follow.json.<pid>.tmp`) e exige `Err` com os bytes anteriores intactos.
- **W3 (linha de troca "sempre, sem `DDC_TRAY_DEBUG`" sem prova em produção):**
  - O PLAN T-4 promete `eprintln!` sempre. Os testes provam a escolha `announce` vs `diagnose` só com o `Recorder`, e o smoke roda com `DDC_TRAY_DEBUG=1` (`smoke-follow-private.sh:276`).
  - Condicionar o `eprintln!` do `Stderr::announce` (`follow.rs:169`) ao switch de diagnóstico passa nas linhas 3 e 8 (comprovado).
  - Sugestão: tornar a escrita do `Stderr` injetável e testar o `announce` com o diagnóstico desligado, ou fazer um passo do smoke sem `DDC_TRAY_DEBUG`.
- **W4 (DRY):**
  - `i18n::input_label` (`apps/ddc-tray/src-tauri/src/i18n.rs:122-130`) repete os nomes dos valores `0x0F`–`0x12` de `0x60`, que já estão em `ddc_core::domain::mccs_catalog::INPUT_SOURCES` (`mccs_catalog.rs:80`, via `value_name`, `:189`). A única diferença é o espaço no lugar do hífen. `demo-data.js` também os tem.
  - Daria para derivar: `value_name(VcpCode::INPUT_SOURCE, code).map(|n| n.replace('-', " "))`.
  - Menor: os ids de teste `046d:c31c:KB0001`/`046d:c077` se repetem em 4 módulos de teste do tray.
- **W5 (consistência do plano):** as 8 tasks do `PLAN.md` continuam `**Status:** pending`, enquanto o SUMMARY diz 8/8. Todas as phases anteriores marcam `completed`.
- **W6 (higiene de testes, menor):**
  - `follow::learning_tests::the_loop_runs_on_a_named_thread_of_its_own` (`follow.rs:1191`) deixa uma thread `usb-follow` real rodando até o fim do binário de teste. Ela usa presença em memória, config padrão e `SystemClock` de 500 ms, e nunca escreve.
  - `the_system_clock_waits_for_real` dorme 5 ms.
  - É inofensivo, mas foge do "relógio injetável, sem dormir" da D-3.
- **W7 (5.10, antigo, informativo):** o `cargo audit` aponta o yank do `yoke-derive 0.8.3`, que já estava em `420a3a1`. Não foi introduzido pela phase.

## DoD Checklist (gate 8)

Os `Verify:` foram extraídos por programa (regex ``\*\*Verify:\*\* `([^`]*)` ``, 10 no CONTEXT e 3 no PROJECT, sem `<FREEZE>`). Rodaram literalmente, a partir da raiz do repositório, com `env -i` + `bash --noprofile --norc`, `LC_ALL=C.UTF-8`, `PATH` com `~/.cargo/bin` e `DISPLAY`/`WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`/`DBUS_SESSION_BUS_ADDRESS` do ambiente.
- Também passei `HOME`. Sem ele, a linha 8 compara `/.config` em vez do `~/.config` real, e o smoke aborta no `set -u`.
- As linhas 1–5, 7, 8 e o PROJECT 1 rodaram dentro do bwrap com as iscas e o observador ligado (0 eventos). A 9 monta o próprio bwrap. A 6, a 10 e os PROJECT 2 e 3 rodaram nativos.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core: `Follower` e `learn` seguem a D-3 com relógio fake (8 testes exatos de `app::usb_follow::tests::`) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 7/7 reprovadas. |
| 2 | Adapter `usb_sysfs` numa raiz fake (5 testes exatos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 3/3. |
| 3 | Laço do follow: escrita exata `[(RTK,0x60,0x10)]`, linha de troca, chegada/desligado `[]`, DDC mudo vai ao `report`, só o monitor configurado e `0x60` + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4 reprovadas; 2 sobreviventes, fora do texto da linha (W1, W3). |
| 4 | Config (D-4): ida e volta, XDG→`~/.config`, ausente = desligado, versão desconhecida não sobrescrita, aprender não liga + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4/4; a atomicidade não é afirmada pela linha (W2). |
| 5 | Menu e rótulos (D-5): check = `enabled`, só no Linux, ida e volta dos ids, literais en/pt-BR, `menu::`/`i18n::` antigos verdes + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4/4. |
| 6 | Popup "em outra entrada" (D-6, D-12b): frases literais, `tests/ui` inteira verde, Playwright inteiro sem `failed`/`flaky` + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Unit: `# fail 0 # cancelled 0 # skipped 0 # todo 0`. Playwright: 152 passed. Mutações 2/2. |
| 7 | Doc de instalação nas duas máquinas (literais exigidos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 3/3. |
| 8 | Smoke release em D-Bus privado, raiz USB fake, as 10 linhas exatas, `~/.config` real idêntico + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 32 s. Mutações 6/6 e 4 condições do script com saída 1. sha256 do `~/.config/autostart` igual antes e depois; `~/.config/ddc-control` inexistente antes e depois. |
| 9 | Suíte dos 3 crates sem o `/sys` real e sem `/dev/i2c-*`, conferido dentro do sandbox + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 2/2. |
| 10 | Sem TODO/FIXME sem referência nos arquivos não-Rust da phase | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4/4; 2 controles negativos OK. |
| 11 | `cargo test --workspace` sai 0 | PROJECT | Auto | PASS | `OK`, 484 passed, 0 failed, 9 ignored. Mutação reprovada. |
| 12 | Cobertura >= 80% de linhas | PROJECT | Auto | PASS | Comando literal `cargo llvm-cov --workspace --summary-only` (sem exclusões): `TOTAL … 5589 614 89.01%` ≥ 80%. Gate 3: 89.31%. Mutação: 77.82%. |
| 13 | Sem `TODO`/`FIXME` sem referência de issue (`.rs`) | PROJECT | Auto | PASS | `OK`, saída 0. Mutação reprovada; controle `TODO(#7)` OK. |
| 14 | CHANGELOG.md atualizado com entrada por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou um `Added` ("Follow USB switch" no menu Linux) e um `Changed` ("(on another input)" no lugar de "(no DDC/CI)"); nenhum cabeçalho `## [versão]` novo (o último é `## [0.1.0] - 2026-09-28`) |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o diff do README põe os itens do follow no menu do clique direito do Linux, acrescenta o tópico "**Follow USB switch** (Linux only)" e a limitação "Linux only, and unproven on real hardware", e troca "(no DDC/CI)" por "(on another input)" em 3 lugares |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod usb-switch-follow` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase. Pelo fluxo autônomo do `/jdi-issue`, os dois itens, junto com o "Deferred to PR review" do CONTEXT (teste real PC + notebook + switch, aprender com o switch real, leitura humana do guia, aparência no KDE/GNOME), ficam para o PR.

## Recommendation
Aprovar com confirmação manual pendente:
- Não há bloqueio. Os gates 1 a 4 e 7 estão limpos, e o clippy do Windows também.
- O core está puro (só `thiserror`), o port `UsbPresence` fica no core e o adapter de sysfs recebe a raiz injetada.
- A regra-chave (dispara uma vez na SAÍDA de todos os aprendidos, rearma ao voltar um, nunca na chegada nem no estado inicial) caiu em todas as 7 mutações do core e nas do smoke.
- O consentimento `Confirm::Yes` só existe no ramo ligado e configurado (D-8). A escrita é única, sem retry, e o erro vai ao `report`.
- A prova de hardware deu zero aberturas.

Antes do ship, de preferência na mesma passada, recomendo fechar as três lacunas de teste achadas por mutação, sem mudar código de produção:
1. W1: o cenário de saída escalonada durante o aprender, sem escrita.
2. W2: a atomicidade do `save`.
3. W3: a linha de troca sem `DDC_TRAY_DEBUG`.

Os demais WARN (W4 DRY, W5 status do plano, W6 e W7) podem ficar para o PR. Se o orquestrador mexer em testes, ele precisa recongelar as árvores no CONTEXT. Os nomes novos devem ficar fora dos prefixos exatos do DoD.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 1). Todo `cargo`, `npm` e Playwright rodaram em `bwrap`, com os 16 `/dev/i2c-*` cobertos e observados, e houve 0 eventos fora dos controles. As linhas 1, 2, 4, 5, 7 e 9 a 13 têm `hollow=false`.

- **Linha 6 — popup "em outra entrada" (objective).** A cláusula "`fallback.spec` e `pseudo-locale.spec` leem o rótulo do monitor mudo no popup, o que prova que o `app.js` usa as funções novas do view-model (D-12b)" é FALSA sobre as specs congeladas. O diff do `app.js` é um refactor puro.
  - Com o `app.js` trocado pelo blob de `420a3a1`, que não usa `pickerOptions`/`messageTip`, o Verify 6 recongelado dá `OK`: unit `# fail 0` e Playwright `152 passed`, com `fallback.spec` 6/6 e `pseudo-locale.spec` 62/62.
  - As frases literais são provadas na função do view-model. O Playwright compara com `t('header.silent')`, ou seja, com o próprio template.
  - Correção: afirmar só o rótulo renderizado e conferir o literal também no e2e.
- **Linha 3 — laço do follow (hollow, NÃO objetiva).**
  - W3 confirmado: o `Stderr::announce` real não é exercido por teste; o `Recorder` fake marca o `announce` por construção.
  - W1 confirmado, fora do texto: sem a guarda "durante o aprender conta como Off", os 196 testes passam. Uma sonda de saída escalonada pega a mutação.
  - Nas duas mutações, só o congelamento derruba o Verify.
- **Linha 8 — smoke (hollow, NÃO objetiva).**
  - O texto diz que o `~/.config/autostart` e o `~/.config/ddc-control` reais "ficam idênticos", mas o Verify compara só os nomes (`ls -A`).
  - O W3 também alcança esta linha, porque o smoke liga `DDC_TRAY_DEBUG=1`.
- **W2, W4–W7:** confirmados como fora do DoD (W2 = atomicidade do `save` sem teste).

**Verdict:** BLOCKED
