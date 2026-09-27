# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 4, revisada por `jdi-reviewer-ddc-control` em 2026-09-27.
- **Branch:** `phase/tray-app`, HEAD `c725c2e`.
- **Host:** Linux (Fedora 44, KDE Plasma 6 Wayland).
- **Escopo:** re-verificação completa (gates 1-8) depois da iteração 4 (`79f5a6a..c725c2e`). Ela trouxe o monitor simulado `DDC_TRAY_FAKE=1` e o smoke da roda (D-2026-09-27-tray-app-5), a checagem de `<select>` em runtime no Gate 7 e as correções de W-2, W-3 e W-4 da iter 3.
- **Reviews anteriores:** estão no histórico (`git show 77e7dc2:.jdi/phases/tray-app/REVIEW.md` = iter 3).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0 (inclui `ddc-tray`). O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked` sai 0 em `x86_64-unknown-linux-gnu` e em `x86_64-pc-windows-msvc`. O `ddc-tray` fica fora do msvc (D-2026-09-26-tray-app-9). Único aviso: o future-incompat de `nom v3.2.1` (via `ddc-hi`), anterior à phase. |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware). A iter 3 tinha 371: +12, sem remoção. Os 3 testes de `shortcut_target` mudaram de `commands/tests.rs` para `panel/tests.rs`. |
| Coverage | PASS | 83.22% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded). Na iter 3 eram 84.01%. O `main.rs` e o `build.rs` do tray não mudaram na iter 4. Lógica: `scroll.rs`, `popup.rs` e `fixture.rs` em 100%, `panel.rs` 99.31%. Cola: `stop_signals.rs` 62.07%, `tray.rs` 57.23%, `commands.rs` 49.32%, `kwin_placement.rs` 45.36%, `lib.rs` 37.75%, `status_item.rs` 23.03%. |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]`/`#[expect(...)]` fora de testes. A lint do workspace segue intacta (`unsafe_code = "deny"`, `unwrap_used`/`expect_used`/`panic` = warn). |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 limpos (detalhe abaixo). A 5.10 (`cargo audit`) sai ≠0 com os mesmos 4 advisories da iter 3 (W-1). |
| Consistency | PASS | D-1, D-2 e D-2026-09-26-tray-app-3/-7 estão conformes, assim como D-2026-09-27-tray-app-3 (agora sem ressalva) e -5. Os commits `79f5a6a..c725c2e` usam o escopo `tray-app`, e os tipos batem com o conteúdo. Os arquivos fora do PLAN estão declarados no SUMMARY. |
| UI Validation | PASS | Suíte Playwright de `apps/ddc-tray` com servidor próprio (`reuseExistingServer: false`, porta 1420 livre antes e depois): 52 passed, 6 skipped (screenshots sem `SCREENSHOTS=1`). Passam os 10 `critical_paths` × tema e os 16 testes do `dropdown.spec.mjs`, com console limpo, axe critical/serious `toEqual([])` e zero `<select>` em runtime. |
| DoD | PASS_PENDING_MANUAL | 20/20 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1–5.4 (hexagonal):** limpo.
  - O `ddc-core` segue só com `thiserror`, sem I/O e sem `cfg` de plataforma. O `crates/` não foi tocado na iter 4.
  - Não há `impl MonitorBackend` no core nem `pub trait` fora dele.
  - Os dois backends são construídos só no composition root:
    - `DdcHiMonitorBackend::new` em `apps/ddc-tray/src-tauri/src/lib.rs:77`;
    - `InMemoryMonitorBackend::builder()` em `lib.rs:87` (`simulated_osd`).
  - `fixture.rs` só tem dados (`FakeMonitor`), que o composition root e os testes consomem.
  - `scroll.rs` e `tray.rs` não importam mais `commands` (o `shortcut_target` foi para `panel.rs`). O acoplamento que a iter 3 observou acabou.
