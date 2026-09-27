# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 3, revisada por `jdi-reviewer-ddc-control` em 2026-09-27.
- **Branch:** `phase/tray-app`, HEAD `0b02ca7`.
- **Host:** Linux (Fedora 44, KDE Plasma 6 Wayland).
- **Escopo:** re-verificação completa (gates 1-8) depois da iteração 3 (`8d26af0..0b02ca7`: dropdown in-page, StatusNotifierItem `ksni`, ancoragem por script do KWin, Playwright com servidor próprio) e do DoD endurecido em `896daf7` (D-2026-09-27-tray-app-1..4).
- **Reviews anteriores:** estão no histórico (`git show 2f16b25:.jdi/phases/tray-app/REVIEW.md` = iter 2).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0 (inclui `ddc-tray`). O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked` sai 0 em `x86_64-unknown-linux-gnu` e em `x86_64-pc-windows-msvc`. O `ddc-tray` fica fora do msvc de propósito (D-2026-09-26-tray-app-9). Único aviso: o future-incompat de `nom v3.2.1` (via `ddc-hi`), anterior à phase. |
| Tests | PASS | 371 passed, 0 failed, 9 ignored (hardware). Na iter 2 eram 344: entraram +27, sem remoção. |
| Coverage | PASS | 84.01% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded). Na iter 2 era 88.79%. A queda vem da cola que só roda com sessão gráfica: `status_item.rs` 24.31%, `lib.rs` 27.64%, `kwin_placement.rs` 36.72%, `commands.rs` 54.60%, `tray.rs` 59.64%. A lógica nova está em `scroll.rs` 100% e `popup.rs` 100%. O `main.rs` do tray não mudou na iter 3. |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]`/`#[expect(...)]` fora de testes. `[lints] workspace = true` no `ddc-tray`. |
| Hexagonal/Safety/Hygiene | WARN | As checagens 5.1–5.9 e 5.11 estão limpas (detalhe abaixo). A 5.10 (`cargo audit`) sai ≠0 com os mesmos 4 advisories da iter 2 (W-1); os crates novos no lock (`ksni`, `pastey`, `tokio-macros`) não têm advisory. |
| Consistency | WARN | D-1, D-2 e D-2026-09-26-tray-app-2/-3/-5/-7 (com as emendas da D-2026-09-27-tray-app-2) conformes, assim como D-2026-09-27-tray-app-1/-4. A D-2026-09-27-tray-app-3 está conforme com ressalva: o script só é descarregado na saída graciosa (W-2). Commits `8d26af0..0b02ca7` com escopo `tray-app` e tipos coerentes. Um desvio menor no SUMMARY (W-4). |
| UI Validation | PASS | Suíte Playwright de `apps/ddc-tray`, com servidor próprio (`reuseExistingServer: false`, porta 1420 livre antes e depois): 52 passed, 6 skipped (screenshots sem `SCREENSHOTS=1`). Os 10 `critical_paths` × tema e os 16 testes do `dropdown.spec.mjs` passam, com console limpo e sem violação axe critical/serious, inclusive com a lista aberta. |
| DoD | PASS_PENDING_MANUAL | 20/20 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1–5.4 (hexagonal):** limpo.
  - `ddc-core` segue só com `thiserror`, sem I/O e sem `cfg` de plataforma.
  - Não há `impl MonitorBackend` no core nem `pub trait` fora dele.
  - `DdcHiMonitorBackend::new` aparece só em `apps/ddc-tray/src-tauri/src/lib.rs:56` (`compose_osd`).
  - O código novo é Rust puro sobre a porta:
    - `scroll.rs`: `scroll_brightness`/`drain_wheel` genéricos em `M: MonitorControl + ?Sized`, testados com `InMemoryMonitorBackend`;
    - `tray::brightness_shortcut`.
  - `status_item.rs` e `kwin_placement.rs` são cola de plataforma do adapter de entrada, sem nenhuma regra de DDC.
  - `crates/` não foi tocado na iter 3.
