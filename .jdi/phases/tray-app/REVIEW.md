# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 5, revisada por `jdi-reviewer-ddc-control` em 2026-09-27.
- **Branch:** `phase/tray-app`, HEAD `d4c6cf6`.
- **Host:** Linux (Fedora 44, KDE Plasma 6 Wayland), rustc/cargo 1.98.1, Node 24.18.0.
- **Escopo:** re-verificação completa (gates 1-8) depois da iteração 5 (`93dcd83..d4c6cf6`). Ela fecha as 2 lacunas do DoD critic da iter 4 (debounce vs throttle; axe travado) e o W-2 da iter 4, e implementa a D-2026-09-27-tray-app-6 (demo só em servidor local; o teste de hardware recusa `DDC_TRAY_FAKE`).
- **Reviews anteriores:** estão no histórico (`git show 839e378:.jdi/phases/tray-app/REVIEW.md` = iter 4).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0 (inclui `ddc-tray`). O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked` sai 0 em `x86_64-unknown-linux-gnu` e em `x86_64-pc-windows-msvc` (e `-p ddc-adapters --features ddc-hi` no msvc também). O `ddc-tray` fica fora do msvc (D-2026-09-26-tray-app-9). Único aviso: o future-incompat de `nom v3.2.1` (via `ddc-hi`), anterior à phase. |
| Tests | PASS | 383 passed, 0 failed, 9 ignored (hardware). Igual à iter 4: a iter 5 não trouxe teste Rust que rode sem hardware (só a guarda nos 2 testes `#[ignore]`). Fora do cargo: `node --test` 129 pass (iter 4: 124), Playwright 56 passed (iter 4: 52). |
| Coverage | PASS | 83.22% lines, threshold 80% (TOTAL row, main.rs/build.rs excluded), exit 0. Igual à iter 4: nenhum `.rs` de `src/` mudou (`git diff --stat c725c2e..HEAD` em `crates/` e `src-tauri/` só lista `tests/rtk_qhd_hdr.rs`). |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]`/`#[expect(...)]` fora de testes. Os 4 crates herdam `[workspace.lints]` (`unsafe_code = "deny"`, `unwrap_used`/`expect_used`/`panic` = warn), intactos. |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 limpos (detalhe abaixo). A 5.10 (`cargo audit`) sai ≠0 com os mesmos 4 advisories das iters 3 e 4 (W-1). |
| Consistency | PASS | D-1, D-2, D-2026-09-26-tray-app-2/-3/-6/-7/-8 e D-2026-09-27-tray-app-6 conformes. Os 5 commits usam o escopo `tray-app`, os tipos batem com o conteúdo (`test` só em testes, `fix` no `bridge.js` com seus testes, `docs` em docs), e todos citam a D-XX. As 8 tasks do PLAN seguem `completed`. |
| UI Validation | PASS | Suíte Playwright de `apps/ddc-tray` com servidor próprio (`reuseExistingServer: false`, porta 1420 livre antes e depois): 56 passed, 6 skipped (screenshots sem `SCREENSHOTS=1`). Passam os 10 `critical_paths` × tema, os 16 testes de `dropdown.spec.mjs` e os 4 novos de `unavailable.spec.mjs`, com console limpo, axe critical/serious `toEqual([])` e zero `<select>` em runtime. |
| DoD | PASS_PENDING_MANUAL | 21/21 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1–5.4 (hexagonal):** limpo.
  - O `ddc-core` segue só com `thiserror`, sem I/O e sem `cfg` de plataforma. O `crates/` não foi tocado na iter 5.
  - Não há `impl MonitorBackend` no core nem `pub trait` fora dele.
  - `DdcHiMonitorBackend::new` aparece só em `apps/ddc-tray/src-tauri/src/lib.rs:77` (composition root). Nenhum adapter importa outro.