- **5.5 (`unsafe`):** a única ocorrência é o comentário `lib.rs:146`. `#![forbid(unsafe_code)]` está em `lib.rs:8` e em `main.rs`.
- **5.6 (pânicos):** fora o doc comment `crates/ddc-adapters/src/ddc_hi_backend/worker.rs:141` (anterior à phase), todos os `unwrap()` ficam dentro de `#[cfg(test)]`:
  - `kwin_placement.rs:246-267`;
  - `lib.rs:269`;
  - `stop_signals.rs:105-108`.
  
  O `std::process::exit(1)` de `stop_signals.rs:78` é deliberado: é o 2º sinal durante a saída, e não é pânico.
- **5.7 (escrita segura):**
  - `Confirm::Yes` só aparece em `commands.rs`. O atalho e a roda gravam só brilho, com `Confirm::No` (`panel::write_brightness`).
  - Os 7 testes de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs` são `#[ignore]` e gated por `DDC_HW_TESTS=1`. O revisor não rodou nenhum deles.
- **5.8, 5.9 e 5.11:** limpos. Nenhum caminho de dispositivo no core, todos os `#[tauri::command]` são `async fn`, e não há segredo nem TODO/FIXME sem issue.
- **5.10:** W-1.

### Revisão dirigida (pedido do orquestrador)

**Sinais (`stop_signals.rs`, `c2ff8aa`) — correto.**
- **Como funciona:**
  - Só existe no Linux (`#[cfg(target_os = "linux")]`, com `tokio` só em `[target.'cfg(target_os = "linux")'.dependencies]`).
  - Os handlers são registrados no `setup`, dentro do runtime do Tauri (`runtime.inner().enter()`), antes de a task nascer. Assim não sobra janela, depois do setup, em que um sinal ainda mata o app sem passar pela saída.
  - A 2ª instância sai no plugin single-instance antes do `setup`, então nunca registra handlers.
  - O 1º sinal chama `app.exit(0)`, que leva ao `RunEvent::Exit` e ao `tray::uninstall`. Um 2º sinal durante a saída faz `process::exit(1)`.
  - O arquivo foi revisto por inteiro, e o comportamento conferido ao vivo com uma instância própria (`DDC_TRAY_FAKE=1`, sem popup):
    | Sinal | Saída | KWin `isScriptLoaded` | Arquivo em `$XDG_RUNTIME_DIR` | stderr |
    |---|---|---|---|---|
    | SIGINT | exit 0 em 33 ms | `b true` → `b false` | removido | `SIGINT received, quitting` / `popup placement unloaded from KWin` |
    | SIGHUP | exit 0 em 33 ms | `b true` → `b false` | removido | idem, com SIGHUP |
    | SIGTERM (fim dos smokes C11 e C15) | — | `b false` depois de cada smoke | — | — |
  - Dois SIGTERM enviados um logo após o outro coalescem num só (sinal POSIX padrão não enfileira): exit 0 em 30 ms, uma linha `received`. Isso é esperado. O caminho do 2º sinal só aparece com uma saída lenta, e o doer o provou com um mutante que trava a saída por 30 s.
- **Timeout do D-Bus na saída:**
  - `kwin_placement::uninstall` envolve conexão + `unloadScript` em `tokio::time::timeout(QUIT_TIMEOUT = 2 s)`. O `block_on` roda na thread principal, fora dos workers do runtime, que tem o driver de tempo (`enable_all`).
  - Com timeout, o app reporta, apaga o arquivo e sai. O doer provou 2026 ms com um mutante `pending()`, e o código confere.
  - Agora o uninstall só roda quando existe `LoadedScript` no estado, ou seja, só quando o app carregou o script. Fora do KDE Wayland ele não chama mais `org.kde.KWin`.
  - O `remove_file` virou `remove_script` (NotFound conta como removido), com a falha reportada.
- **W-3 (iter 3) resolvido:** sem `XDG_RUNTIME_DIR`, ou com um caminho relativo ou vazio, `script_path` → `None`, e nada é escrito. Não há mais fallback para `/tmp`. Há 2 testes puros.