- **5.5 (`unsafe`):** a única ocorrência é o comentário `lib.rs:113`. `#![forbid(unsafe_code)]` está em `lib.rs:8` e em `main.rs:4`.
- **5.6 (pânicos):** só aparecem o doc comment `crates/ddc-adapters/src/ddc_hi_backend/worker.rs:141` (anterior à phase) e um `unwrap()` em `kwin_placement.rs:165`, que fica dentro de `#[cfg(test)] mod tests`.
- **5.7 (escrita segura):**
  - `Confirm::Yes` só aparece em `commands.rs:112`.
  - A roda escreve só brilho, `Safe`, com `Confirm::No` (`scroll.rs:146`): se o core classificasse como `Dangerous`, recusaria.
  - Os 7 testes de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs` são `#[ignore]` e gated por `DDC_HW_TESTS=1`. O revisor não rodou nenhum deles.
  - O smoke só abriu o popup, que só lê.
- **5.8, 5.9 e 5.11:** limpos.
  - Nenhum caminho de dispositivo no core.
  - Todos os `#[tauri::command]` são `async fn`. O `select_monitor` ganhou `AppHandle` e segue `async`.
  - Nenhum segredo e nenhum TODO/FIXME sem issue.
- **Frontend (CSP, D-2026-09-26-tray-app-7):** o `dropdown.js` usa só `textContent`/`setAttribute` e as custom properties `--dropdown-*` via `style.setProperty`. Não há `innerHTML`/`eval`/`new Function` em `src/`.

### Avaliação crítica do script do KWin (D-2026-09-27-tray-app-3)
Arquivos: `apps/ddc-tray/src-tauri/kwin/anchor.js` e `apps/ddc-tray/src-tauri/src/tray/kwin_placement.rs`.

- **Escopo: correto e estreito.** `isPopup` (`anchor.js:16-18`) exige ao mesmo tempo `pid === POPUP_PID`, `resourceClass === "ddc-tray"` e `caption === "DDC Control"`.
  - O script só assina `workspace.windowAdded` e só altera a janela que casa: geometria, `skipTaskbar`, `skipSwitcher`, `skipPager` e `keepAbove`.
  - Não usa `callDBus`, atalhos nem configuração persistente.
  - O conteúdo é constante de compilação (`include_str!`). A única substituição é o PID (`u32` → dígitos), então não há como injetar código.
  - O arquivo vai para `$XDG_RUNTIME_DIR`, que é 0700 e do usuário.
  - Não existe arquivo de ativação D-Bus para `org.kde.KWin` nesta máquina, então chamar KWin fora do KDE não o inicia.
- **Ciclo de vida: parcial.**
  - Na saída graciosa (menu **Sair** → `RunEvent::Exit`), `uninstall` descarrega o script e apaga o arquivo.
  - SIGTERM, SIGKILL ou um crash deixam o script carregado. Prova ao vivo: logo depois do smoke (que encerra com SIGTERM), `isScriptLoaded ddc-tray-anchor` = `b true`, e o arquivo trazia `POPUP_PID = 674938`, o PID do smoke já morto. Ao reabrir a instância do usuário, `load()` descarregou o script órfão pelo nome e carregou o novo (`POPUP_PID = 675118`, `b true`).
  - Uma 2ª instância não mexe no script de quem já roda. O `tauri-plugin-single-instance` 2.5 faz `std::process::exit(0)` no setup do plugin, antes do `setup` do app e sem disparar `RunEvent::Exit`.
- **Riscos para a sessão: baixos.**
  - O script órfão só casa com um PID morto e com a classe e o título do próprio app; na prática, só outro `ddc-tray`, que o substitui ao subir.
  - `loadScript` não persiste nada: o script some no logout ou num restart do KWin.
  - Os riscos residuais estão em W-2 e W-3: não descarrega no SIGTERM, `block_on` sem timeout na saída, e fallback para `/tmp`.
- **Fragilidade para a `release-packaging` (observação):**
  - A `resourceClass` observada é o nome do binário (`ddc-tray`). Não vem do `productName` nem do `identifier`.
  - O teste `the_script_looks_for_the_class_and_title_the_popup_has` prende a classe ao `CARGO_PKG_NAME`, não ao nome do executável instalado.
  - Se um pacote renomear o binário, a ancoragem para de funcionar sem aviso. É uma falha segura: o popup volta a abrir centralizado.

## Blockers (if any)
- Nenhum.

## Warnings (if any)
- **W-1: 5.10 supply chain (`cargo audit` sai ≠0).** São os mesmos 4 advisories da iter 2, nenhum vindo de crate novo da iter 3. O tratamento (`audit.toml`, com cada ignore justificado) fica com a phase `ci-crossbuild`.
  - RUSTSEC-2024-0429: `glib 0.18.5`, unsound.
  - RUSTSEC-2024-0370: `proc-macro-error 1.0.4`, sem manutenção. Este e o anterior vêm via tauri → gtk-rs.
  - RUSTSEC-2018-0005: `serde_yaml 0.7.5`.
  - RUSTSEC-2024-0320: `yaml-rust 0.4.5`. Este e o anterior vêm via `ddc-hi` → `mccs-db`.
