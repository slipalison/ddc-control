# Phase 10: Review  (slug: usb-switch-follow)

**Verdict:** APPROVED_PENDING_MANUAL

Revisado no HEAD `4e57ecd` (código até `02fdd12`), com base `origin/main` = `420a3a1`. Esta é a iteração 2 e substitui o REVIEW da iteração 1, que tinha sido BLOCKED pelo critic na linha 6.

- **O que revisei de novo:** só o código com diff desde `43a831f` (`git diff 43a831f HEAD -- . ':!.jdi'`): `apps/ddc-tray/src-tauri/src/{follow.rs, follow_config.rs, i18n.rs}` e `apps/ddc-tray/tests/e2e/fallback.spec.mjs`.
- **Árvores sem mudança desde `43a831f`:** `crates/ddc-core`, `crates/ddc-adapters`, `apps/ddc-tray/src`, `apps/ddc-tray/scripts`, `docs`, `Cargo.toml` e `Cargo.lock` têm os mesmos ids de árvore. A evidência delas é herdada da iteração 1, e cada ponto herdado está marcado.
- **O que rodou de novo:** todos os gates e todos os Verify.
- **Ambiente:** Rust 1.98.1, Linux (Fedora, kernel 7.2), bash.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` saiu 0. O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` saiu 0; como o host é Linux, o check é nativo. Só aparece o aviso antigo de future-incompat do `nom v3.2.1`. |
| Tests | PASS | 487 passed, 0 failed, 9 ignored (hardware: `real_monitor` 7 e `rtk_qhd_hdr` 2). Rodei primeiro no bwrap da prova de hardware e depois nativo, com o mesmo resultado. Por binário: ddc-adapters lib 83, caching_backend 15, ddc-cli lib 56, cli 32, ddc-core lib 64, monitor_control 36, ddc-tray lib 199, autostart_entry 2. Os +3 sobre os 484 da iteração 1 são os 3 testes Rust novos: `follow::stderr_tests::announce_prints_without_the_diagnostics_switch`, `follow::learning_tests::a_staggered_leave_while_learning_writes_nothing` e `follow_config::rule_tests::a_save_that_cannot_write_its_temporary_file_keeps_the_old_bytes`. Nenhum teste foi removido. |
| Coverage | PASS | 89.54% de linhas, limite 80%, linha TOTAL (`main.rs`/`build.rs` excluídos): `TOTAL 8954 944 89.46% 909 118 87.02% 5679 594 89.54%`, saída 0. Arquivos mexidos: `follow.rs` 98.09%, `follow_config.rs` 98.50%, `i18n.rs` 100%. O `lib.rs` (55.17%) não mudou; o que falta nele é composição. |
| Lint | PASS | `cargo fmt --all --check` saiu 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` saiu 0. O clippy `--target x86_64-pc-windows-gnu` (ddc-tray, ddc-adapters, ddc-core, ddc-cli; `--all-targets -D warnings`), com o `windres`/`ar` falsos do scratchpad, saiu 0. Nenhum `#[allow(...)]` ou `#[expect(...)]` novo. |
| Hexagonal/Safety/Hygiene | PASS (com WARN menor) | Todos os itens de 5.1 a 5.11 passaram (detalhes abaixo). Os WARN são restos menores (W4, W6) e itens informativos (W7, W8). |
| Consistency | PASS | Todos os arquivos do plano estão no log. O W5 fechou: as 8 tasks estão `**Status:** completed` (`137acaf`). Os 6 commits de código da iteração 2 usam `<tipo>(usb-switch-follow):` e citam a D-XX. Nenhuma D-XX violada. |
| UI Validation | PASS | `npm ci --ignore-scripts` + `npx playwright test` inteiro, em bwrap: 154 passed, 6 skipped, 0 failed, 0 flaky, em light e dark. Os 6 pulados são o `screenshots.spec`, preso a `SCREENSHOTS=1`, como antes. O teste novo `a silent monitor reads on another input in the picker` deu ✓ em `[light]` (#18) e `[dark]` (#99). |
| DoD | PASS_PENDING_MANUAL | 13/13 auto (10 do CONTEXT e 3 do PROJECT), 2 manual pendentes (CHANGELOG e README do PROJECT). |

### Gate 5 em detalhe
- **5.1 (D-2), PASS:** sem saída. O `crates/ddc-core/Cargo.toml` só tem `thiserror`, e a árvore do core é a mesma de `43a831f`.
- **5.2, PASS:** sem saída.
- **5.3, PASS:**
  - Não há `impl MonitorBackend` nem `impl UsbPresence` no core.
  - Traits `pub` fora do core: `AutostartEntry` (`autostart.rs:55`, que já existia) e `FollowOutput`, `Executor` e `Clock` (`follow.rs:156/206/228`). São costuras internas do laço do tray, e não ports. A iteração 1 já tinha julgado assim; a iteração 2 não mudou isso.
- **5.4, PASS:**
  - Fora da raiz de composição, `SysfsUsbPresence::new` só aparece em `#[cfg(test)]`: `tray.rs:572/598` e `follow.rs:777` (o `Rig`, dentro de `mod tests`, que começa na `follow.rs:724`). Sempre sobre tempdir.
  - Nenhum adapter importa outro.
- **5.5, PASS:**
  - Nenhum `unsafe`. Os acertos são só comentários: `lib.rs:242` e `tests/autostart_entry.rs:6`, os dois já existentes.
  - O workspace tem `unsafe_code = "deny"`.
- **5.6, PASS:**
  - Os 14 `unwrap` novos do diff ficam em `stderr_tests`, `learning_tests` e `rule_tests`, todos `#[cfg(test)]`.
  - O código de produção novo usa `unwrap_or_else(PoisonError::into_inner)`.
  - O clippy com `unwrap_used`/`expect_used` em `-D warnings` passou.
- **5.7, PASS, com leitura manual:**
  - A grep acha `Confirm::Yes` em `apps/ddc-tray/src-tauri/src/follow.rs:410` e no comentário da linha 9. É o mesmo caminho único da iteração 1: `write_input` ← `switch` ← `tick`, só com `fired` e `target = config.active_target().filter(|_| self.learning.is_none())`. A D-2026-10-02-usb-switch-follow-8 autoriza isso.
  - A guarda "aprender conta como Off" agora tem teste (W1, mutação abaixo).
  - Li os 3 testes Rust novos ou alterados e o e2e novo:
    - `stderr_tests`: `osd_with` → `InMemoryMonitorBackend` e `ScriptedPresence`;
    - `a_staggered_leave…`: `Rig`, com `SysfsUsbPresence` sobre `FakeRoot`/`TempDir` e backend em memória;
    - `the_loop_runs_on_a_named_thread_of_its_own`: `ScriptedPresence`, `osd_with` e config desligada, então não escreve;
    - `a_save_that_cannot_write…`: só tempdir;
    - e2e: bridge demo do navegador.
  - Nenhum teste novo é `#[ignore]`, e nenhum fala com o backend real. O backend real em testes continua só nos 7 `#[ignore]` de `crates/ddc-adapters/tests/real_monitor.rs`, presos a `DDC_TRAY_HW`, que já existiam.
- **5.8, PASS:** sem saída. A raiz real `/sys/bus/usb/devices` continua só em `lib.rs:70` (`REAL_USB_ROOT`), que não mudou.
- **5.9, PASS:** nenhum `#[tauri::command]` novo. O `spawn_until` mantém a thread `usb-follow`, e o `spawn` público só delega.
- **5.10, PASS (W7 informativo):** `cargo audit` 0.22.2 (1280 advisories) achou 0 vulnerabilidades. O único aviso é o yank do `yoke-derive 0.8.3`, que já estava em `420a3a1`. O `Cargo.lock` não mudou desde `43a831f`.
- **5.11, PASS:** nenhum segredo e nenhum TODO/FIXME em `.rs`.
- **DRY, KISS, YAGNI e clean-code (diff da iteração 2):**
  - DRY: o `i18n::input_label` passou a usar `value_name(VcpCode::INPUT_SOURCE, code)` do catálogo do core, com o hífen trocado por espaço. Só é chamado com `FOLLOW_INPUTS` (0x0F–0x12), e a saída é idêntica. Resto menor no W4.
  - KISS: o `spawn_until` genérico (relógio e `keep_going`) é a costura de teste, com 2 usos. Aceito.
  - YAGNI: nada especulativo.
  - clean-code:
    - o `let _ = writeln!(sink, "{line}")` em `follow.rs:190` tem comentário de motivo ("As `eprintln!`, without its panic"). Aceito. A troca do `eprintln!`, que entra em pânico se o stderr falha, pela escrita que ignora o erro é uma melhora;
    - o `diagnose` e o `report` continuam no `crate::diagnose`/`crate::report`; só a linha de troca vai ao sink, como o doc do `Stderr` diz.

### Prova de hardware (D-12), antes de qualquer execução nativa
- **Observador:** um inotify próprio, em Python com ctypes (`scratchpad/rv2/hw/watch.py`).
- **Cobertura do sandbox:** o `bwrap --bind / / --dev /dev` cobre os **16** `/dev/i2c-*` com arquivos-isca observados de fora. Também cobre `/sys/bus/usb/devices` com uma árvore-isca observada (o dispositivo `9-9`).
- **Nós reais:** os 12 legíveis também foram observados. Os nós `i2c-0`, `-6`, `-7` e `-8` são 0600 de root, inacessíveis ao usuário (errno 13 no watch).
- **Controle positivo DENTRO do sandbox:**
  - o sandbox viu 16 entradas `i2c-`, todas arquivos comuns e nenhuma dispositivo de caractere;
  - abri cada uma em leitura e escrita, e o observador registrou 16/16 `OPEN` + `CLOSE_WRITE`;
  - a leitura da raiz USB-isca registrou `DIR OPEN`, `9-9/ OPEN` e `9-9/idVendor OPEN`.
- **Suíte inteira** (`cargo test --workspace --locked`, `env -i`, sem nenhuma `DDC_*`), no mesmo sandbox: 487 passed. **Zero eventos** entre os marcadores `CONTROL_DONE` (14:31:20) e `SUITE_DONE` (14:31:26).
- **Observador ligado até o fim:** ficou ligado durante os gates nativos, o Playwright, os 13 Verify e todas as mutações, até o `STOP` (14:53:03).
  - Saldo: 0 eventos nos nós reais.
  - Os 32 eventos nas iscas i2c e os 7 na raiz USB-isca são todos do controle, às 14:31:20.
  - Logo, nenhum teste, nenhuma linha do DoD e nenhuma mutação abriu um `/dev/i2c-*` ou leu o `/sys/bus/usb/devices` real.
- **Por código:**
  - os testes novos usam `ScriptedPresence` ou `FakeRoot` sobre `TempDir`;
  - o `usb_root()`, que só aceita `DDC_TRAY_USB_ROOT` com `DDC_TRAY_FAKE=1`, não mudou (herdado da iteração 1, teste `switch_tests::the_follow_reads_the_tree_of_ddc_tray_usb_root_only_in_simulation`).
- **Ambiente real:**
  - o `ddc-tray` do usuário (PID 33102) continuou rodando;
  - o `~/.config/autostart` real tem nomes e sha256 idênticos antes e depois;
  - o `~/.config/ddc-control` não existia antes nem depois.

### Mutações (cópia descartável)
- **Ambiente:**
  - repositório em `/var/tmp`, montado com `git archive`: commit 1 = conteúdo de `420a3a1`, com `origin/main` nele; commit 2 = conteúdo de `4e57ecd`. As árvores-raiz e os 9 caminhos do congelamento são idênticos aos reais;
  - `CARGO_TARGET_DIR` próprio, semeado por reflink do btrfs, com um link `target` no repositório apontando para ele, para que o `target/release/ddc-tray` da linha 8 seja o binário recém-construído;
  - todo `cargo`, `npm` e Playwright rodou em bwrap, com as iscas observadas;
  - apaguei a cópia ao terminar.
- **Método:** cada mutação recebe um commit e é **recongelada**, como o orquestrador faz. Depois, o Verify extraído do CONTEXT da cópia roda literal, com `env -i` e `bash --noprofile --norc`. O diagnóstico dos testes que caem vem à parte, para que erro de compilação nunca conte como reprovação.
- **Controles do congelamento:** alteração sem commit dá NO-OK; commit sem recongelar dá NO-OK; comentário inócuo recongelado dá OK.

| Alvo | Mutação | Resultado |
|---|---|---|
| Linha 3 (D-13b) | M3a: o `announce` ignora o sink e usa `eprintln!` | NO-OK. `stderr_tests::announce_prints_without_the_diagnostics_switch` FAILED (`left: ""`). |
| Linha 3 / W3 | M3b: o `announce` só escreve com `DIAGNOSTICS` ligado | NO-OK. O mesmo teste FAILED, inclusive com `DDC_TRAY_DEBUG=1` no ambiente. |
| Linha 8 | M8b: `Stderr::default()` sobre `io::sink()` | Linha 8 NO-OK (`FAIL: '…switching RTK-RTK-QHD-HDR-01010101 to input 0x10' did not happen within 10 s`). A linha 3 dá OK, como esperado: o sink de produção é afirmado pela linha 8 ("NÃO afirmado: a fiação no `run()`, que a linha 8 prova"). |
| Linha 8 (D-13c) | M8a: o smoke acrescenta 1 byte a um arquivo existente do `$HOME/.config/autostart`; os nomes não mudam | Rodou com HOME falso (cópia do `autostart` real) e com o `autostart` real em `--ro-bind`. Verify novo (sha256): **NO-OK**. Verify antigo de `43a831f` (`ls -A`), recongelado no mesmo commit: **OK**. Controle com HOME falso e sem mutação: OK. O furo da iteração 1 está fechado. |
| Linha 6 (D-13a) | M6a: o `app.js` passa um conjunto `silent` vazio ao `monitorPicker` | NO-OK. Unit 166/0. Playwright com 10 failed, entre eles o teste novo ✘ em `[light]` (#19) e `[dark]` (#99). |
| Linha 6 (informativa) | M6b: `app.js` = blob de `420a3a1` | OK, coerente com o novo "NÃO afirmado". O uso das funções do view-model deixou de ser afirmado, e o `app.js` da base renderiza o mesmo rótulo via `t('header.silent')`. |
| W1 | Sem a guarda `.filter(\|_\| self.learning.is_none())` | A linha 3 dá OK, porque o caso fica fora do texto dela. PROJECT 1 NO-OK: `a_staggered_leave_while_learning_writes_nothing` FAILED (`left: [WriteVcp(…RTK-RTK-QHD-HDR-01010101…, VcpCode(96), 16)]`). |
| W2 | `save` grava direto no destino | A linha 4 dá OK, porque a atomicidade não está no texto dela. PROJECT 1 NO-OK: `a_save_that_cannot_write_its_temporary_file_keeps_the_old_bytes` FAILED. |
| W4 | `input_label` mantém o hífen do catálogo | Linha 5 NO-OK: `i18n::tests::follow_labels_are_literal_in_en_and_pt_br` e mais 3 testes de `menu::` FAILED (`left: "DisplayPort-1"`). |
| W6 | O laço ignora o `keep_going` (nunca para) | PROJECT 1 NO-OK: `the_loop_runs_on_a_named_thread_of_its_own` FAILED em 15 s (`left: [Err(Timeout), Err(Timeout), Err(Timeout)]`). O teste falha e não trava. |

**Herdado da iteração 1** (o código coberto não mudou desde `43a831f`):

| Linha | Mutações reprovadas |
|---|---|
| 1 | 7/7 |
| 2 | 3/3 |
| 3 | 4/4 (retry, consentimento ignorando `enabled`, erro sem `report`, laço parado depois do disparo) |
| 4 | 4/4 |
| 5 | 4/4 |
| 7 | 3/3 |
| 8 | 6/6, mais as 4 saídas 1 do script |
| 9 | 2/2 |
| 10 | 4/4 e 2 controles negativos |
| PROJECT 1 | 1 |
| PROJECT 2 | 1 (77.82%) |
| PROJECT 3 | 1 |

## Blockers
- Nenhum.

## Warnings
Situação dos warnings da iteração 1:
- **W1:** fechado (`aac028a`, mutação reprovada).
- **W2:** fechado (`2bcafb2`, mutação reprovada).
- **W3:** fechado (`154824a`, M3a e M3b reprovadas).
- **W5:** fechado (tasks `completed`).
- **W4 e W6:** fechados no que a D-2026-10-02-usb-switch-follow-13(d) pede, com os restos menores abaixo.

Restam:
- **W4 (resto, DRY, menor):**
  - `apps/ddc-tray/src/demo-data.js` ainda escreve os nomes das entradas.
  - Os ids de teste `046d:c31c:KB0001`/`046d:c077` se repetem em vários módulos de teste do tray.
  - Não afeta o comportamento.
- **W6 (resto, higiene de testes, menor):**
  - `follow::learning_tests::the_system_clock_waits_for_real` (`follow.rs:1338`) ainda dorme 5 ms.
  - É o único teste do relógio real; o laço usa relógio injetável em todos os outros (D-2026-10-02-usb-switch-follow-3). Informativo.
- **W7 (5.10, antigo, informativo):** o `cargo audit` aponta o yank do `yoke-derive 0.8.3`. Ele já estava em `420a3a1` e não foi introduzido pela phase.
- **W8 (novo, informativo):**
  - O `follow::spawn` público (`follow.rs:429-435`), que só delega ao `spawn_until`, não é exercido por teste unitário.
  - O caminho de produção é exercido pelo smoke da linha 8: a M8b, no `Stderr::default()` que esse caminho usa, reprovou a linha 8.

## DoD Checklist (gate 8)

- **Extração:** os `Verify:` foram extraídos por programa, com a regex ``\*\*Verify:\*\* `([^`]*)` ``: 10 no CONTEXT e 3 no PROJECT. Não sobrou nenhum `<FREEZE>`, e os 9 fragmentos de congelamento batem com `git rev-parse HEAD:<caminho>`.
- **Execução:** cada um rodou literalmente, a partir da raiz do repositório, com `env -i` + `bash --noprofile --norc`, `LC_ALL=C.UTF-8`, `HOME`, `PATH` com `~/.cargo/bin` e `DISPLAY`/`WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`/`DBUS_SESSION_BUS_ADDRESS` do ambiente.
- **Onde rodou:** as linhas 1–8 e 10 e os PROJECT 1–3 rodaram dentro do bwrap com as iscas e o observador ligado, sempre com 0 eventos. A linha 9 monta o próprio bwrap e rodou nativa.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core: `Follower` e `learn` seguem a D-3 com relógio fake (8 testes exatos de `app::usb_follow::tests::`) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 7/7 herdadas; árvore do core sem mudança. |
| 2 | Adapter `usb_sysfs` numa raiz fake (5 testes exatos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 3/3 herdadas; árvore do adapter sem mudança. |
| 3 | Laço do follow: escrita exata `[(RTK,0x60,0x10)]`, linha de troca, chegada/desligado `[]`, DDC mudo vai ao `report`, só o monitor configurado e `0x60`, `Stderr` REAL com sink injetável imprimindo a linha com o diagnóstico DESLIGADO (D-13b) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. M3a e M3b dão NO-OK. A M8b (sink de produção) cai na linha 8, por desenho. 4/4 herdadas. |
| 4 | Config (D-4): ida e volta, XDG→`~/.config`, ausente = desligado, versão desconhecida não sobrescrita, aprender não liga + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4/4 herdadas. A atomicidade (W2) cai no PROJECT 1. |
| 5 | Menu e rótulos (D-5): check = `enabled`, só no Linux, ida e volta dos ids, literais en/pt-BR, `menu::`/`i18n::` antigos verdes + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. A mutação W4 dá NO-OK. 4/4 herdadas. |
| 6 | Popup "em outra entrada" (D-6, D-12b, D-13a): frases literais, `tests/ui` inteira verde, rótulo RENDERIZADO `LG TV SSCR2 (em outra entrada)` no e2e em light e dark, Playwright inteiro sem `failed`/`flaky` + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 19 s. Unit: `# tests 166 # pass 166 # fail 0 # cancelled 0 # skipped 0 # todo 0` e `ok 159 - a monitor with a readable EDID and no DDC answer reads on another input in en and pt-BR`. Playwright: 154 passed; o teste novo dá ✓ em light e dark. M6a dá NO-OK. M6b dá OK, sem afirmação a contradizer. |
| 7 | Doc de instalação nas duas máquinas (literais exigidos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 3/3 herdadas; `docs` sem mudança. |
| 8 | Smoke release em D-Bus privado, raiz USB fake, as 10 linhas exatas, `~/.config/autostart` e `~/.config/ddc-control` reais idênticos em nomes e sha256 (D-13c) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 33 s. M8a: Verify novo NO-OK, antigo OK. M8b dá NO-OK. 6/6 herdadas. Os snapshots sha256 reais são iguais antes e depois, e o `~/.config/ddc-control` está ausente nos dois momentos. |
| 9 | Suíte dos 3 crates sem o `/sys` real e sem `/dev/i2c-*`, conferido dentro do sandbox + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 4 s. Mutações 2/2 herdadas. A prova por inotify também deu 0 aberturas. |
| 10 | Sem TODO/FIXME sem referência nos arquivos não-Rust da phase | CONTEXT | Auto | PASS | `OK`, saída 0. Mutações 4/4 e 2 controles negativos herdados. |
| 11 | `cargo test --workspace` sai 0 | PROJECT | Auto | PASS | `OK`, 487 passed, 0 failed, 9 ignored. W1, W2 e W6 dão NO-OK aqui. |
| 12 | Cobertura >= 80% de linhas | PROJECT | Auto | PASS | Comando literal `cargo llvm-cov --workspace --summary-only` (sem exclusões): `TOTAL … 5712 614 89.25%` ≥ 80%. Gate 3: 89.54%. Mutação herdada: 77.82%. |
| 13 | Sem `TODO`/`FIXME` sem referência de issue (`.rs`) | PROJECT | Auto | PASS | `OK`, saída 0. Mutação e controle `TODO(#7)` herdados. |
| 14 | CHANGELOG.md atualizado com entrada por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: o CHANGELOG não mudou desde a iteração 1. `## [Unreleased]` tem um `Added` ("Follow USB switch" no menu Linux) e um `Changed` ("(on another input)" no lugar de "(no DDC/CI)"). Nenhum cabeçalho `## [versão]` novo; o último é `## [0.1.0] - 2026-09-28`. |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o README não mudou desde a iteração 1. Põe os itens do follow no menu do clique direito (linha 330) e acrescenta o tópico "**Follow USB switch** (Linux only)" (335) e a limitação "Linux only, and unproven on real hardware" (400). "(on another input)" aparece nas linhas 341 e 359. |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod usb-switch-follow` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

Pelo fluxo autônomo do `/jdi-issue`, os dois itens ficam para o PR, junto com o "Deferred to PR review" do CONTEXT:
- teste real PC + notebook + switch;
- aprender com o switch real;
- leitura humana do guia;
- aparência no KDE/GNOME.

## Recommendation
Aprovar com confirmação manual pendente.

- **Gates:** não há bloqueio. Os gates 1 a 4 e 7 estão limpos, o clippy do Windows também, e a suíte tem 487 testes verdes com cobertura de 89.54%.
- **Linhas emendadas do DoD:** as três provam o que afirmam, e as mutações mostram isso:
  - a 3 cai quando a linha de troca do `Stderr` real deixa de sair com o diagnóstico desligado;
  - a 6 cai quando o seletor deixa de renderizar `LG TV SSCR2 (em outra entrada)`, nos dois temas;
  - a 8 agora pega uma mudança só de conteúdo no `~/.config` real, que o Verify antigo deixava passar, e pega também um sink de produção mudo.
- **Warnings fechados:** W1, W2, W3, W4, W5 e W6 fecharam sem mudar comportamento de produção. Exceções: a troca do `eprintln!` pela escrita no sink, que agora ignora o erro de escrita em vez de entrar em pânico; e o rótulo derivado do catálogo, com saída idêntica.
- **Hardware:** a prova deu zero aberturas.

Os W4 e W6 que sobraram, o W7 e o W8 são menores ou informativos e podem ficar para o PR.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue` (sub-agente somente leitura, iteração 2). Todo `cargo` rodou em `bwrap` com as iscas `/dev/i2c-*` observadas; os 32 eventos registrados são todos do controle positivo. Os achados da iteração 1 (linhas 6 e 8, W3) estão fechados. As linhas 1, 2, 4 a 9 e 11 a 13 têm `hollow=false`.

- **Linha 10 — TODO/FIXME nos arquivos não-Rust (objective).** O padrão `C` aplicado a `en.js`/`pt-BR.js`, `(//|#|<!--).*\b(todo|fixme)s?\b`, não pega a forma `to-do`/`to do`. O `W` da própria linha pega essa forma, e o DoD do projeto conta `to-do(s):` como marcador (D-2026-09-27-tray-app-14/-15).
  - Demonstração, com o Verify 10 literal num repositório descartável: um comentário `// to-do: …` no `pt-BR.js`, `// To do: …` no `en.js` ou `// to do: …` no fim de uma linha dá `OK`.
  - Os controles `// todo:` e `# to-do:` num `.sh` dão NO-OK.
  - Nenhum gate determinístico rejeita esses comentários.
  - Correção: `C='(//|#|<!--).*(\b(todo|fixme)s?\b|\bto[ -]dos?\b[[:space:]]*[:(])'`.
- **Linha 3 — laço do follow (hollow, NÃO objetiva).** A cláusula "a escrita que falha vai para o `report`" só é provada com o `Recorder` falso; o `Stderr::report` real nunca roda em teste.
  - M3r: o `report` real passa pelo `diagnose`. Com ela, os Verify 3 e PROJECT 11 dão `OK` e o smoke, que liga `DDC_TRAY_DEBUG=1`, não distingue.
  - O código congelado cumpre o texto. Sugestão: o `report` do `Stderr` vai ao mesmo sink, com um teste que o afirma com o diagnóstico desligado.

**Verdict:** BLOCKED