**`DDC_TRAY_FAKE` — não muda o padrão.**
- **Ativação:** `switch_on` só aceita exatamente `1`. `""`, `0`, `true`, `1 ` e `yes` são testados como desligados (`lib.rs:289`).
- **Padrão intacto:** com a variável ausente, `compose_osd` segue o caminho real de antes, com `default_cache_dir` e `CachingMonitorBackend`.
- **Aviso:** ligada, o app sempre imprime `SIMULATION_NOTICE` no stderr. Um teste prende o texto que o smoke espera ao do app.
- **Isolamento:** nenhum teste altera o ambiente (`set_var` exigiria `unsafe`), e a variável não é lida em nenhum outro lugar do código de produção.
- **Prova de que o `--fake --scroll` não tocou o hardware:** o backend real cria a thread `ddc-hi-worker` já no `new()` (`crates/ddc-adapters/src/ddc_hi_backend/worker.rs:385`), e a instância do usuário a mostra.
  - Durante o C15, amostrei o PID do smoke (`target/release/ddc-tray`, 46 amostras): 0 amostras com `ddc-hi-worker` e 0 fds `/dev/i2c-*`.
  - Então o backend real nunca foi construído, e as escritas `75 -> 80 -> 75` foram só na memória.
- **Ressalva:** o teste de hardware pode passar oco se a variável vazar no ambiente (W-2).

## Blockers (if any)
- Nenhum.

## Warnings (if any)
- **W-1: 5.10 supply chain (`cargo audit` sai ≠0).** São os mesmos 4 advisories da iter 3, e nenhum vem de dependência da iter 4 (o lock só ganhou a aresta `ddc-tray → tokio 1.53.1`, sem crate novo).
  - RUSTSEC-2024-0429: `glib 0.18.5`, unsound.
  - RUSTSEC-2024-0370: `proc-macro-error 1.0.4`, sem manutenção. Este e o anterior vêm via tauri → gtk-rs.
  - RUSTSEC-2018-0005: `serde_yaml 0.7.5`.
  - RUSTSEC-2024-0320: `yaml-rust 0.4.5`. Este e o anterior vêm via `ddc-hi` → `mccs-db`.
  
  O tratamento (`audit.toml`, com cada ignore justificado) fica com a phase `ci-crossbuild`, como combinado.
- **W-2: a evidência de hardware pode sair oca se `DDC_TRAY_FAKE=1` estiver no ambiente.**
  - **Onde:** `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs:121` monta o core por `compose_osd()`, que respeita a variável (`lib.rs:73`).
  - **Por que passa:** o monitor simulado tem o mesmo id e o mesmo modelo do RTK real (`fixture.rs:12`, `:32`). Com a variável vazada, `rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored` passaria contra a memória (75 → 85 → 75) e produziria a "saída de hardware" que o PR espera.
  - **O que atenua:**
    - no mesmo filtro, `rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads` falharia, porque o fixture não tem monitor mudo;
    - com `--nocapture`, o aviso de simulação aparece;
    - o smoke só define a variável para o processo filho.
  - **Sugestão:**
    - no teste, recusar o modo simulado: `hardware_enabled()` exige também `std::env::var_os("DDC_TRAY_FAKE").is_none()`, ou o teste falha com mensagem clara;
    - até lá, o orquestrador roda o teste de hardware com `env -u DDC_TRAY_FAKE`.
  - **Severidade:** não bloqueia. O modo simulado nunca escreve num monitor real; o risco é só de prova falsa.

Observações sem severidade:
- **W-2, W-3 e W-4 da iter 3: resolvidos.** W-2 e W-3 estão provados acima. O SUMMARY agora cita o `tokio-macros`.
- **Binário que o usuário está testando:** `~/.local/bin/ddc-tray` (01:29) é um build da iter 3 e não tem as correções da iter 4 (0 ocorrências de `received, quitting` e `DDC_TRAY_FAKE` nas strings).
  - Por isso, o `pkill -x` do protocolo deixa o script do KWin dele carregado (`b true` logo depois do `pkill`). O próximo start o substitui.
  - Para testar W-2/W-3 na máquina dele, o usuário precisa reinstalar a partir de `c725c2e`.