- **W-2: a D-2026-09-27-tray-app-3 diz que o script "é descarregado quando o app sai", mas isso só vale na saída graciosa.**
  - **Onde:** `apps/ddc-tray/src-tauri/src/lib.rs:102-106` (só `RunEvent::Exit`) e `tray/status_item.rs:63-65`. Evidência ao vivo na avaliação acima.
  - **O que o SIGTERM pula:** é o sinal normal de `pkill`, de gerenciador de sessão e de systemd. Com ele, o script fica carregado até o próximo start.
  - **Ainda no caminho de saída:**
    - `uninstall` roda `tauri::async_runtime::block_on` na thread principal com chamadas D-Bus sem timeout. O `zbus` 5.19 usa `method_timeout: None` por padrão, então um KWin travado trava o **Sair**.
    - O `uninstall` também não é condicionado a `applies()` e chama `org.kde.KWin` em qualquer sessão Linux. Hoje isso é inócuo: a chamada falha com ServiceUnknown.
    - `kwin_placement.rs:73` descarta o `Result` do `remove_file` com `let _ =` sem comentário (clean-code). A intenção de "melhor esforço" deveria estar escrita.
  - **Sugestão:**
    1. Tratar SIGTERM/SIGINT/SIGHUP com `tokio::signal::unix`, API segura sobre o runtime que já existe, chamando `app.exit(0)` para que o `RunEvent::Exit` rode.
    2. Dar um timeout curto às chamadas de saída, por exemplo via `method_timeout` no builder da conexão.
    3. Qualificar o `README.md:315` e o `CHANGELOG.md:46`. O `docs/hardware-validation.md:165` já documenta o caso.
- **W-3: segurança, defesa em profundidade. Código executável pelo KWin pode ir parar num caminho previsível de `/tmp`.**
  - **Onde:** `script_path()` (`kwin_placement.rs:129-133`) cai em `std::env::temp_dir()` com o nome fixo `ddc-tray-kwin-anchor.js` quando falta `XDG_RUNTIME_DIR`. Depois, `std::fs::write` (que segue symlink) grava o JS e o KWin o executa.
  - **Ataque possível:** num sistema sem `fs.protected_regular`/`fs.protected_symlinks`, outro usuário local poderia criar o arquivo antes e trocar o conteúdo entre o `write` e o `loadScript`. O resultado seria JS rodando no KWin da vítima, com `callDBus` na sessão dela.
  - **Por que é pouco alcançável hoje:**
    - `applies()` exige Wayland, e numa sessão Wayland o `XDG_RUNTIME_DIR` sempre existe.
    - O Fedora do dev tem `fs.protected_regular = 1` e `fs.protected_symlinks = 1`.
  - **Sugestão:** sem `XDG_RUNTIME_DIR`, não carregar o script (fica só o diagnóstico), em vez de cair em `/tmp`.
- **W-4: consistência do SUMMARY (menor).** O SUMMARY diz "O `Cargo.lock` ganhou só `ksni` e `pastey`", mas o diff `2f16b25..0b02ca7 -- Cargo.lock` também adiciona `tokio-macros` 2.7.2. O crate vem da feature `macros` do `tokio`, puxada pelo `ksni` com `tokio`; é oficial do projeto tokio e não tem advisory. Só a descrição está incompleta.

Observações sem severidade:
- **Arquivos fora do `files_modified` do PLAN** (`dropdown.js`, `scroll.rs`, `tray/{status_item,notification_area,kwin_placement}.rs`, `kwin/anchor.js` e testes) foram pedidos pelas D-2026-09-27-tray-app-1..3 nesta iteração. O SUMMARY os lista em "Desvios e observações". Não é inconsistência.
- **Windows:**
  - `tray/notification_area.rs` (159 linhas, código antigo reorganizado) não é compilado por nenhum gate neste host (D-2026-09-26-tray-app-9).
  - O doer o compilou num experimento descartável. A prova real fica para a `ci-crossbuild`.
