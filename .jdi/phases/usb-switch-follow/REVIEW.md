# Phase 10: Review  (slug: usb-switch-follow)

**Verdict:** APPROVED_PENDING_MANUAL

Revisado no HEAD `9310d7c` (código até `c461825`), com base `origin/main` = `420a3a1`. Esta é a iteração 3. Ela substitui o REVIEW da iteração 2, que o critic tinha deixado BLOCKED na linha 10 (forma `to-do`) e que tinha a linha 3 marcada como não objetiva.

- **O que revisei de novo:** só o código com diff desde `4e57ecd` (`git diff 4e57ecd HEAD -- . ':!.jdi'`), que é `apps/ddc-tray/src-tauri/src/follow.rs` (+88/−35, commit `c461825`). Em `.jdi`, revisei as linhas 3 e 10 emendadas do CONTEXT (`e85e5bb`, D-2026-10-02-usb-switch-follow-14) e o recongelamento (`9310d7c`).
- **Árvores sem mudança desde `4e57ecd`:** `crates/ddc-core`, `crates/ddc-adapters`, `crates/ddc-cli`, `apps/ddc-tray/src`, `apps/ddc-tray/tests`, `apps/ddc-tray/scripts`, `docs`, `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`, `clippy.toml` e `.github` têm os mesmos ids. A evidência delas é herdada da iteração 2, e cada ponto herdado está marcado.
- **O que rodou de novo:** os gates 1 a 8, o Playwright inteiro e os 13 Verify, literalmente.
- **Ambiente:** Rust 1.98.1, Linux (Fedora, kernel 7.2), bash.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` saiu 0. O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` saiu 0; como o host é Linux, o check é nativo. Só aparece o aviso antigo de future-incompat do `nom v3.2.1`. Tudo rodou no bwrap da prova de hardware. |
| Tests | PASS | 488 passed, 0 failed, 9 ignored (hardware: `real_monitor` 7 e `rtk_qhd_hdr` 2). Por binário: ddc-adapters lib 83, caching_backend 15, ddc-cli lib 56, cli 32, ddc-core lib 64, monitor_control 36, ddc-tray lib 200, autostart_entry 2. O +1 sobre os 487 da iteração 2 é `follow::stderr_tests::report_prints_without_the_diagnostics_switch`. Nenhum teste foi removido. A suíte deu o mesmo resultado na prova de hardware, no comando literal do gate 2 e no PROJECT 1. |
| Coverage | PASS | 89.64% de linhas, limite 80%, linha TOTAL (`main.rs`/`build.rs` excluídos): `TOTAL 8991 939 89.56% 914 117 87.20% 5707 591 89.64%`, saída 0. Arquivo mexido: `follow.rs` 98.53% (era 98.09%). `follow_config.rs` 98.50% e `i18n.rs` 100%. O `lib.rs` (55.17%) não mudou; o que falta nele é composição. |
| Lint | PASS | `cargo fmt --all --check` saiu 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` saiu 0. O clippy `--target x86_64-pc-windows-gnu` (ddc-tray, ddc-adapters, ddc-core, ddc-cli; `--all-targets -D warnings`), com o `windres`/`ar` falsos de `scratchpad/fakebin`, saiu 0. O módulo `follow` não tem `cfg` de plataforma, então esse clippy cobre o `follow.rs`. Nenhum `#[allow(...)]` ou `#[expect(...)]` novo. |
| Hexagonal/Safety/Hygiene | PASS (com WARN menor) | Todos os itens de 5.1 a 5.11 passaram (detalhes abaixo). Os WARN são restos herdados (W4, W6), itens informativos (W7, W8) e um ponto novo de DRY menor (W9). |
| Consistency | PASS | O `c461825` usa `refactor(usb-switch-follow):` e cita a D-2026-10-02-usb-switch-follow-14; os dois commits de docs também usam o slug. O `follow.rs` está no plano (T-4), e as 8 tasks estão `completed`. Nenhuma D-XX foi violada. Há uma nota sobre a D-7 e a D-14b no W9. |
| UI Validation | PASS | `has_frontend: true` no PROJECT.md. `npm ci --ignore-scripts` + `npx playwright test --reporter=list` inteiro, em bwrap: 154 passed, 6 skipped, 0 failed, 0 flaky, em light e dark. Os 6 pulados são o `screenshots.spec`, preso a `SCREENSHOTS=1`. O teste `a silent monitor reads on another input in the picker` deu ✓ em `[light]` (#19) e `[dark]` (#99). |
| DoD | PASS_PENDING_MANUAL | 13/13 auto (10 do CONTEXT e 3 do PROJECT), 2 manual pendentes (CHANGELOG e README do PROJECT). |

### Gate 5 em detalhe
- **5.1 (D-2), PASS:** sem saída. O `crates/ddc-core/Cargo.toml` só tem `thiserror`, e a árvore do core não mudou.
- **5.2, PASS:** sem saída.
- **5.3, PASS:**
  - Não há `impl MonitorBackend` nem `impl UsbPresence` no core.
  - Traits `pub` fora do core: `AutostartEntry` (`autostart.rs:55`, que já existia) e `FollowOutput`, `Executor` e `Clock` (`follow.rs:156/211/233`; as linhas andaram só pelo diff). São costuras internas do laço do tray, e não ports. O julgamento das iterações anteriores continua valendo.
- **5.4, PASS:**
  - Fora da raiz de composição, `SysfsUsbPresence::new` só aparece em `#[cfg(test)]`: `tray.rs:572/598` e `follow.rs:782` (o `Rig`, dentro de `mod tests`, que começa na `follow.rs:729`).
  - Nenhum adapter importa outro.
- **5.5, PASS:**
  - Nenhum `unsafe`. Os acertos são só comentários que já existiam: `lib.rs:242` e `tests/autostart_entry.rs:6`.
  - O workspace tem `unsafe_code = "deny"`.
- **5.6, PASS:**
  - O único `unwrap` novo do diff é o `TempDir::new().unwrap()` do `StderrRig::new`, dentro de `mod stderr_tests` (`#[cfg(test)]`, `follow.rs:966`).
  - O `print` de produção usa `unwrap_or_else(PoisonError::into_inner)`.
  - O clippy com `unwrap_used`/`expect_used` em `-D warnings` passou.
- **5.7, PASS, com leitura manual:**
  - A grep acha `Confirm::Yes` em `follow.rs:415` e no comentário da linha 9. É o mesmo caminho único de antes: `write_input` ← `switch` ← `tick`, autorizado pela D-2026-10-02-usb-switch-follow-8.
  - Li o teste novo `report_prints_without_the_diagnostics_switch`:
    - o `StderrRig` monta o laço com `osd_with` (`panel/tests.rs:55`, `InMemoryMonitorBackend`) sobre `rtk_monitor().with_vcp_failure(INPUT_SOURCE, Timeout)`, com `ScriptedPresence`, `Inline` e a config num `TempDir`;
    - exige exatamente um `WriteVcp(RTK, 0x60, 0x10)`, ou seja, uma tentativa só (D-7);
    - não é `#[ignore]` e não fala com o backend real.
  - O `announce_prints_…` só passou a usar o mesmo `StderrRig`.
  - O backend real em testes continua só nos 7 `#[ignore]` de `crates/ddc-adapters/tests/real_monitor.rs`, cuja árvore não mudou.
- **5.8, PASS:** sem saída. `/sys/bus/usb/devices` continua só em `lib.rs:70` (`REAL_USB_ROOT`) e no teste dele (`lib.rs:396/398`), sem mudança.
- **5.9, PASS:** nenhum `#[tauri::command]` novo.
- **5.10, PASS (W7 informativo):** `cargo audit` 0.22.2 (1280 advisories) achou 0 vulnerabilidades. O único aviso é o yank do `yoke-derive 0.8.3`, que já estava em `420a3a1`. O `Cargo.lock` não mudou.
- **5.11, PASS:** nenhum segredo e nenhum TODO/FIXME em `.rs`.
- **DRY, KISS, YAGNI e clean-code (diff da iteração 3):**
  - DRY: o formato `ddc-tray: could not {action}: {error}` agora aparece em três lugares (W9).
  - KISS: o `print` privado e o `StderrRig` têm 2 usos cada. Aceito.
  - YAGNI: nada especulativo.
  - clean-code: o `let _ = writeln!` mantém o comentário de motivo (`follow.rs:188`), e o doc do `Stderr` foi atualizado. Não há número mágico.

### Prova de hardware (D-12), antes de qualquer execução nativa
- **Observador:** um inotify próprio, em Python com ctypes (`scratchpad/rv3/hw/watch.py`). Observa 31 caminhos:
  - as 16 iscas;
  - 3 caminhos da raiz USB-isca: o diretório, `9-9/` e `9-9/idVendor`;
  - os 12 nós reais legíveis. Os nós `i2c-0`, `-6`, `-7` e `-8` são 0600 de root, inacessíveis ao usuário (errno 13 no watch).
- **Cobertura do sandbox:** o `bwrap --bind / / --dev /dev` cobre os **16** `/dev/i2c-*` com arquivos-isca e o `/sys/bus/usb/devices` com a árvore-isca.
- **Controle positivo DENTRO do sandbox:**
  - o sandbox viu 16 entradas `i2c-`, todas arquivos comuns e nenhuma dispositivo de caractere;
  - abri cada uma em leitura e escrita, e o observador registrou 16/16 `OPEN` + `CLOSE_WRITE` (32 eventos);
  - a leitura da raiz-isca registrou 9 eventos (`OPEN`/`ACCESS`/`CLOSE_NOWRITE` no diretório, em `9-9/` e em `9-9/idVendor`);
  - tudo às 15:30:32.
- **Suíte inteira** (`cargo test --workspace --locked`, `env -i`, sem nenhuma `DDC_*`), no mesmo sandbox: 488 passed. **Zero eventos** entre os marcadores `CONTROL_DONE` (15:30:32) e `SUITE_DONE` (15:30:37).
- **Observador ligado em duas janelas:**
  - 15:30:20–15:39:40: prova, gates, Playwright, os 13 Verify e as mutações das linhas 10 e 3;
  - 15:40:59–15:41:51: M3a e M3b;
  - entre as duas janelas só rodaram greps e a remoção da primeira cópia, sem `cargo`.
  - Saldo: 41 eventos, todos do controle às 15:30:32, e **0 nos nós reais**. Nenhum teste, nenhuma linha do DoD e nenhuma mutação abriu um `/dev/i2c-*` ou leu o `/sys/bus/usb/devices` real.
- **Ambiente real:**
  - o `ddc-tray` do usuário (PID 33102) continuou rodando;
  - o `~/.config/autostart` real tem nomes e sha256 idênticos antes e depois;
  - o `~/.config/ddc-control` não existia antes nem depois.

### Mutações (cópia descartável)
- **Ambiente:**
  - repositório em `/var/tmp/ddc-mut-rv3.*`, montado com `git archive`: commit 1 = conteúdo de `420a3a1`, com `origin/main` nele; commit 2 = conteúdo de `9310d7c`. As árvores-raiz são idênticas às reais (`bb81e7d…` e `bb6c2e1…`);
  - `CARGO_TARGET_DIR` próprio, semeado por reflink de `target/debug`;
  - todo `cargo` rodou em bwrap, com as iscas observadas;
  - apaguei as duas cópias ao terminar (0 restantes).
- **Método:**
  - cada mutação parte do commit 2, recebe um commit e é **recongelada**, também com commit;
  - o Verify extraído do CONTEXT da cópia roda literal, com `env -i` e `bash --noprofile --norc`;
  - o diagnóstico dos testes vem à parte. Ele mostra que o teste caiu por asserção, e não por erro de compilação.
- **Linha de base (C0):** as linhas 3 e 10 dão OK na cópia sem mudança.
- **Controles do congelamento (linha 3):** alteração sem commit dá NO-OK; commit sem recongelar dá NO-OK; comentário inócuo recongelado dá OK.

| Alvo | Mutação | Resultado |
|---|---|---|
| Linha 10 (D-14a, critic) | T1: `// to-do: revisar a frase do mudo.` em linha própria do `pt-BR.js` | Verify novo: **NO-OK**. Verify 10 de `4e57ecd` no mesmo commit: OK. O furo do critic foi reproduzido e está fechado. |
| Linha 10 (D-14a, critic) | T2: `// To do: check the silent label.` no `en.js` | Novo: **NO-OK**. Antigo: OK. |
| Linha 10 (D-14a, critic) | T3: `// to do: revisar` no fim da linha `header.silent` do `pt-BR.js` | Novo: **NO-OK**. Antigo: OK. |
| Linha 10 (controle positivo) | T4: `// todo: …` no `pt-BR.js` | NO-OK no novo e no antigo. |
| Linha 10 (controles negativos) | K1: `// to-do(#12): …`, com referência; K2: comentário inócuo | OK nos dois. |
| Linha 3 (D-14b; M3r do critic, M14 do doer) | O `Stderr::report` passa pelo `crate::diagnose` | NO-OK. `report_prints_…` FAILED, com `left` só com a linha de troca, também com `DDC_TRAY_DEBUG=1`. |
| Linha 3 | M3o: `Stderr::report` = `crate::report(action, error)`, o código da iteração 2 | NO-OK, pelo mesmo FAILED. |
| Linha 3 | M3d: o sink só recebe o relatório com `DIAGNOSTICS` ligado | NO-OK, também com `DDC_TRAY_DEBUG=1`. |
| Linha 3 | M3p: o relatório sem o prefixo `ddc-tray: ` | NO-OK (`left` com `could not confirm…` sem o prefixo). |
| Linha 3 | M3e: o relatório perde o erro (`: monitor did not respond in time`) | NO-OK. |
| Linha 3 (refeita, o `announce` foi refatorado) | M3a: o `announce` ignora o sink e usa `eprintln!` | NO-OK. Os dois testes `stderr_tests` FAILED (`left: ""`). |
| Linha 3 (refeita) | M3b: o `announce` só escreve com `DIAGNOSTICS` ligado | NO-OK. Os dois FAILED, também com `DDC_TRAY_DEBUG=1`. |

**Herdado da iteração 2** (o código coberto não mudou desde `4e57ecd`):

| Linha | Mutações reprovadas |
|---|---|
| 1 | 7/7 |
| 2 | 3/3 |
| 3 | 4/4 da iteração 1 (retry, consentimento ignorando `enabled`, erro sem `report`, laço parado depois do disparo) |
| 4 | 4/4 |
| 5 | 4/4 e W4 |
| 6 | M6a (NO-OK); M6b informativa |
| 7 | 3/3 |
| 8 | 6/6, M8a e M8b, mais as 4 saídas 1 do script |
| 9 | 2/2 |
| 10 | 4/4 e 2 controles negativos (iteração 1) |
| PROJECT 1 | W1, W2 e W6 da iteração 2 |
| PROJECT 2 | 1 (77.82%) |
| PROJECT 3 | 1, mais o controle `TODO(#7)` |

## Blockers
- Nenhum.

## Warnings
Situação dos achados do critic da iteração 2:
- **Linha 10 (objetivo):** fechado. É a D-14a: T1, T2 e T3 dão NO-OK, e o Verify antigo dá OK nas três.
- **Linha 3 (não objetivo):** fechado. É a D-14b, no `c461825`: M14, M3o, M3d, M3p e M3e dão NO-OK.

Restam:
- **W4 (resto, DRY, menor, herdado):**
  - `apps/ddc-tray/src/demo-data.js:117-118` ainda escreve os nomes das entradas.
  - Os ids de teste `046d:c31c:KB0001`/`046d:c077` se repetem em vários módulos de teste do tray.
  - Não afeta o comportamento.
- **W6 (resto, higiene de testes, menor, herdado):**
  - `follow::learning_tests::the_system_clock_waits_for_real` (`follow.rs:1391`) ainda dorme 5 ms.
  - É o único teste do relógio real; os outros usam relógio injetável (D-2026-10-02-usb-switch-follow-3).
- **W7 (5.10, informativo, herdado):** o `cargo audit` aponta o yank do `yoke-derive 0.8.3`. Ele já estava em `420a3a1`.
- **W8 (informativo, herdado e ampliado):**
  - O `follow::spawn` público (`follow.rs:434-442`), que só delega ao `spawn_until`, não é exercido por teste unitário. O mesmo vale para o `Stderr::default()` (`follow.rs:463`).
  - O caminho de produção do `announce` é provado pelo smoke da linha 8: a M8b herdada, sobre o `Stderr::default()`, reprova a linha 8.
  - O `report` sobre o stderr real não roda no smoke, porque o monitor simulado responde. Mas ele usa o mesmo `print` do `announce`, que a linha 8 prova.
- **W9 (novo, DRY menor, nota sobre a D-7):**
  - O formato `ddc-tray: could not {action}: {error}` aparece em `lib.rs:325` (`crate::report`), em `follow.rs:203` (`Stderr::report`, novo) e em `follow.rs:686` (o `Recorder` de teste, que já existia).
  - A letra da D-2026-10-02-usb-switch-follow-7 diz que "o erro vai a `crate::report`". Os erros do follow agora saem por `Stderr::report`, que imprime a mesma linha no mesmo stderr.
  - A D-2026-10-02-usb-switch-follow-14(b), mais nova e da mesma phase, prescreve exatamente isso. A substância da D-7 continua de pé, e a mudança melhora o "sem pânico": a escrita ignora o erro em vez de entrar em pânico como o `eprintln!`. Logo, não é uma violação.
  - O risco é a deriva: uma mudança futura no `crate::report` não chegaria ao follow, e nenhum teste perceberia.
  - Sugestão para o PR: um `pub(crate) fn report_line(action, error) -> String` no `lib.rs`, usado pelos três.

## DoD Checklist (gate 8)

- **Extração:** os `Verify:` foram extraídos por programa, com a regex ``\*\*Verify:\*\* `([^`]*)` ``: 10 no CONTEXT e 3 no PROJECT. Não sobrou nenhum `<FREEZE>`. Os 9 fragmentos de congelamento das linhas 1–9 são iguais entre si e batem com `git rev-parse HEAD:<caminho>`.
- **Execução:** cada um rodou literalmente, a partir da raiz do repositório, com `env -i` + `bash --noprofile --norc`, `LC_ALL=C.UTF-8`, `HOME`, `PATH` com `~/.cargo/bin` e `DISPLAY`/`WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`/`DBUS_SESSION_BUS_ADDRESS` do ambiente.
- **Onde rodou:** as linhas 1–8 e 10 e os PROJECT 1–3 rodaram no bwrap com as iscas e o observador ligado, com 0 eventos. A linha 9 monta o próprio bwrap e rodou nativa.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | Core: `Follower` e `learn` seguem a D-3 com relógio fake (8 testes exatos de `app::usb_follow::tests::`) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 1 s. Mutações 7/7 herdadas; árvore do core sem mudança. |
| 2 | Adapter `usb_sysfs` numa raiz fake (5 testes exatos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 1 s. Mutações 3/3 herdadas; árvore do adapter sem mudança. |
| 3 | Laço do follow: escrita exata `[(RTK,0x60,0x10)]`, linha de troca, chegada/desligado `[]`, DDC mudo vai ao `report`, só o monitor configurado e `0x60`; o `Stderr` REAL com sink injetável imprime a linha de troca E o relatório `ddc-tray: could not …` com o diagnóstico DESLIGADO (D-13b, D-14b) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 4 s. O conjunto exato de `follow::stderr_tests::` são os 2 testes. M14/M3r, M3o, M3d, M3p e M3e (`report`) e M3a e M3b refeitas (`announce`) dão NO-OK. Os controles do congelamento se comportam. 4/4 herdadas. |
| 4 | Config (D-4): ida e volta, XDG→`~/.config`, ausente = desligado, versão desconhecida não sobrescrita, aprender não liga + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 2 s. Mutações 4/4 herdadas. |
| 5 | Menu e rótulos (D-5): check = `enabled`, só no Linux, ida e volta dos ids, literais en/pt-BR, `menu::`/`i18n::` antigos verdes + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 1 s. W4 e 4/4 herdadas. |
| 6 | Popup "em outra entrada" (D-6, D-12b, D-13a): frases literais, `tests/ui` inteira verde, rótulo RENDERIZADO `LG TV SSCR2 (em outra entrada)` no e2e em light e dark, Playwright inteiro sem `failed`/`flaky` + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 19 s. Unit: `# tests 166 # pass 166 # fail 0 # cancelled 0 # skipped 0 # todo 0` e `ok 159 - a monitor with a readable EDID and no DDC answer reads on another input in en and pt-BR`. Playwright: 154 passed; o teste novo dá ✓ em light (#19) e dark (#99). M6a herdada. |
| 7 | Doc de instalação nas duas máquinas (literais exigidos) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 1 s. Mutações 3/3 herdadas; `docs` sem mudança. |
| 8 | Smoke release em D-Bus privado, raiz USB fake, as 10 linhas exatas, `~/.config/autostart` e `~/.config/ddc-control` reais idênticos em nomes e sha256 (D-13c) + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 31 s. Os snapshots sha256 reais são iguais antes e depois, e o `~/.config/ddc-control` está ausente nos dois momentos. M8a, M8b e 6/6 herdadas. |
| 9 | Suíte dos 3 crates sem o `/sys` real e sem `/dev/i2c-*`, conferido dentro do sandbox + congelamento | CONTEXT | Auto | PASS | `OK`, saída 0, 3 s (nativa, com o bwrap próprio). Mutações 2/2 herdadas. A prova por inotify deu 0 aberturas. |
| 10 | Sem TODO/FIXME sem referência nos arquivos não-Rust da phase, com `en.js`/`pt-BR.js` só em comentário e incluindo `to-do`/`to do` (D-14a) | CONTEXT | Auto | PASS | `OK`, saída 0. T1, T2 e T3 dão NO-OK (o Verify antigo dá OK); T4 dá NO-OK; K1 e K2 dão OK. 4/4 e 2 controles negativos herdados. |
| 11 | `cargo test --workspace` sai 0 | PROJECT | Auto | PASS | `OK`, saída 0: 488 passed, 0 failed, 9 ignored. |
| 12 | Cobertura >= 80% de linhas | PROJECT | Auto | PASS | Comando literal `cargo llvm-cov --workspace --summary-only` (sem exclusões): `TOTAL … 5740 611 89.36%` ≥ 80%. Gate 3: 89.64%. Mutação herdada: 77.82%. |
| 13 | Sem `TODO`/`FIXME` sem referência de issue (`.rs`) | PROJECT | Auto | PASS | `OK`, saída 0. A mutação e o controle `TODO(#7)` são herdados. |
| 14 | CHANGELOG.md atualizado com entrada por release | PROJECT | Manual | MANUAL_REQUIRED | suggested: o CHANGELOG não mudou desde a iteração 2 (mesmo blob). `## [Unreleased]` tem um `Added` na linha 12 ("Follow USB switch" no menu Linux) e um `Changed` na linha 26 ("(on another input)" no lugar de "(no DDC/CI)"). Nenhum cabeçalho `## [versão]` novo; o último é `## [0.1.0] - 2026-09-28` (linha 34). |
| 15 | README descreve o comportamento atual | PROJECT | Manual | MANUAL_REQUIRED | suggested: o README não mudou desde a iteração 2 (mesmo blob). Põe os itens do follow no menu do clique direito (linha 330) e tem o tópico "**Follow USB switch** (Linux only)" (335). "(on another input)" aparece em 341 e 359, e a limitação "Linux only, and unproven on real hardware" em 400. |

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