- **5.5 (`unsafe`):** a única ocorrência é o comentário `lib.rs:146`. `#![forbid(unsafe_code)]` está nas raízes de `ddc-core`, `ddc-cli` e `ddc-tray`.
- **5.6 (pânicos):** os mesmos hits da iter 4, todos em `#[cfg(test)]`:
  - `kwin_placement.rs:246-267` (bloco de teste desde a linha 195);
  - `lib.rs:269` (desde a 239);
  - `stop_signals.rs:105-108` (desde a 82).
  
  O `worker.rs:141` de `ddc-adapters` é doc comment, anterior à phase.
- **5.7 (escrita segura):**
  - `Confirm::Yes` só aparece em `commands.rs` e na CLI.
  - `Dangerous` segue classificado em `crates/ddc-core/src/domain`.
  - Os 7 testes de `crates/ddc-adapters/tests/real_monitor.rs` e os 2 de `apps/ddc-tray/src-tauri/tests/rtk_qhd_hdr.rs` são `#[ignore]` e gated por `DDC_HW_TESTS=1`. O revisor não rodou nenhum deles, nem com `--ignored`, nem com `DDC_HW_TESTS`.
- **5.8, 5.9 e 5.11:** limpos. Nenhum caminho de dispositivo no core, todo `#[tauri::command]` é `async fn`, e não há segredo nem TODO/FIXME sem issue.
- **5.10:** W-1.

### Revisão dirigida da iteração 5

**1 — debounce vs throttle (`93dcd83`): correto e prova o que diz.**
- O teste novo faz uma rajada de 20 pushes a cada 20 ms (400 ms, mais de 4× a janela de 80 ms). Ele exige zero escritas durante a rajada e até 79 ms depois do último push, e exatamente `[[0x10, 95]]` aos 80 ms.
- **Mutação independente**, feita numa cópia no scratchpad (o repo não foi tocado): tirei o `stopTimer(state)` do `push` de `debounce.js`, que é o throttle do critic.
  - Resultado: `not ok 4 - a burst longer than the debounce window writes once, after it ends`, `# pass 13`, `# fail 1`.
  - Só o teste novo pega o mutante, o que confirma a lacuna e o seu fechamento.

**2 — bridge só em servidor local (`4280a59`, D-2026-09-27-tray-app-6): conforme.**
- **Condição:** `createBridge` devolve o demo só com `tauri == null && isLocalDevServer(win?.location)`, ou seja, `http:`/`https:` em `localhost`/`127.0.0.1`. É literalmente o texto da D-6.
- **Fora disso:** o bridge `unavailable` rejeita os 7 comandos com `{ kind: 'backend_unavailable', message: UNAVAILABLE_MESSAGE }`. Ele reaproveita o `api()` comum (sem duplicar a superfície) e não expõe `__ddcDemo`.
- **Listeners:** `onPopupShown`/`onPanelChanged` resolvem um `unlisten` no-op. É razoável, porque o primeiro comando já mostra o erro.
- **Caso-limite:** `__TAURI__` presente sem `invoke` → `unavailable`. É a leitura estrita de "demo só com `__TAURI__` ausente".
- **Origens reais:** `tauri://localhost` tem `protocol` `tauri:`, e `http://tauri.localhost` tem `hostname` `tauri.localhost`. Nenhuma das duas passa pelo `isLocalDevServer`, e os testes cobrem as duas (mais `https://tauri.localhost`, `file:`, IP de LAN, `localhost.example.com`, `[::1]` e `ftp:`).
- **Mutação independente M1** (a volta ao `return demoBridge(…)` incondicional, na cópia do scratchpad): `bridge-demo.test.mjs` dá `# fail 3`, incluindo `not ok 1 - outside a local dev server the bridge never falls back to the demo`.
- **No navegador:** `unavailable.spec.mjs` serve os mesmos arquivos em `http://tauri.localhost`, sob a CSP do app. A rota é registrada depois do catch-all da fixture, então tem prioridade. O spec exige o estado `error`, nenhum `__ddcDemo`, console limpo, nenhum `<select>` e axe limpo. Os 4 testes passaram.
- **`withGlobalTauri: true`** (`tauri.conf.json:9`) está travado pelo Verify do item 17 do DoD.