- **Acoplamento leve:** `scroll.rs:17` importa `crate::commands::shortcut_target`, uma função pura que mora no módulo dos comandos Tauri. Movê-la para `panel.rs` deixaria o `scroll.rs` sem depender de um módulo que importa `tauri`. Não viola a D-1.
- **Cobertura:** a margem sobre 80% caiu de 8.8 para 4.0 pontos. Próximas phases que adicionarem cola de plataforma devem extrair a lógica como foi feito com `scroll.rs` e `popup.rs`.
- **Estado final da máquina:**
  - Só a instância do usuário está viva: `pgrep -xa ddc-tray` → `675118 /home/slipalison/.local/bin/ddc-tray`.
  - O script do KWin está carregado com o PID dela.
  - A porta 1420 está livre.
  - `git status` não mudou, e os screenshots regenerados ficaram byte a byte iguais.

## DoD Checklist (gate 8)

Cada `Verify:` do CONTEXT.md e do PROJECT.md foi copiado literalmente para um script e executado com `bash` a partir da raiz do repo. Antes de rodar, conferi que cada script é substring exata do `.md`. O item 2 do PROJECT também foi rodado literalmente (`cargo llvm-cov --workspace --summary-only`, sem exclusões).

Para o smoke (item 14), segui o protocolo do orquestrador:
1. `pgrep -xa ddc-tray` mostrava `/home/slipalison/.local/bin/ddc-tray`, então a instância foi encerrada com `pkill -x ddc-tray`.
2. O smoke rodou.
3. Logo em seguida, `setsid -f /home/slipalison/.local/bin/ddc-tray` reabriu a instância, que voltou viva como PID 675118.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`: 371 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal: TOTAL lines 83.54% (com `main.rs`); forma do gate: 84.01% (exit 0 com `--fail-under-lines 80`) |
| 3 | No `TODO`/`FIXME` without linked issue reference (Verify novo da D-2026-09-27-tray-app-4) | PROJECT | Auto | PASS | `OK` |
| 4 | fmt + clippy `-D warnings` limpos incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Composition root é o único ponto que constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1 ocorrência, `lib.rs:56`) |
| 7 | `panel.rs` é Rust puro sobre `MonitorControl`, sem o runtime do Tauri | CONTEXT | Auto | PASS | `OK` |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` (`lib.rs:8`, `main.rs:4`) |
| 9 | `node --test` por módulo (inclui `dropdown`), 0 falhas, 0 cancelled, testes fora de `src/` | CONTEXT | Auto | PASS | `OK`. Por módulo: debounce 13, view-model 25, bridge-demo 19, contract 4, i18n-parity 13, i18n-html 13, dropdown 28. Total `# tests 124`, `# pass 124`, `# fail 0`, `# cancelled 0` (inclui kwin-anchor 9) |
| 10 | Paridade i18n en/pt-BR e HTML sem texto literal | CONTEXT | Auto | PASS | `OK` (13 + 13 pass) |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` (base + por plataforma) | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 13 | `tauri-plugin-single-instance` como 1ª chamada do builder | CONTEXT | Auto | PASS | `OK` (`lib.rs:78`) |
| 14 | Smoke Linux/KDE `--activate`: item do próprio PID, vivo, popup mostrado após `Activate`, sem `panicked` | CONTEXT | Auto | PASS | `OK`: o PID 674938 era dono de `org.kde.StatusNotifierItem-674938-1/StatusNotifierItem`. `Activate` fez o app imprimir `ddc-tray: popup shown`; o app continuou vivo e nunca entrou em pânico |
| 15 | Teste `#[ignore]` de hardware RTK existe, compila, gated, sem escrita confirmada | CONTEXT | Auto | PASS | `OK`: `--ignored --list` mostra os 2 testes `rtk_qhd_hdr*`. O revisor não os executou |
| 16 | Gate 7: 0 erro de console, 0 axe critical/serious, servidor próprio, dropdown | CONTEXT | Auto | PASS | `OK`: exit 0, 10/10 `critical_paths` × tema com ✓, 16 ✓ no `dropdown.spec.mjs`, `52 passed`, `6 skipped` |
| 17 | Nenhum `<select>` nativo no popup (D-2026-09-27-tray-app-1) | CONTEXT | Auto | PASS | `OK` |
| 18 | Linux: SNI `ksni`, roda → percentual testado em Rust puro | CONTEXT | Auto | PASS | `OK` (`ksni = …` no Cargo.toml; `test result: ok. 19 passed` com filtro `scroll`) |
| 19 | Nenhum `TODO`/`FIXME` sem issue nos arquivos não-Rust do tray | CONTEXT | Auto | PASS | `OK` |
| 20 | Screenshots claro/escuro regenerados, byte a byte iguais aos versionados | CONTEXT | Auto | PASS | `OK` (720×1120; `git diff --quiet` limpo depois da regeneração; claro ≠ escuro) |
| 21 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (`CHANGELOG.md:8`) com os itens `ddc-tray` (`:40-46`), incluindo `ksni` e o script do KWin. Ainda não há heading `## [version]` (nenhum release) |
| 22 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## Tray app` (`README.md:265`), com clique, roda e ancoragem (`:310-315`), listas in-page (`:318`) e `### Known limitations of the tray app` (`:361`). Ver W-2: `:315` diz "unloads it when it quits" sem a ressalva do SIGTERM |