- **Gates:** não há bloqueio. Os gates 1 a 4 e 7 estão limpos, o clippy do Windows também, e a suíte tem 488 testes verdes com cobertura de 89.64%.
- **Achados do critic:** os dois da iteração 2 estão fechados, e as mutações mostram isso:
  - a linha 10 agora cai com `to-do`/`To do`/`to do` em comentário dos arquivos de tradução, que o Verify antigo deixava passar;
  - a linha 3 cai sempre que o `report` ou o `announce` do `Stderr` real deixa de pôr a linha exata no sink com o diagnóstico desligado. Isso inclui a M3r do critic.
- **Comportamento de produção:** o `c461825` não muda o que o usuário vê; a linha impressa e o destino (stderr) são os mesmos.
- **Hardware:** a prova deu zero aberturas nos nós reais.

Os warnings W4, W6, W7, W8 e W9 são menores ou informativos e podem ficar para o PR. O W9, que unifica o formato do relatório num só lugar, é o mais barato de resolver.

## DoD Critic (enhanced)

Critic forçado pelo `/jdi-issue`, sub-agente somente leitura, iteração 3. Ele rodou num repositório descartável, só com git e bash: sem `cargo`, sem `/dev/i2c`, sem `/sys` real.

As 13 linhas Auto têm `hollow=false`. Os achados das iterações 1 e 2 estão fechados:
- a linha 6 confere o literal renderizado;
- a linha 3 cobre o `announce` e o `report` reais;
- a linha 8 compara por sha256;
- a linha 10 pega `to-do`/`To do`.

Notas opcionais, não objetivas:
- **Linha 6:** `playwright.config.mjs`/`package*.json` ficam fora do congelamento. Mudar o tema por config seria evasão.
- **Linha 8:** o Verify usa `target/release/ddc-tray` fixo, sem considerar `CARGO_TARGET_DIR`. Isso não acontece no ambiente limpo da D-11.

**Verdict:** APPROVED