**3 — W-2 da iter 4 resolvido (`7482711`).**
- **Guarda:** nos 2 testes, `assert!(std::env::var_os("DDC_TRAY_FAKE").is_none(), "{NOT_SIMULATED}")` vem logo depois de `if !hardware_enabled() { return; }` e antes de `compose_osd()`. A asserção dispara antes de qualquer backend.
- **Rigor:** ela recusa qualquer valor, inclusive vazio. É mais estrita que o `switch_on` do app, que só liga com `1`, e segue o texto da D-6.
- **O que não rodei:** a prova com `DDC_HW_TESTS=1` do SUMMARY. A regra do revisor proíbe, e a leitura do código basta.

**4 — axe travado:** o Verify do item 16 do DoD confere as duas linhas literais de `support.mjs` e a ausência de `disableRules`/`exclude`/`include`/`options`/`disableFrameRules`/`setLegacyMode` em `tests/e2e/*.mjs`. Passou.

## Blockers (if any)
- Nenhum.

## Warnings (if any)
- **W-1: 5.10 supply chain (`cargo audit` sai ≠0).** São os mesmos 4 advisories das iters 3 e 4. A iter 5 não mexeu em `Cargo.lock` nem em `package-lock.json`.
  - RUSTSEC-2024-0429: `glib`, unsound.
  - RUSTSEC-2024-0370: `proc-macro-error`, sem manutenção. Este e o anterior vêm via tauri → gtk-rs.
  - RUSTSEC-2018-0005: `serde_yaml`, a vulnerabilidade que faz o `cargo audit` sair 1.
  - RUSTSEC-2024-0320: `yaml-rust`, sem manutenção. Este e o anterior vêm via `ddc-hi` → `mccs-db`.
  
  O tratamento (`audit.toml`, com cada ignore justificado) fica com a phase `ci-crossbuild`, como combinado.

Observações sem severidade:
- **Detalhe do erro em inglês (D-2026-09-26-tray-app-6):**
  - `UNAVAILABLE_MESSAGE` é um literal em inglês no JS, e aparece em `#message-detail`.
  - Não é violação: segue o contrato `{kind, message}` da D-2026-09-26-tray-app-4, em que o título vem traduzido pela chave `error.backend_unavailable` e o detalhe é o texto técnico cru. É o mesmo tratamento das mensagens de `DdcError` que vêm do Rust e do erro do demo (`'no DDC/CI backend could be started (demo)'`), que as reviews anteriores aceitaram.
  - Se algum dia o detalhe passar a ser traduzido, os três entram juntos.
- **Comentário desatualizado em `.jdi/PROJECT.md`:** o bloco `frontend:` (linhas 76-79) ainda diz "bridge JS fake quando `window.__TAURI__` não existe", sem a condição de `localhost` da D-6.
  - Não muda nada no Gate 7, porque `frontend_url` já é `http://localhost:1420`.
  - Vale ajustar a frase quando o PROJECT for editado de novo.
- **Binário que o usuário está testando:** o `~/.local/bin/ddc-tray` é de 02:38.
  - Ele tem as correções da iter 4 (`DDC_TRAY_FAKE` e `received, quitting` estão nas strings), mas é anterior à iter 5 (o `4280a59` é de 02:56).
  - Portanto, ainda não tem o bridge da D-6. Para testar essa mudança, o usuário precisa reinstalar a partir de `d4c6cf6`.
- **Cobertura:** a margem sobre 80% segue em 3.2 pontos. A cola de plataforma continua baixa:

  | Arquivo | Linhas |
  |---|---|
  | `commands.rs` | 49.32% |
  | `lib.rs` | 37.75% |
  | `kwin_placement.rs` | 45.36% |
  | `status_item.rs` | 23.03% |

  A lógica está em 99-100%. As próximas phases devem continuar extraindo lógica pura.