**Totals:** 22 items | Auto: 20 (20 PASS, 0 FAIL) | Manual: 2 pending

Os 2 itens Manual aparecem nas duas DoD (PROJECT e CONTEXT, `Source: PROJECT`) e foram contados uma vez.

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation

A phase está pronta para o ship, pendente só das confirmações humanas.
- **Estado da iter 3:** os dois bugs relatados pelo usuário foram corrigidos com prova objetiva.
  - O `<select>` nativo virou dropdown in-page, com testes unitários e e2e de 0/1 escrita e de Esc.
  - O clique no ícone do Linux passou a usar `ksni`, com o smoke `--activate` verde no PID próprio.
  - A ancoragem no KWin funciona e tem escopo correto.
  - Nenhum gate bloqueia.
- **Antes do `/jdi-ship`:**
  1. `/jdi-confirm-dod tray-app` para os 2 itens Manual (CHANGELOG/README). Ao revisar o README, considere a ressalva do SIGTERM (W-2).
  2. O orquestrador roda `DDC_HW_TESTS=1 cargo test -p ddc-tray --locked --test rtk_qhd_hdr -- --ignored --test-threads=1 --nocapture` e cola a saída no PR.
  3. Validação humana no PR (já em "Deferred to PR review"): clique real no ícone do painel do topo (ancoragem) e roda sobre o ícone no monitor real.
- **Sugestões baratas, que não bloqueiam e podem entrar agora ou numa phase futura:**
  - W-2: tratar SIGTERM/SIGINT/SIGHUP para rodar o `uninstall`, pôr timeout nas chamadas D-Bus de saída e comentar o `let _ =`.
  - W-3: remover o fallback para `/tmp` em `script_path()`.
  - W-4: completar a frase do lock no SUMMARY.
- **W-1** segue para a `ci-crossbuild`. Na `release-packaging`, manter o nome do binário `ddc-tray`, que é a `resourceClass` casada pelo script do KWin, ou derivar a classe do executável.

## DoD Critic (enhanced)

- DoD row «3/19 (TODO/FIXME)»: `# TODO` na lista `members` do `Cargo.toml` raiz e `<!-- TODO -->` no `README.md` passam (o PROJECT só lê `*.rs`; a linha 19 só `apps/ddc-tray`); `TODOs:`/minúsculas escapam (suspeita).
- DoD row «9/10 (node --test)»: `{ skip: … }` ou `{ todo: … }` num teste que falharia dá `# fail 0` e exit 0 — Verify OK (demonstrado com regressão de Esc no dropdown e texto literal no HTML).
- DoD row «15 (hardware)»: `let _ = RestoreBrightness { … }` (guard dropado na hora; monitor fica +10) passa no Verify.
- DoD row «16 (Gate 7)»: `dd -ge 4` aceita 12 dos 16 testes do dropdown pulados com `test.skip` — regressões de contraste com a lista aberta e de Esc passam.
- DoD row «17 (sem `<select>`)»: `element('select')` (o helper do próprio `app.js`, forma usada antes da correção) passa no grep.
- DoD row «18 (ksni + roda)»: `Orientation::Vertical`→`Horizontal` em `status_item.rs` (roda vertical não faz nada) passa — o Verify só prova a lógica pura.
- DoD row «20 (screenshots)»: renomear a guarda do spec para `UPDATE_SCREENSHOTS` faz a regeneração pular (6 skipped) e o `git diff --quiet` passa com PNGs obsoletos.
- Suspeitas (objective:false): «4» módulo Windows fora do clippy Linux (D-2026-09-26-tray-app-9); «14» o smoke aceita `popup shown` seguido de `popup hidden`.

**Verdict:** BLOCKED