- **Corrida pequena:**
  - O script do KWin é carregado numa task assíncrona (`status_item.rs:43-47`), e o `LoadedScript` só entra no estado quando o load termina.
  - Um sinal que chegue nessa janela (dezenas de ms no start) deixa o script carregado, com o mesmo efeito limitado de um SIGKILL: só casa com o PID morto e é substituído no próximo start. Aceitável.
- **Sem marca visual do modo simulado:** o modo simulado só se anuncia no stderr. O popup e o tooltip mostram "RTK QHD HDR" como se fosse real. Isso basta para o propósito (testes). Se o modo algum dia for voltado ao usuário, merece uma marca na UI.
- **`tokio` como dependência direta:** entrou sem D-XX própria. Só no Linux; não é crate novo (só a aresta no lock) e era a correção sugerida pelo W-2 da iter 3. Não contradiz D-2026-09-26-tray-app-2 nem D-2026-09-27-tray-app-3 ("sem pacote novo").
- **Redação do SUMMARY:** diz que o `--fake` exige o aviso "antes de qualquer outra coisa". O script procura o aviso em todo o stderr, depois do registro e antes de `Activate`/`Scroll`. O efeito é o mesmo; só a frase é mais forte.
- **Cobertura:** a margem sobre 80% é de 3.2 pontos. A cola de plataforma segue baixa (ver tabela). Próximas phases devem continuar extraindo lógica pura.
- **Windows:** `tray/notification_area.rs` não mudou na iter 4 e só usa `TRAY_ID`, `run_menu_action` e `toggle_popup`, cujas assinaturas não mudaram. A compilação real fica para a `ci-crossbuild`.
- **Estado final da máquina:**
  - Só a instância do usuário está viva: `pgrep -xa ddc-tray` → `833037 /home/slipalison/.local/bin/ddc-tray`, com o script do KWin recarregado com o PID dela (`b true`).
  - A porta 1420 está livre.
  - `git status` não mudou (fora este REVIEW.md), e os 6 PNGs de `docs/screenshots/` ficaram byte a byte iguais (SHA-1 antes e depois).

## DoD Checklist (gate 8)

Cada `Verify:` do CONTEXT.md e do PROJECT.md foi extraído por script para um arquivo, com checagem de que é substring exata do `.md`, e executado com `bash` a partir da raiz. O item 2 do PROJECT rodou literalmente: `cargo llvm-cov --workspace --summary-only`, sem exclusões.

Os smokes (itens 14 e 15) seguiram o protocolo do orquestrador:
1. `pgrep -xa ddc-tray` mostrava a instância do usuário, que foi encerrada com `pkill -x ddc-tray`.
2. O smoke rodou.
3. Logo depois, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu (PIDs 830964, depois 831768 e, por fim, 833037, após a prova de sinais).