- **Windows:** nada da iter 5 toca Rust de plataforma. A origem `http://tauri.localhost` agora está coberta em JS e no Playwright. A compilação e a execução reais ficam para a `ci-crossbuild`.
- **Estado final da máquina:**
  - Só a instância do usuário está viva: `pgrep -xa ddc-tray` → `926684 /home/slipalison/.local/bin/ddc-tray`, registrada no watcher como `org.kde.StatusNotifierItem-926684-1`.
  - A porta 1420 está livre.
  - `git status` não mudou (fora este REVIEW.md, e o `.idea/` foi ignorado).
  - Os 6 PNGs de `docs/screenshots/` ficaram byte a byte iguais (SHA-1 antes e depois).

## DoD Checklist (gate 8)

Cada `Verify:` do CONTEXT.md e do PROJECT.md foi extraído por script para um arquivo, com checagem de que é substring exata do `.md`, e executado literalmente com `bash` a partir da raiz, sem `DDC_HW_TESTS` e sem `DDC_TRAY_FAKE` no ambiente. O item 2 do PROJECT rodou literalmente: `cargo llvm-cov --workspace --summary-only`, sem exclusões.

Os smokes (itens 14 e 19) seguiram o protocolo do orquestrador, um de cada vez:
1. `pgrep -xa ddc-tray` mostrava a instância do usuário, que foi encerrada com `pkill -x ddc-tray`.
2. O smoke rodou.
3. Logo em seguida, `setsid -f /home/slipalison/.local/bin/ddc-tray` a reabriu. Os PIDs foram 896681 → 925783 → 926684.

Os 2 itens `Manual` do CONTEXT repetem os do PROJECT (Source: PROJECT) e foram contados uma vez só.

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK`; 383 passed, 0 failed, 9 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | literal (sem exclusões): TOTAL lines 82.78%; forma do gate (`--fail-under-lines 80`, sem `main.rs`/`build.rs`): 83.22%, exit 0 |
| 3 | No `TODO`/`FIXME` without linked issue (`*.rs`) | PROJECT | Auto | PASS | `OK` (Verify da D-2026-09-27-tray-app-4) |
| 4 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 5 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 6 | Só o composition root constrói `DdcHiMonitorBackend` | CONTEXT | Auto | PASS | `OK` (1× em `lib.rs:77`) |
| 7 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` (7 `pub fn …<M: MonitorControl + ?Sized>`) |
| 8 | `#![forbid(unsafe_code)]` em `src-tauri` | CONTEXT | Auto | PASS | `OK` |
| 9 | `node --test` (debounce com rajada > janela, view-model, bridge demo…) sem falhas, fora de `src/` | CONTEXT | Auto | PASS | `OK`; `# tests 129`, `# pass 129`, 0 fail/cancelled/skipped/todo. A mutação throttle é pega pelo teste da rajada (acima) |
| 10 | Paridade i18n + HTML sem texto literal | CONTEXT | Auto | PASS | `OK` |
| 11 | CSP sem `unsafe-inline`, `script-src 'self'` | CONTEXT | Auto | PASS | `OK` |
| 12 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 13 | `tauri-plugin-single-instance` no composition root | CONTEXT | Auto | PASS | `OK` (1ª chamada do builder, `lib.rs:108`) |
| 14 | Smoke Linux/KDE `--activate` (PID próprio, popup mostrado e mantido) | CONTEXT | Auto | PASS | `OK`; PID 925552 → item `org.kde.StatusNotifierItem-925552-1`, `popup shown` após `Activate`, ainda mostrado 1.5 s depois; backend real só com leituras |
| 15 | Teste `#[ignore]` de hardware existe, compila, é gated e recusa `DDC_TRAY_FAKE` | CONTEXT | Auto | PASS | `OK`; 2 `#[test]` = 2 `#[ignore]` = 2 listados com `--ignored --list`; `var_os("DDC_TRAY_FAKE").is_none()` presente; não executado (regra) |
| 16 | Gate 7: console limpo e axe sem critical/serious nos `critical_paths` (axe travado) | CONTEXT | Auto | PASS | `OK`; 56 passed, 6 skipped (= screenshots), 10/10 `critical_paths` × tema, 16/16 dropdown |
| 17 | App real nunca mostra dados simulados (D-2026-09-27-tray-app-6) | CONTEXT | Auto | PASS | `OK`; `bridge-demo.test.mjs` 0 fail/0 skipped, `withGlobalTauri == true`; e 4/4 `unavailable.spec.mjs` no Playwright |
| 18 | Nenhum `<select>` nativo no popup | CONTEXT | Auto | PASS | `OK` (estático, inclusive `element('select')`); em runtime, `expectNoNativeSelect` em todo `open()`, antes de cada axe e no teardown |
| 19 | Linux: SNI `ksni`, roda → brilho testada em Rust + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK`; 20 testes `scroll`; `75 -> 80` (vertical +120), horizontal sem escrita em 1.5 s, `80 -> 75` (−120); 0/16 amostras com thread `ddc-hi-worker` ou fd `/dev/i2c-*` no PID do smoke |
| 20 | Nenhum `TODO`/`FIXME` sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | `OK` |
| 21 | Screenshots claro/escuro gerados pela suíte e versionados | CONTEXT | Auto | PASS | `OK`; regeneração com `SCREENSHOTS=1` sem nada pulado, 720×1120, `git diff --quiet` e SHA-1 dos 6 PNGs iguais antes e depois |
| 22 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `CHANGELOG.md:8` `## [Unreleased]` com o tray app (agora cita o demo só em servidor local e a recusa do monitor simulado); ainda sem heading de versão (release em `release-packaging`) |
| 23 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: seção `## Tray app` (`README.md:265`), `### Browser demo` (`:326`, com a condição de servidor local da D-6), `DDC_TRAY_FAKE` (`:361`) e a recusa nos testes de hardware (`:448`) conferidos contra o código; diff `main..HEAD` de README+CHANGELOG: +497/−7 linhas a revisar no PR |