Os 2 itens `Manual` do CONTEXT repetem os do PROJECT (Source: PROJECT) e foram contados uma vez só.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`; 383 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal: TOTAL lines 82.78%; forma do gate (`--fail-under-lines 80`, sem `main.rs`/`build.rs`): 83.22%, exit 0 |
| 3 | No `TODO`/`FIXME` without linked issue (`*.rs`) | PROJECT | Auto | PASS | `OK` (Verify da D-2026-09-27-tray-app-4) |
| 4 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1× em `lib.rs:77`) |
| 7 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` (7 `pub fn …<M: MonitorControl + ?Sized>`) |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` |
| 9 | `node --test` (debounce, view-model, bridge demo…) sem falhas, fora de `src/` | CONTEXT | Auto | PASS | `OK`; `# tests 124`, `# pass 124`, 0 fail/cancelled/skipped/todo |
| 10 | Paridade i18n + HTML sem texto literal | CONTEXT | Auto | PASS | `OK` |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 13 | `tauri-plugin-single-instance` no composition root | CONTEXT | Auto | PASS | `OK` (1ª chamada do builder, `lib.rs:108`) |
| 14 | Smoke Linux/KDE `--activate` (PID próprio, popup mostrado e mantido) | CONTEXT | Auto | PASS | `OK`; PID 831525 → item `org.kde.StatusNotifierItem-831525-1`, `popup shown` após `Activate`, ainda mostrado 1.5 s depois; backend real só com leituras |
| 15 | Teste `#[ignore]` de hardware existe, compila e é gated | CONTEXT | Auto | PASS | `OK`; 2 `#[test]` = 2 `#[ignore]` = 2 listados com `--ignored --list`; não executado (regra) |
| 16 | Gate 7: console limpo e axe sem critical/serious nos `critical_paths` | CONTEXT | Auto | PASS | `OK`; 52 passed, 6 skipped (= screenshots), 10/10 `critical_paths` × tema, 16/16 dropdown |
| 17 | Nenhum `<select>` nativo no popup | CONTEXT | Auto | PASS | `OK` (estático); em runtime, `expectNoNativeSelect` em todo `open()`, antes de cada axe e no teardown |
| 18 | Linux: SNI `ksni`, roda → brilho testada em Rust + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`; `75 -> 80` (vertical +120), horizontal sem escrita em 1.5 s, `80 -> 75` (−120); 0/46 amostras com `ddc-hi-worker` ou fd de i2c |
| 19 | Nenhum `TODO`/`FIXME` sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | `OK` |
| 20 | Screenshots claro/escuro gerados pela suíte e versionados | CONTEXT | Auto | PASS | `OK`; regeneração com `SCREENSHOTS=1` sem nada pulado, 720×1120, `git diff --quiet` e SHA-1 dos 6 PNGs iguais antes e depois |
| 21 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `CHANGELOG.md:8` `## [Unreleased]` com o tray app; ainda sem heading de versão (release em `release-packaging`) |
| 22 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`), com `DDC_TRAY_FAKE`/`DDC_TRAY_DEBUG` (`:358-359`) e sinais (`:315`, `:317`) conferidos contra o código; diff `main..HEAD` +432 linhas a revisar no PR |

**Totals:** 22 items | Auto: 20 (20 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Não há blocker. Os gates 1-7 passam, com W-1 (herdado, fica para a `ci-crossbuild`) e W-2 (novo, de baixa severidade). Os 20 itens Auto do DoD passam, e os 2 Manual aguardam o `/jdi-confirm-dod tray-app`.

Os pontos que o orquestrador pediu para olhar estão sólidos:
- **Sinais:** SIGTERM, SIGINT e SIGHUP fazem saída graciosa em ~30 ms e descarregam o script.
- **Timeout do D-Bus na saída:** limitado a 2 s.
- **`DDC_TRAY_FAKE`:** só liga com `1`, sempre se anuncia, não muda o padrão e, em runtime, não criou o backend real.

Antes de rodar o teste de hardware para o PR (Deferred), garanta `env -u DDC_TRAY_FAKE` ou aplique a guarda do W-2. Isso evita uma evidência de hardware que na verdade veio do monitor simulado. Vale também avisar o usuário de que a instância instalada em `~/.local/bin` é da iter 3.

## DoD Critic (enhanced)

- DoD row «9 (node --test)»: remover `stopTimer(state)` do `push` em `debounce.js` (esquecer o `clearTimeout` — debounce vira throttle: um arrasto de 400 ms gera 17 escritas em vez de 1) passa nos 124 testes e no Verify — nenhum teste tem rajada mais longa que a janela de 80 ms.
- DoD row «16 (Gate 7)»: `.disableRules(['color-contrast'])` no `AxeBuilder` de `support.mjs` (ou `BLOCKING_IMPACTS` reduzido a `critical`) silencia uma violação serious real e o Verify imprime OK.
- Suspeitas (objective:false): «17» `element('select', 'classe')` escapa do grep (o Gate 7 pega em runtime); «15» uma segunda escrita Safe sem restauração passaria nos greps; «3/19» `todo` minúsculo; «7» caminho inline `super::…`. Nota fora do critério: sem `__TAURI__` no app real o bridge cai na demo silenciosamente.

**Verdict:** BLOCKED