**Totals:** 23 items | Auto: 21 (21 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod tray-app` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Não há blocker. Os gates 1-7 passam, só com o W-1 herdado, que fica para a `ci-crossbuild`. Os 21 itens Auto do DoD passam, e os 2 Manual aguardam o `/jdi-confirm-dod tray-app`.

A iteração 5 fecha o que o critic da iter 4 apontou:
- **Debounce vs throttle:** o teste da rajada mata o mutante throttle, conferido de forma independente.
- **Axe:** a configuração está travada pelo Verify.
- **D-2026-09-27-tray-app-6:** implementada ao pé da letra e provada em JS puro, em mutação e no navegador, na origem Windows do app.
- **W-2:** resolvido. O teste de hardware agora falha antes de qualquer backend se `DDC_TRAY_FAKE` estiver definido.

Para o PR:
- anexe a saída real do `rtk_qhd_hdr` com `DDC_HW_TESTS=1` (Deferred);
- avise o usuário de que a instância em `~/.local/bin` é anterior à iter 5.

## DoD Critic (enhanced)

- DoD row «9 (node --test / debounce do slider)»: `queueWrite(entry, value, { now: true })` no `input` do slider (`app.js:508`) faz um arrasto de 400 ms gerar 10 escritas em vez de 1 e todos os Verify passam — o `node --test` só exercita a fila e o único e2e de slider aperta uma tecla.
- DoD row «16 (Gate 7)»: filtrar `Failed to load resource` no coletor de console (`support.mjs:54`) esconde um 404 real em todos os 10 `critical_paths` e o Verify imprime OK — o coletor não está travado como o axe.
- Suspeitas (objective:false): «10» literal em `element(tag, cls, 'texto')` no `app.js` escapa do teste de i18n (contra D-2026-09-26-tray-app-6); «15» o guard de `DDC_TRAY_FAKE` só precisa aparecer uma vez; «3/20» `Todo` em caixa mista.

**Verdict:** BLOCKED
